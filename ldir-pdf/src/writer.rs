//! PDF document builder with TrueType font embedding support.
//!
//! Generates a valid PDF 1.4+ document with embedded TrueType fonts
//! using Type0 (composite) + CIDFontType2 + ToUnicode CMap structure.

#![allow(dead_code)]

use pdf_writer::types::{AnnotationType, CidFontType, FontFlags, SystemInfo, UnicodeCmap};
use pdf_writer::{Content, Filter, Name, Pdf, Rect, Ref, Str, TextStr};

use crate::font::FontFace;

/// An image to be embedded in the PDF.
pub struct PdfImage {
    pub data: Vec<u8>,
    pub format: PdfImageFormat,
}

/// PDF image format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdfImageFormat {
    Png,
    Jpeg,
}

/// Compress data with FlateDecode (zlib/deflate).
fn compress(data: &[u8]) -> Vec<u8> {
    use flate2::Compression;
    use flate2::write::ZlibEncoder;
    use std::io::Write;

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(6));
    let _ = encoder.write_all(data);
    encoder.finish().unwrap_or_else(|_| data.to_vec())
}

fn jpeg_info(data: &[u8]) -> Option<(u32, u32, u8)> {
    let mut i = 2;
    while i + 9 < data.len() {
        if data[i] == 0xFF {
            let marker = data[i + 1];
            if marker == 0xC0 || marker == 0xC2 {
                let h = u16::from_be_bytes([data[i + 5], data[i + 6]]) as u32;
                let w = u16::from_be_bytes([data[i + 7], data[i + 8]]) as u32;
                let components = data[i + 9];
                return Some((w, h, components));
            }
            if i + 3 < data.len() {
                let len = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
                i += 2 + len;
            } else {
                break;
            }
        } else {
            i += 1;
        }
    }
    None
}

fn jpeg_dims(data: &[u8]) -> Option<(u32, u32)> {
    jpeg_info(data).map(|(w, h, _)| (w, h))
}

struct PageState {
    width: f64,
    height: f64,
    content: Content,
    links: Vec<(f64, f64, f64, f64, String, Option<usize>)>,
    images: Vec<(f64, f64, f64, f64, usize)>,
}

/// Describes an embedded font in the PDF.
struct EmbeddedFont {
    /// Font face with parsed metrics.
    face: FontFace,
    /// PDF resource name (e.g. b"F1").
    resource_name: Vec<u8>,
    /// Font size in points.
    size: f32,
}

/// PDF document builder supporting multiple embedded fonts.
pub struct PdfDocumentBuilder {
    title: String,
    author: String,
    subject: String,
    creator: String,
    pages: Vec<PageState>,
    current_page: usize,
    fonts: Vec<EmbeddedFont>,
    /// Index into `fonts` for the currently active font.
    current_font: usize,
    /// Glyph IDs to track for each font (collected during write, merged at build).
    pending_glyphs: Vec<Vec<u32>>,
    /// Image data table for embedding in PDF.
    images: Vec<PdfImage>,
}

impl PdfDocumentBuilder {
    /// Create a new PDF builder with no pages or fonts.
    pub fn new() -> Self {
        Self {
            title: String::new(),
            author: String::new(),
            subject: String::new(),
            creator: String::new(),
            pages: Vec::new(),
            current_page: 0,
            fonts: Vec::new(),
            current_font: 0,
            pending_glyphs: Vec::new(),
            images: Vec::new(),
        }
    }

    /// Set the document title.
    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    /// Set the document author.
    pub fn set_author(&mut self, author: &str) {
        self.author = author.to_string();
    }

    /// Set the document subject.
    pub fn set_subject(&mut self, subject: &str) {
        self.subject = subject.to_string();
    }

    /// Set the document creator (application that generated the PDF).
    pub fn set_creator(&mut self, creator: &str) {
        self.creator = creator.to_string();
    }

    /// Set the image data table for embedding in the PDF.
    pub fn set_images(&mut self, images: Vec<PdfImage>) {
        self.images = images;
    }

    /// Add a new page and return its index.
    pub fn add_page(&mut self, width: f64, height: f64) -> usize {
        let index = self.pages.len();
        self.pages.push(PageState {
            width,
            height,
            content: Content::new(),
            links: Vec::new(),
            images: Vec::new(),
        });
        self.current_page = index;
        index
    }

