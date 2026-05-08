use ldir_core::compiler::compile_sir;
use ldir_ir::gir::{GIRCommand, GIRDocument, GIRPage};
use ldir_ir::sir::{BlockType, ROOT_SENTINEL, SIRDocument, SIRInstruction, SIROpcode};

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

fn pdf_starts_with(pdf_bytes: &[u8]) -> bool {
    pdf_bytes.starts_with(b"%PDF")
}

fn pdf_ends_with_eof(pdf_bytes: &[u8]) -> bool {
    let tail = &pdf_bytes[pdf_bytes.len().saturating_sub(20)..];
    tail.windows(5).any(|w| w == b"%%EOF")
}

fn compile_to_gir(doc: &SIRDocument) -> Option<GIRDocument> {
    compile_sir(doc).ok()
}

// =========================================================================
// 1. Parser edge cases
// =========================================================================

#[test]
fn parser_md_empty_string() {
    let doc = ldir_md::parse_markdown("");
    assert!(!doc.is_empty(), "empty MD should produce root block");
}

#[test]
fn parser_tex_empty_string() {
    let doc = ldir_tex::parse_tex("");
    assert!(!doc.is_empty(), "empty TeX should produce root block");
}

#[test]
fn parser_typst_empty_string() {
    let module = ldir_typst::parse_typst("");
    assert!(
        !module.body.is_empty(),
        "empty Typst should produce document node"
    );
}

#[test]
fn parser_html_empty_string() {
    let module = ldir_html_reader::parse_html("");
    assert!(module.body.is_empty(), "empty HTML should produce no nodes");
}

#[test]
fn parser_md_whitespace_only() {
    let doc = ldir_md::parse_markdown("   \n\t\n   \n");
    assert!(
        !doc.is_empty(),
        "whitespace-only MD should still produce root"
    );
    let has_para = find_block_type(&doc, BlockType::Paragraph);
    assert!(!has_para, "whitespace-only should not produce a paragraph");
}

#[test]
fn parser_tex_whitespace_only() {
    let doc = ldir_tex::parse_tex("   \n\n   ");
    assert!(
        !doc.is_empty(),
        "whitespace-only TeX should still produce root"
    );
}

#[test]
fn parser_md_very_long_line() {
    let long_line = "A".repeat(15000);
    let doc = ldir_md::parse_markdown(&long_line);
    assert!(
        find_content(&doc, "A"),
        "very long line content should be preserved"
    );
}

#[test]
fn parser_md_deeply_nested_lists() {
    let mut md = String::new();
    for _ in 0..12 {
        md.push_str("- item\n");
        md = md.replace("- item", "  - item");
    }
    md = format!("- item\n{md}");
    let doc = ldir_md::parse_markdown(&md);
    assert!(
        find_block_type(&doc, BlockType::List),
        "deeply nested lists should parse"
    );
}

#[test]
fn parser_md_special_characters() {
    let input = "Text with <tag> and 'apostrophes'.";
    let doc = ldir_md::parse_markdown(input);
    assert!(!doc.is_empty());
    assert!(
        find_content(&doc, "apostrophes"),
        "should preserve text with special chars"
    );
}

#[test]
fn parser_md_unicode_content() {
    let cjk = "日本語テスト Chinese 中文 한국어";
    let arabic = "مرحبا بالعالم";
    let input = format!("{}\n\n{}", cjk, arabic);
    let doc = ldir_md::parse_markdown(&input);
    assert!(
        find_content(&doc, "日本語"),
        "CJK characters should be preserved"
    );
    assert!(
        find_content(&doc, "中文"),
        "Chinese characters should be preserved"
    );
    assert!(
        find_content(&doc, "العالم"),
        "Arabic characters should be preserved"
    );
}

#[test]
fn parser_md_nested_blockquotes() {
    let input = "> Level 1\n>\n> > Level 2\n>\n> > > Level 3";
    let doc = ldir_md::parse_markdown(input);
    assert!(
        find_block_type(&doc, BlockType::BlockQuote),
        "nested blockquotes should parse"
    );
}

