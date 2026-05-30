use std::io;

use ldir_ir::sir::v2::module::SIRModuleV2;
use ldir_ir::sir::v2::nodes::*;

#[derive(Debug, thiserror::Error)]
pub enum OdtError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("XML error: {0}")]
    Xml(String),
}

#[derive(Debug, Clone)]
pub struct OdtBuilder;

impl Default for OdtBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl OdtBuilder {
    pub fn new() -> Self {
        Self
    }

    pub fn build(&self, module: &SIRModuleV2) -> Result<Vec<u8>, OdtError> {
        let content_xml = self.render_content(module);
        let styles_xml = self.render_styles();
        let manifest_xml = MANIFEST_XML;
        let mimetype = MIMETYPE;

        let mut zip = SimpleZip::new();
        zip.add_file("mimetype", mimetype.as_bytes(), true);
        zip.add_file("content.xml", content_xml.as_bytes(), false);
        zip.add_file("styles.xml", styles_xml.as_bytes(), false);
        zip.add_file("META-INF/manifest.xml", manifest_xml.as_bytes(), false);

        zip.finish().map_err(OdtError::Xml)
    }

    fn render_content(&self, module: &SIRModuleV2) -> String {
        let mut body = String::new();
        for &root_id in module.body.roots() {
            if let Some(root) = module.body.get(root_id) {
                self.render_node(&mut body, module, root);
            }
        }

        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content
    xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
    xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0"
    xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0"
    xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0"
    xmlns:draw="urn:oasis:names:tc:opendocument:xmlns:drawing:1.0"
    xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0"
    xmlns:xlink="http://www.w3.org/1999/xlink"
    office:version="1.2">
    <office:body>
        <office:text>
{body}
        </office:text>
    </office:body>
</office:document-content>"#
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

            NodeType::Part
            | NodeType::Chapter
            | NodeType::Section
            | NodeType::Subsection
            | NodeType::Subsubsection => {
                let level = node.heading_level().unwrap_or(1);
                out.push_str(&format!(
                    "<text:h text:style-name=\"Heading_20_{}\" text:outline-level=\"{}\">",
                    level, level
                ));
                self.render_children_inline(out, module, node);
                out.push_str("</text:h>");
            }

            NodeType::Paragraph => {
                out.push_str("<text:p>");
                self.render_children_inline(out, module, node);
                out.push_str("</text:p>");
            }

            NodeType::BlockQuote => {
                out.push_str("<text:p text:style-name=\"Blockquote\">");
                self.render_children_inline(out, module, node);
                out.push_str("</text:p>");
            }

            NodeType::CodeBlock { language, content } => {
                if let Some(lang) = language {
                    out.push_str("<text:p text:style-name=\"CodeBlock\">");
                    out.push_str(&escape_xml(&format!("[{}]", lang)));
                    out.push_str("</text:p>");
                }
                for line in content.lines() {
                    out.push_str("<text:p text:style-name=\"CodeBlock\">");
                    out.push_str(&escape_xml(line));
                    out.push_str("</text:p>");
                }
                if content.ends_with('\n') || content.is_empty() {
                    out.push_str("<text:p text:style-name=\"CodeBlock\"/>");
                }
            }

            NodeType::ThematicBreak => {
                out.push_str("<text:p text:style-name=\"HorizontalRule\"/>");
            }

            NodeType::List { ordered, .. } => {
                if *ordered {
                    out.push_str("<text:list text:style-name=\"Ordered_20_List\">");
                } else {
                    out.push_str("<text:list text:style-name=\"Unordered_20_List\">");
                }
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_list_item(out, module, child);
                    }
                }
                out.push_str("</text:list>");
            }

            NodeType::ListItem => {
                self.render_list_item(out, module, node);
            }

            NodeType::Table { .. } => {
                out.push_str("<table:table>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id)
                        && let NodeType::TableRow { .. } = &child.node_type
                    {
                        self.render_table_row(out, module, child);
                    }
                }
                out.push_str("</table:table>");
            }

            NodeType::Figure { .. } => {
                out.push_str("<text:p>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        if let NodeType::Image { source, .. } = &child.node_type {
                            out.push_str(&format!(
                                "<draw:image xlink:href=\"Pictures/{}\"/>",
                                escape_xml(source)
                            ));
                        } else {
                            self.render_node(out, module, child);
                        }
                    }
                }
                out.push_str("</text:p>");
            }

            NodeType::Caption => {
                out.push_str("<text:p text:style-name=\"Caption\">");
                self.render_children_inline(out, module, node);
                out.push_str("</text:p>");
            }

            NodeType::PageBreak => {
                out.push_str("<text:p text:soft-page-break=\"\"/>");
            }

            NodeType::TableOfContents { .. } => {
                out.push_str(
                    "<text:table-of-content text:style-name=\"TOC\" text:name=\"Table of Contents\">
                    <text:table-of-content-source text:outline-level=\"3\">
                        <text:index-body/>
                    </text:table-of-content-source>
                </text:table-of-content>",
                );
            }

            NodeType::Footnote { content } => {
                out.push_str("<text:note text:note-class=\"footnote\">");
                out.push_str("<text:note-citation/>");
                out.push_str("<text:note-body><text:p>");
                out.push_str(&escape_xml(content));
                out.push_str("</text:p></text:note-body>");
                out.push_str("</text:note>");
            }

            NodeType::MathBlock { .. } => {
                let text = module.body.collect_text(node.id);
                out.push_str("<text:p text:style-name=\"CodeBlock\">");
                out.push_str(&escape_xml(&text));
                out.push_str("</text:p>");
            }

            NodeType::Citation { keys, .. } => {
                out.push_str("<text:span text:style-name=\"Citation\">");
                out.push_str(&escape_xml(&keys.join(", ")));
                out.push_str("</text:span>");
            }

            NodeType::FootnoteBlock => {
                for n in module.body.iter() {
                    if let NodeType::Footnote { content } = &n.node_type {
                        out.push_str("<text:p>");
                        out.push_str(&escape_xml(content));
                        out.push_str("</text:p>");
                    }
                }
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

    fn render_children_inline(&self, out: &mut String, module: &SIRModuleV2, node: &Node) {
        for &child_id in &node.child_ids {
            if let Some(child) = module.body.get(child_id) {
                self.render_inline(out, module, child);
            }
        }
    }

    fn render_inline(&self, out: &mut String, module: &SIRModuleV2, node: &Node) {
        match &node.node_type {
            NodeType::Text { content } => {
                out.push_str(&escape_xml(content));
            }

            NodeType::Bold => {
                out.push_str("<text:span text:style-name=\"Bold\">");
                self.render_children_inline(out, module, node);
                out.push_str("</text:span>");
            }

            NodeType::Italic => {
                out.push_str("<text:span text:style-name=\"Italic\">");
                self.render_children_inline(out, module, node);
                out.push_str("</text:span>");
            }

            NodeType::Mono => {
                out.push_str("<text:span text:style-name=\"Code\">");
                self.render_children_inline(out, module, node);
                out.push_str("</text:span>");
            }

            NodeType::Link { url, .. } => {
                out.push_str(&format!(
                    "<text:a xlink:type=\"simple\" xlink:href=\"{}\">",
                    escape_xml(url)
                ));
                let text = module.body.collect_text(node.id);
                out.push_str(&escape_xml(&text));
                out.push_str("</text:a>");
            }

            NodeType::Image { source, alt, .. } => {
                out.push_str(&format!(
                    "<draw:image xlink:href=\"Pictures/{}\"/>",
                    escape_xml(source)
                ));
                if !alt.is_empty() {
                    out.push_str(&format!("<text:span> {} </text:span>", escape_xml(alt)));
                }
            }

            NodeType::MathInline { content } => {
                out.push_str("<text:span text:style-name=\"Code\">");
                out.push_str(&escape_xml(content));
                out.push_str("</text:span>");
            }

            NodeType::LineBreak => {
                out.push_str("<text:line-break/>");
            }

            NodeType::Underline => {
                out.push_str("<text:span text:style-name=\"Underline\">");
                self.render_children_inline(out, module, node);
                out.push_str("</text:span>");
            }

            NodeType::Strikethrough => {
                out.push_str("<text:span text:style-name=\"Strikethrough\">");
                self.render_children_inline(out, module, node);
                out.push_str("</text:span>");
            }

            NodeType::SmallCaps => {
                out.push_str("<text:span text:style-name=\"SmallCaps\">");
                self.render_children_inline(out, module, node);
                out.push_str("</text:span>");
            }

            NodeType::Footnote { content } => {
                out.push_str("<text:note text:note-class=\"footnote\">");
                out.push_str("<text:note-citation/>");
                out.push_str("<text:note-body><text:p>");
                out.push_str(&escape_xml(content));
                out.push_str("</text:p></text:note-body>");
                out.push_str("</text:note>");
            }

            NodeType::Reference { label } => {
                out.push_str(&format!("<text:ref>{}", escape_xml(label)));
                out.push_str("</text:ref>");
            }

            NodeType::Citation { keys, .. } => {
                out.push_str("<text:span text:style-name=\"Citation\">");
                out.push_str(&escape_xml(&keys.join(", ")));
                out.push_str("</text:span>");
            }

            NodeType::Styled { .. } | NodeType::Group => {
                self.render_children_inline(out, module, node);
            }

            _ => {
                let text = module.body.collect_text(node.id);
                if !text.is_empty() {
                    out.push_str(&escape_xml(&text));
                }
            }
        }
    }

    fn render_list_item(&self, out: &mut String, module: &SIRModuleV2, node: &Node) {
        out.push_str("<text:list-item>");
        match &node.node_type {
            NodeType::ListItem => {
                out.push_str("<text:p>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline(out, module, child);
                    }
                }
                out.push_str("</text:p>");
            }
            _ => {
                out.push_str("<text:p>");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_inline(out, module, child);
                    }
                }
                out.push_str("</text:p>");
            }
        }
        out.push_str("</text:list-item>");
    }

    fn render_table_row(&self, out: &mut String, module: &SIRModuleV2, node: &Node) {
        out.push_str("<table:table-row>");
        for &child_id in &node.child_ids {
            if let Some(child) = module.body.get(child_id)
                && let NodeType::TableCell { colspan, rowspan } = &child.node_type
            {
                let mut attrs = String::new();
                if *colspan > 1 {
                    attrs.push_str(&format!(" table:number-columns-spanned=\"{}\"", colspan));
                }
                if *rowspan > 1 {
                    attrs.push_str(&format!(" table:number-rows-spanned=\"{}\"", rowspan));
                }
                out.push_str(&format!("<table:table-cell{}>", attrs));
                out.push_str("<text:p>");
                for &cell_child_id in &child.child_ids {
                    if let Some(cell_child) = module.body.get(cell_child_id) {
                        self.render_inline(out, module, cell_child);
                    }
                }
                out.push_str("</text:p>");
                out.push_str("</table:table-cell>");
            }
        }
        out.push_str("</table:table-row>");
    }

    fn render_styles(&self) -> String {
        STYLES_XML.to_string()
    }
}