    /// Embed a TrueType font for use in the document.
    ///
    /// Returns the font index (0-based) to use with [`set_active_font`].
    pub fn add_font(&mut self, face: FontFace, size: f32) -> usize {
        let resource_name = format!("F{}", self.fonts.len() + 1);
        let index = self.fonts.len();
        self.fonts.push(EmbeddedFont {
            face,
            resource_name: resource_name.into_bytes(),
            size,
        });
        self.pending_glyphs.push(Vec::new());
        index
    }

    /// Set the active font by index (from [`add_font`]).
    pub fn set_active_font(&mut self, font_index: usize) {
        self.current_font = font_index;
    }

    /// Set the active font by name and size (backward compat, no-op without embedded font).
    pub fn set_font(&mut self, _name: &str, _size: f64) {
        // This is kept for backward compatibility. When using the new
        // font embedding path, use add_font() + set_active_font() instead.
    }

    /// Write text at the given position using the active font.
    pub fn write_text(&mut self, x: f64, y: f64, text: &str) {
        if self.pages.is_empty() {
            self.add_page(612.0, 792.0);
        }

        let content = &mut self.pages[self.current_page].content;

        if let Some(font) = self.fonts.get(self.current_font) {
            // Record glyph IDs for ToUnicode CMap
            for ch in text.chars() {
                if let Some(gid) = font.face.glyph_id_for_char(ch) {
                    self.pending_glyphs[self.current_font].push(gid);
                }
            }

            let resource_name = &font.resource_name;
            content
                .begin_text()
                .set_font(Name(resource_name), font.size)
                .set_text_matrix([1.0, 0.0, 0.0, 1.0, x as f32, y as f32])
                .show(Str(text.as_bytes()))
                .end_text();
        } else {
            // Fallback: no font embedded, use /Helvetica (viewer-resident)
            content
                .begin_text()
                .set_font(Name(b"F1"), 12.0)
                .set_text_matrix([1.0, 0.0, 0.0, 1.0, x as f32, y as f32])
                .show(Str(text.as_bytes()))
                .end_text();
        }
    }

    /// Write a glyph at the given position with explicit advance.
    ///
    /// This is used when the compiler has already shaped text via HarfBuzz
    /// and emits individual glyph IDs.
    pub fn write_glyph(&mut self, x: f64, y: f64, glyph_id: u32, advance: f64) {
        if self.pages.is_empty() {
            self.add_page(612.0, 792.0);
        }

        let content = &mut self.pages[self.current_page].content;

        if let Some(font) = self.fonts.get(self.current_font) {
            // Track used glyph
            self.pending_glyphs[self.current_font].push(glyph_id);

            // Convert glyph ID to bytes for the content stream.
            // For TrueType fonts with ToUnicode, we can emit the glyph
            // as a character code (which the ToUnicode CMap will map back).
            // For simplicity, we use the glyph ID as a 2-byte character code.
            let hi = ((glyph_id >> 8) & 0xFF) as u8;
            let lo = (glyph_id & 0xFF) as u8;

            let resource_name = &font.resource_name;
            content
                .begin_text()
                .set_font(Name(resource_name), font.size)
                .set_text_matrix([1.0, 0.0, 0.0, 1.0, x as f32, y as f32])
                .show(Str(&[hi, lo]))
                .end_text();

            // Advance cursor
            let next_x = x + advance;
            content.set_text_matrix([1.0, 0.0, 0.0, 1.0, next_x as f32, y as f32]);
        } else {
            // Fallback: no font
            content
                .begin_text()
                .set_font(Name(b"F1"), 12.0)
                .set_text_matrix([1.0, 0.0, 0.0, 1.0, x as f32, y as f32])
                .show(Str(&[glyph_id as u8]))
                .end_text();
        }
    }

    /// Draw a filled rectangle.
    pub fn draw_rect(&mut self, x: f64, y: f64, w: f64, h: f64) {
        if self.pages.is_empty() {
            self.add_page(612.0, 792.0);
        }
        let content = &mut self.pages[self.current_page].content;
        content
            .rect(x as f32, y as f32, w as f32, h as f32)
            .fill_nonzero();
    }

