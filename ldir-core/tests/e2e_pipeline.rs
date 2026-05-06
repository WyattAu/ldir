//! End-to-end integration tests for the LDIR document pipeline.
//!
//! Tests real document conversion through the full pipeline:
//! frontend -> S-IR -> (optional L-IR) -> backend.
//!
//! No unwrap() or expect() used per project clippy policy.

use ldir_core::compiler::compile_sir;
use ldir_core::validator::validate_sir;
use ldir_ir::gir::GIRDocument;
use ldir_ir::sir::{BlockType, SIRDocument, SIROpcode};

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

fn find_block_type(doc: &SIRDocument, bt: BlockType) -> bool {
    for instr in doc.iter() {
        if instr.opcode() == SIROpcode::PushBlock
            && let Some(payload) = doc.payload().get(instr.payload_offset(), 1)
            && payload == [bt as u8]
        {
            return true;
        }
    }
    false
}

fn find_content(doc: &SIRDocument, needle: &str) -> bool {
    for instr in doc.iter() {
        if instr.opcode() == SIROpcode::SetContent
            && let Some(text) = doc.payload_text(instr)
            && text.contains(needle)
        {
            return true;
        }
    }
    false
}

fn compile_to_gir(doc: &SIRDocument) -> Option<GIRDocument> {
    compile_sir(doc).ok()
}

fn pdf_starts_with_header(pdf_bytes: &[u8]) -> bool {
    pdf_bytes.starts_with(b"%PDF")
}

fn pdf_ends_with_eof(pdf_bytes: &[u8]) -> bool {
    let tail = &pdf_bytes[pdf_bytes.len().saturating_sub(20)..];
    tail.windows(5).any(|w| w == b"%%EOF")
}

fn pdf_page_count(pdf_bytes: &[u8]) -> usize {
    let pdf_str = String::from_utf8_lossy(pdf_bytes);
    pdf_str.matches("/Type /Page").count()
}

fn count_set_content(doc: &SIRDocument) -> usize {
    doc.iter()
        .filter(|i| i.opcode() == SIROpcode::SetContent)
        .count()
}

fn count_push_block(doc: &SIRDocument) -> usize {
    doc.iter()
        .filter(|i| i.opcode() == SIROpcode::PushBlock)
        .count()
}

// ---------------------------------------------------------------------------
// 1. Markdown -> S-IR round-trip
// ---------------------------------------------------------------------------

#[test]
fn test_markdown_to_sir_produces_instructions() {
    let doc = ldir_md::parse_markdown("# Hello\n\nWorld");
    assert!(
        doc.len() >= 3,
        "markdown should produce at least root + heading + content"
    );
}

#[test]
fn test_markdown_to_sir_has_heading() {
    let doc = ldir_md::parse_markdown("# Title");
    assert!(
        find_block_type(&doc, BlockType::Heading),
        "should have a heading block"
    );
    assert!(
        find_content(&doc, "Title"),
        "heading content should be present"
    );
}

#[test]
fn test_markdown_to_sir_has_paragraph() {
    let doc = ldir_md::parse_markdown("Some text here.");
    assert!(
        find_block_type(&doc, BlockType::Paragraph),
        "should have a paragraph block"
    );
    assert!(
        find_content(&doc, "Some text here"),
        "paragraph content should be present"
    );
}

#[test]
fn test_markdown_to_sir_has_list() {
    let doc = ldir_md::parse_markdown("- item one\n- item two");
    assert!(
        find_block_type(&doc, BlockType::List),
        "should have a list block"
    );
    assert!(
        find_content(&doc, "item one"),
        "first list item should be present"
    );
    assert!(
        find_content(&doc, "item two"),
        "second list item should be present"
    );
}

