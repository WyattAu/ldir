//! Preview compilation manager for the LSP server.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, Notify};
use tokio::time::{self};

use tower_lsp::Client;
use tower_lsp::lsp_types;

use crate::detect_extension;

const DEFAULT_DEBOUNCE_MS: u64 = 150;

/// Parameters for the preview status notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewStatusParams {
    /// Document URI.
    pub uri: String,
    /// Status message (e.g., "compiling", "ready", "error: ...").
    pub status: String,
    /// Path to the generated PDF, if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_path: Option<String>,
}

/// LSP notification for preview status updates.
#[derive(Debug)]
pub struct PreviewStatus;

impl lsp_types::notification::Notification for PreviewStatus {
    type Params = PreviewStatusParams;
    const METHOD: &'static str = "ldir/previewStatus";
}

/// A pending compilation request waiting to be debounced.
#[derive(Debug)]
struct PendingCompile {
    text: String,
    extension: String,
    uri: String,
}

/// Manages debounced preview compilation for the LSP server.
#[derive(Debug)]
pub struct PreviewManager {
    output_path: Arc<PathBuf>,
    debounce: Duration,
    notify: Arc<Notify>,
    enabled: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
    pending: Arc<Mutex<Option<PendingCompile>>>,
    #[allow(dead_code)]
    client: Client,
}

impl PreviewManager {
    /// Create a new preview manager that compiles to `output_path`.
    pub fn new(client: Client, output_path: PathBuf) -> Self {
        let notify = Arc::new(Notify::new());
        let running = Arc::new(AtomicBool::new(true));
        let pending = Arc::new(Mutex::new(None));
        let enabled = Arc::new(AtomicBool::new(false));
        let debounce = Duration::from_millis(DEFAULT_DEBOUNCE_MS);
        let output_path = Arc::new(output_path);

        let bg_notify = notify.clone();
        let bg_running = running.clone();
        let bg_pending = pending.clone();
        let bg_output_path = output_path.clone();
        let bg_client = client.clone();

        tokio::spawn(async move {
            Self::background_task(
                bg_notify,
                bg_running,
                bg_pending,
                bg_output_path,
                bg_client,
                debounce,
            )
            .await;
        });

        Self {
            output_path,
            debounce,
            notify,
            enabled,
            running,
            pending,
            client,
        }
    }

    /// Enable or disable preview compilation.
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    /// Check if preview compilation is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Returns the debounce duration.
    pub fn debounce(&self) -> Duration {
        self.debounce
    }

    /// Returns the output directory path.
    pub fn output_path(&self) -> &PathBuf {
        &self.output_path
    }

    /// Trigger a debounced recompilation of the given document.
    pub async fn trigger(&self, text: &str, uri: &str) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }

        let extension = detect_extension(uri).to_string();
        if !matches!(extension.as_str(), "md" | "tex" | "typ") {
            return;
        }

        let mut guard = self.pending.lock().await;
        *guard = Some(PendingCompile {
            text: text.to_string(),
            extension,
            uri: uri.to_string(),
        });
        drop(guard);

        self.notify.notify_one();
    }

    /// Shut down the background compilation task.
    pub fn shutdown(&self) {
        self.running.store(false, Ordering::Relaxed);
        self.notify.notify_one();
    }

    async fn background_task(
        notify: Arc<Notify>,
        running: Arc<AtomicBool>,
        pending: Arc<Mutex<Option<PendingCompile>>>,
        output_path: Arc<PathBuf>,
        #[allow(dead_code)] client: Client,
        debounce: Duration,
    ) {
        loop {
            notify.notified().await;

            if !running.load(Ordering::Relaxed) {
                break;
            }

            loop {
                tokio::select! {
                    biased;
                    _ = notify.notified() => {
                        continue;
                    }
                    _ = time::sleep(debounce) => {
                        break;
                    }
                }
            }

            if !running.load(Ordering::Relaxed) {
                break;
            }

            let compile = {
                let mut guard = pending.lock().await;
                guard.take()
            };

            let Some(compile) = compile else {
                continue;
            };

            let uri = compile.uri.clone();
            let _ = client
                .send_notification::<PreviewStatus>(PreviewStatusParams {
                    uri: uri.clone(),
                    status: "compiling".to_string(),
                    pdf_path: None,
                })
                .await;

            let output = (*output_path).clone();
            let result = tokio::task::spawn_blocking(move || {
                compile_to_pdf(&compile.text, &compile.extension, &compile.uri, &output)
            })
            .await;

            match result {
                Ok(Ok(pdf_path)) => {
                    tracing::info!("preview PDF written: {}", pdf_path.display());
                    let _ = client
                        .send_notification::<PreviewStatus>(PreviewStatusParams {
                            uri,
                            status: "ready".to_string(),
                            pdf_path: Some(pdf_path.to_string_lossy().to_string()),
                        })
                        .await;
                }
                Ok(Err(err)) => {
                    tracing::warn!("preview compile error: {}", err);
                    let _ = client
                        .send_notification::<PreviewStatus>(PreviewStatusParams {
                            uri,
                            status: format!("error: {}", err),
                            pdf_path: None,
                        })
                        .await;
                }
                Err(err) => {
                    tracing::error!("preview task panicked: {}", err);
                    let _ = client
                        .send_notification::<PreviewStatus>(PreviewStatusParams {
                            uri,
                            status: "error: compile task panicked".to_string(),
                            pdf_path: None,
                        })
                        .await;
                }
            }
        }
    }
}

