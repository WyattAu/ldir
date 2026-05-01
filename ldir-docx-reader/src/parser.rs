//! DOCX Reader — converts DOCX (.docx) files to S-IR v2.
//!
//! Extracts `word/document.xml` from the ZIP archive, parses OOXML,
//! and converts to S-IR v2 document structure.

use std::io::{Read, Cursor};
use flate2::read::ZlibDecoder;
use ldir_ir::sir::v2::nodes::{ColSpec, ColumnAlign, Node, NodeType};
use ldir_ir::sir::v2::SIRModuleV2;

pub fn parse_docx(data: &[u8]) -> SIRModuleV2 {
    let document_xml = extract_document_xml(data);
    if document_xml.is_empty() {
        let mut module = SIRModuleV2::from_source("docx", "<input>");
        let doc_id = 0;
        module.body.push(Node::new(doc_id, NodeType::Document));
        return module;
    }
    let tokens = tokenize_xml(&document_xml);
    let dom = build_dom(&tokens);
    convert_dom(&dom)
}

fn extract_document_xml(data: &[u8]) -> String {
    let cursor = Cursor::new(data);
    if let Ok(archive) = SimpleZip::new(cursor) {
        for entry in &archive.entries {
            if entry.name == "word/document.xml" {
                if let Some(raw) = archive.read_entry(entry) {
                    return String::from_utf8_lossy(&raw).to_string();
                }
            }
        }
    }
    String::new()
}

struct ZipEntry {
    name: String,
    offset: u64,
    compressed_size: u64,
    uncompressed_size: u64,
    compression_method: u16,
    _local_header_offset: u64,
}

struct SimpleZip<R: Read> {
    _reader: R,
    data: Vec<u8>,
    entries: Vec<ZipEntry>,
}

impl<R: Read> SimpleZip<R> {
    fn new(mut reader: R) -> std::io::Result<Self> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        let mut entries = Vec::new();
        let mut i = data.len() as i64 - 22;
        while i >= 0 {
            let pos = i as usize;
            if pos + 4 <= data.len()
                && data[pos] == 0x50
                && data[pos + 1] == 0x4B
                && data[pos + 2] == 0x05
                && data[pos + 3] == 0x06
            {
                let cd_offset = u32::from_le_bytes([data[pos + 16], data[pos + 17], data[pos + 18], data[pos + 19]]) as u64;
                let cd_entries = u16::from_le_bytes([data[pos + 10], data[pos + 11]]) as u64;

                let mut cd_pos = cd_offset as usize;
                for _ in 0..cd_entries {
                    if cd_pos + 46 > data.len() {
                        break;
                    }
                    if data[cd_pos] != 0x50 || data[cd_pos + 1] != 0x4B || data[cd_pos + 2] != 0x01 || data[cd_pos + 3] != 0x02 {
                        break;
                    }

                    let compression_method = u16::from_le_bytes([data[cd_pos + 10], data[cd_pos + 11]]);
                    let compressed_size = u32::from_le_bytes([data[cd_pos + 20], data[cd_pos + 21], data[cd_pos + 22], data[cd_pos + 23]]) as u64;
                    let uncompressed_size = u32::from_le_bytes([data[cd_pos + 24], data[cd_pos + 25], data[cd_pos + 26], data[cd_pos + 27]]) as u64;
                    let name_len = u16::from_le_bytes([data[cd_pos + 28], data[cd_pos + 29]]) as usize;
                    let extra_len = u16::from_le_bytes([data[cd_pos + 30], data[cd_pos + 31]]) as usize;
                    let comment_len = u16::from_le_bytes([data[cd_pos + 32], data[cd_pos + 33]]) as usize;
                    let local_header_offset = u32::from_le_bytes([data[cd_pos + 42], data[cd_pos + 43], data[cd_pos + 44], data[cd_pos + 45]]) as u64;

                    let name_start = cd_pos + 46;
                    let name_end = (name_start + name_len).min(data.len());
                    let name = String::from_utf8_lossy(&data[name_start..name_end]).to_string();

                    let local_pos = local_header_offset as usize;
                    let file_data_offset = if local_pos + 30 <= data.len() {
                        let local_name_len = u16::from_le_bytes([data[local_pos + 26], data[local_pos + 27]]) as usize;
                        let local_extra_len = u16::from_le_bytes([data[local_pos + 28], data[local_pos + 29]]) as usize;
                        local_pos + 30 + local_name_len + local_extra_len
                    } else {
                        0
                    };

                    entries.push(ZipEntry {
                        name,
                        offset: file_data_offset as u64,
                        compressed_size,
                        uncompressed_size,
                        compression_method,
                        _local_header_offset: local_header_offset,
                    });

                    cd_pos = name_end + extra_len + comment_len;
                }
                break;
            }
            i -= 1;
        }

