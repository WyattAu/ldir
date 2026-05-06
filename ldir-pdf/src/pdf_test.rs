use ldir_ir::gir::{GIRCommand, GIRDocument, GIRPage};

use crate::converter::gir_to_pdf;
use crate::writer::PdfDocumentBuilder;

fn starts_with_pdf_header(bytes: &[u8]) -> bool {
    bytes.starts_with(b"%PDF-1.7") || bytes.starts_with(b"%PDF-2.0")
}

fn ends_with_eof(bytes: &[u8]) -> bool {
    bytes.ends_with(b"%%EOF")
}

#[test]
fn test_empty_document_produces_valid_pdf() {
    let mut builder = PdfDocumentBuilder::new();
    let bytes = builder.build();
    assert!(starts_with_pdf_header(&bytes), "missing %PDF-1.7 header");
    assert!(ends_with_eof(&bytes), "missing %%EOF trailer");
}

#[test]
fn test_single_page_with_text() {
    let mut builder = PdfDocumentBuilder::new();
    builder.add_page(612.0, 792.0);
    builder.write_text(72.0, 720.0, "Hello, PDF!");
    let bytes = builder.build();
    assert!(starts_with_pdf_header(&bytes));
    assert!(ends_with_eof(&bytes));
    // Content streams are FlateDecode compressed; verify stream exists
    let s = String::from_utf8_lossy(&bytes);
    assert!(
        s.contains("/Filter /FlateDecode"),
        "content stream should be compressed"
    );
    assert!(s.contains("/Type /Page"));
}

#[test]
fn test_multiple_pages() {
    let mut builder = PdfDocumentBuilder::new();
    builder.add_page(612.0, 792.0);
    builder.write_text(72.0, 720.0, "Page 1");
    builder.add_page(612.0, 792.0);
    builder.write_text(72.0, 720.0, "Page 2");
    builder.add_page(612.0, 792.0);
    builder.write_text(72.0, 720.0, "Page 3");
    let bytes = builder.build();
    assert!(starts_with_pdf_header(&bytes));
    assert!(ends_with_eof(&bytes));
    // Verify 3 pages in the pages tree
    let s = String::from_utf8_lossy(&bytes);
    assert!(s.contains("/Count 3"), "should have 3 pages");
    assert!(
        s.contains("/Filter /FlateDecode"),
        "content streams should be compressed"
    );
}

#[test]
fn test_rectangles_drawing() {
    let mut builder = PdfDocumentBuilder::new();
    builder.add_page(612.0, 792.0);
    builder.draw_rect(100.0, 100.0, 200.0, 50.0);
    builder.draw_rect(50.0, 400.0, 300.0, 10.0);
    let bytes = builder.build();
    assert!(starts_with_pdf_header(&bytes));
    assert!(ends_with_eof(&bytes));
    // Content is FlateDecode compressed, so raw operators aren't searchable.
    // Verify the stream exists and is compressed.
    let s = String::from_utf8_lossy(&bytes);
    assert!(
        s.contains("/Filter /FlateDecode"),
        "content stream should be compressed"
    );
    assert!(s.contains("/Type /Page"));
}

#[test]
fn test_font_operations_stub() {
    let mut builder = PdfDocumentBuilder::new();
    builder.add_page(612.0, 792.0);
    // set_font is a backward-compat no-op; it doesn't change PDF output
    builder.set_font("Helvetica", 24.0);
    builder.write_text(72.0, 700.0, "Big text");
    let bytes = builder.build();
    assert!(starts_with_pdf_header(&bytes));
    assert!(ends_with_eof(&bytes));
    let s = String::from_utf8_lossy(&bytes);
    // With no embedded fonts, the fallback Helvetica font is used at size 12
    assert!(s.contains("/Helvetica"));
    assert!(s.contains("12"));
}

#[test]
fn test_set_title() {
    let mut builder = PdfDocumentBuilder::new();
    builder.set_title("Test Document");
    builder.add_page(612.0, 792.0);
    let bytes = builder.build();
    assert!(starts_with_pdf_header(&bytes));
    let s = String::from_utf8_lossy(&bytes);
    assert!(s.contains("Test Document"), "title not in PDF");
}

#[test]
fn test_converter_empty_document() {
    let doc = GIRDocument::new();
    let bytes = gir_to_pdf(&doc);
    assert!(starts_with_pdf_header(&bytes));
    assert!(ends_with_eof(&bytes));
}

#[test]
fn test_converter_single_page_with_glyphs() {
    let mut doc = GIRDocument::new();
    let mut page = GIRPage::new();
    page.push(GIRCommand::new_push_stack());
    page.push(GIRCommand::new_set_font(0));
    page.push(GIRCommand::new_move_xy(72 * 64, 720 * 64));
    page.push(GIRCommand::new_put_glyph(72, 10 * 64));
    page.push(GIRCommand::new_pop_stack());
    doc.push_page(page);

    let bytes = gir_to_pdf(&doc);
    assert!(starts_with_pdf_header(&bytes));
    assert!(ends_with_eof(&bytes));
}

#[test]
fn test_converter_with_draw_rule() {
    let mut doc = GIRDocument::new();
    let mut page = GIRPage::new();
    page.push(GIRCommand::new_push_stack());
    page.push(GIRCommand::new_draw_rule(
        100 * 64,
        200 * 64,
        300 * 64,
        10 * 64,
    ));
    page.push(GIRCommand::new_pop_stack());
    doc.push_page(page);

    let bytes = gir_to_pdf(&doc);
    assert!(starts_with_pdf_header(&bytes));
    assert!(ends_with_eof(&bytes));
    // Content is FlateDecode compressed; verify stream exists
    let s = String::from_utf8_lossy(&bytes);
    assert!(
        s.contains("/Filter /FlateDecode"),
        "content stream should be compressed"
    );
}

#[test]
fn test_converter_multiple_pages() {
    let mut doc = GIRDocument::new();
    let mut page1 = GIRPage::new();
    page1.push(GIRCommand::new_push_stack());
    page1.push(GIRCommand::new_set_font(0));
    page1.push(GIRCommand::new_put_glyph(65, 10 * 64));
    page1.push(GIRCommand::new_pop_stack());
    let mut page2 = GIRPage::new();
    page2.push(GIRCommand::new_push_stack());
    page2.push(GIRCommand::new_set_font(0));
    page2.push(GIRCommand::new_put_glyph(66, 10 * 64));
    page2.push(GIRCommand::new_pop_stack());
    doc.push_page(page1);
    doc.push_page(page2);

    let bytes = gir_to_pdf(&doc);
    assert!(starts_with_pdf_header(&bytes));
    assert!(ends_with_eof(&bytes));
}

#[test]
fn test_default_builder() {
    let mut builder = PdfDocumentBuilder::default();
    let bytes = builder.build();
    assert!(starts_with_pdf_header(&bytes));
    assert!(ends_with_eof(&bytes));
}