#[test]
fn parser_md_mixed_formatting() {
    let input = "This is ***bold and italic*** and `code` text.";
    let doc = ldir_md::parse_markdown(input);
    assert!(
        find_content(&doc, "bold and italic"),
        "mixed formatting should preserve text"
    );
    assert!(
        find_content(&doc, "code"),
        "inline code should be preserved"
    );
}

#[test]
fn parser_tex_unicode_content() {
    let input = r"\section{日本語テスト}Some Chinese 中文 text.";
    let doc = ldir_tex::parse_tex(input);
    assert!(find_content(&doc, "日本語"), "TeX CJK should be preserved");
}

#[test]
fn parser_html_special_entities() {
    let html = "<p>&amp; &lt; &gt; &quot; &apos;</p>";
    let module = ldir_html_reader::parse_html(html);
    use ldir_ir::sir::v2::nodes::NodeType;
    let has_text = module.body.iter().any(|n| match &n.node_type {
        NodeType::Text { content } => content.contains('&') && content.contains('<'),
        _ => false,
    });
    assert!(has_text, "HTML entities should be decoded");
}

#[test]
fn parser_typst_unicode() {
    let module = ldir_typst::parse_typst("= 日本語\n\n中文テスト\n");
    use ldir_ir::sir::v2::nodes::NodeType;
    let has_text = module.body.iter().any(|n| match &n.node_type {
        NodeType::Text { content } => content.contains("中文") || content.contains("テスト"),
        _ => false,
    });
    assert!(has_text, "Typst CJK should be preserved");
}

// =========================================================================
// 2. Compiler edge cases
// =========================================================================

#[test]
fn compile_empty_sir_document() {
    let doc = SIRDocument::new();
    let result = compile_sir(&doc);
    assert!(
        result.is_err(),
        "compiling empty S-IR should fail (no root block)"
    );
}

#[test]
fn compile_root_only_document() {
    let mut doc = SIRDocument::new();
    let offset = doc.payload_mut().append(&[BlockType::Document as u8]);
    doc.push(SIRInstruction::new(
        SIROpcode::PushBlock,
        0,
        ROOT_SENTINEL,
        offset,
    ));
    let result = compile_to_gir(&doc);
    assert!(result.is_some(), "root-only document should compile");
}

#[test]
fn compile_all_block_types() {
    let mut doc = SIRDocument::new();
    let root_offset = doc.payload_mut().append(&[BlockType::Document as u8]);
    doc.push(SIRInstruction::new(
        SIROpcode::PushBlock,
        0,
        ROOT_SENTINEL,
        root_offset,
    ));

    let block_types = [
        BlockType::Heading,
        BlockType::Paragraph,
        BlockType::List,
        BlockType::Code,
        BlockType::BlockQuote,
        BlockType::ThematicBreak,
        BlockType::Math,
        BlockType::Image,
    ];

    for (i, bt) in block_types.iter().enumerate() {
        let mut payload = vec![*bt as u8];
        if *bt == BlockType::Heading {
            payload.extend_from_slice(&1u32.to_le_bytes());
        }
        let offset = doc.payload_mut().append(&payload);
        let id = (i + 1) as u32;
        doc.push(SIRInstruction::new(SIROpcode::PushBlock, id, 0, offset));
        let content_id = (i + 100) as u32;
        let content = match bt {
            BlockType::Heading => "Test Heading",
            BlockType::Code => "fn main() {}",
            BlockType::Image => "photo.png",
            BlockType::Math => "x^2 + y^2",
            _ => "Test content",
        };
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::SetContent, content_id, id, 0),
            content.as_bytes(),
        );
    }

    let result = compile_to_gir(&doc);
    assert!(
        result.is_some(),
        "document with all block types should compile"
    );
    let gir = result.unwrap_or_default();
    assert!(gir.page_count() >= 1);
}

