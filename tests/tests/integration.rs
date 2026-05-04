//! End-to-end integration tests for the ldir compilation pipeline.
//!
//! These tests verify the complete pipeline: parse → compile → PDF.
//! They invoke the `ldc` binary as a subprocess and inspect the resulting PDF.

use std::path::PathBuf;

fn ldc_bin() -> PathBuf {
    let target = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into());
    let target = PathBuf::from(target);
    if target.is_relative() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = manifest_dir
            .parent()
            .expect("tests/ should be inside workspace");
        workspace_root.join(target).join("debug").join("ldc")
    } else {
        target.join("debug").join("ldc")
    }
}

fn compile_to_pdf(input: &str, ext: &str) -> Result<Vec<u8>, String> {
    let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    let input_path = dir.path().join(format!("test.{ext}"));
    let output_path = dir.path().join("output.pdf");
    std::fs::write(&input_path, input).map_err(|e| e.to_string())?;

    let bin = ldc_bin();
    if !bin.exists() {
        return Err(format!(
            "ldc binary not found at {:?}; run `cargo build -p ldc` first",
            bin
        ));
    }

    let status = std::process::Command::new(&bin)
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .output()
        .map_err(|e| e.to_string())?;

    if !status.status.success() {
        let stderr = String::from_utf8_lossy(&status.stderr);
        let stdout = String::from_utf8_lossy(&status.stdout);
        return Err(format!(
            "ldc failed (exit {:?}):\n{}\n{}",
            status.status.code(),
            stderr,
            stdout
        ));
    }

    std::fs::read(&output_path).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// 1. Basic Markdown
// ---------------------------------------------------------------------------

#[test]
fn test_markdown_basic() {
    let pdf = compile_to_pdf("# Hello World\n\nThis is a test paragraph.", "md").unwrap();
    assert!(pdf.starts_with(b"%PDF"), "output must be a valid PDF");
}

// ---------------------------------------------------------------------------
// 2. Markdown with full features
// ---------------------------------------------------------------------------

#[test]
#[ignore = "full-feature markdown is slow"]
fn test_markdown_full_features() {
    let input = "\
# Title

## Section

A paragraph with **bold** and *italic* text.

- Item one
- Item two

> A blockquote

---

| A | B |
|---|---|
| 1 | 2 |
";
    let pdf = compile_to_pdf(input, "md").unwrap();
    assert!(pdf.starts_with(b"%PDF"));
}

// ---------------------------------------------------------------------------
// 3. Basic LaTeX
// ---------------------------------------------------------------------------

#[test]
fn test_latex_basic() {
    let input = "\\documentclass{article}\n\\begin{document}\n\\section{Introduction}\nHello World\n\\end{document}";
    let pdf = compile_to_pdf(input, "tex").unwrap();
    assert!(pdf.starts_with(b"%PDF"));
}

// ---------------------------------------------------------------------------
// 4. LaTeX with math
// ---------------------------------------------------------------------------

#[test]
fn test_latex_math() {
    let input = "\\documentclass{article}\n\\begin{document}\nThe equation $E = mc^2$ is famous.\n\\end{document}";
    let pdf = compile_to_pdf(input, "tex").unwrap();
    assert!(pdf.starts_with(b"%PDF"));
}

// ---------------------------------------------------------------------------
// 5. Basic Typst
// ---------------------------------------------------------------------------

#[test]
fn test_typst_basic() {
    let pdf = compile_to_pdf("= Heading\n\nA paragraph.", "typ").unwrap();
    assert!(pdf.starts_with(b"%PDF"));
}

// ---------------------------------------------------------------------------
// 6. Determinism
// ---------------------------------------------------------------------------

#[test]
fn test_determinism() {
    let input = "# Test\n\nHello world.";
    let pdf1 = compile_to_pdf(input, "md").unwrap();
    let pdf2 = compile_to_pdf(input, "md").unwrap();
    assert_eq!(pdf1, pdf2, "PDF output must be deterministic");
}

// ---------------------------------------------------------------------------
// 7. PDF structure
// ---------------------------------------------------------------------------

#[test]
fn test_pdf_structure() {
    let pdf = compile_to_pdf("# Hello\n\nWorld.", "md").unwrap();
    assert!(pdf.starts_with(b"%PDF"), "must start with %PDF header");
    let pdf_str = String::from_utf8_lossy(&pdf);
    assert!(pdf_str.contains("/Pages"), "PDF must contain /Pages object");
    assert!(
        pdf_str.contains("/Type /Catalog"),
        "PDF must contain /Type /Catalog"
    );
}

// ---------------------------------------------------------------------------
// 8. Font embedding
// ---------------------------------------------------------------------------

#[test]
fn test_font_embedding() {
    let pdf = compile_to_pdf("# Test\n\nSome text content here.", "md").unwrap();
    let pdf_str = String::from_utf8_lossy(&pdf);
    assert!(
        pdf_str.contains("/Font") || pdf_str.contains("/Type /Font"),
        "PDF should contain a /Font dictionary"
    );
}

// ---------------------------------------------------------------------------
// 9. Multi-format same content
// ---------------------------------------------------------------------------

#[test]
fn test_multiformat() {
    let md_pdf = compile_to_pdf("# Hello\n\nWorld.", "md").unwrap();
    let tex_pdf = compile_to_pdf("\\section{Hello}\nWorld.", "tex").unwrap();
    assert!(md_pdf.starts_with(b"%PDF"), "Markdown output must be PDF");
    assert!(tex_pdf.starts_with(b"%PDF"), "LaTeX output must be PDF");
}

// ---------------------------------------------------------------------------
// 10. Empty document
// ---------------------------------------------------------------------------

#[test]
fn test_empty_document() {
    let result = compile_to_pdf("", "md");
    assert!(
        result.is_ok(),
        "empty input should produce a valid PDF: {:?}",
        result.err()
    );
    let pdf = result.unwrap();
    assert!(pdf.starts_with(b"%PDF"));
}

// ---------------------------------------------------------------------------
// 11. Unicode content
// ---------------------------------------------------------------------------

#[test]
fn test_unicode() {
    let input = "# 日本語テスト\n\nПривет мир\n\nHello 世界";
    let pdf = compile_to_pdf(input, "md").unwrap();
    assert!(pdf.starts_with(b"%PDF"));
}

// ---------------------------------------------------------------------------
// 12. Code block
// ---------------------------------------------------------------------------

#[test]
fn test_code_block() {
    let input = "```rust\nfn main() {}\n```";
    let pdf = compile_to_pdf(input, "md").unwrap();
    assert!(pdf.starts_with(b"%PDF"));
}