fn compile_to_pdf(
    text: &str,
    extension: &str,
    uri: &str,
    output_dir: &Path,
) -> Result<PathBuf, String> {
    let module = match extension {
        "md" => {
            let v1 = ldir_md::parse_markdown(text);
            ldir_core::compiler::v1_to_v2::convert_v1_to_v2(&v1)
        }
        "tex" => {
            let v1 = ldir_tex::parse_tex(text);
            ldir_core::compiler::v1_to_v2::convert_v1_to_v2(&v1)
        }
        "typ" => ldir_typst::parse_typst(text),
        _ => return Err(format!("unsupported format: .{}", extension)),
    };

    let mut ctx = ldir_core::compiler::context::CompileContext::new();
    let gir_doc = ldir_core::compiler::v2_compile::compile_v2_document(&module, &mut ctx)
        .map_err(|e| format!("compile error: {e}"))?;

    let pdf_bytes = ldir_pdf::converter::gir_to_pdf(&gir_doc);

    let pdf_path = derive_pdf_path(uri, output_dir)?;
    if let Some(parent) = pdf_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("failed to create directory: {e}"))?;
    }

    std::fs::write(&pdf_path, &pdf_bytes).map_err(|e| format!("failed to write PDF: {e}"))?;

    Ok(pdf_path)
}

fn derive_pdf_path(uri: &str, output_dir: &Path) -> Result<PathBuf, String> {
    let url = url::Url::parse(uri).map_err(|e| format!("invalid URI: {e}"))?;
    let path = url
        .to_file_path()
        .map_err(|_| "URI is not a file path".to_string())?;

    let stem = path
        .file_stem()
        .ok_or("cannot determine file stem")?
        .to_string_lossy()
        .to_string();

    Ok(output_dir.join(format!("{stem}.pdf")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_lsp::lsp_types::notification::Notification;

    #[test]
    fn test_preview_manager_creation() {
        assert_eq!(DEFAULT_DEBOUNCE_MS, 150);
        let path = std::env::temp_dir().join("ldir-preview");
        assert!(path.ends_with("ldir-preview"));
    }

    #[test]
    fn test_debounce_duration() {
        let duration = Duration::from_millis(DEFAULT_DEBOUNCE_MS);
        assert_eq!(duration, Duration::from_millis(150));
    }

    #[test]
    fn test_enable_disable() {
        let enabled = Arc::new(AtomicBool::new(false));
        assert!(!enabled.load(Ordering::Relaxed));
        enabled.store(true, Ordering::Relaxed);
        assert!(enabled.load(Ordering::Relaxed));
        enabled.store(false, Ordering::Relaxed);
        assert!(!enabled.load(Ordering::Relaxed));
    }

    #[test]
    fn test_output_path() {
        let output_dir = std::env::temp_dir().join("ldir-preview");
        // Use a URI with a drive letter on Windows, or an absolute Unix path.
        #[cfg(windows)]
        let uri = "file:///C:/Users/user/doc.md";
        #[cfg(not(windows))]
        let uri = "file:///home/user/doc.md";
        let pdf = derive_pdf_path(uri, &output_dir).unwrap();
        assert_eq!(pdf.file_name().unwrap().to_string_lossy(), "doc.pdf");
        assert!(pdf.parent().unwrap().ends_with("ldir-preview"));

        #[cfg(windows)]
        let uri2 = "file:///C:/Users/user/report.tex";
        #[cfg(not(windows))]
        let uri2 = "file:///home/user/report.tex";
        let pdf = derive_pdf_path(uri2, &output_dir).unwrap();
        assert_eq!(pdf.file_name().unwrap().to_string_lossy(), "report.pdf");
    }

    #[test]
    fn test_derive_pdf_path_invalid_uri() {
        let output_dir = std::env::temp_dir().join("ldir-preview");
        let result = derive_pdf_path("not-a-uri", &output_dir);
        assert!(result.is_err());
    }

    #[test]
    fn test_preview_status_params_serialization() {
        let params = PreviewStatusParams {
            uri: "file:///test/doc.md".to_string(),
            status: "ready".to_string(),
            pdf_path: Some("/tmp/doc.pdf".to_string()),
        };
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains(r#""status":"ready""#));
        assert!(json.contains(r#""uri":"file:///test/doc.md""#));
        assert!(json.contains(r#""pdfPath":"/tmp/doc.pdf""#));
    }

    #[test]
    fn test_preview_status_notification_method() {
        assert_eq!(PreviewStatus::METHOD, "ldir/previewStatus");
    }

    #[test]
    fn test_derive_pdf_path_no_extension() {
        let output_dir = std::env::temp_dir().join("ldir-preview-out");
        #[cfg(windows)]
        let uri = "file:///C:/path/to/README";
        #[cfg(not(windows))]
        let uri = "file:///path/to/README";
        let result = derive_pdf_path(uri, &output_dir);
        assert!(result.is_ok());
        let pdf = result.unwrap();
        assert_eq!(pdf.file_name().unwrap().to_string_lossy(), "README.pdf");
        assert!(pdf.parent().unwrap().ends_with("ldir-preview-out"));
    }

    #[test]
    fn test_preview_status_params_error_no_pdf() {
        let params = PreviewStatusParams {
            uri: "file:///test/doc.md".to_string(),
            status: "error: compile failed".to_string(),
            pdf_path: None,
        };
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains(r#""status":"error: compile failed""#));
        assert!(!json.contains("pdfPath"));
    }
}
