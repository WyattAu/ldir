//! Plugin system for custom frontends and backends.
//!
//! Plugins can register:
//! - Input parsers (frontends) that convert file formats to S-IR v2
//! - Output generators (backends) that convert S-IR v2 to target formats
//!
//! ## Usage
//!
//! ```rust,ignore
//! use ldir_core::plugin::{PluginRegistry, FrontendPlugin, BackendPlugin};
//!
//! let mut registry = PluginRegistry::new();
//! registry.register_frontend(MyFrontend);
//! registry.register_backend(MyBackend);
//! ```

use ldir_ir::sir::v2::SIRModuleV2;
use std::path::Path;

/// Trait for input format plugins (frontends).
pub trait FrontendPlugin: Send + Sync {
    /// Human-readable name (e.g., "Markdown", "LaTeX").
    fn name(&self) -> &str;

    /// File extensions this plugin handles (e.g., ["md", "markdown"]).
    fn extensions(&self) -> &[&str];

    /// Parse a file into S-IR v2.
    fn parse_file(&self, path: &Path) -> Result<SIRModuleV2, PluginError>;

    /// Parse a string into S-IR v2.
    fn parse_string(&self, text: &str, source_name: &str) -> Result<SIRModuleV2, PluginError>;
}

/// Trait for output format plugins (backends).
pub trait BackendPlugin: Send + Sync {
    /// Human-readable name (e.g., "PDF", "HTML").
    fn name(&self) -> &str;

    /// File extension for output (e.g., "pdf").
    fn extension(&self) -> &str;

    /// Convert S-IR v2 to output bytes.
    fn generate(
        &self,
        module: &SIRModuleV2,
        options: &GenerateOptions,
    ) -> Result<Vec<u8>, PluginError>;
}

/// Options for output generation.
#[derive(Debug, Clone, Default)]
pub struct GenerateOptions {
    /// Output file path (for formats that need it).
    pub output_path: Option<std::path::PathBuf>,
    /// Page size ("a4", "letter", etc.).
    pub page_size: Option<String>,
    /// Font family name.
    pub font_family: Option<String>,
    /// Additional key-value options.
    pub extra: std::collections::HashMap<String, String>,
}

/// Plugin error type.
#[derive(Debug, Clone)]
pub enum PluginError {
    /// Parsing the input failed.
    ParseFailed(String),
    /// Generating the output failed.
    GenerateFailed(String),
    /// The requested format is not supported.
    UnsupportedFormat(String),
    /// An I/O error occurred.
    IoError(String),
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseFailed(msg) => write!(f, "parse failed: {msg}"),
            Self::GenerateFailed(msg) => write!(f, "generation failed: {msg}"),
            Self::UnsupportedFormat(fmt) => write!(f, "unsupported format: {fmt}"),
            Self::IoError(msg) => write!(f, "I/O error: {msg}"),
        }
    }
}

impl std::error::Error for PluginError {}

/// Registry of available plugins.
pub struct PluginRegistry {
    frontends: Vec<Box<dyn FrontendPlugin>>,
    backends: Vec<Box<dyn BackendPlugin>>,
}

