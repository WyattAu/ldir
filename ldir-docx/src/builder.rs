use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use ldir_ir::sir::v2::module::SIRModuleV2;
use ldir_ir::sir::v2::nodes::*;

#[derive(Debug, Clone, thiserror::Error)]
/// Errors that can occur during DOCX generation.
pub enum DocxError {
    /// A general build error occurred.
    #[error("DOCX build error: {0}")]
    BuildError(String),
    /// An image reference could not be resolved.
    #[error("image not found: {0}")]
    ImageNotFound(String),
}

#[derive(Debug, Clone)]
struct ImageRef {
    r_id: String,
    source: String,
    content_type: String,
    alt_text: String,
    zip_path: String,
    node_id: u32,
}

impl ImageRef {
    fn is_available(&self, base_dir: &str) -> bool {
        let full_path = Path::new(base_dir).join(&self.source);
        full_path.exists()
    }
}

#[derive(Debug, Clone)]
struct FootnoteEntry {
    id: u32,
    content: String,
}

#[derive(Debug, Clone)]
struct EndnoteEntry {
    id: u32,
    content: String,
}

#[derive(Debug, Clone)]
struct CommentEntry {
    id: u32,
    author: String,
    date: String,
    content: String,
}

struct CollectedNotes {
    footnotes: Vec<FootnoteEntry>,
    endnotes: Vec<EndnoteEntry>,
    comments: Vec<CommentEntry>,
}

#[derive(Debug, Clone)]
/// Builder for generating DOCX documents from S-IR.
pub struct DocxBuilder;

impl Default for DocxBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DocxBuilder {
    /// Create a new `DocxBuilder`.
    pub fn new() -> Self {
        Self
    }

    /// Build a DOCX document from an S-IR module.
    #[must_use = "building DOCX can fail; check the result"]
    pub fn build(&self, module: &SIRModuleV2) -> Result<Vec<u8>, DocxError> {
        self.build_with_base_dir(module, ".")
    }

    /// Build a DOCX document with a specified base directory for resolving image paths.
    #[must_use = "building DOCX can fail; check the result"]
    pub fn build_with_base_dir(
        &self,
        module: &SIRModuleV2,
        base_dir: &str,
    ) -> Result<Vec<u8>, DocxError> {
        let image_refs = self.collect_image_refs(module);

        let image_rid_map: HashMap<u32, &ImageRef> = image_refs
            .iter()
            .filter(|img| img.is_available(base_dir))
            .map(|img| (img.node_id, img))
            .collect();

        let shared_image_map: Arc<HashMap<u32, &ImageRef>> = Arc::new(image_rid_map);

        let mut notes = CollectedNotes {
            footnotes: Vec::new(),
            endnotes: Vec::new(),
            comments: Vec::new(),
        };

        let document_xml =
            self.render_document_with_images_and_notes(module, &shared_image_map, &mut notes);

        let mut content_types_overrides = String::from(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
  <Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
  <Override PartName="/word/numbering.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"/>
  <Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>"#,
        );

        let mut document_rels = String::from(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering" Target="numbering.xml"/>"#,
        );

        let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
</Relationships>"#;

        let styles_xml = self.render_styles();
        let numbering_xml = self.render_numbering();
        let core_xml = self.render_core_properties(module);

        let mut zip = SimpleZip::new();

        for &img in shared_image_map.values() {
            let full_path = Path::new(base_dir).join(&img.source);
            let data = std::fs::read(&full_path).map_err(|e| {
                DocxError::BuildError(format!("failed to read image {}: {e}", img.source))
            })?;
            zip.add_file(&img.zip_path, &data, false);

            content_types_overrides.push_str(&format!(
                "\n  <Override PartName=\"/{}\" ContentType=\"{}\"/>",
                img.zip_path, img.content_type
            ));

            document_rels.push_str(&format!(
                "\n  <Relationship Id=\"{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/image\" Target=\"{}\"/>",
                img.r_id, &img.zip_path[5..]
            ));
        }

        if !notes.footnotes.is_empty() {
            let footnotes_xml = render_footnotes_xml(&notes.footnotes);
            zip.add_file("word/footnotes.xml", footnotes_xml.as_bytes(), false);

            content_types_overrides.push_str(
                "\n  <Override PartName=\"/word/footnotes.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml\"/>",
            );

            let rid = "rIdFootnotes";
            document_rels.push_str(&format!(
                "\n  <Relationship Id=\"{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes\" Target=\"footnotes.xml\"/>",
                rid
            ));
        }

        if !notes.endnotes.is_empty() {
            let endnotes_xml = render_endnotes_xml(&notes.endnotes);
            zip.add_file("word/endnotes.xml", endnotes_xml.as_bytes(), false);

            content_types_overrides.push_str(
                "\n  <Override PartName=\"/word/endnotes.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.endnotes+xml\"/>",
            );

            let rid = "rIdEndnotes";
            document_rels.push_str(&format!(
                "\n  <Relationship Id=\"{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/endnotes\" Target=\"endnotes.xml\"/>",
                rid
            ));
        }

        if !notes.comments.is_empty() {
            let comments_xml = render_comments_xml(&notes.comments);
            zip.add_file("word/comments.xml", comments_xml.as_bytes(), false);

            content_types_overrides.push_str(
                "\n  <Override PartName=\"/word/comments.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml\"/>",
            );

            let rid = "rIdComments";
            document_rels.push_str(&format!(
                "\n  <Relationship Id=\"{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments\" Target=\"comments.xml\"/>",
                rid
            ));
        }

        content_types_overrides.push_str("\n</Types>");
        document_rels.push_str("\n</Relationships>");

        zip.add_file(
            "[Content_Types].xml",
            content_types_overrides.as_bytes(),
            false,
        );
        zip.add_file("_rels/.rels", rels.as_bytes(), false);
        zip.add_file("word/document.xml", document_xml.as_bytes(), false);
        zip.add_file(
            "word/_rels/document.xml.rels",
            document_rels.as_bytes(),
            false,
        );
        zip.add_file("word/styles.xml", styles_xml.as_bytes(), false);
        zip.add_file("word/numbering.xml", numbering_xml.as_bytes(), false);
        zip.add_file("docProps/core.xml", core_xml.as_bytes(), false);

        zip.finish().map_err(DocxError::BuildError)
    }

    fn collect_image_refs(&self, module: &SIRModuleV2) -> Vec<ImageRef> {
        let mut refs = Vec::new();
        let mut counter = 0u32;
        for &root_id in module.body.roots() {
            if let Some(root) = module.body.get(root_id) {
                self.collect_images_walk(module, root, &mut refs, &mut counter);
            }
        }
        refs
    }

    fn collect_images_walk(
        &self,
        module: &SIRModuleV2,
        node: &Node,
        out: &mut Vec<ImageRef>,
        counter: &mut u32,
    ) {
        if let NodeType::Image { source, alt, .. } = &node.node_type {
            *counter += 1;
            let ext = guess_image_extension(source);
            let content_type = extension_to_mime(&ext);
            let r_id = format!("rId{}", *counter + 2);
            let zip_path = format!("word/media/image{}.{}", *counter, ext);

            out.push(ImageRef {
                r_id,
                source: source.clone(),
                content_type,
                alt_text: alt.clone(),
                zip_path,
                node_id: node.id,
            });
        }

        for &child_id in &node.child_ids {
            if let Some(child) = module.body.get(child_id) {
                self.collect_images_walk(module, child, out, counter);
            }
        }
    }