    /// Add a clickable hyperlink annotation on the current page.
    ///
    /// The rectangle `(x, y, x+w, y+h)` defines the clickable area.
    /// `url` is the target URI. If `destination_page` is `Some(page_idx)`,
    /// the link is an internal destination link instead of a URI link.
    pub fn add_link(
        &mut self,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        url: String,
        destination_page: Option<usize>,
    ) {
        if self.pages.is_empty() {
            self.add_page(612.0, 792.0);
        }
        self.pages[self.current_page]
            .links
            .push((x, y, w, h, url, destination_page));
    }

    /// Add an image to be embedded on the current page.
    ///
    /// The image is drawn at `(x, y)` with dimensions `(w, h)`.
    /// `image_index` references the image data table in the G-IR document.
    pub fn add_image(&mut self, x: f64, y: f64, w: f64, h: f64, image_index: usize) {
        if self.pages.is_empty() {
            self.add_page(612.0, 792.0);
        }
        self.pages[self.current_page]
            .images
            .push((x, y, w, h, image_index));
    }

    /// Build the final PDF bytes.
    ///
    /// This allocates PDF object IDs for:
    /// - Catalog (1), Pages (2)
    /// - Per-page: Page object, Content stream
    /// - Per-font: Type0 font, CIDFont, FontDescriptor, FontFile2 stream, ToUnicode CMap stream
    /// - Optional: DocumentInfo
    pub fn build(&mut self) -> Vec<u8> {
        let mut pdf = Pdf::new();

        let catalog_id = Ref::new(1);
        let pages_id = Ref::new(2);
        let mut next_id: i32 = 3;

        pdf.catalog(catalog_id).pages(pages_id);

        let page_count = self.pages.len() as i32;

        // Allocate page and content stream IDs
        let page_ids: Vec<Ref> = (0..page_count)
            .map(|_| {
                let id = Ref::new(next_id);
                next_id += 1;
                id
            })
            .collect();
        let content_ids: Vec<Ref> = (0..page_count)
            .map(|_| {
                let id = Ref::new(next_id);
                next_id += 1;
                id
            })
            .collect();

        // Build pages tree
        {
            let mut pages = pdf.pages(pages_id);
            pages.kids(page_ids.iter().copied());
            pages.count(page_count);
        }

        // Collect used glyphs per font (deduplicated)
        let font_used: Vec<std::collections::HashSet<u32>> = self
            .pending_glyphs
            .iter()
            .map(|gids| {
                let mut set = std::collections::HashSet::new();
                set.insert(0); // .notdef
                for &gid in gids {
                    set.insert(gid);
                }
                set
            })
            .collect();

        // Embed each font
        // Per font we need: Type0, CIDFont, FontDescriptor, FontFile2, ToUnicode
        // = 5 objects per font
        let font_ids: Vec<FontPdfIds> = self
            .fonts
            .iter()
            .zip(font_used.iter())
            .map(|(font, used_gids)| {
                let type0_id = Ref::new(next_id);
                next_id += 1;
                let cid_id = Ref::new(next_id);
                next_id += 1;
                let descriptor_id = Ref::new(next_id);
                next_id += 1;
                let fontfile_id = Ref::new(next_id);
                next_id += 1;
                let tounicode_id = Ref::new(next_id);
                next_id += 1;

                embed_truetype_font(
                    &mut pdf,
                    type0_id,
                    cid_id,
                    descriptor_id,
                    fontfile_id,
                    tounicode_id,
                    &font.face,
                    used_gids,
                    &font.resource_name,
                );

                FontPdfIds {
                    type0_id,
                    resource_name: font.resource_name.clone(),
                }
            })
            .collect();

        // Fallback font (Helvetica) if no fonts embedded
        let fallback_font_id = if font_ids.is_empty() {
            let id = Ref::new(next_id);
            next_id += 1;
            {
                let mut font = pdf.type1_font(id);
                font.base_font(Name(b"Helvetica"));
            }
            Some((id, b"F1".to_vec()))
        } else {
            None
        };

        // Allocate per-page link annotation IDs
        let page_link_ids: Vec<Vec<Ref>> = self
            .pages
            .iter()
            .map(|p| {
                (0..p.links.len())
                    .map(|_| {
                        let id = Ref::new(next_id);
                        next_id += 1;
                        id
                    })
                    .collect()
            })
            .collect();

        // Allocate image XObject IDs
        let image_xobject_ids: Vec<Ref> = self
            .images
            .iter()
            .map(|_| {
                let id = Ref::new(next_id);
                next_id += 1;
                id
            })
            .collect();

        // Write page content streams (and inject image Do operations)
        let content_data: Vec<Vec<u8>> = self
            .pages
            .iter_mut()
            .map(|p| {
                let mut content = std::mem::replace(&mut p.content, Content::new());

                // Inject image Do operations for images on this page
                for &(x, y, w, h, image_index) in &p.images {
                    if image_index < self.images.len() {
                        let resource_name = format!("Im{}", image_index);
                        content.save_state();
                        content.transform([w as f32, 0.0, 0.0, h as f32, x as f32, (y - h) as f32]);
                        content.x_object(Name(resource_name.as_bytes()));
                        content.restore_state();
                    }
                }

                content.finish()
            })
            .collect();

        // Write page objects
        for i in 0..self.pages.len() {
            let page_state = &self.pages[i];
            let mut page = pdf.page(page_ids[i]);
            page.parent(pages_id);
            page.media_box(Rect::new(
                0.0,
                0.0,
                page_state.width as f32,
                page_state.height as f32,
            ));
            page.contents(content_ids[i]);

            {
                let mut resources = page.resources();
                let mut fonts_dict = resources.fonts();
                for font_ref in &font_ids {
                    fonts_dict.pair(Name(&font_ref.resource_name), font_ref.type0_id);
                }
                if let Some((ref id, ref name)) = fallback_font_id {
                    fonts_dict.pair(Name(name), *id);
                }
            }

            // Add XObject resources for images on this page
            let page_image_indices: Vec<usize> = page_state
                .images
                .iter()
                .map(|(_, _, _, _, idx)| *idx)
                .filter(|idx| *idx < self.images.len())
                .collect();
            if !page_image_indices.is_empty() {
                let mut resources = page.resources();
                let mut xobjects = resources.x_objects();
                for &idx in &page_image_indices {
                    let resource_name = format!("Im{}", idx);
                    xobjects.pair(Name(resource_name.as_bytes()), image_xobject_ids[idx]);
                }
            }

            if !page_link_ids[i].is_empty() {
                page.annotations(page_link_ids[i].iter().copied());
            }
        }

        // Write link annotation objects
        for (i, page_state) in self.pages.iter().enumerate() {
            for (j, (x, y, w, h, url, dest_page)) in page_state.links.iter().enumerate() {
                let annot_id = page_link_ids[i][j];
                let mut annot = pdf.annotation(annot_id);
                annot.subtype(AnnotationType::Link);
                annot.rect(Rect::new(
                    *x as f32,
                    *y as f32,
                    (*x + *w) as f32,
                    (*y + *h) as f32,
                ));
                if let Some(page_idx) = dest_page {
                    if *page_idx < page_ids.len() {
                        annot.pair(Name(b"Dest"), page_ids[*page_idx]);
                    }
                } else {
                    annot
                        .action()
                        .action_type(pdf_writer::types::ActionType::Uri)
                        .uri(Str(url.as_bytes()));
                }
            }
        }

        // Write content streams (compressed with FlateDecode)
        for i in 0..self.pages.len() {
            let compressed = compress(&content_data[i]);
            pdf.stream(content_ids[i], &compressed)
                .filter(Filter::FlateDecode);
        }

        // Write image XObjects
        for (img_idx, img) in self.images.iter().enumerate() {
            let xobj_id = image_xobject_ids[img_idx];
            match img.format {
                PdfImageFormat::Png => {
                    let decoded = match crate::image::decode_png(&img.data) {
                        Ok(d) => d,
                        Err(_) => continue,
                    };
                    let color_space_name = match decoded.color_space {
                        crate::image::ColorSpace::RGB => Name(b"DeviceRGB"),
                        crate::image::ColorSpace::Gray => Name(b"DeviceGray"),
                    };
                    let compressed = compress(&decoded.data);
                    pdf.stream(xobj_id, &compressed)
                        .filter(Filter::FlateDecode)
                        .pair(Name(b"Type"), Name(b"XObject"))
                        .pair(Name(b"Subtype"), Name(b"Image"))
                        .pair(Name(b"Width"), decoded.width as i32)
                        .pair(Name(b"Height"), decoded.height as i32)
                        .pair(Name(b"ColorSpace"), color_space_name)
                        .pair(Name(b"BitsPerComponent"), decoded.bits_per_component as i32);
                }
                PdfImageFormat::Jpeg => {
                    let (w, h, components) = jpeg_info(&img.data).unwrap_or((100, 100, 3));
                    let color_space_name = if components == 1 {
                        Name(b"DeviceGray")
                    } else {
                        Name(b"DeviceRGB")
                    };
                    pdf.stream(xobj_id, &img.data)
                        .filter(Filter::DctDecode)
                        .pair(Name(b"Type"), Name(b"XObject"))
                        .pair(Name(b"Subtype"), Name(b"Image"))
                        .pair(Name(b"Width"), w as i32)
                        .pair(Name(b"Height"), h as i32)
                        .pair(Name(b"ColorSpace"), color_space_name)
                        .pair(Name(b"BitsPerComponent"), 8);
                }
            }
        }

        // Document info — always emit so PDF has metadata
        {
            let info_id = Ref::new(next_id);
            let mut info = pdf.document_info(info_id);
            if !self.title.is_empty() {
                info.title(TextStr(&self.title));
            }
            if !self.author.is_empty() {
                info.author(TextStr(&self.author));
            }
            if !self.subject.is_empty() {
                info.subject(TextStr(&self.subject));
            }
            if !self.creator.is_empty() {
                info.creator(TextStr(&self.creator));
            }
        }

        pdf.finish()
    }
}

