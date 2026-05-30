//! Streaming PDF writer that outputs directly to a Write sink.
//!
//! Unlike `pdf_writer::Pdf` (which buffers everything in a single Vec<u8>),
//! this writes PDF objects sequentially and drops page data as it goes,
//! reducing peak memory usage for large documents.

use std::collections::{HashMap, HashSet};
use std::io::Write;

use pdf_writer::types::{SystemInfo, UnicodeCmap};
use pdf_writer::{Name, Str};

use crate::color::icc_alternate_name;
use crate::conformance::PdfConformance;
use crate::structure::StructureNode;
use crate::writer::{PdfDocumentBuilder, PdfImageFormat};

pub struct LinkAnnotation {
    pub rect: (f32, f32, f32, f32),
    pub url: Option<String>,
    pub dest_page: Option<u32>,
}

#[allow(dead_code)]
pub struct ImageXObject {
    pub id: i32,
    pub width: u32,
    pub height: u32,
    pub color_space: String,
    pub bits_per_component: u8,
    pub filter: String,
}

pub(crate) struct StreamingPdfWriter<W: Write> {
    sink: W,
    byte_offset: usize,
    xref_entries: Vec<(i32, usize)>,
    next_id: i32,
    catalog_id: i32,
    pages_id: i32,
    pending_links: Vec<LinkAnnotation>,
    pending_draw_images: Vec<(i32, f32, f32, f32, f32)>,
}

impl<W: Write> StreamingPdfWriter<W> {
    pub(crate) fn new(mut sink: W, conformance: PdfConformance) -> std::io::Result<Self> {
        let version = conformance.pdf_version_str();
        let header_text = format!("%PDF-{version}\n");
        sink.write_all(header_text.as_bytes())?;
        let binary_comment: &[u8] = &[0x25, 0xE2, 0xE3, 0xCF, 0xD3, 0x0A];
        sink.write_all(binary_comment)?;
        let byte_offset = header_text.len() + binary_comment.len();

        let catalog_id = 1;
        let pages_id = 2;

        Ok(Self {
            sink,
            byte_offset,
            xref_entries: Vec::new(),
            next_id: 3,
            catalog_id,
            pages_id,
            pending_links: Vec::new(),
            pending_draw_images: Vec::new(),
        })
    }

    fn alloc_id(&mut self) -> i32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn alloc_ids(&mut self, count: i32) -> i32 {
        let start = self.next_id;
        self.next_id += count;
        start
    }

    fn next_id(&self) -> i32 {
        self.next_id
    }