        Ok(Self { _reader: reader, data, entries })
    }

    fn read_entry(&self, entry: &ZipEntry) -> Option<Vec<u8>> {
        let start = entry.offset as usize;
        let end = (start + entry.compressed_size as usize).min(self.data.len());
        if start >= self.data.len() || end <= start {
            return None;
        }
        let compressed = &self.data[start..end];

        match entry.compression_method {
            0 => Some(compressed.to_vec()),
            8 => {
                let mut decoder = ZlibDecoder::new(compressed);
                let mut decompressed = Vec::with_capacity(entry.uncompressed_size as usize);
                if decoder.read_to_end(&mut decompressed).is_ok() {
                    Some(decompressed)
                } else {
                    let mut decoder2 = ZlibDecoder::new(compressed);
                    let mut buf = Vec::new();
                    if decoder2.read_to_end(&mut buf).is_ok() {
                        Some(buf)
                    } else {
                        None
                    }
                }
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum XmlEvent {
    Open { tag: String, attrs: Vec<(String, String)> },
    Close { tag: String },
    Text { content: String },
    Eof,
}

fn tokenize_xml(input: &str) -> Vec<XmlEvent> {
    let mut tokens = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'<' {
            if i + 9 <= bytes.len() && input[i..i + 9].eq_ignore_ascii_case("<?xml ") {
                if let Some(end) = input[i..].find("?>") {
                    i += end + 2;
                } else {
                    break;
                }
                continue;
            }

            if i + 4 <= bytes.len() && &input[i..i + 4] == "<!--" {
                if let Some(end) = input[i..].find("-->") {
                    i += end + 3;
                } else {
                    break;
                }
                continue;
            }

            if i + 9 <= bytes.len() && input[i..i + 9].eq_ignore_ascii_case("<![CDATA[") {
                if let Some(end) = input[i..].find("]]>") {
                    let cdata_start = i + 9;
                    let cdata_content = &input[cdata_start..i + end];
                    if !cdata_content.is_empty() {
                        tokens.push(XmlEvent::Text { content: cdata_content.to_string() });
                    }
                    i += end + 3;
                } else {
                    break;
                }
                continue;
            }

            i += 1;

            let is_closing = if i < bytes.len() && bytes[i] == b'/' {
                i += 1;
                true
            } else {
                false
            };

            let tag_start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b':' || bytes[i] == b'_') {
                i += 1;
            }
            let tag_name = input[tag_start..i].to_string();

            if tag_name.is_empty() {
                tokens.push(XmlEvent::Text { content: "<".to_string() });
                continue;
            }

            let local_tag = tag_name.split(':').next_back().unwrap_or(&tag_name).to_string();

            if is_closing {
                tokens.push(XmlEvent::Close { tag: local_tag });
                while i < bytes.len() && bytes[i] != b'>' {
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1;
                }
                continue;
            }

            let mut attrs = Vec::new();
            let mut self_closing = false;

            loop {
                while i < bytes.len() && bytes[i] == b' ' {
                    i += 1;
                }
                if i >= bytes.len() {
                    break;
                }
                if bytes[i] == b'>' {
                    i += 1;
                    break;
                }
                if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'>' {
                    i += 2;
                    self_closing = true;
                    break;
                }

                let attr_name_start = i;
                while i < bytes.len()
                    && bytes[i] != b'='
                    && bytes[i] != b'>'
                    && bytes[i] != b'/'
                    && bytes[i] != b' '
                {
                    i += 1;
                }
                let attr_name = input[attr_name_start..i].to_string();

                while i < bytes.len() && bytes[i] == b' ' {
                    i += 1;
                }

                if i < bytes.len() && bytes[i] == b'=' {
                    i += 1;
                    while i < bytes.len() && bytes[i] == b' ' {
                        i += 1;
                    }
                    let attr_value = if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                        let quote = bytes[i];
                        i += 1;
                        let val_start = i;
                        while i < bytes.len() && bytes[i] != quote {
                            i += 1;
                        }
                        let val = input[val_start..i].to_string();
                        if i < bytes.len() {
                            i += 1;
                        }
                        val
                    } else {
                        let val_start = i;
                        while i < bytes.len() && bytes[i] != b'>' && bytes[i] != b' ' && bytes[i] != b'/' {
                            i += 1;
                        }
                        input[val_start..i].to_string()
                    };
                    let local_attr_name = attr_name.split(':').next_back().unwrap_or(&attr_name).to_string();
                    attrs.push((local_attr_name, attr_value));
                }
            }

            tokens.push(XmlEvent::Open { tag: local_tag.clone(), attrs });

            if self_closing {
                tokens.push(XmlEvent::Close { tag: local_tag });
            }
        } else {
            let text_start = i;
            while i < bytes.len() && bytes[i] != b'<' {
                i += 1;
            }
            let raw = &input[text_start..i];
            if !raw.is_empty() {
                tokens.push(XmlEvent::Text { content: raw.to_string() });
            }
        }
    }

    tokens.push(XmlEvent::Eof);
    tokens
}