/// PDF object IDs for an embedded font.
struct FontPdfIds {
    type0_id: Ref,
    resource_name: Vec<u8>,
}

/// Embed a TrueType font into the PDF.
///
/// Creates the following object structure:
/// - `type0_id`: Type0 (composite) font → Encoding: Identity-H
/// - `cid_id`: CIDFont (CIDFontType2 for TrueType)
/// - `descriptor_id`: FontDescriptor with metrics
/// - `fontfile_id`: Stream containing raw TrueType font data
/// - `tounicode_id`: Stream containing ToUnicode CMap
#[allow(clippy::too_many_arguments)]
fn embed_truetype_font(
    pdf: &mut Pdf,
    type0_id: Ref,
    cid_id: Ref,
    descriptor_id: Ref,
    fontfile_id: Ref,
    tounicode_id: Ref,
    face: &FontFace,
    used_glyphs: &std::collections::HashSet<u32>,
    _resource_name: &[u8],
) {
    let info = face.pdf_info();
    let upem = info.units_per_em as f32;
    let scale = 1000.0 / upem;

    // --- Type0 font (composite) ---
    {
        let mut type0 = pdf.type0_font(type0_id);
        let ps_name: Vec<u8> = info.postscript_name.bytes().collect();
        type0.base_font(Name(&ps_name));
        type0.encoding_predefined(Name(b"Identity-H"));
        type0.descendant_font(cid_id);
        type0.to_unicode(tounicode_id);
    }

    // --- CIDFont (Type2 for TrueType) ---
    {
        let mut cid = pdf.cid_font(cid_id);
        let ps_name: Vec<u8> = info.postscript_name.bytes().collect();
        cid.subtype(CidFontType::Type2);
        cid.base_font(Name(&ps_name));
        cid.system_info(SystemInfo {
            registry: Str(b"Adobe"),
            ordering: Str(b"Identity"),
            supplement: 0,
        });
        cid.font_descriptor(descriptor_id);

        // Default width
        cid.default_width(1000.0 / upem);

        // W array: widths for used glyphs
        let mut sorted_glyphs: Vec<u16> = used_glyphs
            .iter()
            .map(|&g| g as u16)
            .filter(|&g| g > 0) // skip .notdef
            .collect();
        sorted_glyphs.sort_unstable();

        if !sorted_glyphs.is_empty() {
            let mut widths = cid.widths();
            // Write individual widths
            let widths_f32: Vec<f32> = sorted_glyphs
                .iter()
                .map(|&gid| {
                    let g = ttf_parser::GlyphId(gid);
                    face.face()
                        .glyph_hor_advance(g)
                        .map(|a| a as f32 * scale)
                        .unwrap_or(500.0)
                })
                .collect();

            // Group consecutive glyph IDs into ranges for efficiency
            let mut start = sorted_glyphs[0];
            let mut run_start_idx = 0;
            for (i, &gid) in sorted_glyphs.iter().enumerate().skip(1) {
                if gid != sorted_glyphs[i - 1] + 1 {
                    // Emit the run
                    let run_widths: Vec<f32> = widths_f32[run_start_idx..i].to_vec();
                    widths.consecutive(start, run_widths);
                    start = gid;
                    run_start_idx = i;
                }
            }
            // Emit final run
            let run_widths: Vec<f32> = widths_f32[run_start_idx..sorted_glyphs.len()].to_vec();
            widths.consecutive(start, run_widths);
        }

        // CIDToGIDMap: Identity
        cid.cid_to_gid_map_predefined(Name(b"Identity"));
    }

    // --- FontDescriptor ---
    {
        let mut desc = pdf.font_descriptor(descriptor_id);
        let ps_name: Vec<u8> = info.postscript_name.bytes().collect();
        desc.name(Name(&ps_name));

        // Flags
        let mut flags = FontFlags::empty();
        if info.is_monospace {
            flags |= FontFlags::FIXED_PITCH;
        }
        flags |= FontFlags::NON_SYMBOLIC;
        desc.flags(flags);

        // Bounding box (scaled to 1000/em)
        desc.bbox(Rect::new(
            info.bbox.x_min as f32 * scale,
            info.bbox.y_min as f32 * scale,
            info.bbox.x_max as f32 * scale,
            info.bbox.y_max as f32 * scale,
        ));

        desc.italic_angle(info.italic_angle);
        desc.ascent(info.ascent as f32 * scale);
        desc.descent(info.descent as f32 * scale);
        desc.cap_height(info.cap_height * scale);
        desc.x_height(info.x_height * scale);

        // StemV estimate (rough heuristic)
        let stem_v = (info.ascent as f32 * scale * 0.05).round().max(50.0);
        desc.stem_v(stem_v);

        // Embed the TrueType font file
        desc.font_file2(fontfile_id);
    }

    // --- FontFile2 (TrueType font stream, FlateDecode compressed) ---
    {
        let subsetted = crate::font::subset_font(face.raw_bytes(), used_glyphs);
        let compressed = compress(&subsetted);
        pdf.stream(fontfile_id, &compressed)
            .filter(Filter::FlateDecode)
            .pair(Name(b"Length1"), subsetted.len() as i32);
    }

    // --- ToUnicode CMap ---
    {
        let system_info = SystemInfo {
            registry: Str(b"Adobe"),
            ordering: Str(b"Identity"),
            supplement: 0,
        };

        let cmap_name: Vec<u8> = format!("{}-ToUnicode", info.postscript_name).into_bytes();

        let mut cmap = UnicodeCmap::new(Name(&cmap_name), system_info);

        // Build glyph_id → unicode mapping from the face's cmap
        let mut sorted_gids: Vec<u32> = used_glyphs.iter().copied().collect();
        sorted_gids.sort_unstable();
        let mut seen = std::collections::HashSet::new();
        for gid in sorted_gids {
            if seen.insert(gid)
                && let Some(ch) = face.glyph_to_unicode(gid)
            {
                cmap.pair(gid as u16, ch);
            }
        }

        let cmap_data = cmap.finish();
        let compressed = compress(&cmap_data);
        pdf.stream(tounicode_id, &compressed)
            .filter(Filter::FlateDecode);
    }
}