    fn render_document_with_images_and_notes(
        &self,
        module: &SIRModuleV2,
        image_map: &HashMap<u32, &ImageRef>,
        notes: &mut CollectedNotes,
    ) -> String {
        let mut body = String::new();
        for &root_id in module.body.roots() {
            if let Some(root) = module.body.get(root_id) {
                self.render_node_with_images_and_notes(&mut body, module, root, image_map, notes);
            }
        }

        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
            xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
            xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
            xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture">
<w:body>
{body}
</w:body>
</w:document>"#
        )
    }

    fn render_node_with_images_and_notes(
        &self,
        out: &mut String,
        module: &SIRModuleV2,
        node: &Node,
        image_map: &HashMap<u32, &ImageRef>,
        notes: &mut CollectedNotes,
    ) {
        match &node.node_type {
            NodeType::Document => {
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_node_with_images_and_notes(
                            out, module, child, image_map, notes,
                        );
                    }
                }
            }

            NodeType::Part => {
                out.push_str("<w:p><w:pPr><w:pStyle w:val=\"Title\"/></w:pPr>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline_with_images_and_notes(
                            out, module, child, image_map, notes,
                        );
                    }
                }
                out.push_str("</w:p>");
            }

            NodeType::Chapter
            | NodeType::Section
            | NodeType::Subsection
            | NodeType::Subsubsection => {
                let style = match &node.node_type {
                    NodeType::Chapter => "Heading1",
                    NodeType::Section => "Heading2",
                    NodeType::Subsection => "Heading3",
                    NodeType::Subsubsection => "Heading4",
                    _ => "Heading2",
                };
                out.push_str("<w:p><w:pPr><w:pStyle w:val=\"");
                out.push_str(style);
                out.push_str("\"/></w:pPr>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline_with_images_and_notes(
                            out, module, child, image_map, notes,
                        );
                    }
                }
                out.push_str("</w:p>");
            }

            NodeType::Paragraph => {
                out.push_str("<w:p>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline_with_images_and_notes(
                            out, module, child, image_map, notes,
                        );
                    }
                }
                out.push_str("</w:p>");
            }

            NodeType::List { ordered, .. } => {
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_list_item_with_notes(out, module, child, *ordered, notes);
                    }
                }
            }

            NodeType::ListItem => {
                self.render_list_item_with_notes(out, module, node, false, notes);
            }

            NodeType::BlockQuote => {
                out.push_str("<w:p><w:pPr><w:pStyle w:val=\"Quote\"/></w:pPr>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline_with_images_and_notes(
                            out, module, child, image_map, notes,
                        );
                    }
                }
                out.push_str("</w:p>");
            }

            NodeType::CodeBlock { language, .. } => {
                out.push_str("<w:p><w:pPr><w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"F5F5F5\"/></w:pPr>");
                if let Some(lang) = language {
                    out.push_str("<w:r><w:rPr><w:rFonts w:ascii=\"Courier New\" w:hAnsi=\"Courier New\"/><w:color w:val=\"606060\"/></w:rPr><w:t xml:space=\"preserve\">");
                    out.push_str(&format!("[{}]\n", lang));
                    out.push_str("</w:t></w:r>");
                }
                let text = module.body.collect_text(node.id);
                out.push_str("<w:r><w:rPr><w:rFonts w:ascii=\"Courier New\" w:hAnsi=\"Courier New\"/></w:rPr><w:t xml:space=\"preserve\">");
                out.push_str(&escape_xml(&text));
                out.push_str("</w:t></w:r></w:p>");
            }

            NodeType::MathBlock { .. } => {
                let text = module.body.collect_text(node.id);
                out.push_str("<w:p><w:r><w:rPr><w:i/><w:color w:val=\"0000CC\"/></w:rPr><w:t xml:space=\"preserve\">");
                out.push_str(&escape_xml(&text));
                out.push_str("</w:t></w:r></w:p>");
            }

            NodeType::Table { .. } => {
                out.push_str("<w:tbl><w:tblPr><w:tblBorders>");
                out.push_str(
                    "<w:top w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"000000\"/>",
                );
                out.push_str(
                    "<w:left w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"000000\"/>",
                );
                out.push_str(
                    "<w:bottom w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"000000\"/>",
                );
                out.push_str(
                    "<w:right w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"000000\"/>",
                );
                out.push_str(
                    "<w:insideH w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"000000\"/>",
                );
                out.push_str(
                    "<w:insideV w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"000000\"/>",
                );
                out.push_str("</w:tblBorders></w:tblPr>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_table_row(out, module, child);
                    }
                }
                out.push_str("</w:tbl>");
            }

            NodeType::ThematicBreak => {
                out.push_str("<w:p><w:pPr><w:pBdr><w:bottom w:val=\"single\" w:sz=\"6\" w:space=\"1\" w:color=\"999999\"/></w:pBdr></w:pPr></w:p>");
            }

            NodeType::FootnoteBlock => {
                for node in module.body.iter() {
                    if let NodeType::Footnote { content } = &node.node_type {
                        out.push_str("<w:p><w:r><w:rPr><w:sz w:val=\"16\"/></w:rPr><w:t>");
                        out.push_str(&escape_xml(content));
                        out.push_str("</w:t></w:r></w:p>");
                    }
                }
            }

            NodeType::Endnote { content } => {
                let id = notes.endnotes.len() as u32 + 1;
                notes.endnotes.push(EndnoteEntry {
                    id,
                    content: content.clone(),
                });
                out.push_str(&format!("<w:r><w:endnoteReference w:id=\"{}\"/></w:r>", id));
            }

            NodeType::Figure { .. } => {
                out.push_str("<w:p>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        if let NodeType::Image { .. } = &child.node_type {
                            if let Some(img_ref) = image_map.get(&child.id) {
                                emit_inline_image(out, img_ref);
                            } else if let NodeType::Image { alt, .. } = &child.node_type {
                                out.push_str("<w:r><w:t>");
                                out.push_str(&escape_xml(alt));
                                out.push_str(" [image not embedded]</w:t></w:r>");
                            }
                        } else {
                            self.render_node_with_images_and_notes(
                                out, module, child, image_map, notes,
                            );
                        }
                    }
                }
                out.push_str("</w:p>");
            }

            NodeType::Caption => {
                out.push_str("<w:p><w:pPr><w:pStyle w:val=\"Caption\"/></w:pPr>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline_with_images_and_notes(
                            out, module, child, image_map, notes,
                        );
                    }
                }
                out.push_str("</w:p>");
            }

            NodeType::PageBreak => {
                out.push_str("<w:p><w:r><w:br w:type=\"page\"/></w:r></w:p>");
            }

            NodeType::TableOfContents { .. } => {
                out.push_str("<w:p><w:pPr><w:pStyle w:val=\"TOCHeading\"/></w:pPr><w:r><w:t>Table of Contents</w:t></w:r></w:p>");
                out.push_str("<w:p><w:r><w:fldChar w:fldCharType=\"begin\"/></w:r><w:r><w:instrText xml:space=\"preserve\"> TOC \\o \"1-3\" \\h \\z \\u </w:instrText><w:r><w:fldChar w:fldCharType=\"separate\"/></w:r><w:r><w:t>[Update this field in Word to generate TOC]</w:t></w:r><w:r><w:fldChar w:fldCharType=\"end\"/></w:r></w:p>");
            }

            NodeType::Citation { keys, .. } => {
                out.push_str("<w:r><w:rPr><w:vertAlign w:val=\"superscript\"/><w:sz w:val=\"16\"/></w:rPr><w:t>");
                out.push_str(&escape_xml(&keys.join(", ")));
                out.push_str("</w:t></w:r>");
            }

            NodeType::Reference { label } => {
                out.push_str("<w:r><w:rPr><w:color w:val=\"0066CC\"/></w:rPr><w:t>");
                out.push_str(&escape_xml(&format!("[ref: {}]", label)));
                out.push_str("</w:t></w:r>");
            }

            NodeType::TrackedInsert {
                author,
                date,
                revision_id,
            } => {
                out.push_str(&format!(
                    "<w:ins w:id=\"{}\" w:author=\"{}\" w:date=\"{}\">",
                    revision_id,
                    escape_xml(author),
                    date
                ));
                let text = module.body.collect_text(node.id);
                out.push_str("<w:r><w:t>");
                out.push_str(&escape_xml(&text));
                out.push_str("</w:t></w:r>");
                out.push_str("</w:ins>");
            }

            NodeType::TrackedDelete {
                author,
                date,
                revision_id,
            } => {
                out.push_str(&format!(
                    "<w:del w:id=\"{}\" w:author=\"{}\" w:date=\"{}\">",
                    revision_id,
                    escape_xml(author),
                    date
                ));
                let text = module.body.collect_text(node.id);
                out.push_str("<w:r><w:delText>");
                out.push_str(&escape_xml(&text));
                out.push_str("</w:delText></w:r>");
                out.push_str("</w:del>");
            }

            _ => {
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_node_with_images_and_notes(
                            out, module, child, image_map, notes,
                        );
                    }
                }
            }
        }
    }

    fn render_inline_with_images_and_notes(
        &self,
        out: &mut String,
        module: &SIRModuleV2,
        node: &Node,
        image_map: &HashMap<u32, &ImageRef>,
        notes: &mut CollectedNotes,
    ) {
        match &node.node_type {
            NodeType::Text { content } => {
                out.push_str("<w:r><w:t>");
                out.push_str(&escape_xml(content));
                out.push_str("</w:t></w:r>");
            }

            NodeType::Bold => {
                out.push_str("<w:r><w:rPr><w:b/></w:rPr>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline_with_images_and_notes(
                            out, module, child, image_map, notes,
                        );
                    }
                }
                out.push_str("</w:r>");
            }

            NodeType::Italic => {
                out.push_str("<w:r><w:rPr><w:i/></w:rPr>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline_with_images_and_notes(
                            out, module, child, image_map, notes,
                        );
                    }
                }
                out.push_str("</w:r>");
            }

            NodeType::Mono => {
                out.push_str("<w:r><w:rPr><w:rFonts w:ascii=\"Courier New\" w:hAnsi=\"Courier New\"/></w:rPr>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline_with_images_and_notes(
                            out, module, child, image_map, notes,
                        );
                    }
                }
                out.push_str("</w:r>");
            }

            NodeType::Underline => {
                out.push_str("<w:r><w:rPr><w:u w:val=\"single\"/></w:rPr>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline_with_images_and_notes(
                            out, module, child, image_map, notes,
                        );
                    }
                }
                out.push_str("</w:r>");
            }

            NodeType::Strikethrough => {
                out.push_str("<w:r><w:rPr><w:strike/></w:rPr>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline_with_images_and_notes(
                            out, module, child, image_map, notes,
                        );
                    }
                }
                out.push_str("</w:r>");
            }

            NodeType::SmallCaps => {
                out.push_str("<w:r><w:rPr><w:smallCaps/></w:rPr>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline_with_images_and_notes(
                            out, module, child, image_map, notes,
                        );
                    }
                }
                out.push_str("</w:r>");
            }

            NodeType::Link { url, title } => {
                let text = module.body.collect_text(node.id);
                out.push_str(
                    "<w:r><w:rPr><w:color w:val=\"0066CC\"/><w:u w:val=\"single\"/></w:rPr><w:t>",
                );
                out.push_str(&escape_xml(&text));
                out.push_str("</w:t></w:r>");
                if let Some(t) = title {
                    out.push_str("<w:r><w:rPr><w:color w:val=\"0066CC\"/></w:rPr><w:t xml:space=\"preserve\"> (");
                    out.push_str(&escape_xml(t));
                    out.push_str(")</w:t></w:r>");
                }
                out.push_str("<w:r><w:rPr><w:sz w:val=\"16\"/><w:color w:val=\"0066CC\"/></w:rPr><w:t xml:space=\"preserve\"> [");
                out.push_str(&escape_xml(url));
                out.push_str("]</w:t></w:r>");
            }

            NodeType::Image { alt, .. } => {
                if let Some(img_ref) = image_map.get(&node.id) {
                    emit_inline_image(out, img_ref);
                } else {
                    out.push_str("<w:r><w:t>[");
                    out.push_str(&escape_xml(alt));
                    out.push_str("]</w:t></w:r>");
                }
            }

            NodeType::MathInline { content } => {
                out.push_str("<w:r><w:rPr><w:i/></w:rPr><w:t>");
                out.push_str(&escape_xml(content));
                out.push_str("</w:t></w:r>");
            }

            NodeType::LineBreak => {
                out.push_str("<w:r><w:br/></w:r>");
            }

            NodeType::Footnote { content } => {
                let id = notes.footnotes.len() as u32 + 1;
                notes.footnotes.push(FootnoteEntry {
                    id,
                    content: content.clone(),
                });
                out.push_str(&format!(
                    "<w:r><w:footnoteReference w:id=\"{}\"/></w:r>",
                    id
                ));
            }

            NodeType::Endnote { content } => {
                let id = notes.endnotes.len() as u32 + 1;
                notes.endnotes.push(EndnoteEntry {
                    id,
                    content: content.clone(),
                });
                out.push_str(&format!("<w:r><w:endnoteReference w:id=\"{}\"/></w:r>", id));
            }

            NodeType::Comment { author, content } => {
                let id = notes.comments.len() as u32;
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                notes.comments.push(CommentEntry {
                    id,
                    author: author.clone(),
                    date: format_iso8601(now),
                    content: content.clone(),
                });
                out.push_str(&format!("<w:commentRangeStart w:id=\"{}\"/>", id));
                out.push_str("<w:r><w:rPr><w:highlight w:val=\"yellow\"/></w:rPr><w:t>");
                out.push_str(&escape_xml(content));
                out.push_str("</w:t></w:r>");
                out.push_str(&format!("<w:commentRangeEnd w:id=\"{}\"/>", id));
                out.push_str(&format!("<w:r><w:commentReference w:id=\"{}\"/></w:r>", id));
            }

            NodeType::Styled { .. } | NodeType::Group => {
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline_with_images_and_notes(
                            out, module, child, image_map, notes,
                        );
                    }
                }
            }

            _ => {
                let text = module.body.collect_text(node.id);
                if !text.is_empty() {
                    out.push_str("<w:r><w:t>");
                    out.push_str(&escape_xml(&text));
                    out.push_str("</w:t></w:r>");
                }
            }
        }
    }

    fn render_list_item_with_notes(
        &self,
        out: &mut String,
        module: &SIRModuleV2,
        node: &Node,
        ordered: bool,
        notes: &mut CollectedNotes,
    ) {
        let num_id = if ordered { "2" } else { "1" };
        out.push_str(&format!(
            "<w:p><w:pPr><w:pStyle w:val=\"ListParagraph\"/><w:numPr><w:ilvl w:val=\"0\"/><w:numId w:val=\"{num_id}\"/></w:numPr></w:pPr>"
        ));
        for &child_id in &node.child_ids {
            if let Some(child) = module.body.get(child_id) {
                self.render_inline_with_notes(out, module, child, notes);
            }
        }
        out.push_str("</w:p>");
    }

    fn render_inline(&self, out: &mut String, module: &SIRModuleV2, node: &Node) {
        match &node.node_type {
            NodeType::Text { content } => {
                out.push_str("<w:r><w:t>");
                out.push_str(&escape_xml(content));
                out.push_str("</w:t></w:r>");
            }

            NodeType::Bold => {
                out.push_str("<w:r><w:rPr><w:b/></w:rPr>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline(out, module, child);
                    }
                }
                out.push_str("</w:r>");
            }

            NodeType::Italic => {
                out.push_str("<w:r><w:rPr><w:i/></w:rPr>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline(out, module, child);
                    }
                }
                out.push_str("</w:r>");
            }

            NodeType::Mono => {
                out.push_str("<w:r><w:rPr><w:rFonts w:ascii=\"Courier New\" w:hAnsi=\"Courier New\"/></w:rPr>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline(out, module, child);
                    }
                }
                out.push_str("</w:r>");
            }

            NodeType::Underline => {
                out.push_str("<w:r><w:rPr><w:u w:val=\"single\"/></w:rPr>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline(out, module, child);
                    }
                }
                out.push_str("</w:r>");
            }

            NodeType::Strikethrough => {
                out.push_str("<w:r><w:rPr><w:strike/></w:rPr>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline(out, module, child);
                    }
                }
                out.push_str("</w:r>");
            }

            NodeType::SmallCaps => {
                out.push_str("<w:r><w:rPr><w:smallCaps/></w:rPr>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline(out, module, child);
                    }
                }
                out.push_str("</w:r>");
            }

            NodeType::Link { url, title } => {
                let text = module.body.collect_text(node.id);
                out.push_str(
                    "<w:r><w:rPr><w:color w:val=\"0066CC\"/><w:u w:val=\"single\"/></w:rPr><w:t>",
                );
                out.push_str(&escape_xml(&text));
                out.push_str("</w:t></w:r>");
                if let Some(t) = title {
                    out.push_str("<w:r><w:rPr><w:color w:val=\"0066CC\"/></w:rPr><w:t xml:space=\"preserve\"> (");
                    out.push_str(&escape_xml(t));
                    out.push_str(")</w:t></w:r>");
                }
                out.push_str("<w:r><w:rPr><w:sz w:val=\"16\"/><w:color w:val=\"0066CC\"/></w:rPr><w:t xml:space=\"preserve\"> [");
                out.push_str(&escape_xml(url));
                out.push_str("]</w:t></w:r>");
            }

            NodeType::Image { alt, .. } => {
                out.push_str("<w:r><w:t>[");
                out.push_str(&escape_xml(alt));
                out.push_str("]</w:t></w:r>");
            }

            NodeType::MathInline { content } => {
                out.push_str("<w:r><w:rPr><w:i/></w:rPr><w:t>");
                out.push_str(&escape_xml(content));
                out.push_str("</w:t></w:r>");
            }

            NodeType::LineBreak => {
                out.push_str("<w:r><w:br/></w:r>");
            }

            NodeType::Footnote { content } => {
                out.push_str("<w:r><w:rPr><w:vertAlign w:val=\"superscript\"/><w:sz w:val=\"16\"/></w:rPr><w:t>");
                out.push_str(&escape_xml(content));
                out.push_str("</w:t></w:r>");
            }

            NodeType::Styled { .. } | NodeType::Group => {
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline(out, module, child);
                    }
                }
            }

            _ => {
                let text = module.body.collect_text(node.id);
                if !text.is_empty() {
                    out.push_str("<w:r><w:t>");
                    out.push_str(&escape_xml(&text));
                    out.push_str("</w:t></w:r>");
                }
            }
        }
    }

    fn render_inline_with_notes(
        &self,
        out: &mut String,
        module: &SIRModuleV2,
        node: &Node,
        notes: &mut CollectedNotes,
    ) {
        match &node.node_type {
            NodeType::Text { content } => {
                out.push_str("<w:r><w:t>");
                out.push_str(&escape_xml(content));
                out.push_str("</w:t></w:r>");
            }

            NodeType::Bold => {
                out.push_str("<w:r><w:rPr><w:b/></w:rPr>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline_with_notes(out, module, child, notes);
                    }
                }
                out.push_str("</w:r>");
            }

            NodeType::Italic => {
                out.push_str("<w:r><w:rPr><w:i/></w:rPr>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline_with_notes(out, module, child, notes);
                    }
                }
                out.push_str("</w:r>");
            }

            NodeType::Mono => {
                out.push_str("<w:r><w:rPr><w:rFonts w:ascii=\"Courier New\" w:hAnsi=\"Courier New\"/></w:rPr>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline_with_notes(out, module, child, notes);
                    }
                }
                out.push_str("</w:r>");
            }

            NodeType::Underline => {
                out.push_str("<w:r><w:rPr><w:u w:val=\"single\"/></w:rPr>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline_with_notes(out, module, child, notes);
                    }
                }
                out.push_str("</w:r>");
            }

            NodeType::Strikethrough => {
                out.push_str("<w:r><w:rPr><w:strike/></w:rPr>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline_with_notes(out, module, child, notes);
                    }
                }
                out.push_str("</w:r>");
            }

            NodeType::SmallCaps => {
                out.push_str("<w:r><w:rPr><w:smallCaps/></w:rPr>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline_with_notes(out, module, child, notes);
                    }
                }
                out.push_str("</w:r>");
            }

            NodeType::Link { url, title } => {
                let text = module.body.collect_text(node.id);
                out.push_str(
                    "<w:r><w:rPr><w:color w:val=\"0066CC\"/><w:u w:val=\"single\"/></w:rPr><w:t>",
                );
                out.push_str(&escape_xml(&text));
                out.push_str("</w:t></w:r>");
                if let Some(t) = title {
                    out.push_str("<w:r><w:rPr><w:color w:val=\"0066CC\"/></w:rPr><w:t xml:space=\"preserve\"> (");
                    out.push_str(&escape_xml(t));
                    out.push_str(")</w:t></w:r>");
                }
                out.push_str("<w:r><w:rPr><w:sz w:val=\"16\"/><w:color w:val=\"0066CC\"/></w:rPr><w:t xml:space=\"preserve\"> [");
                out.push_str(&escape_xml(url));
                out.push_str("]</w:t></w:r>");
            }

            NodeType::Image { alt, .. } => {
                out.push_str("<w:r><w:t>[");
                out.push_str(&escape_xml(alt));
                out.push_str("]</w:t></w:r>");
            }

            NodeType::MathInline { content } => {
                out.push_str("<w:r><w:rPr><w:i/></w:rPr><w:t>");
                out.push_str(&escape_xml(content));
                out.push_str("</w:t></w:r>");
            }

            NodeType::LineBreak => {
                out.push_str("<w:r><w:br/></w:r>");
            }

            NodeType::Footnote { content } => {
                let id = notes.footnotes.len() as u32 + 1;
                notes.footnotes.push(FootnoteEntry {
                    id,
                    content: content.clone(),
                });
                out.push_str(&format!(
                    "<w:r><w:footnoteReference w:id=\"{}\"/></w:r>",
                    id
                ));
            }

            NodeType::Endnote { content } => {
                let id = notes.endnotes.len() as u32 + 1;
                notes.endnotes.push(EndnoteEntry {
                    id,
                    content: content.clone(),
                });
                out.push_str(&format!("<w:r><w:endnoteReference w:id=\"{}\"/></w:r>", id));
            }

            NodeType::Comment { author, content } => {
                let id = notes.comments.len() as u32;
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                notes.comments.push(CommentEntry {
                    id,
                    author: author.clone(),
                    date: format_iso8601(now),
                    content: content.clone(),
                });
                out.push_str(&format!("<w:commentRangeStart w:id=\"{}\"/>", id));
                out.push_str("<w:r><w:rPr><w:highlight w:val=\"yellow\"/></w:rPr><w:t>");
                out.push_str(&escape_xml(content));
                out.push_str("</w:t></w:r>");
                out.push_str(&format!("<w:commentRangeEnd w:id=\"{}\"/>", id));
                out.push_str(&format!("<w:r><w:commentReference w:id=\"{}\"/></w:r>", id));
            }

            NodeType::Styled { .. } | NodeType::Group => {
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline_with_notes(out, module, child, notes);
                    }
                }
            }

            _ => {
                let text = module.body.collect_text(node.id);
                if !text.is_empty() {
                    out.push_str("<w:r><w:t>");
                    out.push_str(&escape_xml(&text));
                    out.push_str("</w:t></w:r>");
                }
            }
        }
    }

    fn render_table_row(&self, out: &mut String, module: &SIRModuleV2, node: &Node) {
        let is_header = matches!(&node.node_type, NodeType::TableRow { is_header: true });
        out.push_str("<w:tr>");
        if is_header {
            out.push_str("<w:trPr><w:tblHeader/></w:trPr>");
        }
        for &child_id in &node.child_ids {
            if let Some(child) = module.body.get(child_id)
                && matches!(&child.node_type, NodeType::TableCell { .. })
            {
                out.push_str("<w:tc><w:p>");
                if is_header {
                    out.push_str("<w:pPr><w:jc w:val=\"center\"/></w:pPr>");
                }
                for &cell_child_id in &child.child_ids {
                    if let Some(cell_child) = module.body.get(cell_child_id) {
                        if is_header {
                            out.push_str("<w:r><w:rPr><w:b/></w:rPr>");
                            self.render_inline(out, module, cell_child);
                            out.push_str("</w:r>");
                        } else {
                            self.render_inline(out, module, cell_child);
                        }
                    }
                }
                out.push_str("</w:p></w:tc>");
            }
        }
        out.push_str("</w:tr>");
    }

    fn render_styles(&self) -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:style w:type="paragraph" w:default="1" w:styleId="Normal">
    <w:name w:val="Normal"/>
    <w:rPr><w:sz w:val="22"/></w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Title">
    <w:name w:val="Title"/>
    <w:basedOn w:val="Normal"/>
    <w:pPr><w:spacing w:before="480" w:after="240"/><w:jc w:val="center"/></w:pPr>
    <w:rPr><w:b/><w:sz w:val="56"/><w:color w:val="2E74B5"/></w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Heading1">
    <w:name w:val="heading 1"/>
    <w:basedOn w:val="Normal"/>
    <w:next w:val="Normal"/>
    <w:pPr><w:keepNext/><w:spacing w:before="240" w:after="120"/></w:pPr>
    <w:rPr><w:b/><w:sz w:val="48"/><w:color w:val="2E74B5"/></w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Heading2">
    <w:name w:val="heading 2"/>
    <w:basedOn w:val="Normal"/>
    <w:next w:val="Normal"/>
    <w:pPr><w:keepNext/><w:spacing w:before="200" w:after="100"/></w:pPr>
    <w:rPr><w:b/><w:sz w:val="36"/><w:color w:val="2E74B5"/></w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Heading3">
    <w:name w:val="heading 3"/>
    <w:basedOn w:val="Normal"/>
    <w:next w:val="Normal"/>
    <w:pPr><w:keepNext/><w:spacing w:before="160" w:after="80"/></w:pPr>
    <w:rPr><w:b/><w:sz w:val="28"/><w:color w:val="2E74B5"/></w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Heading4">
    <w:name w:val="heading 4"/>
    <w:basedOn w:val="Normal"/>
    <w:next w:val="Normal"/>
    <w:pPr><w:keepNext/><w:spacing w:before="120" w:after="60"/></w:pPr>
    <w:rPr><w:b/><w:sz w:val="24"/><w:color w:val="2E74B5"/></w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="ListParagraph">
    <w:name w:val="List Paragraph"/>
    <w:basedOn w:val="Normal"/>
    <w:pPr><w:ind w:left="720" w:hanging="360"/></w:pPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Quote">
    <w:name w:val="Quote"/>
    <w:basedOn w:val="Normal"/>
    <w:pPr><w:ind w:left="720"/></w:pPr>
    <w:rPr><w:i/><w:color w:val="404040"/></w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Caption">
    <w:name w:val="Caption"/>
    <w:basedOn w:val="Normal"/>
    <w:pPr><w:spacing w:before="120" w:after="120"/></w:pPr>
    <w:rPr><w:i/><w:sz w:val="18"/><w:color w:val="404040"/></w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="TOCHeading">
    <w:name w:val="TOC Heading"/>
    <w:basedOn w:val="Heading1"/>
  </w:style>
