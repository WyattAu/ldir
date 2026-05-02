use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::diagnostics;
use crate::preview::PreviewManager;
use crate::symbols;

#[derive(Debug)]
struct DocumentState {
    text: String,
    uri: Url,
}

#[derive(Debug)]
/// The LSP backend that tracks open documents and handles LSP requests.
pub struct Backend {
    client: Client,
    documents: RwLock<HashMap<String, DocumentState>>,
    preview: PreviewManager,
}

impl Backend {
    /// Create a new backend with the given LSP client.
    pub fn new(client: Client) -> Self {
        let output_path = PathBuf::from("/tmp/ldir-preview");
        Self {
            client: client.clone(),
            documents: RwLock::new(HashMap::new()),
            preview: PreviewManager::new(client, output_path),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                completion_provider: Some(CompletionOptions::default()),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        tracing::info!("ldir-lsp initialized");
    }

    async fn shutdown(&self) -> Result<()> {
        self.preview.shutdown();
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let doc = params.text_document;
        let uri = doc.uri.clone();
        let uri_str = uri.to_string();
        let diag = diagnostics::compute_diagnostics(&doc.text, &uri);
        let state = DocumentState {
            text: doc.text,
            uri: doc.uri,
        };
        {
            let mut docs = self.documents.write().await;
            docs.insert(uri_str, state);
        }
        self.client.publish_diagnostics(uri, diag, None).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let uri_str = uri.to_string();
        let text = {
            let mut docs = self.documents.write().await;
            if let Some(state) = docs.get_mut(&uri_str) {
                for change in params.content_changes {
                    state.text = change.text;
                }
                Some(state.text.clone())
            } else {
                None
            }
        };

        let Some(text) = text else {
            return;
        };

        let diag = diagnostics::compute_diagnostics(&text, &uri);
        self.preview.trigger(&text, &uri_str).await;
        self.client.publish_diagnostics(uri, diag, None).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        let uri_str = uri.to_string();
        {
            let mut docs = self.documents.write().await;
            docs.remove(&uri_str);
        }
        self.client.publish_diagnostics(uri, Vec::new(), None).await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let docs = self.documents.read().await;
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        let Some(state) = docs.get(&uri.to_string()) else {
            return Ok(None);
        };
        let syms = symbols::extract_symbols(&state.text, uri);
        for sym in &syms {
            let r = sym.range;
            if pos.line >= r.start.line
                && pos.line <= r.end.line
                && pos.character >= r.start.character
                && pos.character <= r.end.character
            {
                let name = sym.name.clone();
                let hover_text = match &sym.detail {
                    Some(d) => format!("{name}\n\n{d}"),
                    None => name,
                };
                return Ok(Some(Hover {
                    contents: HoverContents::Scalar(MarkedString::String(hover_text)),
                    range: None,
                }));
            }
        }
        Ok(None)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let docs = self.documents.read().await;
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        let Some(state) = docs.get(&uri.to_string()) else {
            return Ok(None);
        };
        let ext = crate::detect_extension(uri.path());
        let Some(line_text) = state.text.lines().nth(pos.line as usize) else {
            return Ok(None);
        };
        match ext {
            "tex" => {
                if let Some(label) = extract_ref_label(line_text) {
                    let pattern = format!("\\label{{{label}}}");
                    for (li, l) in state.text.lines().enumerate() {
                        if let Some(col) = l.find(&pattern) {
                            let start = col as u32;
                            let end = start + pattern.len() as u32;
                            return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                                uri: uri.clone(),
                                range: Range {
                                    start: Position {
                                        line: li as u32,
                                        character: start,
                                    },
                                    end: Position {
                                        line: li as u32,
                                        character: end,
                                    },
                                },
                            })));
                        }
                    }
                }
            }
            "md" => {
                if let Some(target) = extract_md_link_target(line_text) {
                    for (li, l) in state.text.lines().enumerate() {
                        if l.contains(&format!("# {target}"))
                            || l.contains(&format!("## {target}"))
                            || l.contains(&format!("### {target}"))
                            || l.contains(&format!("#{target}"))
                        {
                            return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                                uri: uri.clone(),
                                range: Range {
                                    start: Position {
                                        line: li as u32,
                                        character: 0,
                                    },
                                    end: Position {
                                        line: li as u32,
                                        character: l.len() as u32,
                                    },
                                },
                            })));
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(None)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let docs = self.documents.read().await;
        let uri = &params.text_document.uri;
        let Some(state) = docs.get(&uri.to_string()) else {
            return Ok(None);
        };
        let syms = symbols::extract_symbols(&state.text, uri);
        Ok(Some(DocumentSymbolResponse::Nested(syms)))
    }
}

fn extract_ref_label(line: &str) -> Option<String> {
    for prefix in ["\\ref{", "\\eqref{", "\\cref{", "\\Cref{", "\\autoref{"] {
        if let Some(start) = line.find(prefix) {
            let rest = &line[start + prefix.len()..];
            if let Some(end) = rest.find('}') {
                let label = rest[..end].trim().to_string();
                if !label.is_empty() {
                    return Some(label);
                }
            }
        }
    }
    None
}

fn extract_md_link_target(line: &str) -> Option<String> {
    if let Some(start) = line.find("[[") {
        let rest = &line[start + 2..];
        if let Some(end) = rest.find("]]") {
            let target = rest[..end].trim().to_string();
            if !target.is_empty() {
                return Some(target);
            }
        }
    }
    None
}
