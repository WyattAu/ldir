//! G-IR to PDF converter with TrueType font embedding.
//!
//! Converts a G-IR document into a valid PDF with embedded fonts,
//! ToUnicode CMaps, and proper glyph rendering.
//!
//! Supports multiple font variants (Regular, Bold, Italic, Mono) via
//! font ID convention in G-IR SetFont commands.

use ldir_ir::gir::{GIRDocument, GIROpcode, ImageFormat};

use crate::conformance::PdfConformance;
use crate::font::FontFace;
use crate::writer::{PdfDocumentBuilder, PdfImage, PdfImageFormat};

/// Options for PDF generation.
pub struct PdfOptions {
    /// Document title.
    pub title: Option<String>,
    /// Document author.
    pub author: Option<String>,
    /// Document subject.
    pub subject: Option<String>,
    /// Application that created the PDF.
    pub creator: Option<String>,
    /// Header left template.
    pub header_left: Option<String>,
    /// Header right template.
    pub header_right: Option<String>,
    /// Footer left template.
    pub footer_left: Option<String>,
    /// Footer right template.
    pub footer_right: Option<String>,
    /// Suppress header on the first page.
    pub suppress_first_header: bool,
    /// PDF/A conformance level.
    pub conformance: PdfConformance,
}

impl Default for PdfOptions {
    fn default() -> Self {
        Self {
            title: None,
            author: None,
            subject: None,
            creator: Some("ldir".to_string()),
            header_left: None,
            header_right: None,
            footer_left: None,
            footer_right: None,
            suppress_first_header: true,
            conformance: PdfConformance::default(),
        }
    }
}

fn expand_template(template: &str, page: usize, pages: usize, options: &PdfOptions) -> String {
    let mut result = template.to_string();
    result = result.replace("{page}", &page.to_string());
    result = result.replace("{pages}", &pages.to_string());
    result = result.replace("{title}", options.title.as_deref().unwrap_or(""));
    result = result.replace("{author}", options.author.as_deref().unwrap_or(""));
    result = result.replace("{date}", &date_now_str());
    result
}

fn date_now_str() -> String {
    let epoch = std::time::SystemTime::UNIX_EPOCH;
    std::time::SystemTime::now()
        .duration_since(epoch)
        .map(|d| {
            let secs = d.as_secs();
            let days = secs / 86400;
            let z = (days + 719528) as i64;
            let era = if z >= 0 {
                z / 146097
            } else {
                (z - 146096) / 146097
            };
            let doe = z - era * 146097;
            let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
            let y = yoe + era * 400;
            let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
            let mp = (5 * doy + 2) / 153;
            let d = doy - (153 * mp + 2) / 5 + 1;
            let m = if mp < 10 { mp + 3 } else { mp - 9 };
            let y = if m <= 2 { y + 1 } else { y };
            format!("{:04}-{:02}-{:02}", y, m, d)
        })
        .unwrap_or_else(|_| "1970-01-01".to_string())
}

/// Convert a G-IR document to PDF bytes using a viewer-resident font (fallback).
pub fn gir_to_pdf(gir_doc: &GIRDocument) -> Vec<u8> {
    gir_to_pdf_with_fonts(gir_doc, &[])
}

/// Convert a G-IR document to PDF bytes with optional embedded fonts.
///
/// When `fonts` is non-empty, the fonts are embedded in the PDF as
/// TrueType fonts (Type0 + CIDFontType2) with ToUnicode CMaps.
/// The first font is the default (font_id=0); additional fonts are
/// selected by G-IR SetFont commands.
///
/// When `fonts` is empty, falls back to viewer-resident Helvetica.
pub fn gir_to_pdf_with_fonts(gir_doc: &GIRDocument, fonts: &[FontFace]) -> Vec<u8> {
    gir_to_pdf_with_fonts_and_options(gir_doc, fonts, &PdfOptions::default())
}

