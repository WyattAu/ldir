//! WASM build verification test.
//!
//! Validates that the ldir-wasm crate compiles for `wasm32-unknown-unknown`
//! and that exported WASM functions behave correctly.
//!
//! Run with:
//!   cargo test -p ldir-wasm --target wasm32-unknown-unknown
//! (requires `wasm-pack test` or `wasm-bindgen-test` runner)

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn version_returns_non_empty() {
    let v = ldir_wasm::version();
    if let Err(e) = (|| {
        if v.is_empty() {
            return Err("version returned empty string");
        }
        if !v.contains('.') {
            return Err("version missing dot separator");
        }
        Ok(())
    })() {
        panic!("{e}");
    }
}

#[wasm_bindgen_test]
fn compile_empty_markdown() {
    let result = ldir_wasm::compile_markdown_to_html("");
    if !result.is_empty() {
        panic!("expected empty output for empty input, got: {result}");
    }
}

#[wasm_bindgen_test]
fn compile_whitespace_only() {
    let result = ldir_wasm::compile_markdown_to_html("   \n\t  ");
    if !result.is_empty() {
        panic!("expected empty output for whitespace input, got: {result}");
    }
}

#[wasm_bindgen_test]
fn compile_heading_produces_html() {
    let result = ldir_wasm::compile_markdown_to_html("# Hello");
    let mut missing = Vec::new();
    if !result.contains("<h1>") {
        missing.push("<h1>");
    }
    if !result.contains("Hello") {
        missing.push("text 'Hello'");
    }
    if !result.contains("</h1>") {
        missing.push("</h1>");
    }
    if !result.contains("ldir-document") {
        missing.push("ldir-document wrapper");
    }
    if !missing.is_empty() {
        panic!("missing in output: {missing:?}");
    }
}

#[wasm_bindgen_test]
fn compile_md_to_html_basic() {
    let result = ldir_wasm::compile_markdown_to_html("# Hello\nworld");
    let mut missing = Vec::new();
    if !result.contains("<h1") {
        missing.push("<h1> heading");
    }
    if !result.contains("</p>") {
        missing.push("</p>");
    }
    if !result.contains("Hello") {
        missing.push("text 'Hello'");
    }
    if !missing.is_empty() {
        panic!("missing in output: {missing:?}");
    }
}

#[wasm_bindgen_test]
fn compile_paragraph_produces_html() {
    let result = ldir_wasm::compile_markdown_to_html("Hello world");
    let mut missing = Vec::new();
    if !result.contains("<p>") {
        missing.push("<p>");
    }
    if !result.contains("</p>") {
        missing.push("</p>");
    }
    if !result.contains("Hello world") {
        missing.push("text 'Hello world'");
    }
    if let Err(e) = (|| {
        if missing.is_empty() {
            Ok(())
        } else {
            Err(format!("missing in output: {missing:?}"))
        }
    })() {
        panic!("{e}");
    }
}

#[wasm_bindgen_test]
#[cfg(feature = "unstable")]
fn compile_and_render_returns_empty_for_empty_input() {
    let result = ldir_wasm::compile_and_render(&[]);
    if !result.is_empty() {
        panic!(
            "expected empty buffer for empty input, got {} bytes",
            result.len()
        );
    }
}