</w:styles>"#
            .to_string()
    }

    fn render_numbering(&self) -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:abstractNum w:abstractNumId="0">
    <w:lvl w:ilvl="0"><w:start w:val="1"/><w:numFmt w:val="bullet"/><w:lvlText w:val="&#x2022;"/><w:lvlJc w:val="left"/></w:lvl>
    <w:lvl w:ilvl="1"><w:start w:val="1"/><w:numFmt w:val="bullet"/><w:lvlText w:val="-"/><w:lvlJc w:val="left"/></w:lvl>
  </w:abstractNum>
  <w:abstractNum w:abstractNumId="1">
    <w:lvl w:ilvl="0"><w:start w:val="1"/><w:numFmt w:val="decimal"/><w:lvlText w:val="%1."/><w:lvlJc w:val="left"/></w:lvl>
    <w:lvl w:ilvl="1"><w:start w:val="1"/><w:numFmt w:val="lowerLetter"/><w:lvlText w:val="%2)"/><w:lvlJc w:val="left"/></w:lvl>
  </w:abstractNum>
  <w:num w:numId="1"><w:abstractNumId w:val="0"/></w:num>
  <w:num w:numId="2"><w:abstractNumId w:val="1"/></w:num>
</w:numbering>"#.to_string()
    }

    fn render_core_properties(&self, module: &SIRModuleV2) -> String {
        let title = module.metadata.title.as_deref().unwrap_or("");
        let author = module.metadata.author.as_deref().unwrap_or("");
        let subject = module.metadata.subject.as_deref().unwrap_or("");
        let _date = module.metadata.date.as_deref().unwrap_or("");
        let created = if module.header.created > 0 {
            format_iso8601(module.header.created)
        } else {
            String::new()
        };

        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:dcmitype="http://purl.org/dc/dcmitype/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <dc:title>{title}</dc:title>
  <dc:creator>{author}</dc:creator>
  <dc:subject>{subject}</dc:subject>
  <dcterms:created xsi:type="dcterms:W3CDTF">{created}</dcterms:created>
</cp:coreProperties>"#
        )
    }
}

