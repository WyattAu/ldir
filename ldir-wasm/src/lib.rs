//! LDIR WASM module — browser-based document compilation and preview.

#![warn(clippy::unwrap_used, clippy::expect_used)]
#![deny(unsafe_code)]
#![warn(missing_docs)]

use wasm_bindgen::prelude::*;

mod html_renderer;

pub mod bridge;
pub mod sandbox;
pub mod versioning;

use ldir_ir::sir::SIRDocument;

/// Initialize the WASM module. Call this once from JS.
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Compile a Markdown document and return HTML preview.
///
/// This is the main entry point for the browser playground.
/// It parses the Markdown using ldir-md and converts the
/// S-IR instructions to styled HTML.
#[wasm_bindgen]
pub fn compile_markdown_to_html(markdown: &str) -> String {
    html_renderer::render_markdown(markdown)
}

/// Get the version of the WASM module.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Compile a document from the given input format to HTML.
///
/// Supported formats: `"markdown"`, `"latex"`.
/// Returns an HTML string, or an error message for unsupported formats.
#[wasm_bindgen]
pub fn compile(input: &str, format: &str) -> String {
    match format {
        "markdown" => html_renderer::render_markdown(input),
        "latex" => {
            let doc: SIRDocument = ldir_tex::parse_tex(input);
            html_renderer::render_sir_document(&doc)
        }
        _ => format!("<p>Unsupported format: {format}</p>"),
    }
}

/// Compile S-IR bytes and render to pixels.
///
/// This is a convenience function that:
/// 1. Deserializes S-IR from the input bytes
/// 2. Validates the S-IR document
/// 3. Compiles S-IR to G-IR
/// 4. Renders G-IR to an RGBA pixel buffer
///
/// Returns raw RGBA pixel data (width * height * 4 bytes).
///
/// **Unstable:** This function is a placeholder for future GPU renderer
/// integration. Currently returns an empty buffer.
#[cfg(feature = "unstable")]
#[wasm_bindgen]
pub fn compile_and_render(_sir_bytes: &[u8]) -> Vec<u8> {
    // Placeholder: full implementation requires ldir-vello GPU renderer
    // integration in WASM. See ROADMAP_NEXT.md Phase W for status.
    Vec::new()
}
