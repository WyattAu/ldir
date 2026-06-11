//! ldir-lsp — Language Server Protocol server for ldir document formats.
//!
//! Provides IDE features (diagnostics, go-to-definition, hover, document
//! symbols) for Markdown, TeX, and Typst files via the LSP protocol.

#![deny(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

/// LSP backend implementation using tower-lsp.
pub mod backend;

/// Diagnostic computation for supported document formats.
pub mod diagnostics;

/// Live PDF preview support with debounced compilation.
pub mod preview;

/// CRDT-based collaborative editing support.
pub mod crdt;

/// Folding range computation for collapsible document regions.
pub mod folding;

/// Semantic token computation for syntax highlighting.
pub mod semantic_tokens;

/// Document symbol extraction from headings and environments.
pub mod symbols;

pub(crate) fn detect_extension(path: &str) -> &str {
    path.rsplit('.').next().map_or("", |s| s)
}
