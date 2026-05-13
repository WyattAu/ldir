//! WASM text shaping tests (W-1).
//!
//! Validates that the shaping pipeline correctly handles various Unicode scripts
//! when compiled to WASM. Uses the fast_path / ttf_parser fallback path.
//!
//! These tests exercise the shaping functions indirectly through the
//! compile_markdown_to_html pipeline, verifying that text content appears
//! correctly in the rendered output.
//!
//! Run with: cargo test -p ldir-wasm

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// ASCII-only text should render all characters.
/// Tests the fast-path mono advance shaping (byte-value glyph IDs).
#[wasm_bindgen_test]
fn wasm_shape_ascii_text() {
    let result =
        ldir_wasm::compile_markdown_to_html("The quick brown fox jumps over the lazy dog.");
    // Verify all ASCII characters appear
    assert!(result.contains("quick"), "missing 'quick'");
    assert!(result.contains("brown"), "missing 'brown'");
    assert!(result.contains("lazy"), "missing 'lazy'");
    assert!(result.contains("dog"), "missing 'dog'");
    // Should produce paragraph tags
    assert!(result.contains("<p>"), "should have <p> tags");
}

/// Unicode Latin text with accented characters.
/// Tests the ttf_parser cmap fallback for proper glyph IDs.
#[wasm_bindgen_test]
fn wasm_shape_unicode_latin() {
    let result = ldir_wasm::compile_markdown_to_html("Café résumé — naïve façade überstraße");
    // Accented characters should appear in output (not garbled)
    assert!(result.contains("Caf"), "missing 'Caf'");
    assert!(result.contains("résumé"), "missing 'résumé'");
    assert!(result.contains("façade"), "missing 'façade'");
    assert!(result.contains("über"), "missing 'über'");
}

/// CJK text — Chinese characters.
/// Tests that CJK characters produce visible output (even without
/// proper kerning/ligatures, the cmap lookup should find valid glyphs).
#[wasm_bindgen_test]
fn wasm_shape_cjk_chinese() {
    let result = ldir_wasm::compile_markdown_to_html("你好世界");
    // Chinese characters should appear in the output
    assert!(result.contains("你好"), "missing '你好'");
    assert!(result.contains("世界"), "missing '世界'");
}

/// CJK text — Japanese text.
#[wasm_bindgen_test]
fn wasm_shape_cjk_japanese() {
    let result = ldir_wasm::compile_markdown_to_html("こんにちは世界");
    assert!(result.contains("こんにちは"), "missing Japanese text");
}

/// CJK text — Korean text.
#[wasm_bindgen_test]
fn wasm_shape_cjk_korean() {
    let result = ldir_wasm::compile_markdown_to_html("안녕하세요");
    assert!(result.contains("안녕하세요"), "missing Korean text");
}

/// Mixed CJK and Latin text.
/// Tests that mixed-script documents render correctly.
#[wasm_bindgen_test]
fn wasm_shape_mixed_script() {
    let result = ldir_wasm::compile_markdown_to_html("Hello 你好 こんにちは");
    assert!(result.contains("Hello"), "missing English");
    assert!(result.contains("你好"), "missing Chinese");
    assert!(result.contains("こんにちは"), "missing Japanese");
}

/// Empty heading should not crash.
#[wasm_bindgen_test]
fn wasm_shape_empty_heading() {
    let result = ldir_wasm::compile_markdown_to_html("# ");
    // Should produce h1 tag (even with empty content)
    assert!(result.contains("<h1"), "should have <h1> tag");
}

/// Numbers and special characters.
#[wasm_bindgen_test]
fn wasm_shape_numbers_and_symbols() {
    let result = ldir_wasm::compile_markdown_to_html("Price: $42.50, count: 100, ratio: 3:1");
    assert!(result.contains("42.50"), "missing decimal number");
    assert!(result.contains("3:1"), "missing ratio");
    assert!(result.contains("$42.50"), "missing dollar sign");
}

/// Long paragraph should not truncate.
#[wasm_bindgen_test]
fn wasm_shape_long_paragraph() {
    let text = "A".repeat(500); // 500 'A' characters
    let result = ldir_wasm::compile_markdown_to_html(&text);
    // Should have content (not empty)
    assert!(!result.is_empty(), "long paragraph produced empty output");
    assert!(result.contains("<p>"), "should have <p> tag");
}

/// Multiple paragraphs with various content.
#[wasm_bindgen_test]
fn wasm_shape_multi_paragraph() {
    let result = ldir_wasm::compile_markdown_to_html(
        "First paragraph.\n\nSecond paragraph with **bold** text.\n\nThird paragraph.",
    );
    assert!(result.contains("First paragraph"), "missing first");
    assert!(result.contains("Second paragraph"), "missing second");
    assert!(result.contains("Third paragraph"), "missing third");
}
