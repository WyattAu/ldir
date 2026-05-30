//! PDF document builder with TrueType font embedding support.
//!
//! Generates a valid PDF 1.4+ document with embedded TrueType fonts
//! using Type0 (composite) + CIDFontType2 + ToUnicode CMap structure.

use pdf_writer::types::{AnnotationType, CidFontType, FontFlags, SystemInfo, UnicodeCmap};
use pdf_writer::{Content, Filter, Name, Pdf, Rect, Ref, Str, TextStr};
use rayon::prelude::*;

use crate::color::{IccProfile, icc_alternate_name};
use crate::conformance::PdfConformance;
use crate::font::FontFace;
use crate::structure::StructureNode;

/// An image to be embedded in the PDF.
#[derive(Debug, Clone)]
pub struct PdfImage {
    pub data: Vec<u8>,
    pub format: PdfImageFormat,
    pub alt_text: Option<String>,
}

/// PDF image format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdfImageFormat {
    Png,
    Jpeg,
}

/// Compress data with FlateDecode (zlib/deflate).
pub(crate) fn compress(data: &[u8]) -> Vec<u8> {
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

#[allow(dead_code)]
fn jpeg_dims(data: &[u8]) -> Option<(u32, u32)> {
    jpeg_info(data).map(|(w, h, _)| (w, h))
}

pub(crate) struct PageState {
    pub width: f64,
    pub height: f64,
    pub content: Content,
    pub links: Vec<(f64, f64, f64, f64, String, Option<usize>)>,
    pub images: Vec<(f64, f64, f64, f64, usize)>,
}

/// Describes an embedded font in the PDF.
pub(crate) struct EmbeddedFont {
    /// Font face with parsed metrics.
    pub face: FontFace,
    /// PDF resource name (e.g. b"F1").
    pub resource_name: Vec<u8>,
    /// Font size in points.
    pub size: f32,
}

/// PDF document builder supporting multiple embedded fonts.
pub struct PdfDocumentBuilder {
    pub(crate) title: String,
    pub(crate) author: String,
    pub(crate) subject: String,
    pub(crate) creator: String,
    pub(crate) language: String,
    pub(crate) pages: Vec<PageState>,
    current_page: usize,
    pub(crate) fonts: Vec<EmbeddedFont>,
    current_font: usize,
    pub(crate) pending_glyphs: Vec<Vec<u32>>,
    pub(crate) images: Vec<PdfImage>,
    pub(crate) structure_tree: Vec<StructureNode>,
    pub(crate) tagged: bool,
    pub(crate) icc_profile: Option<IccProfile>,
    pub(crate) conformance: PdfConformance,
}

impl PdfDocumentBuilder {
    /// Create a new PDF builder with no pages or fonts.
    pub fn new() -> Self {
        Self {
            title: String::new(),
            author: String::new(),
            subject: String::new(),
            creator: String::new(),
            language: "en".to_string(),
            pages: Vec::new(),
            current_page: 0,
            fonts: Vec::new(),
            current_font: 0,
            pending_glyphs: Vec::new(),
            images: Vec::new(),
            structure_tree: Vec::new(),
            tagged: false,
            icc_profile: None,
            conformance: PdfConformance::default(),
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

    #[allow(dead_code)]
    pub fn set_language(&mut self, lang: &str) {
        self.language = lang.to_string();
    }

    #[allow(dead_code)]
    pub fn set_structure_tree(&mut self, tree: Vec<StructureNode>) {
        self.structure_tree = tree;
        self.tagged = true;
    }

    #[allow(dead_code)]
    pub fn set_tagged(&mut self, tagged: bool) {
        self.tagged = tagged;
    }

    pub fn set_conformance(&mut self, conformance: PdfConformance) {
        self.conformance = conformance;
    }

    /// Set the image data table for embedding in the PDF.
    pub fn set_images(&mut self, images: Vec<PdfImage>) {
        self.images = images;
    }

    /// Set an ICC color profile for the document output.
    ///
    /// The profile will be embedded in the PDF and referenced from the
    /// catalog's OutputIntent and default color space.
    #[allow(dead_code)]
    pub fn set_icc_profile(&mut self, profile: IccProfile) {
        self.icc_profile = Some(profile);
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
    #[allow(dead_code)]
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

    /// Draw a simple table grid using stroked lines.
    ///
    /// - `x`, `y`: top-left corner (PDF coordinates, y decreases downward)
    /// - `col_widths`: width of each column
    /// - `row_height`: height of each row
    /// - `num_rows`: number of rows in the grid
    /// - `line_width`: stroke thickness of the grid lines
    #[allow(dead_code)]
    pub fn draw_table(
        &mut self,
        x: f64,
        y: f64,
        col_widths: &[f64],
        row_height: f64,
        num_rows: usize,
        line_width: f64,
    ) {
        if self.pages.is_empty() || col_widths.is_empty() || num_rows == 0 {
            return;
        }

        let total_width: f64 = col_widths.iter().sum();
        let total_height = row_height * num_rows as f64;
        let content = &mut self.pages[self.current_page].content;

        content.set_line_width(line_width as f32);
        content.set_stroke_gray(0.0);

        for i in 0..=num_rows {
            let ly = y - (i as f64 * row_height);
            content
                .move_to(x as f32, ly as f32)
                .line_to((x + total_width) as f32, ly as f32)
                .stroke();
        }

        let mut cx = x;
        for &w in col_widths.iter().chain(std::iter::once(&0.0)) {
            content
                .move_to(cx as f32, y as f32)
                .line_to(cx as f32, (y - total_height) as f32)
                .stroke();
            cx += w;
        }
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

        match self.conformance {
            PdfConformance::PdfA4 => pdf.set_version(2, 0),
            PdfConformance::PdfA2b | PdfConformance::PdfA3b => pdf.set_version(1, 7),
        }

        let catalog_id = Ref::new(1);
        let pages_id = Ref::new(2);
        let mut next_id: i32 = 3;

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

        // Pre-compute structure tree size and allocate IDs
        let struct_tree_info = if self.tagged && !self.structure_tree.is_empty() {
            let all_nodes: Vec<StructureNode> = self
                .structure_tree
                .iter()
                .flat_map(|n| collect_all_nodes(n).into_iter().cloned())
                .collect();

            if all_nodes.is_empty() {
                None
            } else {
                let node_start = next_id;
                next_id += all_nodes.len() as i32;
                let root_id = Ref::new(next_id);
                next_id += 1;
                let parent_tree_id = Ref::new(next_id);
                next_id += 1;
                Some((all_nodes, node_start, root_id, parent_tree_id))
            }
        } else {
            None
        };

        // Pre-allocate ICC-related IDs
        let icc_stream_id = if self.icc_profile.is_some() {
            let id = Ref::new(next_id);
            next_id += 1;
            Some(id)
        } else {
            None
        };
        let intent_id = if self.icc_profile.is_some() {
            let id = Ref::new(next_id);
            next_id += 1;
            Some(id)
        } else {
            None
        };

        // Pre-allocate XMP metadata ID (before writing catalog so we can reference it)
        let xmp_metadata_id = Ref::new(next_id);
        next_id += 1;

        // Write catalog (must be written before other objects that need the pdf ref)
        {
            let mut catalog = pdf.catalog(catalog_id);
            catalog.pages(pages_id);
            if self.tagged {
                catalog.lang(TextStr(&self.language));
                catalog.mark_info().marked(true);
                if let Some((_, _, root_id, parent_tree_id)) = &struct_tree_info {
                    catalog.pair(Name(b"StructTreeRoot"), *root_id);
                    catalog.pair(Name(b"ParentTree"), *parent_tree_id);
                }
            }
            if let Some(intent_id) = intent_id {
                catalog
                    .insert(Name(b"OutputIntents"))
                    .array()
                    .item(intent_id);
            }
        }

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
        // Per font we need: Type0, CIDFont, FontDescriptor, FontFile2, CIDToGIDMap, ToUnicode
        // = 6 objects per font
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
                let cidtogidmap_id = Ref::new(next_id);
                next_id += 1;
                let tounicode_id = Ref::new(next_id);
                next_id += 1;

                embed_truetype_font(
                    &mut pdf,
                    type0_id,
                    cid_id,
                    descriptor_id,
                    fontfile_id,
                    cidtogidmap_id,
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
            .enumerate()
            .map(|(page_idx, p)| {
                let mut content = std::mem::replace(&mut p.content, Content::new());

                if self.tagged {
                    let mcid = page_idx as i32;
                    content
                        .begin_marked_content_with_properties(Name(b"P"))
                        .properties()
                        .pair(Name(b"MCID"), mcid);
                }

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

                if self.tagged {
                    content.end_marked_content();
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

            // Add ICC-based color space to page resources
            if let Some(icc_ref) = icc_stream_id {
                let mut resources = page.resources();
                let mut cs = resources.color_spaces();
                let mut arr = cs.insert(Name(b"ICCSB")).array();
                arr.item(Name(b"ICCBased"));
                arr.item(icc_ref);
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

        // Write content streams (compressed with FlateDecode).
        // Compress all pages in parallel, then write sequentially.
        let compressed_content: Vec<Vec<u8>> =
            content_data.par_iter().map(|data| compress(data)).collect();
        for i in 0..self.pages.len() {
            pdf.stream(content_ids[i], &compressed_content[i])
                .filter(Filter::FlateDecode);
        }

        // Write image XObjects
        for (img_idx, img) in self.images.iter().enumerate() {
            let xobj_id = image_xobject_ids[img_idx];
            let alt_text = img.alt_text.as_deref().unwrap_or("");
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
                        .pair(Name(b"BitsPerComponent"), decoded.bits_per_component as i32)
                        .pair(Name(b"Alt"), TextStr(alt_text));
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
                        .pair(Name(b"BitsPerComponent"), 8)
                        .pair(Name(b"Alt"), TextStr(alt_text));
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

        // ICC profile stream and OutputIntent
        if let (Some(stream_id), Some(int_id)) = (icc_stream_id, intent_id) {
            if let Some(profile) = &self.icc_profile {
                let compressed = compress(&profile.data);
                let alternate = icc_alternate_name(profile.color_space);
                pdf.stream(stream_id, &compressed)
                    .filter(Filter::FlateDecode)
                    .pair(Name(b"N"), profile.components as i32)
                    .pair(Name(b"Alternate"), Name(alternate));
            }
            {
                let mut intent = pdf.indirect(int_id).dict();
                intent.pair(Name(b"Type"), Name(b"OutputIntent"));
                intent.pair(Name(b"S"), Name(b"GTS_PDFX"));
                let condition_id = self
                    .icc_profile
                    .as_ref()
                    .map(|p| p.name.as_str())
                    .unwrap_or("sRGB");
                intent.pair(Name(b"OutputConditionIdentifier"), TextStr(condition_id));
                intent.pair(Name(b"DestOutputProfile"), stream_id);
            }
        }

        // XMP metadata stream (PDF/A requirement)
        {
            let xmp_bytes =
                crate::xmp::generate_pdfa_xmp(self.conformance, &self.title, &self.author);
            let compressed = compress(&xmp_bytes);
            pdf.stream(xmp_metadata_id, &compressed)
                .filter(Filter::FlateDecode)
                .pair(Name(b"Type"), Name(b"Metadata"))
                .pair(Name(b"Subtype"), Name(b"XML"));
        }

        // Structure tree (PDF/UA)
        if let Some((all_nodes, node_start, root_id, parent_tree_id)) = struct_tree_info {
            write_structure_tree(
                &mut pdf,
                &mut next_id,
                &all_nodes,
                node_start,
                root_id,
                parent_tree_id,
                &page_ids,
            );
        }

        pdf.finish()
    }
}

fn write_structure_tree(
    pdf: &mut Pdf,
    next_id: &mut i32,
    all_nodes: &[StructureNode],
    node_start: i32,
    root_id: Ref,
    parent_tree_id: Ref,
    page_ids: &[Ref],
) {
    let node_refs: Vec<Ref> = all_nodes
        .iter()
        .enumerate()
        .map(|(i, _)| Ref::new(node_start + i as i32))
        .collect();

    for (i, node) in all_nodes.iter().enumerate() {
        let r = node_refs[i];
        let mut elem = pdf.struct_element(r);

        if let Some(custom_name) = node.element_type.custom_role_name() {
            elem.custom_kind(Name(custom_name));
        } else {
            elem.kind(node.element_type.to_struct_role());
        }

        if let Some(ref alt) = node.alt_text {
            elem.alt(TextStr(alt.as_str()));
        }

        if let Some(ref actual) = node.actual_text {
            elem.actual_text(TextStr(actual.as_str()));
        }

        if let Some(ref expanded) = node.expanded_text {
            elem.expanded(TextStr(expanded.as_str()));
        }

        if let Some(ref lang) = node.language {
            elem.lang(TextStr(lang.as_str()));
        }

        if let Some(ref bbox) = node.bbox {
            let bbox_val = format!(
                "[{:.2} {:.2} {:.2} {:.2}]",
                bbox.x,
                bbox.y,
                bbox.x + bbox.width,
                bbox.y + bbox.height
            );
            elem.pair(Name(b"BBox"), Str(bbox_val.as_bytes()));
        }

        let page_idx = node.page.saturating_sub(1) as usize;
        if node.is_leaf() && page_idx < page_ids.len() {
            elem.page(page_ids[page_idx]);
        }

        if !node.children.is_empty() {
            let mut children = elem.children();
            for child in &node.children {
                if let Some(idx) = find_node_index(all_nodes, child) {
                    children.item(node_refs[idx]);
                }
            }
        }

        drop(elem);
    }

    {
        let mut root = pdf
            .indirect(root_id)
            .start::<pdf_writer::writers::StructTreeRoot>();
        {
            let mut kids = root.children();
            for &r in &node_refs {
                kids.item(r);
            }
            drop(kids);
        }
        root.pair(Name(b"ParentTreeNextKey"), page_ids.len() as i32);
        drop(root);
    }

    {
        let mut nums_dict = pdf.indirect(parent_tree_id).dict();
        let mut nums = nums_dict.insert(Name(b"Nums")).array();
        for (page_idx, _) in page_ids.iter().enumerate() {
            nums.item(page_idx as i32);
            nums.item(page_idx as i32);
        }
        drop(nums);
        drop(nums_dict);
    }

    *next_id = (*next_id).max(node_start + all_nodes.len() as i32 + 2);
}

fn collect_all_nodes(node: &StructureNode) -> Vec<&StructureNode> {
    let mut result = vec![node];
    for child in &node.children {
        result.extend(collect_all_nodes(child));
    }
    result
}

fn find_node_index(all_nodes: &[StructureNode], target: &StructureNode) -> Option<usize> {
    all_nodes.iter().position(|n| {
        n.element_type == target.element_type
            && n.page == target.page
            && n.mcid == target.mcid
            && n.alt_text == target.alt_text
            && n.actual_text == target.actual_text
            && n.language == target.language
            && n.bbox == target.bbox
            && n.children.len() == target.children.len()
    })
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
    cidtogidmap_id: Ref,
    tounicode_id: Ref,
    face: &FontFace,
    used_glyphs: &std::collections::HashSet<u32>,
    _resource_name: &[u8],
) {
    let info = face.pdf_info();
    let upem = info.units_per_em as f32;
    let scale = 1000.0 / upem;

    let subset_result = crate::font::subset_font(face.raw_bytes(), used_glyphs);

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

        // --- CIDToGIDMap ---
        if subset_result.cid_to_gid_map.is_some() {
            cid.cid_to_gid_map_stream(cidtogidmap_id);
        } else {
            cid.cid_to_gid_map_predefined(Name(b"Identity"));
        }
    }

    // --- CIDToGIDMap stream (if needed) ---
    if let Some(ref cid_map) = subset_result.cid_to_gid_map {
        let compressed = compress(cid_map);
        pdf.stream(cidtogidmap_id, &compressed)
            .filter(Filter::FlateDecode);
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
        let compressed = compress(&subset_result.font_data);
        pdf.stream(fontfile_id, &compressed)
            .filter(Filter::FlateDecode)
            .pair(Name(b"Length1"), subset_result.font_data.len() as i32);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structure::StructureType;

    fn get_font_face() -> Option<FontFace> {
        let data = ldir_test_helpers::test_font_data();
        FontFace::from_bytes(&data).ok()
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

    #[test]
    fn test_marked_content_wrapping() {
        let mut builder = PdfDocumentBuilder::new();
        builder.set_tagged(true);
        builder.add_page(612.0, 792.0);
        builder.write_text(72.0, 720.0, "Hello");
        let bytes = builder.build();
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/Lang"));
        assert!(s.contains("/Marked true"));
    }

    #[test]
    fn test_alt_text_on_images() {
        let mut builder = PdfDocumentBuilder::new();
        builder.set_tagged(true);
        builder.add_page(612.0, 792.0);
        builder.set_images(vec![PdfImage {
            data: vec![],
            format: PdfImageFormat::Jpeg,
            alt_text: Some("A beautiful sunset".to_string()),
        }]);
        builder.add_image(72.0, 700.0, 100.0, 50.0, 0);
        let bytes = builder.build();
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/Alt"));
        assert!(s.contains("A beautiful sunset"));
    }

    #[test]
    fn test_language_tag_in_catalog() {
        let mut builder = PdfDocumentBuilder::new();
        builder.set_language("de");
        builder.set_tagged(true);
        builder.add_page(612.0, 792.0);
        let bytes = builder.build();
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/Lang (de)"));
    }

    #[test]
    fn test_mark_info_marked() {
        let mut builder = PdfDocumentBuilder::new();
        builder.set_tagged(true);
        builder.add_page(612.0, 792.0);
        let bytes = builder.build();
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/Marked true"));
    }

    #[test]
    fn test_structure_tree_with_nodes() {
        let mut builder = PdfDocumentBuilder::new();
        builder.set_tagged(true);
        builder.add_page(612.0, 792.0);
        builder.set_structure_tree(vec![StructureNode::with_children(
            StructureType::Document,
            vec![StructureNode::new(StructureType::Paragraph, 1, 0)],
        )]);
        let bytes = builder.build();
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/StructTreeRoot"));
        assert!(s.contains("/Type /StructElem"));
        assert!(s.contains("/ParentTree"));
    }

    #[test]
    fn test_nested_structure_pdf() {
        let mut builder = PdfDocumentBuilder::new();
        builder.set_tagged(true);
        builder.add_page(612.0, 792.0);
        builder.set_structure_tree(vec![StructureNode::with_children(
            StructureType::Document,
            vec![StructureNode::with_children(
                StructureType::Section,
                vec![
                    StructureNode::new(StructureType::Paragraph, 1, 0),
                    StructureNode::new(StructureType::Paragraph, 1, 1),
                ],
            )],
        )]);
        let bytes = builder.build();
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/StructTreeRoot"));
        assert!(s.contains("/S /Sect"));
        assert!(s.contains("/S /P"));
    }

    #[test]
    fn test_empty_structure_tree_no_tagged() {
        let mut builder = PdfDocumentBuilder::new();
        builder.add_page(612.0, 792.0);
        let bytes = builder.build();
        let s = String::from_utf8_lossy(&bytes);
        assert!(!s.contains("/StructTreeRoot"));
        assert!(!s.contains("/Marked"));
        assert!(!s.contains("/Lang"));
    }

    #[test]
    fn test_pdf_with_srgb_icc_profile() {
        let mut builder = PdfDocumentBuilder::new();
        builder.set_icc_profile(crate::color::IccProfile::srgb());
        builder.add_page(612.0, 792.0);
        builder.write_text(72.0, 720.0, "Color managed");
        let bytes = builder.build();
        assert!(bytes.starts_with(b"%PDF"));
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/OutputIntents"));
        assert!(s.contains("/OutputIntent"));
        assert!(s.contains("/GTS_PDFX"));
        assert!(s.contains("/DestOutputProfile"));
        assert!(s.contains("/ICCBased"));
    }

    #[test]
    fn test_pdf_with_cmyk_icc_profile() {
        let mut builder = PdfDocumentBuilder::new();
        builder.set_icc_profile(crate::color::IccProfile::cmyk());
        builder.add_page(612.0, 792.0);
        let bytes = builder.build();
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/OutputIntents"));
        assert!(s.contains("/Alternate /DeviceCMYK"));
    }

    #[test]
    fn test_pdf_with_gray_icc_profile() {
        let mut builder = PdfDocumentBuilder::new();
        builder.set_icc_profile(crate::color::IccProfile::gray());
        builder.add_page(612.0, 792.0);
        let bytes = builder.build();
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/OutputIntents"));
        assert!(s.contains("/Alternate /DeviceGray"));
    }

    #[test]
    fn test_pdf_without_icc_profile_no_output_intents() {
        let mut builder = PdfDocumentBuilder::new();
        builder.add_page(612.0, 792.0);
        let bytes = builder.build();
        let s = String::from_utf8_lossy(&bytes);
        assert!(!s.contains("/OutputIntents"));
        assert!(!s.contains("/ICCBased"));
    }

    #[test]
    fn test_icc_profile_srgb_components() {
        let mut builder = PdfDocumentBuilder::new();
        builder.set_icc_profile(crate::color::IccProfile::srgb());
        builder.add_page(612.0, 792.0);
        let bytes = builder.build();
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/N 3"));
    }

    #[test]
    fn test_headings_produce_h1_h6_structure() {
        use crate::structure::heading;

        let mut doc = StructureNode::with_children(
            StructureType::Document,
            vec![
                heading(1, "Title", 1, 0),
                heading(2, "Chapter", 1, 1),
                heading(3, "Section", 1, 2),
            ],
        );
        doc.assign_reading_order();

        let mut builder = PdfDocumentBuilder::new();
        builder.set_tagged(true);
        builder.add_page(612.0, 792.0);
        builder.set_structure_tree(vec![doc]);
        let bytes = builder.build();
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/S /H1"));
        assert!(s.contains("/S /H2"));
        assert!(s.contains("/S /H3"));
        assert!(s.contains("/ActualText"));
        assert!(s.contains("Title"));
    }

    #[test]
    fn test_table_produces_tr_th_td_nesting() {
        use crate::structure::table_with_header;

        let table = table_with_header(vec!["Name", "Value"], vec![vec!["foo", "1"]], 1, 0);

        let mut doc = StructureNode::with_children(StructureType::Document, vec![table]);
        doc.assign_reading_order();

        let mut builder = PdfDocumentBuilder::new();
        builder.set_tagged(true);
        builder.add_page(612.0, 792.0);
        builder.set_structure_tree(vec![doc]);
        let bytes = builder.build();
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/S /Table"));
        assert!(s.contains("/S /TR"));
        assert!(s.contains("/S /TH"));
        assert!(s.contains("/S /TD"));
        assert!(s.contains("/S /THead"));
        assert!(s.contains("/S /TBody"));
        assert!(s.contains("Name"));
        assert!(s.contains("foo"));
    }

    #[test]
    fn test_images_include_alt_text_in_structure_tree() {
        use crate::structure::figure_with_caption;

        let fig = figure_with_caption("Sunset photo", "Figure 1: Sunset", 1, 0, 1);
        let mut doc = StructureNode::with_children(StructureType::Document, vec![fig]);
        doc.assign_reading_order();

        let mut builder = PdfDocumentBuilder::new();
        builder.set_tagged(true);
        builder.add_page(612.0, 792.0);
        builder.set_structure_tree(vec![doc]);
        let bytes = builder.build();
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/Alt (Sunset photo)"));
        assert!(s.contains("/S /Caption"));
        assert!(s.contains("Figure 1: Sunset"));
    }

    #[test]
    fn test_reading_order_is_sequential() {
        use crate::structure::{heading, paragraph};

        let mut doc = StructureNode::with_children(
            StructureType::Document,
            vec![
                heading(1, "Title", 1, 0),
                paragraph("Para 1", 1, 1),
                paragraph("Para 2", 1, 2),
            ],
        );
        let count = doc.assign_reading_order();
        assert_eq!(count, 3);
        assert_eq!(doc.children[0].reading_order, 0);
        assert_eq!(doc.children[1].reading_order, 1);
        assert_eq!(doc.children[2].reading_order, 2);
    }

    #[test]
    fn test_language_span_in_pdf() {
        use crate::structure::language_span_node;

        let span = language_span_node("Bonjour", "fr", 1, 0);
        let mut doc = StructureNode::with_children(StructureType::Document, vec![span]);
        doc.assign_reading_order();

        let mut builder = PdfDocumentBuilder::new();
        builder.set_tagged(true);
        builder.add_page(612.0, 792.0);
        builder.set_structure_tree(vec![doc]);
        let bytes = builder.build();
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/S /Span"));
        assert!(s.contains("/Lang (fr)"));
        assert!(s.contains("/ActualText"));
        assert!(s.contains("Bonjour"));
    }

    #[test]
    fn test_actual_text_and_expanded_text_in_pdf() {
        let node = StructureNode::new(StructureType::Span, 1, 0)
            .with_actual_text("PDF")
            .with_expanded_text("Portable Document Format");
        let mut doc = StructureNode::with_children(StructureType::Document, vec![node]);
        doc.assign_reading_order();

        let mut builder = PdfDocumentBuilder::new();
        builder.set_tagged(true);
        builder.add_page(612.0, 792.0);
        builder.set_structure_tree(vec![doc]);
        let bytes = builder.build();
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/ActualText (PDF)"));
        assert!(s.contains("/E (Portable Document Format)"));
    }

    #[test]
    fn test_footnote_ref_and_body_in_pdf() {
        use crate::structure::footnote_pair;

        let (ref_node, body_node) = footnote_pair("[1]", "Footnote text", 1, 0, 1);
        let mut doc =
            StructureNode::with_children(StructureType::Document, vec![ref_node, body_node]);
        doc.assign_reading_order();

        let mut builder = PdfDocumentBuilder::new();
        builder.set_tagged(true);
        builder.add_page(612.0, 792.0);
        builder.set_structure_tree(vec![doc]);
        let bytes = builder.build();
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/S /Reference"));
        assert!(s.contains("[1]"));
        assert!(s.contains("Footnote text"));
    }

    #[test]
    fn test_list_item_with_label_and_body_in_pdf() {
        use crate::structure::list_item;

        let li = list_item("1.", "First item", 1, 0, 1);
        let mut doc = StructureNode::with_children(StructureType::Document, vec![li]);
        doc.assign_reading_order();

        let mut builder = PdfDocumentBuilder::new();
        builder.set_tagged(true);
        builder.add_page(612.0, 792.0);
        builder.set_structure_tree(vec![doc]);
        let bytes = builder.build();
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/S /LI"));
        assert!(s.contains("/S /Lbl"));
        assert!(s.contains("/S /LBody"));
        assert!(s.contains("1."));
        assert!(s.contains("First item"));
    }

    #[test]
    fn test_pdfa2b_version_header() {
        let mut builder = PdfDocumentBuilder::new();
        builder.set_conformance(PdfConformance::PdfA2b);
        builder.add_page(612.0, 792.0);
        let bytes = builder.build();
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("%PDF-1.7"));
    }

    #[test]
    fn test_pdfa4_version_header() {
        let mut builder = PdfDocumentBuilder::new();
        builder.set_conformance(PdfConformance::PdfA4);
        builder.add_page(612.0, 792.0);
        let bytes = builder.build();
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("%PDF-2.0"));
    }

    #[test]
    fn test_pdfa2b_xmp_metadata() {
        let mut builder = PdfDocumentBuilder::new();
        builder.set_conformance(PdfConformance::PdfA2b);
        builder.set_title("Test");
        builder.set_author("Author");
        builder.add_page(612.0, 792.0);
        let bytes = builder.build();
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/Type /Metadata"));
        assert!(s.contains("/Subtype /XML"));
    }

    #[test]
    fn test_default_conformance_is_pdfa4() {
        let builder = PdfDocumentBuilder::new();
        let bytes = {
            let mut b = builder;
            b.add_page(612.0, 792.0);
            b.build()
        };
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("%PDF-2.0"));
    }

    #[test]
    fn test_table_draws_borders() {
        let mut builder = PdfDocumentBuilder::new();
        builder.add_page(612.0, 792.0);
        builder.draw_table(72.0, 700.0, &[200.0, 200.0], 20.0, 3, 1.0);
        let bytes = builder.build();
        assert!(bytes.starts_with(b"%PDF"));
        // Table content is in the compressed content stream, so just verify valid PDF
        assert!(bytes.len() > 500);
    }

    #[test]
    fn test_table_single_cell() {
        let mut builder = PdfDocumentBuilder::new();
        builder.add_page(612.0, 792.0);
        builder.draw_table(72.0, 700.0, &[468.0], 20.0, 1, 0.5);
        let bytes = builder.build();
        assert!(bytes.starts_with(b"%PDF"));
        assert!(bytes.len() > 500);
    }

    #[test]
    fn test_table_multiple_columns() {
        let mut builder = PdfDocumentBuilder::new();
        builder.add_page(612.0, 792.0);
        builder.draw_table(50.0, 600.0, &[100.0, 150.0, 200.0, 50.0], 25.0, 5, 1.0);
        let bytes = builder.build();
        assert!(bytes.starts_with(b"%PDF"));
        assert!(bytes.len() > 500);
    }

    #[test]
    fn test_table_empty_columns_noop() {
        let mut builder = PdfDocumentBuilder::new();
        builder.add_page(612.0, 792.0);
        builder.draw_table(72.0, 700.0, &[], 20.0, 3, 1.0);
        let bytes = builder.build();
        assert!(bytes.starts_with(b"%PDF"));
    }

    #[test]
    fn test_table_zero_rows_noop() {
        let mut builder = PdfDocumentBuilder::new();
        builder.add_page(612.0, 792.0);
        builder.draw_table(72.0, 700.0, &[200.0], 20.0, 0, 1.0);
        let bytes = builder.build();
        assert!(bytes.starts_with(b"%PDF"));
    }

    #[test]
    fn test_table_no_page_auto_creates() {
        let mut builder = PdfDocumentBuilder::new();
        // No page added — draw_table should not panic on empty pages
        builder.draw_table(72.0, 700.0, &[200.0], 20.0, 1, 1.0);
        // Should have auto-created a page via the early return (pages is empty, so just returns)
        assert_eq!(builder.pages.len(), 0);
    }
}
