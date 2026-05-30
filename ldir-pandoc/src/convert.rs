use ldir_ir::sir::v2::module::SIRModuleV2;
use ldir_ir::sir::v2::nodes::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PandocError {
    #[error("JSON serialization failed: {0}")]
    JsonError(#[from] serde_json::Error),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PandocAttr {
    pub identifier: String,
    #[serde(rename = "classes")]
    pub classes: Vec<String>,
    #[serde(rename = "key-value")]
    pub key_value: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct PandocDoc {
    #[serde(rename = "pandoc-api-version")]
    pub api_version: [u32; 2],
    pub meta: HashMap<String, serde_json::Value>,
    pub blocks: Vec<PandocBlock>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "t", content = "c")]
pub enum PandocBlock {
    #[serde(rename = "Plain")]
    Plain(Vec<PandocInline>),
    #[serde(rename = "Para")]
    Para(Vec<PandocInline>),
    #[serde(rename = "Header")]
    Header(u32, Vec<PandocAttr>, Vec<PandocInline>),
    #[serde(rename = "BulletList")]
    BulletList(Vec<Vec<PandocBlock>>),
    #[serde(rename = "OrderedList")]
    OrderedList(Vec<PandocAttr>, Vec<Vec<PandocBlock>>),
    #[serde(rename = "BlockQuote")]
    BlockQuote(Vec<PandocBlock>),
    #[serde(rename = "CodeBlock")]
    CodeBlock(PandocAttr, String),
    #[serde(rename = "Table")]
    Table(
        Vec<PandocAttr>,
        Vec<PandocBlock>,
        Vec<Vec<Vec<PandocBlock>>>,
        Vec<Vec<Vec<PandocBlock>>>,
    ),
    #[serde(rename = "Div")]
    Div(PandocAttr, Vec<PandocBlock>),
    #[serde(rename = "HorizontalRule")]
    HorizontalRule,
    #[serde(rename = "Null")]
    Null,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "t", content = "c")]
pub enum PandocInline {
    #[serde(rename = "Str")]
    Str(String),
    #[serde(rename = "Emph")]
    Emph(Vec<PandocInline>),
    #[serde(rename = "Strong")]
    Strong(Vec<PandocInline>),
    #[serde(rename = "Strikeout")]
    Strikeout(Vec<PandocInline>),
    #[serde(rename = "Code")]
    Code(PandocAttr, String),
    #[serde(rename = "Link")]
    Link(PandocAttr, Vec<PandocInline>, serde_json::Value),
    #[serde(rename = "Image")]
    Image(PandocAttr, Vec<PandocInline>, serde_json::Value),
    #[serde(rename = "Math")]
    Math(String, String),
    #[serde(rename = "Space")]
    Space,
    #[serde(rename = "SoftBreak")]
    SoftBreak,
    #[serde(rename = "LineBreak")]
    LineBreak,
    #[serde(rename = "Span")]
    Span(PandocAttr, Vec<PandocInline>),
}

fn empty_attr() -> PandocAttr {
    PandocAttr {
        identifier: String::new(),
        classes: Vec::new(),
        key_value: Vec::new(),
    }
}

fn make_str(s: &str) -> PandocInline {
    PandocInline::Str(s.to_string())
}

fn slugify(text: &str) -> String {
    let mut slug = String::new();
    for ch in text.chars() {
        if ch.is_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else if !slug.is_empty() && slug.ends_with('-') {
            continue;
        } else if slug.is_empty() || ch.is_whitespace() || ch == '-' || ch == '_' {
            if !slug.is_empty() {
                slug.push('-');
            }
        } else {
            slug.push(ch);
        }
    }
    if slug.ends_with('-') {
        slug.pop();
    }
    slug
}

pub fn sir_to_pandoc_json(module: &SIRModuleV2) -> Result<String, PandocError> {
    let mut converter = PandocConverter::new();
    let doc = converter.convert(module);
    serde_json::to_string_pretty(&doc).map_err(PandocError::from)
}

struct PandocConverter {
    heading_counter: [u32; 6],
}

impl PandocConverter {
    fn new() -> Self {
        Self {
            heading_counter: [0; 6],
        }
    }