const MIMETYPE: &str = "application/vnd.oasis.opendocument.text";

const MANIFEST_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0" manifest:version="1.2">
    <manifest:file-entry manifest:media-type="application/vnd.oasis.opendocument.text" manifest:version="1.2" manifest:full-path="/"/>
    <manifest:file-entry manifest:media-type="text/xml" manifest:full-path="content.xml"/>
    <manifest:file-entry manifest:media-type="text/xml" manifest:full-path="styles.xml"/>
</manifest:manifest>"#;

const STYLES_XML: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<office:document-styles
    xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
    xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0"
    xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0"
    xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0"
    office:version="1.2">
    <office:styles>
        <style:style style:name="Heading_20_1" style:family="paragraph" style:parent-style-name="Heading">
            <style:text-properties fo:font-size="2em" fo:font-weight="bold"/>
        </style:style>
        <style:style style:name="Heading_20_2" style:family="paragraph" style:parent-style-name="Heading">
            <style:text-properties fo:font-size="1.5em" fo:font-weight="bold"/>
        </style:style>
        <style:style style:name="Heading_20_3" style:family="paragraph" style:parent-style-name="Heading">
            <style:text-properties fo:font-size="1.2em" fo:font-weight="bold"/>
        </style:style>
        <style:style style:name="Heading_20_4" style:family="paragraph" style:parent-style-name="Heading">
            <style:text-properties fo:font-size="1.1em" fo:font-weight="bold"/>
        </style:style>
        <style:style style:name="Bold" style:family="text">
            <style:text-properties fo:font-weight="bold"/>
        </style:style>
        <style:style style:name="Italic" style:family="text">
            <style:text-properties fo:font-style="italic"/>
        </style:style>
        <style:style style:name="Code" style:family="text">
            <style:text-properties style:font-name="Courier New" fo:font-family="Courier New"/>
        </style:style>
        <style:style style:name="Underline" style:family="text">
            <style:text-properties style:text-underline-style="solid" style:text-underline-width="auto"/>
        </style:style>
        <style:style style:name="Strikethrough" style:family="text">
            <style:text-properties style:text-line-through-style="solid"/>
        </style:style>
        <style:style style:name="SmallCaps" style:family="text">
            <style:text-properties fo:font-variant="small-caps"/>
        </style:style>
        <style:style style:name="CodeBlock" style:family="paragraph">
            <style:paragraph-properties fo:margin-left="0.5cm" fo:background-color="#F5F5F5"/>
            <style:text-properties style:font-name="Courier New" fo:font-family="Courier New" fo:font-size="0.9em"/>
        </style:style>
        <style:style style:name="Blockquote" style:family="paragraph">
            <style:paragraph-properties fo:margin-left="1.5cm" fo:margin-right="1.5cm" fo:margin-top="0.5cm" fo:margin-bottom="0.5cm" fo:border-left="0.2cm solid #999999" fo:padding-left="0.5cm"/>
            <style:text-properties fo:font-style="italic" fo:color="#404040"/>
        </style:style>
        <style:style style:name="HorizontalRule" style:family="paragraph">
            <style:paragraph-properties fo:border-bottom="1pt solid #999999" fo:margin-top="0.5cm" fo:margin-bottom="0.5cm"/>
        </style:style>
        <style:style style:name="Caption" style:family="paragraph">
            <style:paragraph-properties fo:margin-top="0.2cm" fo:margin-bottom="0.2cm"/>
            <style:text-properties fo:font-style="italic" fo:font-size="0.9em" fo:color="#404040"/>
        </style:style>
        <style:style style:name="Citation" style:family="text">
            <style:text-properties fo:font-size="0.8em" fo:vertical-align="super" fo:color="#0066CC"/>
        </style:style>
        <style:style style:name="Unordered_20_List" style:family="paragraph">
            <style:paragraph-properties fo:margin-left="1cm"/>
        </style:style>
        <style:style style:name="Ordered_20_List" style:family="paragraph">
            <style:paragraph-properties fo:margin-left="1cm"/>
        </style:style>
    </office:styles>