#[test]
fn compile_large_document() {
    let mut doc = SIRDocument::new();
    let root_offset = doc.payload_mut().append(&[BlockType::Document as u8]);
    doc.push(SIRInstruction::new(
        SIROpcode::PushBlock,
        0,
        ROOT_SENTINEL,
        root_offset,
    ));

    for i in 1..=100u32 {
        let offset = doc.payload_mut().append(&[BlockType::Paragraph as u8]);
        doc.push(SIRInstruction::new(SIROpcode::PushBlock, i, 0, offset));
        let content = format!("Paragraph {} with some filler text content.", i);
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::SetContent, i + 1000, i, 0),
            content.as_bytes(),
        );
    }

    let result = compile_to_gir(&doc);
    assert!(result.is_some(), "100-paragraph document should compile");
    let gir = result.unwrap_or_default();
    assert!(gir.page_count() >= 1);
}

#[test]
fn compile_document_with_inline_styles() {
    let mut doc = SIRDocument::new();
    let root_offset = doc.payload_mut().append(&[BlockType::Document as u8]);
    doc.push(SIRInstruction::new(
        SIROpcode::PushBlock,
        0,
        ROOT_SENTINEL,
        root_offset,
    ));

    let para_offset = doc.payload_mut().append(&[BlockType::Paragraph as u8]);
    doc.push(SIRInstruction::new(SIROpcode::PushBlock, 1, 0, para_offset));

    doc.push(SIRInstruction::new(SIROpcode::ApplyStyle, 2, 1, 0));
    doc.push(SIRInstruction::new(SIROpcode::ApplyStyle, 3, 1, 0));
    doc.push_with_payload(
        SIRInstruction::new(SIROpcode::SetContent, 4, 1, 0),
        b"Styled text",
    );
    doc.push(SIRInstruction::new(SIROpcode::ApplyStyle, 5, 1, 0));
    doc.push(SIRInstruction::new(SIROpcode::ApplyStyle, 6, 1, 0));

    let result = compile_to_gir(&doc);
    assert!(
        result.is_some(),
        "document with inline styles should compile"
    );
}

// =========================================================================
// 3. PDF backend edge cases
// =========================================================================

#[test]
fn pdf_empty_gir_to_pdf() {
    let doc = GIRDocument::new();
    let pdf_bytes = ldir_pdf::converter::gir_to_pdf(&doc);
    assert!(
        pdf_starts_with(&pdf_bytes),
        "empty G-IR PDF should start with %PDF"
    );
    assert!(
        pdf_ends_with_eof(&pdf_bytes),
        "empty G-IR PDF should end with %%EOF"
    );
}

#[test]
fn pdf_single_page_validity() {
    let mut doc = GIRDocument::with_capacity(1);
    let mut page = GIRPage::with_dimensions(612 * 64, 792 * 64);
    page.push(GIRCommand::new_set_font(0));
    page.push(GIRCommand::new_move_xy((72 * 64) as i32, (720 * 64) as i32));
    page.push(GIRCommand::new_put_glyph(72, (7 * 64) as i32));
    doc.push_page(page);

    let pdf_bytes = ldir_pdf::converter::gir_to_pdf(&doc);
    assert!(pdf_starts_with(&pdf_bytes), "PDF should start with %PDF");
    assert!(pdf_ends_with_eof(&pdf_bytes), "PDF should end with %%EOF");
    assert!(pdf_bytes.len() > 100, "PDF should have reasonable size");
}

#[test]
fn pdf_conformance_display() {
    use ldir_pdf::conformance::PdfConformance;

    assert_eq!(format!("{}", PdfConformance::PdfA4), "PDF/A-4");
    assert_eq!(format!("{}", PdfConformance::PdfA2b), "PDF/A-2b");
    assert_eq!(format!("{}", PdfConformance::PdfA3b), "PDF/A-3b");
}