impl PluginRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            frontends: Vec::new(),
            backends: Vec::new(),
        }
    }

    /// Register a frontend plugin.
    pub fn register_frontend(&mut self, plugin: Box<dyn FrontendPlugin>) {
        self.frontends.push(plugin);
    }

    /// Register a backend plugin.
    pub fn register_backend(&mut self, plugin: Box<dyn BackendPlugin>) {
        self.backends.push(plugin);
    }

    /// Find a frontend plugin by file extension.
    pub fn find_frontend(&self, ext: &str) -> Option<&dyn FrontendPlugin> {
        self.frontends
            .iter()
            .find(|f| f.extensions().contains(&ext))
            .map(|f| f.as_ref())
    }

    /// Find a backend plugin by name or extension.
    pub fn find_backend(&self, name: &str) -> Option<&dyn BackendPlugin> {
        self.backends
            .iter()
            .find(|b| b.name() == name || b.extension() == name)
            .map(|b| b.as_ref())
    }

    /// List all registered frontend names.
    pub fn frontend_names(&self) -> Vec<&str> {
        self.frontends.iter().map(|f| f.name()).collect()
    }

    /// List all registered backend names.
    pub fn backend_names(&self) -> Vec<&str> {
        self.backends.iter().map(|b| b.name()).collect()
    }

    /// Number of registered frontends.
    pub fn frontend_count(&self) -> usize {
        self.frontends.len()
    }

    /// Number of registered backends.
    pub fn backend_count(&self) -> usize {
        self.backends.len()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockFrontend {
        name_str: &'static str,
        exts: &'static [&'static str],
    }

    impl MockFrontend {
        fn new(name: &'static str, exts: &'static [&'static str]) -> Self {
            Self {
                name_str: name,
                exts,
            }
        }
    }

    impl FrontendPlugin for MockFrontend {
        fn name(&self) -> &str {
            self.name_str
        }

        fn extensions(&self) -> &[&str] {
            self.exts
        }

        fn parse_file(&self, _path: &Path) -> Result<SIRModuleV2, PluginError> {
            Ok(SIRModuleV2::new())
        }

        fn parse_string(
            &self,
            _text: &str,
            _source_name: &str,
        ) -> Result<SIRModuleV2, PluginError> {
            Ok(SIRModuleV2::new())
        }
    }

    struct MockBackend {
        name_str: &'static str,
        ext: &'static str,
    }

    impl MockBackend {
        fn new(name: &'static str, ext: &'static str) -> Self {
            Self {
                name_str: name,
                ext,
            }
        }
    }

    impl BackendPlugin for MockBackend {
        fn name(&self) -> &str {
            self.name_str
        }

        fn extension(&self) -> &str {
            self.ext
        }

        fn generate(
            &self,
            _module: &SIRModuleV2,
            _options: &GenerateOptions,
        ) -> Result<Vec<u8>, PluginError> {
            Ok(vec![b'<', b'h', b't', b'm', b'l', b'>'])
        }
    }

    #[test]
    fn test_empty_registry() {
        let registry = PluginRegistry::new();
        assert_eq!(registry.frontend_count(), 0);
        assert_eq!(registry.backend_count(), 0);
        assert!(registry.frontend_names().is_empty());
        assert!(registry.backend_names().is_empty());
    }

    #[test]
    fn test_register_and_count_frontend() {
        let mut registry = PluginRegistry::new();
        registry.register_frontend(Box::new(MockFrontend::new("Markdown", &["md", "markdown"])));
        assert_eq!(registry.frontend_count(), 1);
        assert_eq!(registry.backend_count(), 0);
    }

    #[test]
    fn test_register_and_count_backend() {
        let mut registry = PluginRegistry::new();
        registry.register_backend(Box::new(MockBackend::new("HTML", "html")));
        assert_eq!(registry.backend_count(), 1);
        assert_eq!(registry.frontend_count(), 0);
    }

    #[test]
    fn test_find_frontend_by_extension() {
        let mut registry = PluginRegistry::new();
        registry.register_frontend(Box::new(MockFrontend::new("Markdown", &["md", "markdown"])));
        registry.register_frontend(Box::new(MockFrontend::new("LaTeX", &["tex", "latex"])));

        let found = registry.find_frontend("md");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name(), "Markdown");

        let found_latex = registry.find_frontend("latex");
        assert!(found_latex.is_some());
        assert_eq!(found_latex.unwrap().name(), "LaTeX");
    }

    #[test]
    fn test_find_frontend_missing() {
        let registry = PluginRegistry::new();
        assert!(registry.find_frontend("docx").is_none());
    }

    #[test]
    fn test_find_backend_by_name() {
        let mut registry = PluginRegistry::new();
        registry.register_backend(Box::new(MockBackend::new("HTML", "html")));
        registry.register_backend(Box::new(MockBackend::new("PDF", "pdf")));

        let found = registry.find_backend("PDF");
        assert!(found.is_some());
        assert_eq!(found.unwrap().extension(), "pdf");
    }

    #[test]
    fn test_find_backend_by_extension() {
        let mut registry = PluginRegistry::new();
        registry.register_backend(Box::new(MockBackend::new("HTML", "html")));

        let found = registry.find_backend("html");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name(), "HTML");
    }

    #[test]
    fn test_frontend_names_list() {
        let mut registry = PluginRegistry::new();
        registry.register_frontend(Box::new(MockFrontend::new("Markdown", &["md"])));
        registry.register_frontend(Box::new(MockFrontend::new("LaTeX", &["tex"])));
        registry.register_frontend(Box::new(MockFrontend::new("Org", &["org"])));

        let names = registry.frontend_names();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"Markdown"));
        assert!(names.contains(&"LaTeX"));
        assert!(names.contains(&"Org"));
    }

    #[test]
    fn test_backend_names_list() {
        let mut registry = PluginRegistry::new();
        registry.register_backend(Box::new(MockBackend::new("HTML", "html")));
        registry.register_backend(Box::new(MockBackend::new("PDF", "pdf")));

        let names = registry.backend_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"HTML"));
        assert!(names.contains(&"PDF"));
    }

    #[test]
    fn test_duplicate_frontend_registration() {
        let mut registry = PluginRegistry::new();
        registry.register_frontend(Box::new(MockFrontend::new("Markdown", &["md"])));
        registry.register_frontend(Box::new(MockFrontend::new("Markdown", &["md"])));
        assert_eq!(registry.frontend_count(), 2);
    }

    #[test]
    fn test_duplicate_backend_registration() {
        let mut registry = PluginRegistry::new();
        registry.register_backend(Box::new(MockBackend::new("HTML", "html")));
        registry.register_backend(Box::new(MockBackend::new("HTML", "html")));
        assert_eq!(registry.backend_count(), 2);
    }

    #[test]
    fn test_parse_string_returns_module() {
        let mut registry = PluginRegistry::new();
        registry.register_frontend(Box::new(MockFrontend::new("Markdown", &["md"])));

        let frontend = registry.find_frontend("md").unwrap();
        let module = frontend.parse_string("# Hello", "test.md").unwrap();
        assert_eq!(module.header.version, (2, 0, 0));
    }

    #[test]
    fn test_backend_generate() {
        let mut registry = PluginRegistry::new();
        registry.register_backend(Box::new(MockBackend::new("HTML", "html")));

        let backend = registry.find_backend("html").unwrap();
        let module = SIRModuleV2::new();
        let result = backend
            .generate(&module, &GenerateOptions::default())
            .unwrap();
        assert_eq!(result, b"<html>");
    }

    #[test]
    fn test_plugin_error_display() {
        let err = PluginError::ParseFailed("unexpected token".into());
        assert_eq!(format!("{err}"), "parse failed: unexpected token");

        let err = PluginError::GenerateFailed("font not found".into());
        assert_eq!(format!("{err}"), "generation failed: font not found");

        let err = PluginError::UnsupportedFormat("docx".into());
        assert_eq!(format!("{err}"), "unsupported format: docx");

        let err = PluginError::IoError("file not found".into());
        assert_eq!(format!("{err}"), "I/O error: file not found");
    }

    #[test]
    fn test_generate_options_default() {
        let opts = GenerateOptions::default();
        assert!(opts.output_path.is_none());
        assert!(opts.page_size.is_none());
        assert!(opts.font_family.is_none());
        assert!(opts.extra.is_empty());
    }

    #[test]
    fn test_registry_default() {
        let registry = PluginRegistry::default();
        assert_eq!(registry.frontend_count(), 0);
        assert_eq!(registry.backend_count(), 0);
    }
}