    fn write_raw(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.sink.write_all(data)?;
        self.byte_offset += data.len();
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> std::io::Result<()> {
        self.write_raw(s.as_bytes())
    }

    fn begin_object(&mut self, id: i32) -> std::io::Result<()> {
        let obj_header = format!("{id} 0 obj\n");
        self.xref_entries.push((id, self.byte_offset));
        self.write_str(&obj_header)
    }

    fn end_object(&mut self) -> std::io::Result<()> {
        self.write_str("endobj\n")
    }

    fn write_indirect_dict(&mut self, id: i32, dict_content: &str) -> std::io::Result<()> {
        self.begin_object(id)?;
        self.write_str(dict_content)?;
        self.end_object()
    }

    fn write_stream(
        &mut self,
        id: i32,
        dict_entries: &str,
        data: &[u8],
        filter: Option<&str>,
    ) -> std::io::Result<()> {
        let has_flate = filter == Some("FlateDecode");
        let stream_data = if has_flate {
            crate::writer::compress(data)
        } else {
            data.to_vec()
        };

        self.begin_object(id)?;

        if dict_entries.is_empty() {
            self.write_str("<< ")?;
        } else {
            self.write_str(dict_entries)?;
        }
        if let Some(f) = filter {
            self.write_str(&format!("/Filter /{f} "))?;
        }

        self.write_str(&format!("/Length {} >>\nstream\n", stream_data.len()))?;
        self.write_raw(&stream_data)?;
        self.write_str("\nendstream\n")?;
        self.end_object()
    }

    fn write_ref(&self, id: i32) -> String {
        format!("{id} 0 R")
    }

    fn write_xref_and_trailer(&mut self, info_id: i32) -> std::io::Result<()> {
        let max_id = self
            .xref_entries
            .iter()
            .map(|(id, _)| *id)
            .max()
            .unwrap_or(1)
            + 1;

        let xref_offset = self.byte_offset;
        self.write_str("xref\n")?;
        self.write_str(&format!("0 {max_id}\n"))?;

        self.write_str("0000000000 65535 f \n")?;

        let mut sorted: Vec<(i32, usize)> = self.xref_entries.clone();
        sorted.sort_by_key(|(id, _)| *id);

        let mut idx = 0;
        for obj_num in 1..max_id {
            if idx < sorted.len() && sorted[idx].0 == obj_num {
                self.write_str(&format!("{:010} 00000 n \n", sorted[idx].1))?;
                idx += 1;
            } else {
                self.write_str("0000000000 00000 f \n")?;
            }
        }

        self.write_str("trailer\n")?;
        self.write_str(&format!(
            "<< /Size {max_id} /Root {} 0 R /Info {} 0 R >>\n",
            self.catalog_id, info_id
        ))?;
        self.write_str(&format!("startxref\n{xref_offset}\n"))?;
        self.write_str("%%EOF\n")
    }

    pub(crate) fn into_inner(self) -> W {
        self.sink
    }

    pub fn add_link(&mut self, rect: (f32, f32, f32, f32), url: &str) {
        self.pending_links.push(LinkAnnotation {
            rect,
            url: Some(url.to_string()),
            dest_page: None,
        });
    }

    pub fn add_internal_link(&mut self, rect: (f32, f32, f32, f32), dest_page: u32) {
        self.pending_links.push(LinkAnnotation {
            rect,
            url: None,
            dest_page: Some(dest_page),
        });
    }

    pub fn embed_image_jpeg(&mut self, data: &[u8]) -> std::io::Result<ImageXObject> {
        let id = self.alloc_id();
        let (width, height, components) = jpeg_info(data).unwrap_or((100, 100, 3));
        let cs_name = if components == 1 {
            "DeviceGray"
        } else {
            "DeviceRGB"
        };
        let dict = format!(
            "/Type /XObject /Subtype /Image /Width {width} /Height {height} /ColorSpace /{cs_name} /BitsPerComponent 8"
        );
        self.write_stream(id, &dict, data, Some("DctDecode"))?;
        Ok(ImageXObject {
            id,
            width,
            height,
            color_space: cs_name.to_string(),
            bits_per_component: 8,
            filter: "DctDecode".to_string(),
        })
    }

    pub fn embed_image_png(&mut self, data: &[u8]) -> std::io::Result<ImageXObject> {
        let decoded = crate::image::decode_png(data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let id = self.alloc_id();
        let cs_name = match decoded.color_space {
            crate::image::ColorSpace::RGB => "DeviceRGB",
            crate::image::ColorSpace::Gray => "DeviceGray",
        };
        let dict = format!(
            "/Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /{cs_name} /BitsPerComponent {}",
            decoded.width, decoded.height, decoded.bits_per_component
        );
        self.write_stream(id, &dict, &decoded.data, Some("FlateDecode"))?;
        Ok(ImageXObject {
            id,
            width: decoded.width,
            height: decoded.height,
            color_space: cs_name.to_string(),
            bits_per_component: decoded.bits_per_component,
            filter: "FlateDecode".to_string(),
        })
    }

    pub fn draw_image(&mut self, img_ref: i32, x: f32, y: f32, width: f32, height: f32) {
        self.pending_draw_images
            .push((img_ref, x, y, width, height));
    }

    pub fn take_page_links(&mut self) -> Vec<LinkAnnotation> {
        std::mem::take(&mut self.pending_links)
    }

    pub fn take_page_draw_images(&mut self) -> Vec<(i32, f32, f32, f32, f32)> {
        std::mem::take(&mut self.pending_draw_images)
    }
}

struct StreamingFontIds {
    type0_id: i32,
    cid_id: i32,
    descriptor_id: i32,
    fontfile_id: i32,
    cidtogidmap_id: i32,
    tounicode_id: i32,
    resource_name: Vec<u8>,
}

pub fn build_streaming<W: Write>(builder: &PdfDocumentBuilder, sink: W) -> std::io::Result<W> {
    let page_count = builder.pages.len() as i32;

    let mut w = StreamingPdfWriter::new(sink, builder.conformance)?;

    // --- Allocate all IDs upfront ---
    let page_ids: Vec<i32> = (0..page_count).map(|_| w.alloc_id()).collect();
    let content_ids: Vec<i32> = (0..page_count).map(|_| w.alloc_id()).collect();

    // Structure tree
    let struct_tree_info = if builder.tagged && !builder.structure_tree.is_empty() {
        let all_nodes: Vec<StructureNode> = builder
            .structure_tree
            .iter()
            .flat_map(|n| collect_all_nodes(n).into_iter().cloned())
            .collect();
        if all_nodes.is_empty() {
            None
        } else {
            let node_start = w.next_id();
            w.alloc_ids(all_nodes.len() as i32);
            let root_id = w.alloc_id();
            let parent_tree_id = w.alloc_id();
            Some((all_nodes, node_start, root_id, parent_tree_id))
        }
    } else {
        None
    };

    // ICC
    let icc_stream_id = if builder.icc_profile.is_some() {
        Some(w.alloc_id())
    } else {
        None
    };
    let intent_id = if builder.icc_profile.is_some() {
        Some(w.alloc_id())
    } else {
        None
    };

    // XMP metadata
    let xmp_metadata_id = w.alloc_id();

    // Fallback font
    let fallback_font_id = if builder.fonts.is_empty() {
        Some(w.alloc_id())
    } else {
        None
    };

    // Link annotations are allocated dynamically per-page (no pre-allocation needed).

    // Image XObjects -- embed before pages so IDs are available for resource dicts.

    // Font IDs (6 per font)
    let font_ids: Vec<StreamingFontIds> = builder
        .fonts
        .iter()
        .map(|font| StreamingFontIds {
            type0_id: w.alloc_id(),
            cid_id: w.alloc_id(),
            descriptor_id: w.alloc_id(),
            fontfile_id: w.alloc_id(),
            cidtogidmap_id: w.alloc_id(),
            tounicode_id: w.alloc_id(),
            resource_name: font.resource_name.clone(),
        })
        .collect();

    let catalog_id = w.catalog_id;
    let pages_id = w.pages_id;

    // --- Write Catalog ---
    {
        let mut dict = String::from("<< /Type /Catalog /Pages ");
        dict.push_str(&w.write_ref(pages_id));
        dict.push_str(" 0 R");
        if builder.tagged {
            dict.push_str(&format!(" /Lang ({})", builder.language));
            dict.push_str(" /MarkInfo << /Marked true >>");
            if let Some((_, _, root_id, parent_tree_id)) = &struct_tree_info {
                dict.push_str(&format!(" /StructTreeRoot {} 0 R", root_id));
                dict.push_str(&format!(" /ParentTree {} 0 R", parent_tree_id));
            }
        }
        if let Some(intent_id) = intent_id {
            dict.push_str(&format!(" /OutputIntents [{} 0 R]", intent_id));
        }
        dict.push_str(" >>");
        w.write_indirect_dict(catalog_id, &dict)?;
    }

    // --- Write Pages tree ---
    {
        let mut kids = String::from("[");
        for &pid in &page_ids {
            kids.push_str(&format!("{} 0 R ", pid));
        }
        kids.push(']');
        let dict = format!("<< /Type /Pages /Kids {kids} /Count {page_count} >>");
        w.write_indirect_dict(pages_id, &dict)?;
    }

    // --- Write Fallback font ---
    if let Some(fid) = fallback_font_id {
        w.write_indirect_dict(
            fid,
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
        )?;
    }

    // --- Collect used glyphs per font ---
    let font_used: Vec<HashSet<u32>> = builder
        .pending_glyphs
        .iter()
        .map(|gids| {
            let mut set = HashSet::new();
            set.insert(0);
            for &gid in gids {
                set.insert(gid);
            }
            set
        })
        .collect();

    // --- Write Font objects ---
    for (font_idx, font) in builder.fonts.iter().enumerate() {
        let ids = &font_ids[font_idx];
        let used_gids = &font_used[font_idx];
        write_streaming_font(&mut w, font, used_gids, ids)?;
    }

    // --- Embed image XObjects (before pages so IDs are available) ---
    let image_xobject_ids: Vec<Option<i32>> = builder
        .images
        .iter()
        .map(|img| match img.format {
            PdfImageFormat::Jpeg => w.embed_image_jpeg(&img.data).ok().map(|x| x.id),
            PdfImageFormat::Png => w.embed_image_png(&img.data).ok().map(|x| x.id),
        })
        .collect();

    let img_ref_to_idx: HashMap<i32, usize> = image_xobject_ids
        .iter()
        .enumerate()
        .filter_map(|(idx, id)| id.map(|id| (id, idx)))
        .collect();

    // --- Write pages, content streams, and link annotations ---
    for i in 0..builder.pages.len() {
        let page_state = &builder.pages[i];

        // Add link annotations to streaming writer
        for (x, y, lw, lh, url, dest_page) in &page_state.links {
            let rect = (*x as f32, *y as f32, (*x + *lw) as f32, (*y + *lh) as f32);
            if let Some(dp) = dest_page {
                w.add_internal_link(rect, *dp as u32);
            } else {
                w.add_link(rect, url);
            }
        }

        // Add image draw commands
        for &(x, y, w_dim, h_dim, image_index) in &page_state.images {
            if let Some(Some(id)) = image_xobject_ids.get(image_index) {
                w.draw_image(*id, x as f32, y as f32, w_dim as f32, h_dim as f32);
            }
        }

        // Take draw commands for content stream and resource dict
        let draw_cmds = w.take_page_draw_images();

        // Build content bytes
        let mut content = pdf_writer::Content::new();
        if builder.tagged {
            let mcid = i as i32;
            content
                .begin_marked_content_with_properties(Name(b"P"))
                .properties()
                .pair(Name(b"MCID"), mcid);
        }

        // Inject image Do operations
        for &(img_ref, x, y, w_dim, h_dim) in &draw_cmds {
            let name_idx = img_ref_to_idx.get(&img_ref).copied().unwrap_or(0);
            let resource_name = format!("Im{}", name_idx);
            content.save_state();
            content.transform([w_dim, 0.0, 0.0, h_dim, x, y - h_dim]);
            content.x_object(Name(resource_name.as_bytes()));
            content.restore_state();
        }

        if builder.tagged {
            content.end_marked_content();
        }

        let content_bytes = content.finish();

        // Write content stream
        w.write_stream(content_ids[i], "", &content_bytes, Some("FlateDecode"))?;

        // Take link annotations for page dict
        let links = w.take_page_links();

        // Allocate annotation IDs dynamically
        let annot_ids: Vec<i32> = (0..links.len()).map(|_| w.alloc_id()).collect();

        // Write page dict
        {
            let mut dict = String::new();
            dict.push_str(&format!(
                "<< /Type /Page /Parent {} 0 R /MediaBox [0 0 {:.4} {:.4}] /Contents {} 0 R",
                pages_id, page_state.width, page_state.height, content_ids[i]
            ));

            // Resources - Fonts
            dict.push_str(" /Resources << /Font << ");
            for fids in &font_ids {
                let name = String::from_utf8_lossy(&fids.resource_name);
                dict.push_str(&format!("/{} {} 0 R ", name, fids.type0_id));
            }
            if let Some((ref fid, _)) = fallback_font_id.zip(Some(b"F1".to_vec())) {
                dict.push_str(&format!("/F1 {fid} 0 R "));
            }
            dict.push_str(">> ");

            // XObjects
            if !draw_cmds.is_empty() {
                dict.push_str("/XObject << ");
                let mut seen: HashSet<i32> = HashSet::new();
                for &(img_ref, _, _, _, _) in &draw_cmds {
                    if seen.insert(img_ref) {
                        let name_idx = img_ref_to_idx.get(&img_ref).copied().unwrap_or(0);
                        let name = format!("Im{}", name_idx);
                        dict.push_str(&format!("/{name} {img_ref} 0 R "));
                    }
                }
                dict.push_str(">> ");
            }

            // ICC color space
            if let Some(icc_ref) = icc_stream_id {
                dict.push_str(&format!(
                    "/ColorSpace << /ICCSB [/ICCBased {} 0 R] >> ",
                    icc_ref
                ));
            }

            dict.push_str(">> ");

            // Annotations
            if !annot_ids.is_empty() {
                dict.push_str("/Annots [");
                for &aid in &annot_ids {
                    dict.push_str(&format!("{} 0 R ", aid));
                }
                dict.push(']');
            }

            dict.push_str(">>");
            w.write_indirect_dict(page_ids[i], &dict)?;
        }

        // Write link annotation objects
        for (j, link) in links.iter().enumerate() {
            let aid = annot_ids[j];
            let mut dict = format!(
                "<< /Type /Annot /Subtype /Link /Rect [{:.4} {:.4} {:.4} {:.4}]",
                link.rect.0, link.rect.1, link.rect.2, link.rect.3
            );
            if let Some(page_idx) = link.dest_page {
                if (page_idx as usize) < page_ids.len() {
                    dict.push_str(&format!(" /Dest {} 0 R", page_ids[page_idx as usize]));
                }
            } else if let Some(ref url) = link.url {
                dict.push_str(&format!(" /A << /Type /Action /S /URI /URI ({url}) >>"));
            }
            dict.push_str(" >>");
            w.write_indirect_dict(aid, &dict)?;
        }
    }

    // --- Document Info ---
    let info_id = w.alloc_id();
    {
        let mut dict = String::from("<< ");
        if !builder.title.is_empty() {
            dict.push_str(&format!("/Title ({})", escape_pdf_string(&builder.title)));
        }
        if !builder.author.is_empty() {
            dict.push_str(&format!("/Author ({})", escape_pdf_string(&builder.author)));
        }
        if !builder.subject.is_empty() {
            dict.push_str(&format!(
                "/Subject ({})",
                escape_pdf_string(&builder.subject)
            ));
        }
        if !builder.creator.is_empty() {
            dict.push_str(&format!(
                "/Creator ({})",
                escape_pdf_string(&builder.creator)
            ));
        }
        dict.push_str(" >>");
        w.write_indirect_dict(info_id, &dict)?;
    }

    // --- ICC profile and OutputIntent ---
    if let (Some(stream_id), Some(int_id)) = (icc_stream_id, intent_id)
        && let Some(profile) = &builder.icc_profile
    {
        let compressed = crate::writer::compress(&profile.data);
        let alternate = icc_alternate_name(profile.color_space);
        let alt_name = std::str::from_utf8(alternate).unwrap_or("DeviceRGB");
        let dict = format!("/N {} /Alternate /{alt_name}", profile.components);
        w.write_stream(stream_id, &dict, &compressed, Some("FlateDecode"))?;

        let condition_id = profile.name.as_str();
        let intent_dict = format!(
            "<< /Type /OutputIntent /S /GTS_PDFX /OutputConditionIdentifier ({condition_id}) /DestOutputProfile {} 0 R >>",
            stream_id
        );
        w.write_indirect_dict(int_id, &intent_dict)?;
    }

    // --- XMP metadata ---
    {
        let xmp_bytes =
            crate::xmp::generate_pdfa_xmp(builder.conformance, &builder.title, &builder.author);
        w.write_stream(
            xmp_metadata_id,
            "/Type /Metadata /Subtype /XML",
            &xmp_bytes,
            Some("FlateDecode"),
        )?;
    }

    // --- Structure tree ---
    if let Some((all_nodes, node_start, root_id, parent_tree_id)) = struct_tree_info {
        let node_refs: Vec<i32> = (0..all_nodes.len())
            .map(|i| node_start + i as i32)
            .collect();

        for (i, node) in all_nodes.iter().enumerate() {
            let r = node_refs[i];
            let mut dict = String::from("<< /Type /StructElem");
            if let Some(custom_name) = node.element_type.custom_role_name() {
                dict.push_str(&format!(
                    " /S /{}",
                    std::str::from_utf8(custom_name).unwrap_or("")
                ));
            } else {
                // Use the StructRole name
                let role_name = struct_role_name(node.element_type);
                dict.push_str(&format!(" /S /{role_name}"));
            }
            if let Some(ref alt) = node.alt_text {
                dict.push_str(&format!(" /Alt ({})", escape_pdf_string(alt)));
            }
            if let Some(ref actual) = node.actual_text {
                dict.push_str(&format!(" /ActualText ({})", escape_pdf_string(actual)));
            }
            if let Some(ref expanded) = node.expanded_text {
                dict.push_str(&format!(" /E ({})", escape_pdf_string(expanded)));
            }
            if let Some(ref lang) = node.language {
                dict.push_str(&format!(" /Lang ({lang})"));
            }
            if let Some(ref bbox) = node.bbox {
                dict.push_str(&format!(
                    " /BBox [{:.2} {:.2} {:.2} {:.2}]",
                    bbox.x,
                    bbox.y,
                    bbox.x + bbox.width,
                    bbox.y + bbox.height
                ));
            }
            let page_idx = node.page.saturating_sub(1) as usize;
            if node.is_leaf() && page_idx < page_ids.len() {
                dict.push_str(&format!(" /P {} 0 R", page_ids[page_idx]));
            }
            if !node.children.is_empty() {
                dict.push_str(" /K [");
                for child in &node.children {
                    if let Some(idx) = find_node_index(&all_nodes, child) {
                        dict.push_str(&format!("{} 0 R ", node_refs[idx]));
                    }
                }
                dict.push(']');
            }
            dict.push_str(" >>");
            w.write_indirect_dict(r, &dict)?;
        }

        // StructTreeRoot
        {
            let mut kids = String::from("[");
            for &r in &node_refs {
                kids.push_str(&format!("{} 0 R ", r));
            }
            kids.push(']');
            let root_dict = format!(
                "<< /Type /StructTreeRoot /K {kids} /ParentTreeNextKey {} >>",
                page_count
            );
            w.write_indirect_dict(root_id, &root_dict)?;
        }

        // ParentTree
        {
            let mut nums = String::from("<< /Nums [");
            for page_idx in 0..page_count as usize {
                nums.push_str(&format!("{} {} 0 R ", page_idx, page_idx));
            }
            nums.push_str("] >>");
            w.write_indirect_dict(parent_tree_id, &nums)?;
        }
    }

    // --- Xref table and trailer ---
    w.write_xref_and_trailer(info_id)?;

    Ok(w.into_inner())
}

fn write_streaming_font<W: Write>(
    w: &mut StreamingPdfWriter<W>,
    font: &crate::writer::EmbeddedFont,
    used_gids: &HashSet<u32>,
    ids: &StreamingFontIds,
) -> std::io::Result<()> {
    let info = font.face.pdf_info();
    let upem = info.units_per_em as f32;
    let scale = 1000.0 / upem;

    let ps_name = &info.postscript_name;

    let subset_result = crate::font::subset_font(font.face.raw_bytes(), used_gids);

    // Type0 font
    {
        let dict = format!(
            "<< /Type /Font /Subtype /Type0 /BaseFont /{ps_name} /Encoding /Identity-H /DescendantFonts [{} 0 R] /ToUnicode {} 0 R >>",
            ids.cid_id, ids.tounicode_id
        );
        w.write_indirect_dict(ids.type0_id, &dict)?;
    }

    // CIDFont
    {
        let default_width = 1000.0 / upem;
        let mut dict = format!(
            "<< /Type /Font /Subtype /CIDFontType2 /BaseFont /{ps_name} /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> /FontDescriptor {} 0 R /DW {default_width:.4}",
            ids.descriptor_id
        );

        // W array
        let mut sorted_glyphs: Vec<u16> = used_gids
            .iter()
            .map(|&g| g as u16)
            .filter(|&g| g > 0)
            .collect();
        sorted_glyphs.sort_unstable();

        if !sorted_glyphs.is_empty() {
            let widths_f32: Vec<f32> = sorted_glyphs
                .iter()
                .map(|&gid| {
                    let g = ttf_parser::GlyphId(gid);
                    font.face
                        .face()
                        .glyph_hor_advance(g)
                        .map(|a| a as f32 * scale)
                        .unwrap_or(500.0)
                })
                .collect();

            let mut w_array = String::from(" /W [");
            let mut start = sorted_glyphs[0];
            let mut run_start_idx = 0;
            for (i, &gid) in sorted_glyphs.iter().enumerate().skip(1) {
                if gid != sorted_glyphs[i - 1] + 1 {
                    // Emit run
                    let run_widths: Vec<f32> = widths_f32[run_start_idx..i].to_vec();
                    w_array.push_str(&format!("{} [", start));
                    for width in &run_widths {
                        w_array.push_str(&format!("{:.4} ", width));
                    }
                    w_array.push_str("] ");
                    start = gid;
                    run_start_idx = i;
                }
            }
            // Final run
            let run_widths: Vec<f32> = widths_f32[run_start_idx..sorted_glyphs.len()].to_vec();
            w_array.push_str(&format!("{} [", start));
            for width in &run_widths {
                w_array.push_str(&format!("{:.4} ", width));
            }
            w_array.push_str("] ");
            w_array.push(']');
            dict.push_str(&w_array);
        }

        if let Some(ref _cid_map) = subset_result.cid_to_gid_map {
            let ref_str = w.write_ref(ids.cidtogidmap_id);
            dict.push_str(&format!(" /CIDToGIDMap {ref_str} >>"));
        } else {
            dict.push_str(" /CIDToGIDMap /Identity >>");
        }
        w.write_indirect_dict(ids.cid_id, &dict)?;
    }

    // FontDescriptor
    {
        let mut flags: u32 = 0x00000004; // NON_SYMBOLIC
        if info.is_monospace {
            flags |= 0x00000001; // FIXED_PITCH
        }

        let stem_v = (info.ascent as f32 * scale * 0.05).round().max(50.0);

        let dict = format!(
            "<< /Type /FontDescriptor /FontName /{ps_name} /Flags {flags} /FontBBox [{:.4} {:.4} {:.4} {:.4}] /ItalicAngle {:.4} /Ascent {:.4} /Descent {:.4} /CapHeight {:.4} /XHeight {:.4} /StemV {stem_v:.4} /FontFile2 {} 0 R >>",
            info.bbox.x_min as f32 * scale,
            info.bbox.y_min as f32 * scale,
            info.bbox.x_max as f32 * scale,
            info.bbox.y_max as f32 * scale,
            info.italic_angle,
            info.ascent as f32 * scale,
            info.descent as f32 * scale,
            info.cap_height * scale,
            info.x_height * scale,
            ids.fontfile_id
        );
        w.write_indirect_dict(ids.descriptor_id, &dict)?;
    }

    // FontFile2 (TrueType font stream, FlateDecode compressed)
    {
        let dict = format!("/Length1 {}", subset_result.font_data.len());
        w.write_stream(
            ids.fontfile_id,
            &dict,
            &subset_result.font_data,
            Some("FlateDecode"),
        )?;
    }

    // CIDToGIDMap stream (if needed)
    if let Some(ref cid_map) = subset_result.cid_to_gid_map {
        w.write_stream(ids.cidtogidmap_id, "", cid_map, Some("FlateDecode"))?;
    }

    // ToUnicode CMap
    {
        let system_info = SystemInfo {
            registry: Str(b"Adobe"),
            ordering: Str(b"Identity"),
            supplement: 0,
        };
        let cmap_name: Vec<u8> = format!("{ps_name}-ToUnicode").into_bytes();

        let mut cmap = UnicodeCmap::new(Name(&cmap_name), system_info);

        let mut sorted_gids: Vec<u32> = used_gids.iter().copied().collect();
        sorted_gids.sort_unstable();
        let mut seen = HashSet::new();
        for gid in sorted_gids {
            if seen.insert(gid)
                && let Some(ch) = font.face.glyph_to_unicode(gid)
            {
                cmap.pair(gid as u16, ch);
            }
        }

        let cmap_data = cmap.finish();
        w.write_stream(ids.tounicode_id, "", &cmap_data, Some("FlateDecode"))?;
    }

    Ok(())
}

fn struct_role_name(st: crate::structure::StructureType) -> &'static str {
    use crate::structure::StructureType::*;
    match st {
        Document => "Document",
        Part => "Part",
        Chapter => "Chapter",
        Section => "Sect",
        Subsection => "Subsection",
        H1 => "H1",
        H2 => "H2",
        H3 => "H3",
        H4 => "H4",
        H5 => "H5",
        H6 => "H6",
        Paragraph => "P",
        List => "L",
        ListItem => "LI",
        ListLabel => "Lbl",
        ListBody => "LBody",
        Table => "Table",
        TableHeader => "THead",
        TableBody => "TBody",
        TableRow => "TR",
        TableHeaderCell => "TH",
        TableDataCell => "TD",
        Figure => "Figure",
        Caption => "Caption",
        CodeBlock => "CodeBlock",
        BlockQuote => "BlockQuote",
        MathBlock => "MathBlock",
        Footnote => "Note",
        FootnoteRef => "Reference",
        FootnoteBody => "FootnoteBody",
        TOC => "TOC",
        ThematicBreak => "NonStruct",
        Span => "Span",
        Artifact => "Artifact",
    }
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

fn escape_pdf_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '(' | ')' | '\\' => {
                out.push('\\');
                out.push(ch);
            }
            c => out.push(c),
        }
    }
    out
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::IccProfile;
    use crate::font::FontFace;
    use crate::structure::StructureType;
    use crate::writer::PdfDocumentBuilder;

    fn get_font_face() -> Option<FontFace> {
        let data = ldir_test_helpers::test_font_data();
        FontFace::from_bytes(&data).ok()
    }

    #[test]
    fn test_streaming_empty_pdf() {
        let mut builder = PdfDocumentBuilder::new();
        builder.add_page(612.0, 792.0);
        let mut buf = Vec::new();
        let result = build_streaming(&builder, &mut buf);
        assert!(result.is_ok());
        assert!(!buf.is_empty());
        assert!(buf.starts_with(b"%PDF"));
        assert!(buf.ends_with(b"%%EOF\n"));
    }

    #[test]
    fn test_streaming_pdf_with_text_no_font() {
        let mut builder = PdfDocumentBuilder::new();
        builder.add_page(612.0, 792.0);
        builder.write_text(72.0, 720.0, "Hello");
        let mut buf = Vec::new();
        let result = build_streaming(&builder, &mut buf);
        assert!(result.is_ok());
        assert!(buf.starts_with(b"%PDF"));
    }

    #[test]
    fn test_streaming_pdf_with_embedded_font() {
        let Some(face) = get_font_face() else { return };
        let mut builder = PdfDocumentBuilder::new();
        let font_idx = builder.add_font(face, 12.0);
        builder.set_active_font(font_idx);
        builder.add_page(612.0, 792.0);
        builder.write_text(72.0, 720.0, "Hello World");
        let mut buf = Vec::new();
        let result = build_streaming(&builder, &mut buf);
        assert!(result.is_ok());
        assert!(buf.starts_with(b"%PDF"));
        let pdf_str = String::from_utf8_lossy(&buf);
        assert!(pdf_str.contains("/FontDescriptor"));
        assert!(pdf_str.contains("/FontFile2"));
    }

    #[test]
    fn test_streaming_pdf_with_glyphs() {
        let Some(face) = get_font_face() else { return };
        let mut builder = PdfDocumentBuilder::new();
        let font_idx = builder.add_font(face, 12.0);
        builder.set_active_font(font_idx);
        builder.add_page(612.0, 792.0);
        builder.write_glyph(72.0, 720.0, 36, 7.0);
        builder.write_glyph(79.0, 720.0, 56, 6.0);
        let mut buf = Vec::new();
        let result = build_streaming(&builder, &mut buf);
        assert!(result.is_ok());
        let pdf_str = String::from_utf8_lossy(&buf);
        assert!(pdf_str.contains("/ToUnicode"));
    }

    #[test]
    fn test_streaming_pdf_with_rect() {
        let mut builder = PdfDocumentBuilder::new();
        builder.add_page(612.0, 792.0);
        builder.draw_rect(72.0, 100.0, 468.0, 1.0);
        let mut buf = Vec::new();
        build_streaming(&builder, &mut buf).unwrap();
        assert!(buf.starts_with(b"%PDF"));
    }

    #[test]
    fn test_streaming_pdf_with_title() {
        let mut builder = PdfDocumentBuilder::new();
        builder.set_title("Test Document");
        builder.add_page(612.0, 792.0);
        let mut buf = Vec::new();
        build_streaming(&builder, &mut buf).unwrap();
        let pdf_str = String::from_utf8_lossy(&buf);
        assert!(pdf_str.contains("Test Document"));
    }

    #[test]
    fn test_streaming_pdf_with_all_metadata() {
        let mut builder = PdfDocumentBuilder::new();
        builder.set_title("Full Meta");
        builder.set_author("Jane Doe");
        builder.set_subject("Testing");
        builder.set_creator("ldir");
        builder.add_page(612.0, 792.0);
        let mut buf = Vec::new();
        build_streaming(&builder, &mut buf).unwrap();
        let pdf_str = String::from_utf8_lossy(&buf);
        assert!(pdf_str.contains("Full Meta"));
        assert!(pdf_str.contains("Jane Doe"));
        assert!(pdf_str.contains("Testing"));
        assert!(pdf_str.contains("ldir"));
    }

    #[test]
    fn test_streaming_pdf_with_links() {
        let mut builder = PdfDocumentBuilder::new();
        builder.add_page(612.0, 792.0);
        builder.add_link(
            72.0,
            700.0,
            100.0,
            20.0,
            "https://example.com".to_string(),
            None,
        );
        let mut buf = Vec::new();
        build_streaming(&builder, &mut buf).unwrap();
        let pdf_str = String::from_utf8_lossy(&buf);
        assert!(pdf_str.contains("/Subtype /Link"));
        assert!(pdf_str.contains("https://example.com"));
    }

    #[test]
    fn test_streaming_pdf_with_icc_profile() {
        let mut builder = PdfDocumentBuilder::new();
        builder.set_icc_profile(IccProfile::srgb());
        builder.add_page(612.0, 792.0);
        let mut buf = Vec::new();
        build_streaming(&builder, &mut buf).unwrap();
        let pdf_str = String::from_utf8_lossy(&buf);
        assert!(pdf_str.contains("/OutputIntents"));
        assert!(pdf_str.contains("/ICCBased"));
    }

    #[test]
    fn test_streaming_pdf_multiple_pages() {
        let mut builder = PdfDocumentBuilder::new();
        builder.add_page(612.0, 792.0);
        builder.write_text(72.0, 720.0, "Page 1");
        builder.add_page(612.0, 792.0);
        builder.write_text(72.0, 720.0, "Page 2");
        builder.add_page(612.0, 792.0);
        builder.write_text(72.0, 720.0, "Page 3");
        let mut buf = Vec::new();
        build_streaming(&builder, &mut buf).unwrap();
        assert!(buf.starts_with(b"%PDF"));
        assert!(buf.ends_with(b"%%EOF\n"));
    }

    #[test]
    fn test_streaming_pdf_with_tagged_content() {
        let mut builder = PdfDocumentBuilder::new();
        builder.set_tagged(true);
        builder.add_page(612.0, 792.0);
        builder.write_text(72.0, 720.0, "Hello");
        let mut buf = Vec::new();
        build_streaming(&builder, &mut buf).unwrap();
        let pdf_str = String::from_utf8_lossy(&buf);
        assert!(pdf_str.contains("/Lang"));
        assert!(pdf_str.contains("/Marked true"));
    }

    #[test]
    fn test_streaming_pdf_pdfa4_version() {
        let mut builder = PdfDocumentBuilder::new();
        builder.set_conformance(PdfConformance::PdfA4);
        builder.add_page(612.0, 792.0);
        let mut buf = Vec::new();
        build_streaming(&builder, &mut buf).unwrap();
        let pdf_str = String::from_utf8_lossy(&buf);
        assert!(pdf_str.contains("%PDF-2.0"));
    }

    #[test]
    fn test_streaming_pdf_pdfa2b_version() {
        let mut builder = PdfDocumentBuilder::new();
        builder.set_conformance(PdfConformance::PdfA2b);
        builder.add_page(612.0, 792.0);
        let mut buf = Vec::new();
        build_streaming(&builder, &mut buf).unwrap();
        let pdf_str = String::from_utf8_lossy(&buf);
        assert!(pdf_str.contains("%PDF-1.7"));
    }

    #[test]
    fn test_streaming_pdf_with_structure_tree() {
        let mut builder = PdfDocumentBuilder::new();
        builder.set_tagged(true);
        builder.add_page(612.0, 792.0);
        builder.set_structure_tree(vec![StructureNode::with_children(
            StructureType::Document,
            vec![StructureNode::new(StructureType::Paragraph, 1, 0)],
        )]);
        let mut buf = Vec::new();
        build_streaming(&builder, &mut buf).unwrap();
        let pdf_str = String::from_utf8_lossy(&buf);
        assert!(pdf_str.contains("/StructTreeRoot"));
        assert!(pdf_str.contains("/S /P"));
    }

    #[test]
    fn test_streaming_produces_valid_xref() {
        let mut builder = PdfDocumentBuilder::new();
        builder.add_page(612.0, 792.0);
        let mut buf = Vec::new();
        build_streaming(&builder, &mut buf).unwrap();
        let pdf_str = String::from_utf8_lossy(&buf);
        assert!(pdf_str.contains("xref"));
        assert!(pdf_str.contains("trailer"));
        assert!(pdf_str.contains("startxref"));
    }

    #[test]
    fn test_escape_pdf_string() {
        assert_eq!(escape_pdf_string("hello"), "hello");
        assert_eq!(escape_pdf_string("(test)"), "\\(test\\)");
        assert_eq!(escape_pdf_string("back\\slash"), "back\\\\slash");
    }

    fn make_test_jpeg_data() -> Vec<u8> {
        let mut data = Vec::new();
        // SOI
        data.extend_from_slice(&[0xFF, 0xD8]);
        // APP0 (JFIF header)
        data.extend_from_slice(&[0xFF, 0xE0, 0x00, 0x10]);
        data.extend_from_slice(b"JFIF\x00\x01\x01\x00\x00\x01\x00\x01\x00\x00");
        // SOF0: 8x4, 3 components (RGB)
        data.extend_from_slice(&[0xFF, 0xC0, 0x00, 0x0B, 0x08]);
        data.extend_from_slice(&[0x00, 0x04]); // height = 4
        data.extend_from_slice(&[0x00, 0x08]); // width = 8
        data.push(3); // components = 3 (RGB)
        data.extend_from_slice(&[0x01, 0x11, 0x00]); // Y
        data.extend_from_slice(&[0x02, 0x11, 0x00]); // Cb
        data.extend_from_slice(&[0x03, 0x11, 0x00]); // Cr
        // EOI
        data.extend_from_slice(&[0xFF, 0xD9]);
        data
    }

    fn make_test_png_data() -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut buf, 8, 4);
            encoder.set_color(png::ColorType::Rgb);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().expect("header");
            let pixel_data: Vec<u8> = (0..8 * 4).flat_map(|_| [0xFF, 0x00, 0x00]).collect();
            writer.write_image_data(&pixel_data).expect("write data");
        }
        buf
    }

    #[test]
    fn test_streaming_pdf_link_external() {
        let mut builder = PdfDocumentBuilder::new();
        builder.add_page(612.0, 792.0);
        builder.add_link(
            72.0,
            700.0,
            100.0,
            20.0,
            "https://example.com".to_string(),
            None,
        );
        let mut buf = Vec::new();
        build_streaming(&builder, &mut buf).unwrap();
        let pdf_str = String::from_utf8_lossy(&buf);
        assert!(pdf_str.contains("/Annots ["));
        assert!(pdf_str.contains("/Subtype /Link"));
        assert!(pdf_str.contains("/S /URI"));
        assert!(pdf_str.contains("https://example.com"));
        assert!(pdf_str.contains("/Rect ["));
    }

    #[test]
    fn test_streaming_pdf_link_internal() {
        let mut builder = PdfDocumentBuilder::new();
        builder.add_page(612.0, 792.0);
        builder.add_page(612.0, 792.0);
        builder.add_link(72.0, 700.0, 100.0, 20.0, String::new(), Some(1));
        let mut buf = Vec::new();
        build_streaming(&builder, &mut buf).unwrap();
        let pdf_str = String::from_utf8_lossy(&buf);
        assert!(pdf_str.contains("/Subtype /Link"));
        assert!(pdf_str.contains("/Dest"));
    }

    #[test]
    fn test_streaming_pdf_link_rect_coordinates() {
        let mut builder = PdfDocumentBuilder::new();
        builder.add_page(612.0, 792.0);
        builder.add_link(
            100.0,
            200.0,
            50.0,
            30.0,
            "https://test.com".to_string(),
            None,
        );
        let mut buf = Vec::new();
        build_streaming(&builder, &mut buf).unwrap();
        let pdf_str = String::from_utf8_lossy(&buf);
        assert!(pdf_str.contains("/Rect [100.0000 200.0000 150.0000 230.0000]"));
    }

    #[test]
    fn test_streaming_pdf_embed_jpeg() {
        let jpeg_data = make_test_jpeg_data();
        let mut builder = PdfDocumentBuilder::new();
        builder.add_page(612.0, 792.0);
        builder.set_images(vec![crate::writer::PdfImage {
            data: jpeg_data,
            format: crate::writer::PdfImageFormat::Jpeg,
            alt_text: None,
        }]);
        let mut buf = Vec::new();
        build_streaming(&builder, &mut buf).unwrap();
        let pdf_str = String::from_utf8_lossy(&buf);
        assert!(pdf_str.contains("/Type /XObject"));
        assert!(pdf_str.contains("/Subtype /Image"));
        assert!(pdf_str.contains("/Filter /DctDecode"));
        assert!(pdf_str.contains("/Width 8"));
        assert!(pdf_str.contains("/Height 4"));
        assert!(pdf_str.contains("/ColorSpace /DeviceRGB"));
    }

    #[test]
    fn test_streaming_pdf_draw_image() {
        let jpeg_data = make_test_jpeg_data();
        let mut builder = PdfDocumentBuilder::new();
        builder.add_page(612.0, 792.0);
        builder.set_images(vec![crate::writer::PdfImage {
            data: jpeg_data,
            format: crate::writer::PdfImageFormat::Jpeg,
            alt_text: None,
        }]);
        builder.add_image(72.0, 700.0, 100.0, 50.0, 0);
        let mut buf = Vec::new();
        build_streaming(&builder, &mut buf).unwrap();
        let pdf_str = String::from_utf8_lossy(&buf);
        assert!(pdf_str.contains("/XObject <<"));
        assert!(pdf_str.contains("/Im0"));
    }

    #[test]
    fn test_streaming_pdf_multiple_images() {
        let jpeg1 = make_test_jpeg_data();
        let jpeg2 = make_test_png_data();
        let mut builder = PdfDocumentBuilder::new();
        builder.add_page(612.0, 792.0);
        builder.set_images(vec![
            crate::writer::PdfImage {
                data: jpeg1,
                format: crate::writer::PdfImageFormat::Jpeg,
                alt_text: None,
            },
            crate::writer::PdfImage {
                data: jpeg2,
                format: crate::writer::PdfImageFormat::Png,
                alt_text: None,
            },
        ]);
        builder.add_image(72.0, 700.0, 100.0, 50.0, 0);
        builder.add_image(200.0, 600.0, 80.0, 40.0, 1);
        let mut buf = Vec::new();
        build_streaming(&builder, &mut buf).unwrap();
        let pdf_str = String::from_utf8_lossy(&buf);
        assert!(pdf_str.contains("/Im0"));
        assert!(pdf_str.contains("/Im1"));
    }
}