#[derive(Clone)]
struct XmlNode {
    tag: String,
    attrs: Vec<(String, String)>,
    children: Vec<XmlNode>,
    text: Option<String>,
}

fn build_dom(tokens: &[XmlEvent]) -> XmlNode {
    let root = XmlNode {
        tag: String::new(),
        attrs: Vec::new(),
        children: Vec::new(),
        text: None,
    };
    let mut stack: Vec<XmlNode> = vec![root];

    for token in tokens {
        match token {
            XmlEvent::Open { tag, attrs } => {
                let node = XmlNode {
                    tag: tag.clone(),
                    attrs: attrs.clone(),
                    children: Vec::new(),
                    text: None,
                };
                stack.push(node);
            }
            XmlEvent::Close { tag } => {
                let mut found = false;
                for j in (1..stack.len()).rev() {
                    if stack[j].tag == *tag {
                        let popped = stack.remove(j);
                        if let Some(parent) = stack.last_mut() {
                            parent.children.push(popped);
                        }
                        found = true;
                        break;
                    }
                }
                let _ = found;
            }
            XmlEvent::Text { content } => {
                if let Some(current) = stack.last_mut() {
                    current.text.get_or_insert_with(String::new).push_str(content);
                }
            }
            XmlEvent::Eof => break,
        }
    }

    while stack.len() > 1 {
        let popped = stack.pop().unwrap();
        stack.last_mut().unwrap().children.push(popped);
    }

    stack.pop().unwrap()
}

fn xml_attr(attrs: &[(String, String)], name: &str) -> Option<String> {
    attrs.iter().find(|(k, _)| k == name).map(|(_, v)| v.clone())
}

struct DocxConverter {
    module: SIRModuleV2,
    next_id: u32,
}

impl DocxConverter {
    fn new() -> Self {
        let module = SIRModuleV2::from_source("docx", "<input>");
        Self {
            module,
            next_id: 1,
        }
    }