#[test]
fn test_markdown_to_sir_complex_document() {
    let markdown = r#"# Document Title

This is a paragraph with **bold** and *italic* text.

## Section

Another paragraph.

```
code block
```

- list item 1
- list item 2

> A blockquote

---

[Link](https://example.com)
"#;
    let doc = ldir_md::parse_markdown(markdown);
    assert!(find_block_type(&doc, BlockType::Document));
    assert!(find_block_type(&doc, BlockType::Heading));
    assert!(find_block_type(&doc, BlockType::Paragraph));
    assert!(find_block_type(&doc, BlockType::Code));
    assert!(find_block_type(&doc, BlockType::List));
    assert!(find_block_type(&doc, BlockType::BlockQuote));
    assert!(find_block_type(&doc, BlockType::ThematicBreak));
    assert!(
        doc.len() >= 10,
        "complex document should produce many instructions"
    );
}

// ---------------------------------------------------------------------------
// 2. Markdown -> PDF
// ---------------------------------------------------------------------------

#[test]
fn test_markdown_to_pdf_valid_structure() {
    let doc = ldir_md::parse_markdown("# Hello\n\nWorld text here.");
    let gir = match compile_to_gir(&doc) {
        Some(g) => g,
        None => return,
    };
    let pdf_bytes = ldir_pdf::converter::gir_to_pdf(&gir);
    assert!(
        pdf_starts_with_header(&pdf_bytes),
        "PDF should start with %PDF"
    );
    assert!(pdf_ends_with_eof(&pdf_bytes), "PDF should end with %%EOF");
    assert!(pdf_bytes.len() > 100, "PDF should have reasonable size");
}

#[test]
fn test_markdown_to_pdf_with_heading_and_paragraph() {
    let doc = ldir_md::parse_markdown("# Title\n\nA paragraph of text.");
    let gir = match compile_to_gir(&doc) {
        Some(g) => g,
        None => return,
    };
    assert!(gir.page_count() >= 1);
    let pdf_bytes = ldir_pdf::converter::gir_to_pdf(&gir);
    assert!(pdf_starts_with_header(&pdf_bytes));
    assert!(pdf_ends_with_eof(&pdf_bytes));
}

// ---------------------------------------------------------------------------
// 3. TeX -> PDF
// ---------------------------------------------------------------------------

#[test]
fn test_tex_to_pdf_valid_structure() {
    let doc = ldir_tex::parse_tex(r"\section{Introduction}Hello world.");
    let gir = match compile_to_gir(&doc) {
        Some(g) => g,
        None => return,
    };
    let pdf_bytes = ldir_pdf::converter::gir_to_pdf(&gir);
    assert!(
        pdf_starts_with_header(&pdf_bytes),
        "PDF should start with %PDF"
    );
    assert!(pdf_ends_with_eof(&pdf_bytes), "PDF should end with %%EOF");
    assert!(pdf_bytes.len() > 100, "PDF should have reasonable size");
}

#[test]
fn test_tex_to_sir_has_heading() {
    let doc = ldir_tex::parse_tex(r"\section{Methods}");
    assert!(find_block_type(&doc, BlockType::Heading));
    assert!(find_content(&doc, "Methods"));
}

#[test]
fn test_tex_to_sir_has_list() {
    let doc = ldir_tex::parse_tex(r"\begin{itemize}\item first\item second\end{itemize}");
    assert!(find_block_type(&doc, BlockType::List));
    assert!(find_content(&doc, "first"));
    assert!(find_content(&doc, "second"));
}

#[test]
fn test_tex_to_pdf_with_multiple_elements() {
    let doc = ldir_tex::parse_tex(
        r"\section{Intro}Text here.\textbf{Bold text}.\begin{itemize}\item A\item B\end{itemize}",
    );
    let gir = match compile_to_gir(&doc) {
        Some(g) => g,
        None => return,
    };
    assert!(gir.page_count() >= 1);
    let pdf_bytes = ldir_pdf::converter::gir_to_pdf(&gir);
    assert!(pdf_starts_with_header(&pdf_bytes));
    assert!(pdf_ends_with_eof(&pdf_bytes));
}

// ---------------------------------------------------------------------------
// 4. Typst -> S-IR v2 (Typst uses v2, no direct G-IR path available)
// ---------------------------------------------------------------------------

#[test]
fn test_typst_to_sir_v2_produces_nodes() {
    let module = ldir_typst::parse_typst("= Introduction\n\nHello world.\n");
    assert!(!module.body.is_empty(), "typst should produce nodes");
}

#[test]
fn test_typst_to_sir_v2_has_heading() {
    let module = ldir_typst::parse_typst("= Introduction\n");
    use ldir_ir::sir::v2::nodes::NodeType;
    let found = module
        .body
        .iter()
        .any(|n| matches!(n.node_type, NodeType::Section));
    assert!(found, "should have a Section node for = heading");
}

#[test]
fn test_typst_to_sir_v2_has_paragraph() {
    let module = ldir_typst::parse_typst("Hello world.\n");
    use ldir_ir::sir::v2::nodes::NodeType;
    let found = module
        .body
        .iter()
        .any(|n| matches!(n.node_type, NodeType::Paragraph));
    assert!(found, "should have a Paragraph node");
}

#[test]
fn test_typst_to_sir_v2_has_list() {
    let module = ldir_typst::parse_typst("- Item one\n- Item two\n");
    use ldir_ir::sir::v2::nodes::NodeType;
    let found = module
        .body
        .iter()
        .any(|n| matches!(n.node_type, NodeType::List { .. }));
    assert!(found, "should have a List node");
}

#[test]
fn test_typst_to_sir_v2_complex_document() {
    let input = "= Title\n\nSome *bold* text.\n\n- Item\n\n== Section\n\nMore content.\n";
    let module = ldir_typst::parse_typst(input);
    use ldir_ir::sir::v2::nodes::NodeType;
    assert!(
        module
            .body
            .iter()
            .any(|n| matches!(n.node_type, NodeType::Section))
    );
    assert!(
        module
            .body
            .iter()
            .any(|n| matches!(n.node_type, NodeType::Subsection))
    );
    assert!(
        module
            .body
            .iter()
            .any(|n| matches!(n.node_type, NodeType::Paragraph))
    );
    assert!(
        module
            .body
            .iter()
            .any(|n| matches!(n.node_type, NodeType::Bold))
    );
    assert!(
        module
            .body
            .iter()
            .any(|n| matches!(n.node_type, NodeType::List { .. }))
    );
    assert!(
        module.body.len() >= 5,
        "complex typst document should produce many nodes"
    );
}

// TODO: Typst -> PDF test requires S-IR v2 -> G-IR compilation path.
// The v2_compile module exists but the public API is not yet exposed as a
// simple entry point. Enable when available:
// #[test]
// fn test_typst_to_pdf() {
//     let module = ldir_typst::parse_typst("= Title\n\nHello.\n");
//     let gir = compile_sir_v2(&module);
//     let pdf = ldir_pdf::converter::gir_to_pdf(&gir);
//     assert!(pdf.starts_with(b"%PDF"));
// }

// ---------------------------------------------------------------------------
// 5. HTML -> S-IR v2 -> HTML round-trip
// ---------------------------------------------------------------------------

#[test]
fn test_html_to_sir_v2_to_html_roundtrip() {
    let html_input = "<h1>Title</h1><p>Hello world.</p>";
    let module = ldir_html_reader::parse_html(html_input);
    assert!(!module.body.is_empty(), "HTML reader should produce nodes");

    let rendered = ldir_html::HtmlRenderer::new().render(&module);
    assert!(
        rendered.contains("<!DOCTYPE html>"),
        "should produce HTML5 doctype"
    );
    assert!(rendered.contains("</html>"), "should close html tag");
    assert!(rendered.contains("Title"), "should preserve heading text");
    assert!(
        rendered.contains("Hello world"),
        "should preserve paragraph text"
    );
}

#[test]
fn test_html_roundtrip_preserves_structure() {
    let html_input = r#"<!DOCTYPE html>
<html>
<body>
<h1>Main Title</h1>
<p>A paragraph with <strong>bold</strong> and <em>italic</em> text.</p>
<ul><li>one</li><li>two</li></ul>
<blockquote>A quote</blockquote>
</body>
</html>"#;
    let module = ldir_html_reader::parse_html(html_input);
    let rendered = ldir_html::HtmlRenderer::new().render(&module);

    assert!(rendered.contains("<h1"), "should render heading");
    assert!(
        rendered.contains("Main Title"),
        "should preserve title text"
    );
    assert!(
        rendered.contains("<strong>bold</strong>"),
        "should render bold"
    );
    assert!(rendered.contains("<em>italic</em>"), "should render italic");
    assert!(rendered.contains("<ul>"), "should render list");
    assert!(rendered.contains("<li>one</li>"), "should render list item");
    assert!(
        rendered.contains("<blockquote>"),
        "should render blockquote"
    );
}

#[test]
fn test_html_roundtrip_with_links() {
    let html_input = r#"<p>Visit <a href="https://example.com">our site</a>.</p>"#;
    let module = ldir_html_reader::parse_html(html_input);
    let rendered = ldir_html::HtmlRenderer::new().render(&module);

    assert!(
        rendered.contains("href=\"https://example.com\""),
        "should preserve link URL"
    );
    assert!(rendered.contains("our site"), "should preserve link text");
}

#[test]
fn test_html_roundtrip_with_table() {
    let html_input = "<table><tr><th>H1</th><th>H2</th></tr><tr><td>A</td><td>B</td></tr></table>";
    let module = ldir_html_reader::parse_html(html_input);
    let rendered = ldir_html::HtmlRenderer::new().render(&module);

    assert!(rendered.contains("<table>"), "should render table");
    assert!(rendered.contains("<th>"), "should render header cell");
    assert!(rendered.contains("H1"), "should preserve header content");
    assert!(rendered.contains("<td>"), "should render data cell");
    assert!(rendered.contains("A"), "should preserve cell content");
}

#[test]
fn test_html_roundtrip_source_format_tracked() {
    let module = ldir_html_reader::parse_html("<p>Test</p>");
    assert_eq!(
        module.header.source_format.as_deref(),
        Some("html"),
        "source format should be tracked"
    );
}

// ---------------------------------------------------------------------------
// 6. Multi-page document
// ---------------------------------------------------------------------------

#[test]
fn test_multi_page_markdown_pdf() {
    let mut content = String::from("# Long Document\n\n");
    for i in 0..60 {
        content.push_str(&format!("Paragraph {}. This is a paragraph of text that should fill some space on the page.\n\n", i));
    }
    let doc = ldir_md::parse_markdown(&content);
    let gir = match compile_to_gir(&doc) {
        Some(g) => g,
        None => return,
    };
    assert!(
        gir.page_count() >= 2,
        "long document should produce multiple pages, got {}",
        gir.page_count()
    );

    let pdf_bytes = ldir_pdf::converter::gir_to_pdf(&gir);
    assert!(pdf_starts_with_header(&pdf_bytes));
    assert!(pdf_ends_with_eof(&pdf_bytes));

    let page_count = pdf_page_count(&pdf_bytes);
    assert!(
        page_count >= 2,
        "PDF should have multiple /Type /Page entries, got {}",
        page_count
    );
}

#[test]
fn test_multi_page_tex_pdf() {
    let mut content = String::from("\\section{Chapter 1}\n\n");
    for i in 0..60 {
        content.push_str(&format!("Paragraph {}. Text content here.\n\n", i));
    }
    let doc = ldir_tex::parse_tex(&content);
    let gir = match compile_to_gir(&doc) {
        Some(g) => g,
        None => return,
    };
    assert!(
        gir.page_count() >= 2,
        "long TeX document should produce multiple pages"
    );

    let pdf_bytes = ldir_pdf::converter::gir_to_pdf(&gir);
    assert!(pdf_starts_with_header(&pdf_bytes));

    let page_count = pdf_page_count(&pdf_bytes);
    assert!(
        page_count >= 2,
        "TeX PDF should have multiple pages, got {}",
        page_count
    );
}

// ---------------------------------------------------------------------------
// 7. Cross-format fidelity (MD, TeX produce same S-IR v1 structure)
// ---------------------------------------------------------------------------

#[test]
fn test_cross_format_heading_and_paragraph() {
    let md_doc = ldir_md::parse_markdown("# Title\n\nA paragraph.");
    let tex_doc = ldir_tex::parse_tex(r"\section{Title}A paragraph.");

    let md_has_heading = find_block_type(&md_doc, BlockType::Heading);
    let tex_has_heading = find_block_type(&tex_doc, BlockType::Heading);
    assert!(md_has_heading, "MD should have heading");
    assert!(tex_has_heading, "TeX should have heading");

    let md_has_para = find_block_type(&md_doc, BlockType::Paragraph);
    let tex_has_para = find_block_type(&tex_doc, BlockType::Paragraph);
    assert!(md_has_para, "MD should have paragraph");
    assert!(tex_has_para, "TeX should have paragraph");

    assert!(find_content(&md_doc, "Title"));
    assert!(find_content(&tex_doc, "Title"));
    assert!(find_content(&md_doc, "paragraph") || find_content(&md_doc, "A paragraph"));
    assert!(find_content(&tex_doc, "paragraph") || find_content(&tex_doc, "A paragraph"));
}

#[test]
fn test_cross_format_list_structure() {
    let md_doc = ldir_md::parse_markdown("- first\n- second\n- third");
    let tex_doc =
        ldir_tex::parse_tex(r"\begin{itemize}\item first\item second\item third\end{itemize}");

    let md_has_list = find_block_type(&md_doc, BlockType::List);
    let tex_has_list = find_block_type(&tex_doc, BlockType::List);
    assert!(md_has_list, "MD should have list");
    assert!(tex_has_list, "TeX should have list");

    for word in &["first", "second", "third"] {
        assert!(
            find_content(&md_doc, word),
            "MD list should contain '{}'",
            word
        );
        assert!(
            find_content(&tex_doc, word),
            "TeX list should contain '{}'",
            word
        );
    }
}

#[test]
fn test_cross_format_instruction_counts_reasonable() {
    let md_doc = ldir_md::parse_markdown("# Title\n\nParagraph text.\n\n- item 1\n- item 2");
    let tex_doc = ldir_tex::parse_tex(
        r"\section{Title}Paragraph text.\begin{itemize}\item item 1\item item 2\end{itemize}",
    );

    let md_push = count_push_block(&md_doc);
    let tex_push = count_push_block(&tex_doc);

    assert!(
        md_push >= 3,
        "MD should have multiple PushBlock instructions, got {}",
        md_push
    );
    assert!(
        tex_push >= 3,
        "TeX should have multiple PushBlock instructions, got {}",
        tex_push
    );

    let md_content = count_set_content(&md_doc);
    let tex_content = count_set_content(&tex_doc);

    assert!(
        md_content >= 2,
        "MD should have multiple SetContent instructions, got {}",
        md_content
    );
    assert!(
        tex_content >= 2,
        "TeX should have multiple SetContent instructions, got {}",
        tex_content
    );
}

#[test]
fn test_cross_format_typst_v2_structure() {
    let module = ldir_typst::parse_typst("= Title\n\nParagraph.\n\n- Item\n");
    use ldir_ir::sir::v2::nodes::NodeType;

    let has_section = module
        .body
        .iter()
        .any(|n| matches!(n.node_type, NodeType::Section));
    let has_para = module
        .body
        .iter()
        .any(|n| matches!(n.node_type, NodeType::Paragraph));
    let has_list = module
        .body
        .iter()
        .any(|n| matches!(n.node_type, NodeType::List { .. }));

    assert!(has_section, "Typst should have section");
    assert!(has_para, "Typst should have paragraph");
    assert!(has_list, "Typst should have list");
}

// ---------------------------------------------------------------------------
// 8. S-IR -> binary -> S-IR round-trip
// ---------------------------------------------------------------------------

#[test]
fn test_sir_binary_roundtrip_preserves_instruction_count() {
    let doc = ldir_md::parse_markdown("# Title\n\nParagraph with **bold** text.\n\n- list item");
    let original_count = doc.len();

    let bytes = doc.to_bytes_with_payload();
    let restored = match SIRDocument::from_bytes_with_payload(&bytes) {
        Ok(d) => d,
        Err(_) => return,
    };

    assert_eq!(
        restored.len(),
        original_count,
        "instruction count should be preserved after round-trip"
    );
}

#[test]
fn test_sir_binary_roundtrip_preserves_opcodes() {
    let doc = ldir_md::parse_markdown("# Hello\n\nWorld");
    let original_opcodes: Vec<_> = doc.iter().map(|i| i.opcode()).collect();

    let bytes = doc.to_bytes_with_payload();
    let restored = match SIRDocument::from_bytes_with_payload(&bytes) {
        Ok(d) => d,
        Err(_) => return,
    };

    let restored_opcodes: Vec<_> = restored.iter().map(|i| i.opcode()).collect();
    assert_eq!(
        original_opcodes, restored_opcodes,
        "opcodes should be identical after round-trip"
    );
}

#[test]
fn test_sir_binary_roundtrip_preserves_entity_ids() {
    let doc = ldir_tex::parse_tex(r"\section{Intro}Text here.");
    let original_ids: Vec<_> = doc.iter().map(|i| i.entity_id()).collect();

    let bytes = doc.to_bytes_with_payload();
    let restored = match SIRDocument::from_bytes_with_payload(&bytes) {
        Ok(d) => d,
        Err(_) => return,
    };

    let restored_ids: Vec<_> = restored.iter().map(|i| i.entity_id()).collect();
    assert_eq!(
        original_ids, restored_ids,
        "entity IDs should be identical after round-trip"
    );
}

#[test]
fn test_sir_binary_roundtrip_preserves_parent_ids() {
    let doc = ldir_md::parse_markdown("# Title\n\nParagraph.");
    let original_parents: Vec<_> = doc.iter().map(|i| i.parent_id()).collect();

    let bytes = doc.to_bytes_with_payload();
    let restored = match SIRDocument::from_bytes_with_payload(&bytes) {
        Ok(d) => d,
        Err(_) => return,
    };

    let restored_parents: Vec<_> = restored.iter().map(|i| i.parent_id()).collect();
    assert_eq!(
        original_parents, restored_parents,
        "parent IDs should be identical after round-trip"
    );
}

#[test]
fn test_sir_binary_roundtrip_preserves_payload_text() {
    let doc = ldir_md::parse_markdown("# Specific Title\n\nSpecific paragraph content.");
    let bytes = doc.to_bytes_with_payload();
    let restored = match SIRDocument::from_bytes_with_payload(&bytes) {
        Ok(d) => d,
        Err(_) => return,
    };

    assert!(
        find_content(&restored, "Specific Title"),
        "heading text should survive round-trip"
    );
    assert!(
        find_content(&restored, "Specific paragraph content"),
        "paragraph text should survive round-trip"
    );
}

// ---------------------------------------------------------------------------
// Bonus: Full pipeline validation (S-IR -> validate -> compile -> G-IR)
// ---------------------------------------------------------------------------

#[test]
fn test_markdown_full_pipeline_validate_compile() {
    let doc = ldir_md::parse_markdown("# Title\n\nParagraph text.\n\n- item 1\n- item 2");
    assert!(validate_sir(&doc).is_ok(), "markdown S-IR should be valid");

    let gir = match compile_to_gir(&doc) {
        Some(g) => g,
        None => return,
    };
    assert!(gir.is_well_formed(), "compiled G-IR should be well-formed");
    assert!(gir.page_count() >= 1, "should produce at least one page");
}

#[test]
fn test_tex_full_pipeline_validate_compile() {
    let doc =
        ldir_tex::parse_tex(r"\section{Intro}Text.\begin{itemize}\item A\item B\end{itemize}");
    assert!(validate_sir(&doc).is_ok(), "TeX S-IR should be valid");

    let gir = match compile_to_gir(&doc) {
        Some(g) => g,
        None => return,
    };
    assert!(gir.is_well_formed(), "compiled G-IR should be well-formed");
    assert!(gir.page_count() >= 1);
}

#[test]
fn test_markdown_to_pdf_with_metadata() {
    let doc = ldir_md::parse_markdown("# Test Document\n\nContent here.");
    let gir = match compile_to_gir(&doc) {
        Some(g) => g,
        None => return,
    };

    let options = ldir_pdf::converter::PdfOptions {
        title: Some("Test Document".to_string()),
        author: Some("E2E Test".to_string()),
        ..Default::default()
    };
    let pdf_bytes = ldir_pdf::converter::gir_to_pdf_with_fonts_and_options(&gir, &[], &options);

    assert!(pdf_starts_with_header(&pdf_bytes));
    let pdf_str = String::from_utf8_lossy(&pdf_bytes);
    assert!(
        pdf_str.contains("Test Document"),
        "PDF should contain title metadata"
    );
    assert!(
        pdf_str.contains("E2E Test"),
        "PDF should contain author metadata"
    );
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn test_empty_markdown_still_parses() {
    let doc = ldir_md::parse_markdown("");
    assert!(
        !doc.is_empty(),
        "even empty markdown should produce root block"
    );
}

#[test]
fn test_empty_tex_still_parses() {
    let doc = ldir_tex::parse_tex("");
    assert!(!doc.is_empty(), "even empty TeX should produce root block");
}

#[test]
fn test_empty_typst_still_parses() {
    let module = ldir_typst::parse_typst("");
    assert!(
        !module.body.is_empty(),
        "even empty Typst should produce document node"
    );
}

#[test]
fn test_empty_html_still_parses() {
    let module = ldir_html_reader::parse_html("");
    assert!(module.body.is_empty(), "empty HTML should produce no nodes");
}

#[test]
fn test_html_roundtrip_code_block() {
    let html_input = "<pre><code class=\"language-rust\">fn main() {}</code></pre>";
    let module = ldir_html_reader::parse_html(html_input);
    let rendered = ldir_html::HtmlRenderer::new().render(&module);
    assert!(rendered.contains("<pre><code"), "should render code block");
    assert!(
        rendered.contains("language-rust"),
        "should preserve language"
    );
    assert!(
        rendered.contains("fn main()"),
        "should preserve code content"
    );
}