fn guess_image_extension(source: &str) -> String {
    let path = Path::new(source);
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
    {
        Some(ext)
            if matches!(
                ext.as_str(),
                "png"
                    | "jpg"
                    | "jpeg"
                    | "gif"
                    | "bmp"
                    | "svg"
                    | "tiff"
                    | "tif"
                    | "webp"
                    | "emf"
                    | "wmf"
            ) =>
        {
            if ext == "jpg" {
                "jpg".to_string()
            } else {
                ext
            }
        }
        _ => "png".to_string(),
    }
}

fn extension_to_mime(ext: &str) -> String {
    match ext {
        "png" => "image/png".to_string(),
        "jpg" | "jpeg" => "image/jpeg".to_string(),
        "gif" => "image/gif".to_string(),
        "bmp" => "image/bmp".to_string(),
        "svg" => "image/svg+xml".to_string(),
        "tiff" | "tif" => "image/tiff".to_string(),
        "webp" => "image/webp".to_string(),
        "emf" => "image/x-emf".to_string(),
        "wmf" => "image/x-wmf".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}

fn emit_inline_image(out: &mut String, img: &ImageRef) {
    out.push_str("<w:r>");
    out.push_str("<w:drawing>");
    out.push_str("<wp:inline distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\">");
    out.push_str("<wp:extent cx=\"457200\" cy=\"457200\"/>");
    out.push_str("<wp:effectExtent l=\"0\" t=\"0\" r=\"0\" b=\"0\"/>");
    out.push_str("<wp:docPr id=\"0\" name=\"Picture\"/>");
    out.push_str("<wp:cNvGraphicFramePr><a:graphicFrameLocks xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" noChangeAspect=\"1\"/></wp:cNvGraphicFramePr>");
    out.push_str("<a:graphic xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">");
    out.push_str(
        "<a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/picture\">",
    );
    out.push_str(
        "<pic:pic xmlns:pic=\"http://schemas.openxmlformats.org/drawingml/2006/picture\">",
    );
    out.push_str("<pic:nvPicPr>");
    out.push_str(&format!(
        "<pic:cNvPr id=\"0\" name=\"{}\"/>",
        escape_xml(&img.alt_text)
    ));
    out.push_str("<pic:cNvPicPr/>");
    out.push_str("</pic:nvPicPr>");
    out.push_str("<pic:blipFill>");
    out.push_str("<a:blip r:embed=\"");
    out.push_str(&img.r_id);
    out.push_str(
        "\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\"/>",
    );
    out.push_str("<a:stretch><a:fillRect/></a:stretch>");
    out.push_str("</pic:blipFill>");
    out.push_str("<pic:spPr>");
    out.push_str("<a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"457200\" cy=\"457200\"/></a:xfrm>");
    out.push_str("<a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom>");
    out.push_str("</pic:spPr>");
    out.push_str("</pic:pic>");
    out.push_str("</a:graphicData>");
    out.push_str("</a:graphic>");
    out.push_str("</wp:inline>");
    out.push_str("</w:drawing>");
    out.push_str("</w:r>");
}

fn escape_xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

fn render_footnotes_xml(footnotes: &[FootnoteEntry]) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:footnote w:type="separator" w:id="-1">
    <w:p><w:separator/></w:p>
  </w:footnote>
  <w:footnote w:type="continuationSeparator" w:id="0">
    <w:p><w:continuationSeparator/></w:p>
  </w:footnote>"#,
    );
    for fn_entry in footnotes {
        xml.push_str(&format!(
            r#"
  <w:footnote w:id="{}">
    <w:p>
      <w:r><w:rPr><w:vertAlign w:val="superscript"/></w:rPr><w:t>{}</w:t></w:r>
    </w:p>
  </w:footnote>"#,
            fn_entry.id,
            escape_xml(&fn_entry.content)
        ));
    }
    xml.push_str("\n</w:footnotes>");
    xml
}