    fn alloc_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn convert(&mut self, dom: &XmlNode) {
        let body_children: Vec<XmlNode> = find_body_recursive(dom)
            .unwrap_or_else(|| dom.children.clone());

        let doc_id = self.alloc_id();
        self.module.body.push(Node::new(doc_id, NodeType::Document));

        for child in &body_children {
            let tag = child.tag.as_str();

            if tag == "p" {
                let style = get_paragraph_style(child);
                let is_heading = style.as_ref().is_some_and(|s| s.starts_with("Heading"));

                if is_heading {
                    let text = collect_run_text(child);
                    let level: u8 = style.as_ref()
                        .and_then(|s| s.strip_prefix("Heading"))
                        .and_then(|rest| rest.chars().next())
                        .and_then(|c| c.to_digit(10))
                        .map(|d| d as u8)
                        .unwrap_or(1);

                    let heading_type = match level {
                        1 => NodeType::Chapter,
                        2 => NodeType::Section,
                        3 => NodeType::Subsection,
                        _ => NodeType::Subsubsection,
                    };

                    let heading_id = self.alloc_id();
                    let mut heading_node = Node::new(heading_id, heading_type).with_parent(doc_id);
                    if !text.is_empty() {
                        heading_node.counter = Some(text);
                    }
                    self.module.body.push(heading_node);
                    if let Some(doc) = self.module.body.get_mut(doc_id) {
                        doc.add_child(heading_id);
                    }
                    continue;
                }

                let run_nodes = convert_runs(child);
                if run_nodes.is_empty() {
                    continue;
                }

                let para_id = self.alloc_id();
                self.module.body.push(Node::new(para_id, NodeType::Paragraph).with_parent(doc_id));
                if let Some(doc) = self.module.body.get_mut(doc_id) {
                    doc.add_child(para_id);
                }

                for node_type in run_nodes {
                    let child_id = self.alloc_id();
                    self.module.body.push(Node::new(child_id, node_type).with_parent(para_id));
                    if let Some(para) = self.module.body.get_mut(para_id) {
                        para.add_child(child_id);
                    }
                }
            } else if tag == "tbl" {
                let table_id = self.alloc_id();
                self.module.body.push(
                    Node::new(table_id, NodeType::Table {
                        col_specs: Vec::new(),
                        num_cols: 0,
                    })
                    .with_parent(doc_id),
                );
                if let Some(doc) = self.module.body.get_mut(doc_id) {
                    doc.add_child(table_id);
                }

                let num_cols = count_table_columns(child);
                if let Some(tbl) = self.module.body.get_mut(table_id) {
                    let col_specs: Vec<ColSpec> = (0..num_cols)
                        .map(|_| ColSpec { align: ColumnAlign::Left, width: None })
                        .collect();
                    tbl.node_type = NodeType::Table { col_specs, num_cols };
                }

                let mut row_idx = 0;
                for tr in &child.children {
                    if tr.tag != "tr" {
                        continue;
                    }
                    let in_header_row = row_idx == 0;
                    let row_id = self.alloc_id();
                    self.module.body.push(
                        Node::new(row_id, NodeType::TableRow { is_header: in_header_row })
                            .with_parent(table_id),
                    );
                    if let Some(tbl) = self.module.body.get_mut(table_id) {
                        tbl.add_child(row_id);
                    }

                    for tc in &tr.children {
                        if tc.tag != "tc" {
                            continue;
                        }
                        let tc_id = self.alloc_id();
                        self.module.body.push(
                            Node::new(tc_id, NodeType::TableCell { colspan: 1, rowspan: 1 })
                                .with_parent(row_id),
                        );
                        if let Some(row_node) = self.module.body.get_mut(row_id) {
                            row_node.add_child(tc_id);
                        }

                        let cell_text = collect_element_text(tc);
                        if !cell_text.is_empty() {
                            let text_id = self.alloc_id();
                            self.module.body.push(
                                Node::new(text_id, NodeType::Text { content: cell_text })
                                    .with_parent(tc_id),
                            );
                            if let Some(tc_node) = self.module.body.get_mut(tc_id) {
                                tc_node.add_child(text_id);
                            }
                        }
                    }
                    row_idx += 1;
                }
            }
        }
    }
}