/// Convert a G-IR document to PDF bytes with embedded fonts and metadata options.
pub fn gir_to_pdf_with_fonts_and_options(
    gir_doc: &GIRDocument,
    fonts: &[FontFace],
    options: &PdfOptions,
) -> Vec<u8> {
    let mut builder = PdfDocumentBuilder::new();

    if let Some(ref title) = options.title {
        builder.set_title(title);
    }
    if let Some(ref author) = options.author {
        builder.set_author(author);
    }
    if let Some(ref subject) = options.subject {
        builder.set_subject(subject);
    }
    if let Some(ref creator) = options.creator {
        builder.set_creator(creator);
    }
    builder.set_conformance(options.conformance);

    let has_embedded_fonts = !fonts.is_empty();
    for face in fonts {
        builder.add_font((*face).clone(), 12.0);
    }
    if has_embedded_fonts {
        builder.set_active_font(0);
    }

    let page_count = gir_doc.page_count();
    for (page_idx, page) in gir_doc.iter().enumerate() {
        let width = page.width as f64 / 64.0;
        let height = page.height as f64 / 64.0;
        builder.add_page(width, height);

        let mut cursor_x: f64 = 0.0;
        let mut cursor_y: f64 = 0.0;

        for cmd in page.iter() {
            match cmd.opcode() {
                GIROpcode::SetFont => {
                    let font_id = cmd.arg(0).unwrap_or(0) as usize;
                    if has_embedded_fonts && font_id < fonts.len() {
                        builder.set_active_font(font_id);
                    }
                }
                GIROpcode::MoveXY => {
                    cursor_x = cmd.arg(0).unwrap_or(0) as f64 / 64.0;
                    cursor_y = cmd.arg(1).unwrap_or(0) as f64 / 64.0;
                }
                GIROpcode::PutGlyph => {
                    let glyph_id = cmd.arg(0).unwrap_or(0);
                    let advance = cmd.arg(1).unwrap_or(0) as f64 / 64.0;

                    if has_embedded_fonts {
                        builder.write_glyph(cursor_x, cursor_y, glyph_id as u32, advance);
                    } else {
                        let ch = char::from_u32(glyph_id as u32).unwrap_or('?');
                        builder.write_text(cursor_x, cursor_y, &ch.to_string());
                    }
                    cursor_x += advance;
                }
                GIROpcode::DrawRule => {
                    let raw_x = cmd.arg(0).unwrap_or(0);
                    let raw_y = cmd.arg(1).unwrap_or(0);
                    let raw_w = cmd.arg(2).unwrap_or(0);
                    let raw_h = cmd.arg(3).unwrap_or(0);

                    // Image sentinel: args[0] = -1 signals an image placeholder
                    if raw_x == -1 {
                        let image_index = raw_y as usize;
                        let w = raw_w as f64 / 64.0;
                        let h = raw_h as f64 / 64.0;
                        builder.add_image(cursor_x, cursor_y, w, h, image_index);
                    } else {
                        let x = raw_x as f64 / 64.0;
                        let y = raw_y as f64 / 64.0;
                        let w = raw_w as f64 / 64.0;
                        let h = raw_h as f64 / 64.0;
                        builder.draw_rect(x, y, w, h);
                    }
                }
                GIROpcode::PushStack | GIROpcode::PopStack | GIROpcode::AttachMetadata => {}
            }
        }

        for link in &page.links {
            builder.add_link(
                link.x,
                link.y,
                link.width,
                link.height,
                link.url.clone(),
                link.destination_page,
            );
        }

        // Header/footer support
        let has_header = options.header_left.is_some() || options.header_right.is_some();
        let has_custom_footer = options.footer_left.is_some() || options.footer_right.is_some();

        if has_header && !(options.suppress_first_header && page_idx == 0) {
            if let Some(ref tmpl) = options.header_left {
                let text = expand_template(tmpl, page_idx + 1, page_count, options);
                if has_embedded_fonts {
                    builder.set_active_font(0);
                }
                builder.write_text(72.0, height - 36.0, &text);
            }
            if let Some(ref tmpl) = options.header_right {
                let text = expand_template(tmpl, page_idx + 1, page_count, options);
                if has_embedded_fonts {
                    builder.set_active_font(0);
                }
                let approx_width = text.len() as f64 * 3.5;
                builder.write_text(width - 72.0 - approx_width, height - 36.0, &text);
            }
        }

        if has_custom_footer {
            if let Some(ref tmpl) = options.footer_left {
                let text = expand_template(tmpl, page_idx + 1, page_count, options);
                if has_embedded_fonts {
                    builder.set_active_font(0);
                }
                builder.write_text(72.0, 24.0, &text);
            }
            if let Some(ref tmpl) = options.footer_right {
                let text = expand_template(tmpl, page_idx + 1, page_count, options);
                if has_embedded_fonts {
                    builder.set_active_font(0);
                }
                let approx_width = text.len() as f64 * 3.5;
                builder.write_text(width - 72.0 - approx_width, 24.0, &text);
            }
        }

        if page_count > 1 && !has_custom_footer {
            let page_num = page_idx + 1;
            let text = page_num.to_string();
            let font_size = 10.0;
            let text_width = text.len() as f64 * font_size * 0.5;
            let center_x = width / 2.0 - text_width / 2.0;
            let bottom_y = 30.0;

            if has_embedded_fonts {
                builder.set_active_font(0);
            }
            builder.write_text(center_x, bottom_y, &text);
        }
    }

    // Pass images to the builder
    if !gir_doc.images().is_empty() {
        let pdf_images: Vec<PdfImage> = gir_doc
            .images()
            .iter()
            .map(|img| PdfImage {
                data: img.data.clone(),
                format: match img.format {
                    ImageFormat::Png => PdfImageFormat::Png,
                    ImageFormat::Jpeg => PdfImageFormat::Jpeg,
                },
                alt_text: None,
            })
            .collect();
        builder.set_images(pdf_images);
    }

    builder.build()
}