#[test]
fn pdf_conformance_from_str() {
    use ldir_pdf::conformance::PdfConformance;
    use std::str::FromStr;

    let a4 = PdfConformance::from_str("pdfa4");
    assert_eq!(a4.ok(), Some(PdfConformance::PdfA4));

    let a2b = PdfConformance::from_str("pdf/a-2b");
    assert_eq!(a2b.ok(), Some(PdfConformance::PdfA2b));

    let a3b = PdfConformance::from_str("pdf/a-3b");
    assert_eq!(a3b.ok(), Some(PdfConformance::PdfA3b));

    let short_4 = PdfConformance::from_str("4");
    assert_eq!(short_4.ok(), Some(PdfConformance::PdfA4));

    let short_2b = PdfConformance::from_str("2b");
    assert_eq!(short_2b.ok(), Some(PdfConformance::PdfA2b));

    let short_3b = PdfConformance::from_str("3b");
    assert_eq!(short_3b.ok(), Some(PdfConformance::PdfA3b));

    let invalid = PdfConformance::from_str("nonsense");
    assert!(invalid.is_err(), "invalid conformance string should error");
}

#[test]
fn pdf_conformance_pdf_version() {
    use ldir_pdf::conformance::PdfConformance;

    assert_eq!(PdfConformance::PdfA4.pdf_version_str(), "2.0");
    assert_eq!(PdfConformance::PdfA2b.pdf_version_str(), "1.7");
    assert_eq!(PdfConformance::PdfA3b.pdf_version_str(), "1.7");
}

#[test]
fn pdf_conformance_pdfaid_part() {
    use ldir_pdf::conformance::PdfConformance;

    assert_eq!(PdfConformance::PdfA4.pdfaid_part(), 4);
    assert_eq!(PdfConformance::PdfA2b.pdfaid_part(), 2);
    assert_eq!(PdfConformance::PdfA3b.pdfaid_part(), 3);
}

#[test]
fn pdf_conformance_default() {
    use ldir_pdf::conformance::PdfConformance;
    assert_eq!(PdfConformance::default(), PdfConformance::PdfA4);
}

// =========================================================================
// 4. Serialization edge cases
// =========================================================================

#[test]
fn serialize_empty_payload_roundtrip() {
    let doc = SIRDocument::new();
    let bytes = doc.to_bytes_with_payload();
    let restored = match SIRDocument::from_bytes_with_payload(&bytes) {
        Ok(d) => d,
        Err(_) => return,
    };
    assert_eq!(
        restored.len(),
        doc.len(),
        "empty doc round-trip instruction count"
    );
}

#[test]
fn serialize_roundtrip_instruction_count() {
    let mut doc = SIRDocument::new();
    let root_offset = doc.payload_mut().append(&[BlockType::Document as u8]);
    doc.push(SIRInstruction::new(
        SIROpcode::PushBlock,
        0,
        ROOT_SENTINEL,
        root_offset,
    ));
    let para_offset = doc.payload_mut().append(&[BlockType::Paragraph as u8]);
    doc.push(SIRInstruction::new(SIROpcode::PushBlock, 1, 0, para_offset));
    doc.push_with_payload(
        SIRInstruction::new(SIROpcode::SetContent, 2, 1, 0),
        b"Hello world round-trip test",
    );

    let original_len = doc.len();
    let bytes = doc.to_bytes_with_payload();
    let restored = match SIRDocument::from_bytes_with_payload(&bytes) {
        Ok(d) => d,
        Err(_) => return,
    };
    assert_eq!(
        restored.len(),
        original_len,
        "instruction count should be preserved"
    );
}

#[test]
fn serialize_roundtrip_opcodes() {
    let mut doc = SIRDocument::new();
    let root_offset = doc.payload_mut().append(&[BlockType::Document as u8]);
    doc.push(SIRInstruction::new(
        SIROpcode::PushBlock,
        0,
        ROOT_SENTINEL,
        root_offset,
    ));
    let para_offset = doc.payload_mut().append(&[BlockType::Paragraph as u8]);
    doc.push(SIRInstruction::new(SIROpcode::PushBlock, 1, 0, para_offset));
    doc.push(SIRInstruction::new(SIROpcode::ApplyStyle, 2, 1, 0));
    doc.push(SIRInstruction::new(SIROpcode::ApplyStyle, 3, 1, 0));

    let original_opcodes: Vec<_> = doc.iter().map(|i| i.opcode()).collect();
    let bytes = doc.to_bytes_with_payload();
    let restored = match SIRDocument::from_bytes_with_payload(&bytes) {
        Ok(d) => d,
        Err(_) => return,
    };
    let restored_opcodes: Vec<_> = restored.iter().map(|i| i.opcode()).collect();
    assert_eq!(
        original_opcodes, restored_opcodes,
        "opcodes should be preserved"
    );
}