fn get_paragraph_style(p: &XmlNode) -> Option<String> {
    for child in &p.children {
        if child.tag == "pPr" {
            for pp_child in &child.children {
                if pp_child.tag == "pStyle" {
                    return xml_attr(&pp_child.attrs, "val");
                }
            }
        }
    }
    None
}

fn collect_run_text(p: &XmlNode) -> String {
    let mut text = String::new();
    collect_run_text_recursive(p, &mut text);
    text.trim().to_string()
}

fn collect_run_text_recursive(node: &XmlNode, text: &mut String) {
    if let Some(ref t) = node.text {
        if node.tag == "t" {
            text.push_str(t);
        }
    }
    for child in &node.children {
        collect_run_text_recursive(child, text);
    }
}

fn collect_element_text(node: &XmlNode) -> String {
    let mut text = String::new();
    collect_all_text(node, &mut text);
    text.trim().to_string()
}

fn collect_all_text(node: &XmlNode, text: &mut String) {
    if let Some(ref t) = node.text {
        text.push_str(t);
    }
    for child in &node.children {
        collect_all_text(child, text);
    }
}

fn convert_runs(p: &XmlNode) -> Vec<NodeType> {
    let mut nodes = Vec::new();
    for child in &p.children {
        if child.tag == "r" {
            let is_bold = run_has_property(child, "b");
            let is_italic = run_has_property(child, "i");
            let is_underline = run_has_property(child, "u");
            let is_strike = run_has_property(child, "strike");

            let text = collect_run_text_of(child);
            if text.is_empty() {
                continue;
            }

            if is_bold {
                nodes.push(NodeType::Bold);
            }
            if is_italic {
                nodes.push(NodeType::Italic);
            }
            if is_underline {
                nodes.push(NodeType::Underline);
            }
            if is_strike {
                nodes.push(NodeType::Strikethrough);
            }

            nodes.push(NodeType::Text { content: text });
            } else if child.tag == "hyperlink" {
                let url = xml_attr(&child.attrs, "anchor")
                    .or_else(|| xml_attr(&child.attrs, "id"));
                let text = collect_run_text(child);
                if let Some(ref url_str) = url {
                if !url_str.is_empty() {
                    nodes.push(NodeType::Link {
                        url: url_str.clone(),
                        title: if text.is_empty() { None } else { Some(text.clone()) },
                    });
                    continue;
                }
            }
            if !text.is_empty() {
                nodes.push(NodeType::Text { content: text });
            }
        }
    }
    nodes
}

fn collect_run_text_of(r: &XmlNode) -> String {
    let mut text = String::new();
    for child in &r.children {
        if child.tag == "t" {
            if let Some(ref t) = child.text {
                text.push_str(t);
            }
        }
    }
    text
}

fn run_has_property(r: &XmlNode, prop: &str) -> bool {
    for child in &r.children {
        if child.tag == "rPr" {
            for rp_child in &child.children {
                if rp_child.tag == prop {
                    let val = xml_attr(&rp_child.attrs, "val");
                    return val.as_deref() != Some("0") && val.as_deref() != Some("false");
                }
            }
        }
    }
    false
}

fn count_table_columns(tbl: &XmlNode) -> usize {
    if let Some(first_row) = tbl.children.iter().find(|c| c.tag == "tr") {
        first_row.children.iter().filter(|c| c.tag == "tc").count()
    } else {
        0
    }
}

fn find_body_recursive(dom: &XmlNode) -> Option<Vec<XmlNode>> {
    if dom.tag == "body" {
        return Some(dom.children.clone());
    }
    for child in &dom.children {
        if let Some(result) = find_body_recursive(child) {
            return Some(result);
        }
    }
    None
}