/// Backward-compatible single-font API.
pub fn gir_to_pdf_with_font(gir_doc: &GIRDocument, font_data: Option<&[u8]>) -> Vec<u8> {
    match font_data {
        Some(data) => match FontFace::from_bytes(data) {
            Ok(face) => gir_to_pdf_with_fonts(gir_doc, &[face]),
            Err(_) => gir_to_pdf(gir_doc),
        },
        None => gir_to_pdf(gir_doc),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldir_ir::gir::{GIRCommand, GIRDocument, GIRPage};

    fn get_font_data() -> Option<Vec<u8>> {
        let paths = [
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        ];
        for path in &paths {
            if let Ok(data) = std::fs::read(path) {
                return Some(data);
            }
        }
        None
    }

    #[test]
    fn test_gir_to_pdf_fallback() {
        let mut doc = GIRDocument::with_capacity(1);
        let mut page = GIRPage::with_dimensions(612 * 64, 792 * 64);
        page.push(GIRCommand::new_set_font(0));
        page.push(GIRCommand::new_move_xy((72 * 64) as i32, (720 * 64) as i32));
        page.push(GIRCommand::new_put_glyph(72, (7 * 64) as i32)); // 'H'
        doc.push_page(page);

        let bytes = gir_to_pdf(&doc);
        assert!(bytes.starts_with(b"%PDF"));
    }

    #[test]
    fn test_gir_to_pdf_with_embedded_font() {
        let Some(font_data) = get_font_data() else {
            return;
        };

        let mut doc = GIRDocument::with_capacity(1);
        let mut page = GIRPage::with_dimensions(612 * 64, 792 * 64);
        page.push(GIRCommand::new_set_font(0));
        page.push(GIRCommand::new_move_xy((72 * 64) as i32, (720 * 64) as i32));
        // 'H' glyph in DejaVu Sans = glyph 36
        page.push(GIRCommand::new_put_glyph(36, (7 * 64) as i32));
        doc.push_page(page);

        let bytes = gir_to_pdf_with_font(&doc, Some(&font_data));
        assert!(bytes.starts_with(b"%PDF"));

        // Should contain embedded font markers
        let pdf_str = String::from_utf8_lossy(&bytes);
        assert!(pdf_str.contains("/FontDescriptor"));
        assert!(pdf_str.contains("/FontFile2"));
        assert!(pdf_str.contains("/ToUnicode"));
    }

    #[test]
    fn test_font_subset_from_gir() {
        use crate::font::FontSubset;

        let mut doc = GIRDocument::with_capacity(1);
        let mut page = GIRPage::with_dimensions(612 * 64, 792 * 64);
        page.push(GIRCommand::new_put_glyph(36, 500)); // 'H'
        page.push(GIRCommand::new_put_glyph(45, 500)); // 'e'
        page.push(GIRCommand::new_put_glyph(56, 500)); // 'l'
        doc.push_page(page);

        let subset = FontSubset::from_gir(&doc);
        assert!(subset.contains(36));
        assert!(subset.contains(45));
        assert!(subset.contains(56));
        assert!(subset.contains(0)); // .notdef always
        assert_eq!(subset.len(), 3);
    }

    #[test]
    fn test_gir_to_pdf_with_multiple_fonts() {
        let Some(font_data) = get_font_data() else {
            return;
        };
        let face = FontFace::from_bytes(&font_data).unwrap();

        let mut doc = GIRDocument::with_capacity(1);
        let mut page = GIRPage::with_dimensions(612 * 64, 792 * 64);
        // Use font 0 (Regular)
        page.push(GIRCommand::new_set_font(0));
        page.push(GIRCommand::new_move_xy((72 * 64) as i32, (720 * 64) as i32));
        page.push(GIRCommand::new_put_glyph(36, (7 * 64) as i32));
        // Switch to font 1 (Bold)
        page.push(GIRCommand::new_set_font(1));
        page.push(GIRCommand::new_move_xy(
            (100 * 64) as i32,
            (720 * 64) as i32,
        ));
        page.push(GIRCommand::new_put_glyph(36, (8 * 64) as i32));
        doc.push_page(page);

        // Embed the same font twice (Regular and Bold would be different in practice)
        let bytes = gir_to_pdf_with_fonts(&doc, &[face.clone(), face]);
        assert!(bytes.starts_with(b"%PDF"));

        let pdf_str = String::from_utf8_lossy(&bytes);
        // Should have two Type0 fonts
        let type0_count =
            pdf_str.matches("/Type /Font").count() + pdf_str.matches("/Type /Font ").count();
        assert!(
            type0_count >= 2,
            "should have at least 2 fonts, got {}",
            type0_count
        );
    }

    #[test]
    fn test_pdf_options_metadata() {
        let mut doc = GIRDocument::with_capacity(1);
        let mut page = GIRPage::with_dimensions(612 * 64, 792 * 64);
        page.push(GIRCommand::new_move_xy((72 * 64) as i32, (720 * 64) as i32));
        page.push(GIRCommand::new_put_glyph(72, (7 * 64) as i32));
        doc.push_page(page);

        let mut options = PdfOptions::default();
        options.title = Some("My Title".to_string());
        options.author = Some("Test Author".to_string());

        let bytes = gir_to_pdf_with_fonts_and_options(&doc, &[], &options);
        let pdf_str = String::from_utf8_lossy(&bytes);
        assert!(pdf_str.contains("My Title"));
        assert!(pdf_str.contains("Test Author"));
    }

    #[test]
    fn test_pdf_options_default_creator() {
        let mut doc = GIRDocument::with_capacity(1);
        let mut page = GIRPage::with_dimensions(612 * 64, 792 * 64);
        page.push(GIRCommand::new_move_xy((72 * 64) as i32, (720 * 64) as i32));
        page.push(GIRCommand::new_put_glyph(72, (7 * 64) as i32));
        doc.push_page(page);

        let bytes = gir_to_pdf_with_fonts_and_options(&doc, &[], &PdfOptions::default());
        let pdf_str = String::from_utf8_lossy(&bytes);
        assert!(pdf_str.contains("ldir"));
    }

    #[test]
    fn test_expand_template() {
        let mut opts = PdfOptions::default();
        opts.title = Some("Test Doc".to_string());
        opts.author = Some("Author".to_string());

        let result = expand_template("{title} - {page}/{pages}", 1, 5, &opts);
        assert_eq!(result, "Test Doc - 1/5");

        let result = expand_template("{author} {date}", 1, 1, &opts);
        assert!(result.starts_with("Author "));
        assert!(result.contains("-"));
    }

    #[test]
    fn test_date_now_str() {
        let date = date_now_str();
        assert!(date.len() == 10, "date should be YYYY-MM-DD, got: {}", date);
        assert_eq!(&date[4..5], "-");
        assert_eq!(&date[7..8], "-");
    }

    #[test]
    fn test_pdf_header_footer_options() {
        let mut doc = GIRDocument::with_capacity(1);
        let mut page = GIRPage::with_dimensions(612 * 64, 792 * 64);
        page.push(GIRCommand::new_move_xy((72 * 64) as i32, (720 * 64) as i32));
        page.push(GIRCommand::new_put_glyph(72, (7 * 64) as i32));
        doc.push_page(page);

        let mut options = PdfOptions::default();
        options.header_left = Some("{title}".to_string());
        options.footer_left = Some("{page}/{pages}".to_string());
        options.title = Some("My Paper".to_string());

        let bytes = gir_to_pdf_with_fonts_and_options(&doc, &[], &options);
        assert!(bytes.starts_with(b"%PDF"));
    }
}