    fn convert(&mut self, module: &SIRModuleV2) -> PandocDoc {
        let mut meta: HashMap<String, serde_json::Value> = HashMap::new();
        if let Some(ref title) = module.metadata.title {
            meta.insert(
                "title".to_string(),
                serde_json::json!({
                    "t": "MetaInlines",
                    "c": [{"t": "Str", "c": title}]
                }),
            );
        }
        if let Some(ref author) = module.metadata.author {
            meta.insert(
                "author".to_string(),
                serde_json::json!({
                    "t": "MetaInlines",
                    "c": [{"t": "Str", "c": author}]
                }),
            );
        }

        let mut blocks = Vec::new();
        for &root_id in module.body.roots() {
            if let Some(root) = module.body.get(root_id) {
                self.convert_block(&mut blocks, module, root);
            }
        }

        PandocDoc {
            api_version: [1, 23],
            meta,
            blocks,
        }
    }

    fn convert_block(&mut self, blocks: &mut Vec<PandocBlock>, module: &SIRModuleV2, node: &Node) {
        match &node.node_type {
            NodeType::Document => {
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.convert_block(blocks, module, child);
                    }
                }
            }

            NodeType::Part
            | NodeType::Chapter
            | NodeType::Section
            | NodeType::Subsection
            | NodeType::Subsubsection => {
                let level = node.heading_level().unwrap_or(2);
                if level > 0 && (level as usize) < 6 {
                    self.heading_counter[level as usize] += 1;
                    for i in (level as usize + 1)..6 {
                        self.heading_counter[i] = 0;
                    }
                }
                let pandoc_level = (level + 1).min(6) as u32;
                let text = module.body.collect_text(node.id);
                let attr = PandocAttr {
                    identifier: slugify(&text),
                    classes: Vec::new(),
                    key_value: Vec::new(),
                };
                let inlines = if text.is_empty() {
                    Vec::new()
                } else {
                    vec![make_str(&text)]
                };
                blocks.push(PandocBlock::Header(pandoc_level, vec![attr], inlines));

                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.convert_block(blocks, module, child);
                    }
                }
            }

            NodeType::Paragraph => {
                let inlines = self.collect_inlines(module, node);
                if !inlines.is_empty() {
                    blocks.push(PandocBlock::Para(inlines));
                }
            }

            NodeType::Text { content } => {
                if !content.is_empty() {
                    blocks.push(PandocBlock::Plain(vec![make_str(content)]));
                }
            }

            NodeType::List { ordered, .. } => {
                let mut items: Vec<Vec<PandocBlock>> = Vec::new();
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id)
                        && let NodeType::ListItem = &child.node_type
                    {
                        let mut item_blocks = Vec::new();
                        for &gc_id in &child.child_ids {
                            if let Some(gc) = module.body.get(gc_id) {
                                self.convert_block(&mut item_blocks, module, gc);
                            }
                        }
                        if item_blocks.is_empty() {
                            let inlines = self.collect_inlines(module, child);
                            if !inlines.is_empty() {
                                item_blocks.push(PandocBlock::Plain(inlines));
                            }
                        }
                        items.push(item_blocks);
                    }
                }
                if *ordered {
                    blocks.push(PandocBlock::OrderedList(vec![empty_attr()], items));
                } else {
                    blocks.push(PandocBlock::BulletList(items));
                }
            }

            NodeType::ListItem => {
                let inlines = self.collect_inlines(module, node);
                if !inlines.is_empty() {
                    blocks.push(PandocBlock::Plain(inlines));
                }
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id)
                        && matches!(&child.node_type, NodeType::List { .. })
                    {
                        self.convert_block(blocks, module, child);
                    }
                }
            }

            NodeType::BlockQuote => {
                let mut inner = Vec::new();
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.convert_block(&mut inner, module, child);
                    }
                }
                blocks.push(PandocBlock::BlockQuote(inner));
            }

            NodeType::CodeBlock { language, content } => {
                let mut attr = empty_attr();
                if let Some(lang) = language {
                    attr.classes.push(lang.clone());
                }
                let text = if content.is_empty() {
                    module.body.collect_text(node.id)
                } else {
                    content.clone()
                };
                blocks.push(PandocBlock::CodeBlock(attr, text));
            }

            NodeType::MathBlock { .. } => {
                let text = module.body.collect_text(node.id);
                blocks.push(PandocBlock::Para(vec![PandocInline::Math(
                    "DisplayMath".to_string(),
                    text,
                )]));
            }

            NodeType::Table { .. } => {
                let mut header_row: Vec<Vec<Vec<PandocBlock>>> = Vec::new();
                let mut body_rows: Vec<Vec<Vec<PandocBlock>>> = Vec::new();
                let mut col_widths: Vec<PandocAttr> = Vec::new();
                let mut num_cols = 0;

                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id)
                        && let NodeType::TableRow { is_header } = &child.node_type
                    {
                        let mut row: Vec<Vec<PandocBlock>> = Vec::new();
                        for &cell_id in &child.child_ids {
                            if let Some(cell_node) = module.body.get(cell_id) {
                                let inlines = self.collect_inlines(module, cell_node);
                                row.push(vec![PandocBlock::Plain(inlines)]);
                            }
                        }
                        num_cols = num_cols.max(row.len());
                        if *is_header {
                            header_row.push(row);
                        } else {
                            body_rows.push(row);
                        }
                    }
                }

                for _ in 0..num_cols {
                    col_widths.push(empty_attr());
                }

                let caption = vec![PandocBlock::Plain(Vec::new())];
                blocks.push(PandocBlock::Table(
                    col_widths, caption, header_row, body_rows,
                ));
            }

            NodeType::ThematicBreak => {
                blocks.push(PandocBlock::HorizontalRule);
            }

            NodeType::Figure { .. } => {
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.convert_block(blocks, module, child);
                    }
                }
            }

            NodeType::Image { source, alt, .. } => {
                let target = serde_json::json!({ "t": "Plain", "c": source });
                blocks.push(PandocBlock::Para(vec![PandocInline::Image(
                    empty_attr(),
                    if alt.is_empty() {
                        Vec::new()
                    } else {
                        vec![make_str(alt)]
                    },
                    target,
                )]));
            }

            NodeType::Link { url, .. } => {
                let target = serde_json::json!({ "t": "Plain", "c": url });
                blocks.push(PandocBlock::Para(vec![PandocInline::Link(
                    empty_attr(),
                    Vec::new(),
                    target,
                )]));
            }

            NodeType::MathInline { content } => {
                blocks.push(PandocBlock::Para(vec![PandocInline::Math(
                    "InlineMath".to_string(),
                    content.clone(),
                )]));
            }

            NodeType::Group => {
                let inlines = self.collect_inlines(module, node);
                if !inlines.is_empty() {
                    blocks.push(PandocBlock::Para(inlines));
                }
            }

            _ => {
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.convert_block(blocks, module, child);
                    }
                }
            }
        }
    }

    fn collect_inlines(&self, module: &SIRModuleV2, node: &Node) -> Vec<PandocInline> {
        let mut inlines = Vec::new();
        for &child_id in &node.child_ids {
            if let Some(child) = module.body.get(child_id) {
                self.collect_inlines_recursive(module, child, &mut inlines);
            }
        }
        inlines
    }

    fn collect_inlines_recursive(
        &self,
        module: &SIRModuleV2,
        node: &Node,
        out: &mut Vec<PandocInline>,
    ) {
        match &node.node_type {
            NodeType::Text { content } => {
                if !content.is_empty() {
                    out.push(make_str(content));
                }
            }

            NodeType::Bold => {
                let inner = self.collect_inlines(module, node);
                if !inner.is_empty() {
                    out.push(PandocInline::Strong(inner));
                }
            }

            NodeType::Italic => {
                let inner = self.collect_inlines(module, node);
                if !inner.is_empty() {
                    out.push(PandocInline::Emph(inner));
                }
            }

            NodeType::Mono => {
                let text = module.body.collect_text(node.id);
                out.push(PandocInline::Code(empty_attr(), text));
            }

            NodeType::Strikethrough => {
                let inner = self.collect_inlines(module, node);
                if !inner.is_empty() {
                    out.push(PandocInline::Strikeout(inner));
                }
            }

            NodeType::Underline => {
                let inner = self.collect_inlines(module, node);
                if !inner.is_empty() {
                    out.push(PandocInline::Span(empty_attr(), inner));
                }
            }

            NodeType::SmallCaps => {
                let inner = self.collect_inlines(module, node);
                out.extend(inner);
            }

            NodeType::Link { url, title } => {
                let inner = self.collect_inlines(module, node);
                let link_title = title.as_deref().unwrap_or("").to_string();
                let target = serde_json::json!({ "t": "Plain", "c": url });
                out.push(PandocInline::Link(
                    PandocAttr {
                        identifier: String::new(),
                        classes: Vec::new(),
                        key_value: if link_title.is_empty() {
                            Vec::new()
                        } else {
                            vec!["title".to_string(), link_title]
                        },
                    },
                    if inner.is_empty() {
                        vec![make_str(url)]
                    } else {
                        inner
                    },
                    target,
                ));
            }

            NodeType::Image { source, alt, .. } => {
                let inner = self.collect_inlines(module, node);
                let target = serde_json::json!({ "t": "Plain", "c": source });
                out.push(PandocInline::Image(
                    empty_attr(),
                    if inner.is_empty() && !alt.is_empty() {
                        vec![make_str(alt)]
                    } else {
                        inner
                    },
                    target,
                ));
            }

            NodeType::MathInline { content } => {
                out.push(PandocInline::Math(
                    "InlineMath".to_string(),
                    content.clone(),
                ));
            }

            NodeType::LineBreak => {
                out.push(PandocInline::LineBreak);
            }

            NodeType::Footnote { content } => {
                out.push(PandocInline::Str(format!("[^{}]", content)));
            }

            NodeType::Styled { .. } => {
                let inner = self.collect_inlines(module, node);
                out.extend(inner);
            }

            NodeType::Group => {
                let inner = self.collect_inlines(module, node);
                out.extend(inner);
            }

            NodeType::Reference { label } => {
                out.push(PandocInline::Str(label.clone()));
            }

            _ => {
                let inner = self.collect_inlines(module, node);
                out.extend(inner);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_doc_module() -> SIRModuleV2 {
        let mut m = SIRModuleV2::new();
        m.metadata.title = Some("Test Document".into());
        m.metadata.author = Some("Test Author".into());
        m.body.push(Node::new(0, NodeType::Document));
        m.body.push(
            Node::new(1, NodeType::Section)
                .with_parent(0)
                .with_label("sec:intro"),
        );
        m.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "Introduction".into(),
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
                    content: "Hello, world!".into(),
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

    #[test]
    fn test_basic_conversion() {
        let m = make_doc_module();
        let json = sir_to_pandoc_json(&m).expect("conversion succeeded");
        let doc: serde_json::Value = serde_json::from_str(&json).expect("valid json");

        assert_eq!(doc["pandoc-api-version"], serde_json::json!([1, 23]));
        assert!(doc["meta"]["title"]["c"][0]["c"].as_str() == Some("Test Document"));
        assert!(doc["meta"]["author"]["c"][0]["c"].as_str() == Some("Test Author"));

        let blocks = doc["blocks"].as_array().expect("blocks array");
        assert!(!blocks.is_empty());
    }

    #[test]
    fn test_header_block() {
        let m = make_doc_module();
        let json = sir_to_pandoc_json(&m).expect("conversion succeeded");
        let doc: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        let blocks = doc["blocks"].as_array().expect("blocks array");

        let header = &blocks[0];
        assert_eq!(header["t"], "Header");
        let c = header["c"].as_array().expect("header content");
        assert_eq!(c[0], 3);
    }

    #[test]
    fn test_nested_formatting() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::Paragraph).with_parent(0));
        m.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "Normal ".into(),
                },
            )
            .with_parent(1),
        );
        m.body.push(Node::new(3, NodeType::Bold).with_parent(1));
        m.body.push(
            Node::new(
                4,
                NodeType::Text {
                    content: "bold".into(),
                },
            )
            .with_parent(3),
        );
        m.body.push(Node::new(5, NodeType::Italic).with_parent(1));
        m.body.push(
            Node::new(
                6,
                NodeType::Text {
                    content: "italic".into(),
                },
            )
            .with_parent(5),
        );
        m.body.push(Node::new(7, NodeType::Mono).with_parent(1));
        m.body.push(
            Node::new(
                8,
                NodeType::Text {
                    content: "code".into(),
                },
            )
            .with_parent(7),
        );
        if let Some(n) = m.body.get_mut(0) {
            n.add_child(1);
        }
        if let Some(n) = m.body.get_mut(1) {
            n.add_child(2);
            n.add_child(3);
            n.add_child(5);
            n.add_child(7);
        }
        if let Some(n) = m.body.get_mut(3) {
            n.add_child(4);
        }
        if let Some(n) = m.body.get_mut(5) {
            n.add_child(6);
        }
        if let Some(n) = m.body.get_mut(7) {
            n.add_child(8);
        }

        let json = sir_to_pandoc_json(&m).expect("conversion succeeded");
        let doc: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        let blocks = doc["blocks"].as_array().expect("blocks array");
        let para = &blocks[0];
        assert_eq!(para["t"], "Para");
        let inlines = para["c"].as_array().expect("para inlines");

        let mut found_strong = false;
        let mut found_emph = false;
        let mut found_code = false;
        for inline in inlines {
            if inline["t"] == "Strong" {
                found_strong = true;
            }
            if inline["t"] == "Emph" {
                found_emph = true;
            }
            if inline["t"] == "Code" {
                found_code = true;
            }
        }
        assert!(found_strong);
        assert!(found_emph);
        assert!(found_code);
    }

    #[test]
    fn test_table() {
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
                    content: "A".into(),
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
                    content: "B".into(),
                },
            )
            .with_parent(5),
        );
        m.body
            .push(Node::new(7, NodeType::TableRow { is_header: false }).with_parent(1));
        m.body.push(
            Node::new(
                8,
                NodeType::TableCell {
                    colspan: 1,
                    rowspan: 1,
                },
            )
            .with_parent(7),
        );
        m.body.push(
            Node::new(
                9,
                NodeType::Text {
                    content: "1".into(),
                },
            )
            .with_parent(8),
        );
        if let Some(n) = m.body.get_mut(0) {
            n.add_child(1);
        }
        if let Some(n) = m.body.get_mut(1) {
            n.add_child(2);
            n.add_child(7);
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
        if let Some(n) = m.body.get_mut(7) {
            n.add_child(8);
        }
        if let Some(n) = m.body.get_mut(8) {
            n.add_child(9);
        }

        let json = sir_to_pandoc_json(&m).expect("conversion succeeded");
        let doc: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        let blocks = doc["blocks"].as_array().expect("blocks array");
        let table = &blocks[0];
        assert_eq!(table["t"], "Table");
        let c = table["c"].as_array().expect("table content");
        assert_eq!(c.len(), 4);
    }

    #[test]
    fn test_list() {
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
                    content: "Item 1".into(),
                },
            )
            .with_parent(2),
        );
        m.body.push(Node::new(4, NodeType::ListItem).with_parent(1));
        m.body.push(
            Node::new(
                5,
                NodeType::Text {
                    content: "Item 2".into(),
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

        let json = sir_to_pandoc_json(&m).expect("conversion succeeded");
        let doc: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        let blocks = doc["blocks"].as_array().expect("blocks array");
        assert_eq!(blocks[0]["t"], "BulletList");
        let items = blocks[0]["c"].as_array().expect("list items");
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_math() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body.push(
            Node::new(
                1,
                NodeType::MathBlock {
                    math_type: MathType::Equation,
                    numbered: false,
                },
            )
            .with_parent(0),
        );
        m.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "x^2 + y^2 = z^2".into(),
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

        let json = sir_to_pandoc_json(&m).expect("conversion succeeded");
        let doc: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        let blocks = doc["blocks"].as_array().expect("blocks array");
        assert_eq!(blocks[0]["t"], "Para");
        let inlines = blocks[0]["c"].as_array().expect("inlines");
        assert_eq!(inlines[0]["t"], "Math");
        assert_eq!(inlines[0]["c"][0], "DisplayMath");
    }

    #[test]
    fn test_code_block() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body.push(
            Node::new(
                1,
                NodeType::CodeBlock {
                    language: Some("python".into()),
                    content: "print('hello')".into(),
                },
            )
            .with_parent(0),
        );
        if let Some(n) = m.body.get_mut(0) {
            n.add_child(1);
        }

        let json = sir_to_pandoc_json(&m).expect("conversion succeeded");
        let doc: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        let blocks = doc["blocks"].as_array().expect("blocks array");
        assert_eq!(blocks[0]["t"], "CodeBlock");
        let attr = &blocks[0]["c"][0];
        assert!(
            attr["classes"]
                .as_array()
                .expect("classes")
                .contains(&serde_json::json!("python"))
        );
        assert_eq!(blocks[0]["c"][1], "print('hello')");
    }
}