fn render_endnotes_xml(endnotes: &[EndnoteEntry]) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:endnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:endnote w:type="separator" w:id="-1">
    <w:p><w:separator/></w:p>
  </w:endnote>
  <w:endnote w:type="continuationSeparator" w:id="0">
    <w:p><w:continuationSeparator/></w:p>
  </w:endnote>"#,
    );
    for en_entry in endnotes {
        xml.push_str(&format!(
            r#"
  <w:endnote w:id="{}">
    <w:p>
      <w:r><w:rPr><w:vertAlign w:val="superscript"/></w:rPr><w:t>{}</w:t></w:r>
    </w:p>
  </w:endnote>"#,
            en_entry.id,
            escape_xml(&en_entry.content)
        ));
    }
    xml.push_str("\n</w:endnotes>");
    xml
}

fn render_comments_xml(comments: &[CommentEntry]) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">"#,
    );
    for c in comments {
        xml.push_str(&format!(
            r#"
  <w:comment w:id="{}" w:author="{}" w:date="{}">
    <w:p><w:r><w:t>{}</w:t></w:r></w:p>
  </w:comment>"#,
            c.id,
            escape_xml(&c.author),
            c.date,
            escape_xml(&c.content)
        ));
    }
    xml.push_str("\n</w:comments>");
    xml
}

