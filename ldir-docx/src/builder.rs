use ldir_ir::sir::v2::module::SIRModuleV2;
use ldir_ir::sir::v2::nodes::*;

#[derive(Debug, Clone, thiserror::Error)]
pub enum DocxError {
    #[error("DOCX build error: {0}")]
    BuildError(String),
}

#[derive(Debug, Clone)]
pub struct DocxBuilder;

impl Default for DocxBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DocxBuilder {
    pub fn new() -> Self {
        Self
    }

    #[must_use = "building DOCX can fail; check the result"]
    pub fn build(&self, module: &SIRModuleV2) -> Result<Vec<u8>, DocxError> {
        let document_xml = self.render_document(module);

        let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
  <Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
  <Override PartName="/word/numbering.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"/>
  <Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
</Types>"#;

        let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
</Relationships>"#;

        let document_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering" Target="numbering.xml"/>
</Relationships>"#;

        let styles_xml = self.render_styles();
        let numbering_xml = self.render_numbering();
        let core_xml = self.render_core_properties(module);

        let mut zip = SimpleZip::new();
        zip.add_file("[Content_Types].xml", content_types.as_bytes(), false);
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

    fn render_document(&self, module: &SIRModuleV2) -> String {
        let mut body = String::new();
        for &root_id in module.body.roots() {
            if let Some(root) = module.body.get(root_id) {
                self.render_node(&mut body, module, root);
            }
        }

        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body>
{body}
</w:body>
</w:document>"#
        )
    }

    fn render_node(&self, out: &mut String, module: &SIRModuleV2, node: &Node) {
        match &node.node_type {
            NodeType::Document => {
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_node(out, module, child);
                    }
                }
            }

            NodeType::Part => {
                out.push_str("<w:p><w:pPr><w:pStyle w:val=\"Title\"/></w:pPr>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline(out, module, child);
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
                        self.render_inline(out, module, child);
                    }
                }
                out.push_str("</w:p>");
            }

            NodeType::Paragraph => {
                out.push_str("<w:p>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline(out, module, child);
                    }
                }
                out.push_str("</w:p>");
            }

            NodeType::List { ordered, .. } => {
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_list_item(out, module, child, *ordered);
                    }
                }
            }

            NodeType::ListItem => {
                self.render_list_item(out, module, node, false);
            }

            NodeType::BlockQuote => {
                out.push_str("<w:p><w:pPr><w:pStyle w:val=\"Quote\"/></w:pPr>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline(out, module, child);
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

            NodeType::Figure { .. } => {
                out.push_str("<w:p>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        match &child.node_type {
                            NodeType::Image { alt, .. } => {
                                out.push_str("<w:r><w:t>");
                                out.push_str(&escape_xml(alt));
                                out.push_str(" [image not embedded]</w:t></w:r>");
                            }
                            _ => self.render_node(out, module, child),
                        }
                    }
                }
                out.push_str("</w:p>");
            }

            NodeType::Caption => {
                out.push_str("<w:p><w:pPr><w:pStyle w:val=\"Caption\"/></w:pPr>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline(out, module, child);
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

            _ => {
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_node(out, module, child);
                    }
                }
            }
        }
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

    fn render_list_item(&self, out: &mut String, module: &SIRModuleV2, node: &Node, ordered: bool) {
        let num_id = if ordered { "2" } else { "1" };
        out.push_str(&format!(
            "<w:p><w:pPr><w:pStyle w:val=\"ListParagraph\"/><w:numPr><w:ilvl w:val=\"0\"/><w:numId w:val=\"{num_id}\"/></w:numPr></w:pPr>"
        ));
        for &child_id in &node.child_ids {
            if let Some(child) = module.body.get(child_id) {
                self.render_inline(out, module, child);
            }
        }
        out.push_str("</w:p>");
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
}