fn convert_dom(dom: &XmlNode) -> SIRModuleV2 {
    let mut converter = DocxConverter::new();
    converter.convert(dom);
    converter.module
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find_nodes(module: &SIRModuleV2, pred: impl Fn(&NodeType) -> bool) -> Vec<&Node> {
        module.body.find_by_type(pred)
    }

    fn make_simple_docx(body_content: &str) -> Vec<u8> {
        let xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<w:body>
{}
</w:body>
</w:document>"#,
            body_content
        );

        let xml_bytes = xml.as_bytes();

        let filename = b"word/document.xml";
        let local_header = build_local_file_header(filename, xml_bytes.len() as u32, 0);
        let central_dir_entry = build_central_dir_entry(filename, xml_bytes.len() as u32, 0);
        let end_of_central_dir = build_eocd(1, central_dir_entry.len() as u32, local_header.len() as u32);

        let mut out = Vec::new();
        out.extend_from_slice(&local_header);
        out.extend_from_slice(xml_bytes);
        out.extend_from_slice(&central_dir_entry);
        out.extend_from_slice(&end_of_central_dir);
        out
    }

    fn build_local_file_header(name: &[u8], size: u32, _crc: u32) -> Vec<u8> {
        let mut h = Vec::new();
        h.extend_from_slice(&[0x50, 0x4B, 0x03, 0x04]);
        h.extend_from_slice(&20u16.to_le_bytes());
        h.extend_from_slice(&0u16.to_le_bytes());
        h.extend_from_slice(&0u16.to_le_bytes());
        h.extend_from_slice(&0u16.to_le_bytes());
        h.extend_from_slice(&size.to_le_bytes());
        h.extend_from_slice(&size.to_le_bytes());
        h.extend_from_slice(&0u32.to_le_bytes());
        h.extend_from_slice(&(name.len() as u16).to_le_bytes());
        h.extend_from_slice(&0u16.to_le_bytes());
        h.extend_from_slice(name);
        h
    }

    fn build_central_dir_entry(name: &[u8], size: u32, _crc: u32) -> Vec<u8> {
        let mut h = Vec::new();
        h.extend_from_slice(&[0x50, 0x4B, 0x01, 0x02]);
        h.extend_from_slice(&20u16.to_le_bytes());
        h.extend_from_slice(&20u16.to_le_bytes());
        h.extend_from_slice(&0u16.to_le_bytes());
        h.extend_from_slice(&0u16.to_le_bytes());
        h.extend_from_slice(&0u16.to_le_bytes());
        h.extend_from_slice(&size.to_le_bytes());
        h.extend_from_slice(&size.to_le_bytes());
        h.extend_from_slice(&0u32.to_le_bytes());
        h.extend_from_slice(&(name.len() as u16).to_le_bytes());
        h.extend_from_slice(&0u16.to_le_bytes());
        h.extend_from_slice(&0u16.to_le_bytes());
        h.extend_from_slice(&0u16.to_le_bytes());
        h.extend_from_slice(&0u32.to_le_bytes());
        h.extend_from_slice(&0u32.to_le_bytes());
        h.extend_from_slice(&0u32.to_le_bytes());
        h.extend_from_slice(&0u32.to_le_bytes());
        h.extend_from_slice(name);
        h
    }

    fn build_eocd(entries: u16, cd_size: u32, cd_offset: u32) -> Vec<u8> {
        let mut h = Vec::new();
        h.extend_from_slice(&[0x50, 0x4B, 0x05, 0x06]);
        h.extend_from_slice(&0u16.to_le_bytes());
        h.extend_from_slice(&0u16.to_le_bytes());
        h.extend_from_slice(&entries.to_le_bytes());
        h.extend_from_slice(&entries.to_le_bytes());
        h.extend_from_slice(&cd_size.to_le_bytes());
        h.extend_from_slice(&cd_offset.to_le_bytes());
        h.extend_from_slice(&0u16.to_le_bytes());
        h
    }

    #[test]
    fn test_xml_tokenizer_basic() {
        let xml = "<root><child>hello</child></root>";
        let tokens = tokenize_xml(xml);
        assert!(matches!(&tokens[0], XmlEvent::Open { tag, .. } if tag == "root"));
        assert!(matches!(&tokens[1], XmlEvent::Open { tag, .. } if tag == "child"));
        assert!(matches!(&tokens[2], XmlEvent::Text { content } if content == "hello"));
        assert!(matches!(&tokens[3], XmlEvent::Close { tag } if tag == "child"));
        assert!(matches!(&tokens[4], XmlEvent::Close { tag } if tag == "root"));
    }

    #[test]
    fn test_xml_tokenizer_with_attrs() {
        let xml = r#"<w:p w:val="test">text</w:p>"#;
        let tokens = tokenize_xml(xml);
        if let XmlEvent::Open { tag, attrs } = &tokens[0] {
            assert_eq!(tag, "p");
            assert_eq!(attrs.len(), 1);
            assert_eq!(attrs[0].0, "val");
            assert_eq!(attrs[0].1, "test");
        } else {
            panic!("expected Open event");
        }
    }

    #[test]
    fn test_xml_tokenizer_self_closing() {
        let xml = "<br/>";
        let tokens = tokenize_xml(xml);
        assert!(matches!(&tokens[0], XmlEvent::Open { tag, .. } if tag == "br"));
        assert!(matches!(&tokens[1], XmlEvent::Close { tag } if tag == "br"));
    }

    #[test]
    fn test_xml_tokenizer_cdata() {
        let xml = "<root><![CDATA[some <raw> data]]></root>";
        let tokens = tokenize_xml(xml);
        assert!(matches!(&tokens[1], XmlEvent::Text { content } if content == "some <raw> data"));
    }

    #[test]
    fn test_xml_tokenizer_comment() {
        let xml = "<!-- a comment --><root/>";
        let tokens = tokenize_xml(xml);
        assert!(matches!(&tokens[0], XmlEvent::Open { tag, .. } if tag == "root"));
    }

    #[test]
    fn test_build_dom() {
        let xml = "<a>hello</a><b>world</b>";
        let tokens = tokenize_xml(xml);
        let dom = build_dom(&tokens);
        assert_eq!(dom.children.len(), 2);
        assert_eq!(dom.children[0].tag, "a");
        assert_eq!(dom.children[0].text.as_deref(), Some("hello"));
        assert_eq!(dom.children[1].tag, "b");
        assert_eq!(dom.children[0].tag, "a");
        assert_eq!(dom.children[0].text.as_deref(), Some("hello"));
        assert_eq!(dom.children[1].tag, "b");
    }

    #[test]
    fn test_empty_docx() {
        let module = parse_docx(&[]);
        let docs = find_nodes(&module, |nt| matches!(nt, NodeType::Document));
        assert_eq!(docs.len(), 1);
    }

    #[test]
    fn test_simple_docx_paragraph() {
        let xml = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Hello World</w:t></w:r></w:p></w:body></w:document>"#;
        let tokens = tokenize_xml(xml);
        let dom = build_dom(&tokens);
        let module = convert_dom(&dom);
        let paras = find_nodes(&module, |nt| matches!(nt, NodeType::Paragraph));
        assert_eq!(paras.len(), 1);
        let texts = find_nodes(&module, |nt| matches!(nt, NodeType::Text { content } if content.contains("Hello")));
        assert_eq!(texts.len(), 1);
    }

    #[test]
    fn test_docx_bold() {
        let xml = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:rPr><w:b/></w:rPr><w:t>bold text</w:t></w:r></w:p></w:body></w:document>"#;
        let tokens = tokenize_xml(xml);
        let dom = build_dom(&tokens);
        let module = convert_dom(&dom);
        let bolds = find_nodes(&module, |nt| matches!(nt, NodeType::Bold));
        assert_eq!(bolds.len(), 1);
    }

    #[test]
    fn test_docx_italic() {
        let xml = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:rPr><w:i/></w:rPr><w:t>italic text</w:t></w:r></w:p></w:body></w:document>"#;
        let tokens = tokenize_xml(xml);
        let dom = build_dom(&tokens);
        let module = convert_dom(&dom);
        let italics = find_nodes(&module, |nt| matches!(nt, NodeType::Italic));
        assert_eq!(italics.len(), 1);
    }

    #[test]
    fn test_docx_heading() {
        let xml = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Title</w:t></w:r></w:p></w:body></w:document>"#;
        let tokens = tokenize_xml(xml);
        let dom = build_dom(&tokens);
        let module = convert_dom(&dom);
        let chapters = find_nodes(&module, |nt| matches!(nt, NodeType::Chapter));
        assert_eq!(chapters.len(), 1);
        assert_eq!(chapters[0].counter.as_deref(), Some("Title"));
    }

    #[test]
    fn test_docx_table() {
        let xml = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:tbl><w:tr><w:tc><w:p><w:r><w:t>H1</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>H2</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:p><w:r><w:t>C1</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>C2</w:t></w:r></w:p></w:tc></w:tr></w:tbl></w:body></w:document>"#;
        let tokens = tokenize_xml(xml);
        let dom = build_dom(&tokens);
        let module = convert_dom(&dom);
        let tables = find_nodes(&module, |nt| matches!(nt, NodeType::Table { .. }));
        assert_eq!(tables.len(), 1);
        if let NodeType::Table { num_cols, .. } = &tables[0].node_type {
            assert_eq!(*num_cols, 2);
        }
        let rows = find_nodes(&module, |nt| matches!(nt, NodeType::TableRow { .. }));
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn test_docx_hyperlink() {
        let xml = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:hyperlink w:anchor="https://example.com"><w:r><w:t>click</w:t></w:r></w:hyperlink></w:p></w:body></w:document>"#;
        let tokens = tokenize_xml(xml);
        let dom = build_dom(&tokens);
        let module = convert_dom(&dom);
        let links = find_nodes(&module, |nt| matches!(nt, NodeType::Link { .. }));
        assert_eq!(links.len(), 1);
        if let NodeType::Link { url, title, .. } = &links[0].node_type {
            assert_eq!(url, "https://example.com");
            assert_eq!(title.as_deref(), Some("click"));
        }
    }

    #[test]
    fn test_docx_multiple_paragraphs() {
        let xml = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>First</w:t></w:r></w:p><w:p><w:r><w:t>Second</w:t></w:r></w:p></w:body></w:document>"#;
        let tokens = tokenize_xml(xml);
        let dom = build_dom(&tokens);
        let module = convert_dom(&dom);
        let paras = find_nodes(&module, |nt| matches!(nt, NodeType::Paragraph));
        assert_eq!(paras.len(), 2);
    }

    #[test]
    fn test_docx_underline() {
        let xml = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:rPr><w:u/></w:rPr><w:t>underlined</w:t></w:r></w:p></w:body></w:document>"#;
        let tokens = tokenize_xml(xml);
        let dom = build_dom(&tokens);
        let module = convert_dom(&dom);
        let underlines = find_nodes(&module, |nt| matches!(nt, NodeType::Underline));
        assert_eq!(underlines.len(), 1);
    }

    #[test]
    fn test_docx_strikethrough() {
        let xml = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:rPr><w:strike/></w:rPr><w:t>strikethrough</w:t></w:r></w:p></w:body></w:document>"#;
        let tokens = tokenize_xml(xml);
        let dom = build_dom(&tokens);
        let module = convert_dom(&dom);
        let strikes = find_nodes(&module, |nt| matches!(nt, NodeType::Strikethrough));
        assert_eq!(strikes.len(), 1);
    }

    #[test]
    fn test_docx_source_format() {
        let docx = make_simple_docx("<w:p/>");
        let module = parse_docx(&docx);
        assert_eq!(module.header.source_format.as_deref(), Some("docx"));
    }
}