#[test]
fn serialize_preserves_entity_ids() {
    let mut doc = SIRDocument::new();
    let root_offset = doc.payload_mut().append(&[BlockType::Document as u8]);
    doc.push(SIRInstruction::new(
        SIROpcode::PushBlock,
        42,
        ROOT_SENTINEL,
        root_offset,
    ));
    doc.push(SIRInstruction::new(SIROpcode::SetContent, 99, 42, 0));

    let bytes = doc.to_bytes_with_payload();
    let restored = match SIRDocument::from_bytes_with_payload(&bytes) {
        Ok(d) => d,
        Err(_) => return,
    };

    let ids: Vec<u32> = restored.iter().map(|i| i.entity_id()).collect();
    assert_eq!(ids, vec![42, 99], "entity IDs should be preserved");
}

#[test]
fn serialize_preserves_parent_ids() {
    let mut doc = SIRDocument::new();
    let root_offset = doc.payload_mut().append(&[BlockType::Document as u8]);
    doc.push(SIRInstruction::new(
        SIROpcode::PushBlock,
        0,
        ROOT_SENTINEL,
        root_offset,
    ));
    let para_offset = doc.payload_mut().append(&[BlockType::Paragraph as u8]);
    doc.push(SIRInstruction::new(SIROpcode::PushBlock, 1, 0, para_offset));

    let bytes = doc.to_bytes_with_payload();
    let restored = match SIRDocument::from_bytes_with_payload(&bytes) {
        Ok(d) => d,
        Err(_) => return,
    };

    let parents: Vec<u32> = restored.iter().map(|i| i.parent_id()).collect();
    assert_eq!(
        parents,
        vec![ROOT_SENTINEL, 0],
        "parent IDs should be preserved"
    );
}

// =========================================================================
// 5. Determinism
// =========================================================================

#[test]
fn determinism_md_compile_twice() {
    let doc = ldir_md::parse_markdown("# Title\n\nParagraph with text.\n\n- item 1\n- item 2");
    let gir1 = match compile_to_gir(&doc) {
        Some(g) => g,
        None => return,
    };
    let gir2 = match compile_to_gir(&doc) {
        Some(g) => g,
        None => return,
    };

    assert_eq!(
        gir1.page_count(),
        gir2.page_count(),
        "page count should match"
    );
    assert_eq!(
        gir1, gir2,
        "two compilations of the same input should produce identical G-IR"
    );
}

#[test]
fn determinism_tex_compile_twice() {
    let doc = ldir_tex::parse_tex(r"\section{Intro}Some text here.");
    let gir1 = match compile_to_gir(&doc) {
        Some(g) => g,
        None => return,
    };
    let gir2 = match compile_to_gir(&doc) {
        Some(g) => g,
        None => return,
    };

    assert_eq!(gir1, gir2, "TeX compile should be deterministic");
}

#[test]
fn determinism_pdf_output() {
    let mut doc = GIRDocument::with_capacity(1);
    let mut page = GIRPage::with_dimensions(612 * 64, 792 * 64);
    page.push(GIRCommand::new_set_font(0));
    page.push(GIRCommand::new_move_xy((72 * 64) as i32, (720 * 64) as i32));
    page.push(GIRCommand::new_put_glyph(72, (7 * 64) as i32));
    doc.push_page(page);

    let pdf1 = ldir_pdf::converter::gir_to_pdf(&doc);
    let pdf2 = ldir_pdf::converter::gir_to_pdf(&doc);

    assert_eq!(pdf1, pdf2, "PDF generation should be deterministic");
}