fn format_iso8601(secs: u64) -> String {
    let secs = secs as i64;
    let year = 1970 + secs / 31_536_000;
    let rem = secs % 31_536_000;
    let month = 1 + rem / 2_592_000;
    let rem = rem % 2_592_000;
    let day = 1 + rem / 86_400;
    format!("{year:04}-{month:02}-{day:02}T00:00:00Z")
}

struct SimpleZip {
    files: Vec<(String, Vec<u8>, bool)>,
}

impl SimpleZip {
    fn new() -> Self {
        Self { files: Vec::new() }
    }

    fn add_file(&mut self, path: &str, data: &[u8], stored: bool) {
        self.files.push((path.to_string(), data.to_vec(), stored));
    }

    fn finish(&self) -> Result<Vec<u8>, String> {
        let mut buf = Vec::new();
        let mut central_offsets: Vec<(u64, u32, u32, String)> = Vec::new();

        for (path, data, stored) in &self.files {
            let offset = buf.len() as u64;
            let crc = crc32(data);
            let size = data.len() as u32;

            buf.extend_from_slice(&[0x50, 0x4B, 0x03, 0x04]);
            buf.extend_from_slice(&[20, 0]);
            buf.extend_from_slice(&[0, 0]);
            buf.extend_from_slice(if *stored { &[0, 0] } else { &[8, 0] });
            buf.extend_from_slice(&[0, 0]);
            buf.extend_from_slice(&[0, 0]);
            buf.extend_from_slice(&crc.to_le_bytes());
            buf.extend_from_slice(&size.to_le_bytes());
            buf.extend_from_slice(&size.to_le_bytes());
            let name_bytes = path.as_bytes();
            buf.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
            buf.extend_from_slice(&[0u8; 10]);
            buf.extend_from_slice(name_bytes);
            buf.extend_from_slice(data);

            central_offsets.push((offset, size, crc, path.clone()));
        }

        let central_start = buf.len() as u64;

        for (offset, size, crc, name) in &central_offsets {
            buf.extend_from_slice(&[0x50, 0x4B, 0x01, 0x02]);
            buf.extend_from_slice(&[20, 0]);
            buf.extend_from_slice(&[20, 0]);
            buf.extend_from_slice(&[0, 0]);
            buf.extend_from_slice(&[0, 0]);
            buf.extend_from_slice(&[0, 0]);
            buf.extend_from_slice(&[0, 0]);
            buf.extend_from_slice(&crc.to_le_bytes());
            buf.extend_from_slice(&size.to_le_bytes());
            buf.extend_from_slice(&size.to_le_bytes());
            let name_bytes = name.as_bytes();
            buf.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
            buf.extend_from_slice(&[0u8; 10]);
            buf.extend_from_slice(&[0, 0, 0]);
            buf.extend_from_slice(&(*offset as u32).to_le_bytes());
            buf.extend_from_slice(name_bytes);
        }

        let central_end = buf.len() as u64;
        let central_count = central_offsets.len() as u16;

        buf.extend_from_slice(&[0x50, 0x4B, 0x05, 0x06]);
        buf.extend_from_slice(&[0, 0]);
        buf.extend_from_slice(&[0, 0]);
        buf.extend_from_slice(&central_count.to_le_bytes());
        buf.extend_from_slice(&central_count.to_le_bytes());
        buf.extend_from_slice(&(central_start as u32).to_le_bytes());
        buf.extend_from_slice(&(central_end as u32).to_le_bytes());
        buf.extend_from_slice(&[0u8; 2]);

        Ok(buf)
    }
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_simple_module() -> SIRModuleV2 {
        let mut m = SIRModuleV2::new();
        m.metadata.title = Some("Test Document".into());
        m.body.push(Node::new(0, NodeType::Document));
        m.body.push(Node::new(1, NodeType::Section).with_parent(0));
        m.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "Chapter 1".into(),
                },
            )
            .with_parent(1),
        );
        m.body
            .push(Node::new(3, NodeType::Paragraph).with_parent(0));
        m.body.push(
            Node::new(
                4,
                NodeType::Text {
                    content: "Hello DOCX!".into(),
                },
            )
            .with_parent(3),
        );
        if let Some(n) = m.body.get_mut(0) {
            n.add_child(1);
        }
        if let Some(n) = m.body.get_mut(0) {
            n.add_child(3);
        }
        if let Some(n) = m.body.get_mut(1) {
            n.add_child(2);
        }
        if let Some(n) = m.body.get_mut(3) {
            n.add_child(4);
        }
        m
    }

    #[test]
    fn test_docx_builds() -> Result<(), Box<dyn std::error::Error>> {
        let m = make_simple_module();
        let docx = DocxBuilder::new().build(&m)?;
        assert!(docx.len() > 100);
        Ok(())
    }

    #[test]
    fn test_docx_starts_with_pk() -> Result<(), Box<dyn std::error::Error>> {
        let m = make_simple_module();
        let docx = DocxBuilder::new().build(&m)?;
        assert_eq!(&docx[0..4], b"PK\x03\x04");
        Ok(())
    }

    #[test]
    fn test_docx_contains_content_types() -> Result<(), Box<dyn std::error::Error>> {
        let m = make_simple_module();
        let docx = DocxBuilder::new().build(&m)?;
        let text = String::from_utf8_lossy(&docx);
        assert!(text.contains("[Content_Types].xml"));
        Ok(())
    }

    #[test]
    fn test_docx_contains_document_xml() -> Result<(), Box<dyn std::error::Error>> {
        let m = make_simple_module();
        let docx = DocxBuilder::new().build(&m)?;
        let text = String::from_utf8_lossy(&docx);
        assert!(text.contains("word/document.xml"));
        assert!(text.contains("Hello DOCX!"));
        Ok(())
    }

    #[test]
    fn test_docx_contains_heading() -> Result<(), Box<dyn std::error::Error>> {
        let m = make_simple_module();
        let docx = DocxBuilder::new().build(&m)?;
        let text = String::from_utf8_lossy(&docx);
        assert!(text.contains("Chapter 1"));
        assert!(text.contains("Heading2"));
        Ok(())
    }

    #[test]
    fn test_docx_bold_italic() -> Result<(), Box<dyn std::error::Error>> {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::Paragraph).with_parent(0));
        m.body.push(Node::new(2, NodeType::Bold).with_parent(1));
        m.body.push(
            Node::new(
                3,
                NodeType::Text {
                    content: "bold".into(),
                },
            )
            .with_parent(2),
        );
        m.body.push(Node::new(4, NodeType::Italic).with_parent(1));
        m.body.push(
            Node::new(
                5,
                NodeType::Text {
                    content: "italic".into(),
                },
            )
            .with_parent(4),
        );
        if let Some(n) = m.body.get_mut(0) {
            n.add_child(1);
        }
        if let Some(n) = m.body.get_mut(1) {
            n.add_child(2);
        }
        if let Some(n) = m.body.get_mut(1) {
            n.add_child(4);
        }
        if let Some(n) = m.body.get_mut(2) {
            n.add_child(3);
        }
        if let Some(n) = m.body.get_mut(4) {
            n.add_child(5);
        }

        let docx = DocxBuilder::new().build(&m)?;
        let text = String::from_utf8_lossy(&docx);
        assert!(text.contains("<w:b/>"));
        assert!(text.contains("<w:i/>"));
        assert!(text.contains("bold"));
        assert!(text.contains("italic"));
        Ok(())
    }

    #[test]
    fn test_docx_list() -> Result<(), Box<dyn std::error::Error>> {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body.push(
            Node::new(
                1,
                NodeType::List {
                    list_type: ListType::Unordered,
                    ordered: false,
                    start: None,
                },
            )
            .with_parent(0),
        );
        m.body.push(Node::new(2, NodeType::ListItem).with_parent(1));
        m.body.push(
            Node::new(
                3,
                NodeType::Text {
                    content: "item 1".into(),
                },
            )
            .with_parent(2),
        );
        if let Some(n) = m.body.get_mut(0) {
            n.add_child(1);
        }
        if let Some(n) = m.body.get_mut(1) {
            n.add_child(2);
        }
        if let Some(n) = m.body.get_mut(2) {
            n.add_child(3);
        }

        let docx = DocxBuilder::new().build(&m)?;
        let text = String::from_utf8_lossy(&docx);
        assert!(text.contains("item 1"));
        assert!(text.contains("<w:numPr>"));
        Ok(())
    }

    #[test]
    fn test_docx_table() -> Result<(), Box<dyn std::error::Error>> {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body.push(
            Node::new(
                1,
                NodeType::Table {
                    col_specs: vec![],
                    num_cols: 2,
                    caption: None,
                    column_widths: vec![],
                    header_row: false,
                },
            )
            .with_parent(0),
        );
        m.body
            .push(Node::new(2, NodeType::TableRow { is_header: true }).with_parent(1));
        m.body.push(
            Node::new(
                3,
                NodeType::TableCell {
                    colspan: 1,
                    rowspan: 1,
                },
            )
            .with_parent(2),
        );
        m.body.push(
            Node::new(
                4,
                NodeType::Text {
                    content: "Cell1".into(),
                },
            )
            .with_parent(3),
        );
        if let Some(n) = m.body.get_mut(0) {
            n.add_child(1);
        }
        if let Some(n) = m.body.get_mut(1) {
            n.add_child(2);
        }
        if let Some(n) = m.body.get_mut(2) {
            n.add_child(3);
        }
        if let Some(n) = m.body.get_mut(3) {
            n.add_child(4);
        }

        let docx = DocxBuilder::new().build(&m)?;
        let text = String::from_utf8_lossy(&docx);
        assert!(text.contains("<w:tbl>"));
        assert!(text.contains("<w:tc>"));
        assert!(text.contains("Cell1"));
        Ok(())
    }

    #[test]
    fn test_docx_code_block() -> Result<(), Box<dyn std::error::Error>> {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body.push(
            Node::new(
                1,
                NodeType::CodeBlock {
                    language: Some("rust".into()),
                    content: String::new(),
                },
            )
            .with_parent(0),
        );
        m.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "fn main() {}".into(),
                },
            )
            .with_parent(1),
        );
        if let Some(n) = m.body.get_mut(0) {
            n.add_child(1);
        }
        if let Some(n) = m.body.get_mut(1) {
            n.add_child(2);
        }

        let docx = DocxBuilder::new().build(&m)?;
        let text = String::from_utf8_lossy(&docx);
        assert!(text.contains("Courier New"));
        assert!(text.contains("fn main()"));
        Ok(())
    }

    #[test]
    fn test_docx_thematic_break() -> Result<(), Box<dyn std::error::Error>> {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::ThematicBreak).with_parent(0));
        if let Some(n) = m.body.get_mut(0) {
            n.add_child(1);
        }

        let docx = DocxBuilder::new().build(&m)?;
        let text = String::from_utf8_lossy(&docx);
        assert!(text.contains("<w:pBdr>"));
        Ok(())
    }

    #[test]
    fn test_docx_xml_escaping() -> Result<(), Box<dyn std::error::Error>> {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::Paragraph).with_parent(0));
        m.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "a < b & c > d".into(),
                },
            )
            .with_parent(1),
        );
        if let Some(n) = m.body.get_mut(0) {
            n.add_child(1);
        }
        if let Some(n) = m.body.get_mut(1) {
            n.add_child(2);
        }

        let docx = DocxBuilder::new().build(&m)?;
        let text = String::from_utf8_lossy(&docx);
        assert!(text.contains("&lt;"));
        assert!(text.contains("&amp;"));
        assert!(text.contains("&gt;"));
        assert!(!text.contains("a < b"));
        Ok(())
    }

    #[test]
    fn test_crc32() {
        assert_eq!(crc32(b""), 0x00000000);
        assert_eq!(crc32(b"123456789"), 0xCBF43926);
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
        assert_eq!(escape_xml("a & b"), "a &amp; b");
        assert_eq!(escape_xml("\"q\""), "&quot;q&quot;");
        assert_eq!(escape_xml("it's"), "it&apos;s");
    }

    #[test]
    fn test_docx_empty_module() -> Result<(), Box<dyn std::error::Error>> {
        let m = SIRModuleV2::new();
        let docx = DocxBuilder::new().build(&m)?;
        assert!(docx.len() > 100);
        assert_eq!(&docx[0..4], b"PK\x03\x04");
        Ok(())
    }

    #[test]
    fn test_collect_image_refs() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::Paragraph).with_parent(0));
        m.body.push(
            Node::new(
                2,
                NodeType::Image {
                    source: "photo.png".into(),
                    alt: "A photo".into(),
                    width: None,
                    height: None,
                    placement: FloatPlacement::Top,
                },
            )
            .with_parent(1),
        );
        m.body.push(
            Node::new(
                3,
                NodeType::Image {
                    source: "diagram.jpg".into(),
                    alt: "A diagram".into(),
                    width: None,
                    height: None,
                    placement: FloatPlacement::Top,
                },
            )
            .with_parent(1),
        );
        if let Some(n) = m.body.get_mut(0) {
            n.add_child(1);
        }
        if let Some(n) = m.body.get_mut(1) {
            n.add_child(2);
        }
        if let Some(n) = m.body.get_mut(1) {
            n.add_child(3);
        }

        let builder = DocxBuilder::new();
        let refs = builder.collect_image_refs(&m);
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].r_id, "rId3");
        assert_eq!(refs[0].source, "photo.png");
        assert_eq!(refs[0].content_type, "image/png");
        assert_eq!(refs[0].alt_text, "A photo");
        assert_eq!(refs[0].zip_path, "word/media/image1.png");
        assert_eq!(refs[1].r_id, "rId4");
        assert_eq!(refs[1].source, "diagram.jpg");
        assert_eq!(refs[1].content_type, "image/jpeg");
        assert_eq!(refs[1].zip_path, "word/media/image2.jpg");
    }

    #[test]
    fn test_docx_with_images() -> Result<(), Box<dyn std::error::Error>> {
        let tmp_dir = tempfile::tempdir()?;
        let img_path = tmp_dir.path().join("test_image.png");

        let tiny_png: &[u8] = &[
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
            0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78,
            0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00,
            0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
        ];
        std::fs::write(&img_path, tiny_png)?;

        let mut m = SIRModuleV2::new();
        m.metadata.title = Some("Doc with Image".into());
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::Paragraph).with_parent(0));
        m.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "Before image".into(),
                },
            )
            .with_parent(1),
        );
        m.body.push(
            Node::new(
                3,
                NodeType::Figure {
                    placement: FloatPlacement::Top,
                },
            )
            .with_parent(0),
        );
        m.body.push(
            Node::new(
                4,
                NodeType::Image {
                    source: "test_image.png".into(),
                    alt: "Test PNG".into(),
                    width: None,
                    height: None,
                    placement: FloatPlacement::Top,
                },
            )
            .with_parent(3),
        );
        m.body
            .push(Node::new(5, NodeType::Paragraph).with_parent(0));
        m.body.push(
            Node::new(
                6,
                NodeType::Text {
                    content: "After image".into(),
                },
            )
            .with_parent(5),
        );

        if let Some(n) = m.body.get_mut(0) {
            n.add_child(1);
        }
        if let Some(n) = m.body.get_mut(0) {
            n.add_child(3);
        }
        if let Some(n) = m.body.get_mut(0) {
            n.add_child(5);
        }
        if let Some(n) = m.body.get_mut(1) {
            n.add_child(2);
        }
        if let Some(n) = m.body.get_mut(3) {
            n.add_child(4);
        }
        if let Some(n) = m.body.get_mut(5) {
            n.add_child(6);
        }

        let docx = DocxBuilder::new().build_with_base_dir(&m, tmp_dir.path().to_str().unwrap())?;
        assert!(docx.len() > 200);
        assert_eq!(&docx[0..4], b"PK\x03\x04");

        let text = String::from_utf8_lossy(&docx);
        assert!(text.contains("word/media/image1.png"));
        assert!(text.contains("r:embed=\"rId3\""));
        assert!(text.contains("image/png"));
        assert!(text.contains("Before image"));
        assert!(text.contains("After image"));
        assert!(text.contains("<w:drawing>"));
        assert!(text.contains("<a:blip"));
        Ok(())
    }

    #[test]
    fn test_docx_with_missing_image_fallback() -> Result<(), Box<dyn std::error::Error>> {
        let mut m = SIRModuleV2::new();
        m.metadata.title = Some("Missing Image".into());
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::Paragraph).with_parent(0));
        m.body.push(
            Node::new(
                2,
                NodeType::Image {
                    source: "nonexistent.png".into(),
                    alt: "Missing".into(),
                    width: None,
                    height: None,
                    placement: FloatPlacement::Top,
                },
            )
            .with_parent(1),
        );
        if let Some(n) = m.body.get_mut(0) {
            n.add_child(1);
        }
        if let Some(n) = m.body.get_mut(1) {
            n.add_child(2);
        }

        let docx = DocxBuilder::new().build_with_base_dir(&m, "/nonexistent")?;
        let text = String::from_utf8_lossy(&docx);
        assert!(text.contains("[Missing]"));
        assert!(!text.contains("<w:drawing>"));
        Ok(())
    }

    #[test]
    fn test_docx_footnote() -> Result<(), Box<dyn std::error::Error>> {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::Paragraph).with_parent(0));
        m.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "Hello".into(),
                },
            )
            .with_parent(1),
        );
        m.body.push(
            Node::new(
                3,
                NodeType::Footnote {
                    content: "A footnote".into(),
                },
            )
            .with_parent(1),
        );
        if let Some(n) = m.body.get_mut(0) {
            n.add_child(1);
        }
        if let Some(n) = m.body.get_mut(1) {
            n.add_child(2);
        }
        if let Some(n) = m.body.get_mut(1) {
            n.add_child(3);
        }

        let docx = DocxBuilder::new().build(&m)?;
        let text = String::from_utf8_lossy(&docx);
        assert!(text.contains("word/footnotes.xml"));
        assert!(text.contains("A footnote"));
        assert!(text.contains("w:id=\"1\""));
        assert!(text.contains("w:type=\"separator\""));
        Ok(())
    }

    #[test]
    fn test_docx_endnote() -> Result<(), Box<dyn std::error::Error>> {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::Paragraph).with_parent(0));
        m.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "Hello".into(),
                },
            )
            .with_parent(1),
        );
        m.body.push(
            Node::new(
                3,
                NodeType::Endnote {
                    content: "An endnote".into(),
                },
            )
            .with_parent(1),
        );
        if let Some(n) = m.body.get_mut(0) {
            n.add_child(1);
        }
        if let Some(n) = m.body.get_mut(1) {
            n.add_child(2);
        }
        if let Some(n) = m.body.get_mut(1) {
            n.add_child(3);
        }

        let docx = DocxBuilder::new().build(&m)?;
        let text = String::from_utf8_lossy(&docx);
        assert!(text.contains("word/endnotes.xml"));
        assert!(text.contains("An endnote"));
        assert!(text.contains("w:id=\"1\""));
        assert!(text.contains("<w:endnoteReference"));
        Ok(())
    }

    #[test]
    fn test_docx_footnote_reference_in_body() -> Result<(), Box<dyn std::error::Error>> {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::Paragraph).with_parent(0));
        m.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "Some text".into(),
                },
            )
            .with_parent(1),
        );
        m.body.push(
            Node::new(
                3,
                NodeType::Footnote {
                    content: "Note 1".into(),
                },
            )
            .with_parent(1),
        );
        if let Some(n) = m.body.get_mut(0) {
            n.add_child(1);
        }
        if let Some(n) = m.body.get_mut(1) {
            n.add_child(2);
        }
        if let Some(n) = m.body.get_mut(1) {
            n.add_child(3);
        }

        let docx = DocxBuilder::new().build(&m)?;
        let text = String::from_utf8_lossy(&docx);
        assert!(text.contains("<w:footnoteReference w:id=\"1\"/>"));
        Ok(())
    }

    #[test]
    fn test_docx_comment() -> Result<(), Box<dyn std::error::Error>> {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::Paragraph).with_parent(0));
        m.body.push(
            Node::new(
                2,
                NodeType::Comment {
                    author: "Test Author".into(),
                    content: "A comment".into(),
                },
            )
            .with_parent(1),
        );
        if let Some(n) = m.body.get_mut(0) {
            n.add_child(1);
        }
        if let Some(n) = m.body.get_mut(1) {
            n.add_child(2);
        }

        let docx = DocxBuilder::new().build(&m)?;
        let text = String::from_utf8_lossy(&docx);
        assert!(text.contains("word/comments.xml"));
        assert!(text.contains("Test Author"));
        assert!(text.contains("A comment"));
        assert!(text.contains("<w:commentRangeStart"));
        assert!(text.contains("<w:commentRangeEnd"));
        assert!(text.contains("<w:commentReference"));
        Ok(())
    }

    #[test]
    fn test_docx_multiple_footnotes() -> Result<(), Box<dyn std::error::Error>> {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::Paragraph).with_parent(0));
        m.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "Text".into(),
                },
            )
            .with_parent(1),
        );
        m.body.push(
            Node::new(
                3,
                NodeType::Footnote {
                    content: "First".into(),
                },
            )
            .with_parent(1),
        );
        m.body.push(
            Node::new(
                4,
                NodeType::Footnote {
                    content: "Second".into(),
                },
            )
            .with_parent(1),
        );
        if let Some(n) = m.body.get_mut(0) {
            n.add_child(1);
        }
        if let Some(n) = m.body.get_mut(1) {
            n.add_child(2);
        }
        if let Some(n) = m.body.get_mut(1) {
            n.add_child(3);
        }
        if let Some(n) = m.body.get_mut(1) {
            n.add_child(4);
        }

        let docx = DocxBuilder::new().build(&m)?;
        let text = String::from_utf8_lossy(&docx);
        assert!(text.contains("<w:footnoteReference w:id=\"1\"/>"));
        assert!(text.contains("<w:footnoteReference w:id=\"2\"/>"));
        assert!(text.contains("First"));
        assert!(text.contains("Second"));
        Ok(())
    }

    #[test]
    fn test_docx_no_footnotes_no_file() -> Result<(), Box<dyn std::error::Error>> {
        let m = make_simple_module();
        let docx = DocxBuilder::new().build(&m)?;
        let text = String::from_utf8_lossy(&docx);
        assert!(!text.contains("word/footnotes.xml"));
        assert!(!text.contains("word/endnotes.xml"));
        assert!(!text.contains("word/comments.xml"));
        Ok(())
    }
}