impl Default for PdfDocumentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
fn escape_pdf_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '(' => out.push_str("\\("),
            ')' => out.push_str("\\)"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_font_face() -> Option<FontFace> {
        let paths = [
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        ];
        for path in &paths {
            if let Ok(data) = std::fs::read(path) {
                if let Ok(face) = FontFace::from_bytes(&data) {
                    return Some(face);
                }
            }
        }
        None
    }

    #[test]
    fn test_empty_pdf_builds() {
        let mut builder = PdfDocumentBuilder::new();
        builder.add_page(612.0, 792.0);
        let bytes = builder.build();
        assert!(!bytes.is_empty());
        // Should start with %PDF
        assert!(bytes.starts_with(b"%PDF"));
    }

    #[test]
    fn test_pdf_with_text_no_font() {
        let mut builder = PdfDocumentBuilder::new();
        builder.add_page(612.0, 792.0);
        builder.write_text(72.0, 720.0, "Hello");
        let bytes = builder.build();
        assert!(!bytes.is_empty());
        assert!(bytes.starts_with(b"%PDF"));
    }

    #[test]
    fn test_pdf_with_embedded_font() {
        let Some(face) = get_font_face() else { return };
        let mut builder = PdfDocumentBuilder::new();
        let font_idx = builder.add_font(face, 12.0);
        builder.set_active_font(font_idx);
        builder.add_page(612.0, 792.0);
        builder.write_text(72.0, 720.0, "Hello World");
        let bytes = builder.build();
        assert!(!bytes.is_empty());
        assert!(bytes.starts_with(b"%PDF"));
        // Should contain FontDescriptor (embedded font marker)
        let pdf_str = String::from_utf8_lossy(&bytes);
        assert!(pdf_str.contains("/FontDescriptor"));
        assert!(pdf_str.contains("/FontFile2"));
    }

    #[test]
    fn test_pdf_with_glyphs() {
        let Some(face) = get_font_face() else { return };
        let mut builder = PdfDocumentBuilder::new();
        let font_idx = builder.add_font(face, 12.0);
        builder.set_active_font(font_idx);
        builder.add_page(612.0, 792.0);
        // Write individual glyphs (as the converter would)
        builder.write_glyph(72.0, 720.0, 36, 7.0); // 'A' glyph in DejaVu
        builder.write_glyph(79.0, 720.0, 56, 6.0); // 'B' glyph
        let bytes = builder.build();
        assert!(!bytes.is_empty());
        assert!(bytes.starts_with(b"%PDF"));
        // Should have ToUnicode CMap
        let pdf_str = String::from_utf8_lossy(&bytes);
        assert!(pdf_str.contains("/ToUnicode"));
    }

    #[test]
    fn test_pdf_with_rect() {
        let mut builder = PdfDocumentBuilder::new();
        builder.add_page(612.0, 792.0);
        builder.draw_rect(72.0, 100.0, 468.0, 1.0);
        let bytes = builder.build();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_pdf_with_title() {
        let mut builder = PdfDocumentBuilder::new();
        builder.set_title("Test Document");
        builder.add_page(612.0, 792.0);
        let bytes = builder.build();
        let pdf_str = String::from_utf8_lossy(&bytes);
        assert!(pdf_str.contains("Test Document"));
    }

    #[test]
    fn test_pdf_with_author() {
        let mut builder = PdfDocumentBuilder::new();
        builder.set_author("Test Author");
        builder.add_page(612.0, 792.0);
        let bytes = builder.build();
        let pdf_str = String::from_utf8_lossy(&bytes);
        assert!(pdf_str.contains("Test Author"));
    }

    #[test]
    fn test_pdf_with_subject() {
        let mut builder = PdfDocumentBuilder::new();
        builder.set_subject("Test Subject");
        builder.add_page(612.0, 792.0);
        let bytes = builder.build();
        let pdf_str = String::from_utf8_lossy(&bytes);
        assert!(pdf_str.contains("Test Subject"));
    }

    #[test]
    fn test_pdf_with_creator() {
        let mut builder = PdfDocumentBuilder::new();
        builder.set_creator("ldir");
        builder.add_page(612.0, 792.0);
        let bytes = builder.build();
        let pdf_str = String::from_utf8_lossy(&bytes);
        assert!(pdf_str.contains("ldir"));
    }

    #[test]
    fn test_pdf_with_all_metadata() {
        let mut builder = PdfDocumentBuilder::new();
        builder.set_title("Full Meta");
        builder.set_author("Jane Doe");
        builder.set_subject("Testing");
        builder.set_creator("ldir");
        builder.add_page(612.0, 792.0);
        let bytes = builder.build();
        let pdf_str = String::from_utf8_lossy(&bytes);
        assert!(pdf_str.contains("Full Meta"));
        assert!(pdf_str.contains("Jane Doe"));
        assert!(pdf_str.contains("Testing"));
        assert!(pdf_str.contains("ldir"));
    }

    #[test]
    fn test_pdf_image_support() {
        let mut builder = PdfDocumentBuilder::new();
        builder.add_page(612.0, 792.0);
        builder.add_image(72.0, 700.0, 100.0, 50.0, 0);
        let bytes = builder.build();
        assert!(bytes.starts_with(b"%PDF"));
    }
}
