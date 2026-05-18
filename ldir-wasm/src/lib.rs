//! LDIR WASM module — browser-based document compilation and preview.

#![warn(clippy::unwrap_used, clippy::expect_used)]
#![deny(unsafe_code)]
#![warn(missing_docs)]

use wasm_bindgen::prelude::*;

mod html_renderer;

pub mod bridge;
pub mod sandbox;
pub mod versioning;

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
/// **Unstable:** This function is not yet implemented. Enable with
/// `features = ["unstable"]` to access the API surface.
#[cfg(feature = "unstable")]
#[wasm_bindgen]
pub fn compile_and_render(_sir_bytes: &[u8]) -> Vec<u8> {
    todo!("compile_and_render requires GPU renderer integration (ldir-vello)")
}