</office:document-styles>"##;

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
                    content: "Section Title".into(),
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
                    content: "Hello ODT!".into(),
                },
            )
            .with_parent(3),
        );
        if let Some(n) = m.body.get_mut(0) {
            n.add_child(1);
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

    fn build_text(module: &SIRModuleV2) -> String {
        String::from_utf8_lossy(&OdtBuilder::new().build(module).unwrap()).into_owned()
    }

    #[test]
    fn test_odt_valid_zip() {
        let m = make_simple_module();
        let odt = OdtBuilder::new().build(&m).unwrap();
        assert_eq!(&odt[0..4], b"PK\x03\x04");
        assert!(odt.len() > 100);
    }

    #[test]
    fn test_odt_contains_mimetype() {
        let m = make_simple_module();
        let text = build_text(&m);
        let zip_text = &text;
        assert!(zip_text.contains("mimetype"));
        assert!(zip_text.contains("application/vnd.oasis.opendocument.text"));
    }

    #[test]
    fn test_odt_contains_content_xml() {
        let m = make_simple_module();
        let text = build_text(&m);
        assert!(text.contains("content.xml"));
        assert!(text.contains("office:document-content"));
        assert!(text.contains("Hello ODT!"));
    }

    #[test]
    fn test_odt_heading() {
        let m = make_simple_module();
        let text = build_text(&m);
        assert!(text.contains("<text:h"));
        assert!(text.contains("text:outline-level=\"2\""));
        assert!(text.contains("Heading_20_2"));
        assert!(text.contains("Section Title"));
    }

    #[test]
    fn test_odt_bold_italic() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::Paragraph).with_parent(0));
        m.body.push(Node::new(2, NodeType::Bold).with_parent(1));
        m.body.push(
            Node::new(
                3,
                NodeType::Text {
                    content: "bold text".into(),
                },
            )
            .with_parent(2),
        );
        m.body.push(Node::new(4, NodeType::Italic).with_parent(1));
        m.body.push(
            Node::new(
                5,
                NodeType::Text {
                    content: "italic text".into(),
                },
            )
            .with_parent(4),
        );
        m.body.push(Node::new(6, NodeType::Mono).with_parent(1));
        m.body.push(
            Node::new(
                7,
                NodeType::Text {
                    content: "code".into(),
                },
            )
            .with_parent(6),
        );
        if let Some(n) = m.body.get_mut(0) {
            n.add_child(1);
        }
        if let Some(n) = m.body.get_mut(1) {
            n.add_child(2);
            n.add_child(4);
            n.add_child(6);
        }
        if let Some(n) = m.body.get_mut(2) {
            n.add_child(3);
        }
        if let Some(n) = m.body.get_mut(4) {
            n.add_child(5);
        }
        if let Some(n) = m.body.get_mut(6) {
            n.add_child(7);
        }

        let text = build_text(&m);
        assert!(text.contains(r#"text:style-name="Bold""#));
        assert!(text.contains(r#"text:style-name="Italic""#));
        assert!(text.contains(r#"text:style-name="Code""#));
        assert!(text.contains("bold text"));
        assert!(text.contains("italic text"));
        assert!(text.contains("code"));
    }

    #[test]
    fn test_odt_table() {
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
        m.body.push(
            Node::new(
                5,
                NodeType::TableCell {
                    colspan: 1,
                    rowspan: 1,
                },
            )
            .with_parent(2),
        );
        m.body.push(
            Node::new(
                6,
                NodeType::Text {
                    content: "Cell2".into(),
                },
            )
            .with_parent(5),
        );
        if let Some(n) = m.body.get_mut(0) {
            n.add_child(1);
        }
        if let Some(n) = m.body.get_mut(1) {
            n.add_child(2);
        }
        if let Some(n) = m.body.get_mut(2) {
            n.add_child(3);
            n.add_child(5);
        }
        if let Some(n) = m.body.get_mut(3) {
            n.add_child(4);
        }
        if let Some(n) = m.body.get_mut(5) {
            n.add_child(6);
        }

        let text = build_text(&m);
        assert!(text.contains("<table:table>"));
        assert!(text.contains("<table:table-row>"));
        assert!(text.contains("<table:table-cell>"));
        assert!(text.contains("Cell1"));
        assert!(text.contains("Cell2"));
    }

    #[test]
    fn test_odt_list() {
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
        m.body.push(Node::new(4, NodeType::ListItem).with_parent(1));
        m.body.push(
            Node::new(
                5,
                NodeType::Text {
                    content: "item 2".into(),
                },
            )
            .with_parent(4),
        );
        if let Some(n) = m.body.get_mut(0) {
            n.add_child(1);
        }
        if let Some(n) = m.body.get_mut(1) {
            n.add_child(2);
            n.add_child(4);
        }
        if let Some(n) = m.body.get_mut(2) {
            n.add_child(3);
        }
        if let Some(n) = m.body.get_mut(4) {
            n.add_child(5);
        }

        let text = build_text(&m);
        assert!(text.contains("<text:list"));
        assert!(text.contains("<text:list-item>"));
        assert!(text.contains("item 1"));
        assert!(text.contains("item 2"));
    }

    #[test]
    fn test_odt_link() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::Paragraph).with_parent(0));
        m.body.push(
            Node::new(
                2,
                NodeType::Link {
                    url: "https://example.com".into(),
                    title: Some("Example".into()),
                },
            )
            .with_parent(1),
        );
        m.body.push(
            Node::new(
                3,
                NodeType::Text {
                    content: "link text".into(),
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

        let text = build_text(&m);
        assert!(text.contains("<text:a"));
        assert!(text.contains("xlink:href=\"https://example.com\""));
        assert!(text.contains("link text"));
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
    fn test_odt_empty_module() {
        let m = SIRModuleV2::new();
        let text = build_text(&m);
        assert_eq!(&text.as_bytes()[0..4], b"PK\x03\x04");
    }

    #[test]
    fn test_odt_code_block() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body.push(
            Node::new(
                1,
                NodeType::CodeBlock {
                    language: Some("rust".into()),
                    content: "fn main() {}".into(),
                },
            )
            .with_parent(0),
        );
        if let Some(n) = m.body.get_mut(0) {
            n.add_child(1);
        }

        let text = build_text(&m);
        assert!(text.contains("text:style-name=\"CodeBlock\""));
        assert!(text.contains("[rust]"));
        assert!(text.contains("fn main()"));
    }

    #[test]
    fn test_odt_blockquote() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::BlockQuote).with_parent(0));
        m.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "A quote".into(),
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

        let text = build_text(&m);
        assert!(text.contains("text:style-name=\"Blockquote\""));
        assert!(text.contains("A quote"));
    }

    #[test]
    fn test_odt_thematic_break() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::ThematicBreak).with_parent(0));
        if let Some(n) = m.body.get_mut(0) {
            n.add_child(1);
        }

        let text = build_text(&m);
        assert!(text.contains("text:style-name=\"HorizontalRule\""));
    }

    #[test]
    fn test_odt_xml_escaping() {
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

        let text = build_text(&m);
        assert!(text.contains("&lt;"));
        assert!(text.contains("&amp;"));
        assert!(text.contains("&gt;"));
    }

    #[test]
    fn test_odt_contains_styles_xml() {
        let m = make_simple_module();
        let text = build_text(&m);
        assert!(text.contains("styles.xml"));
        assert!(text.contains("office:document-styles"));
        assert!(text.contains("style:name=\"Bold\""));
        assert!(text.contains("style:name=\"Italic\""));
    }
}
