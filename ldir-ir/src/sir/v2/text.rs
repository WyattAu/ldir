//! Text format (.ldir) for S-IR v2.
//!
//! A simple, human-readable format that can be version-controlled,
//! manually authored, and diffed.

use super::module::SIRModuleV2;

pub fn text_to_module(text: &str) -> Result<SIRModuleV2, String> {
    use crate::sir::v2::{annotations, nodes, resources, styles};

    let mut module = SIRModuleV2::new();
    let mut next_id: u32 = 0;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(";;") {
            continue;
        }

        if line.starts_with("@meta") {
            continue;
        }
        if line.starts_with("@body") {
            continue;
        }

        if line.starts_with("@font") {
            if let Some(name) = extract_quoted(line, "font ") {
                let family = extract_quoted(line, "family = ").unwrap_or_default();
                module.resources.fonts.push(resources::FontDecl {
                    name,
                    family,
                    weight: resources::FontWeight::Regular,
                    style: resources::FontStyle::Normal,
                    source: resources::FontSource::System,
                    features: Vec::new(),
                });
            }
            continue;
        }

        if line.starts_with("@counter") {
            if let Some(name) = extract_quoted(line, "counter ") {
                module.resources.counters.push(resources::CounterDecl {
                    name,
                    format: resources::CounterFormat::Arabic,
                    reset_scope: resources::CounterReset::PerSection,
                });
            }
            continue;
        }

        if line.starts_with("@style") {
            if let Some(name) = extract_quoted(line, "style ") {
                let parent = extract_quoted(line, "parent = ");
                module.styles.styles.push(styles::StyleDecl {
                    name,
                    parent,
                    properties: styles::StyleProperties::default(),
                });
            }
            continue;
        }

        if let Some((tag, attrs_str, body_str)) = parse_node_line(line) {
            let id = extract_attr_num(attrs_str, "id=").unwrap_or(next_id);
            let parent_id = extract_attr_num(attrs_str, "parent=");
            let label = extract_attr_quoted(attrs_str, "label=");
            let style = extract_attr_quoted(attrs_str, "style=");
            let counter = extract_attr_quoted(attrs_str, "counter=");

            if id >= next_id {
                next_id = id + 1;
            }

            let node_type = match tag {
                "document" => nodes::NodeType::Document,
                "part" => nodes::NodeType::Part,
                "chapter" => nodes::NodeType::Chapter,
                "section" => nodes::NodeType::Section,
                "subsection" => nodes::NodeType::Subsection,
                "subsubsection" => nodes::NodeType::Subsubsection,
                "paragraph" => nodes::NodeType::Paragraph,
                "list" => nodes::NodeType::List {
                    list_type: nodes::ListType::Unordered,
                    ordered: false,
                    start: None,
                },
                "list-item" => nodes::NodeType::ListItem,
                "blockquote" => nodes::NodeType::BlockQuote,
                "equation" => nodes::NodeType::MathBlock {
                    math_type: nodes::MathType::Equation,
                    numbered: body_str.contains("numbered=true"),
                },
                "text" => nodes::NodeType::Text {
                    content: extract_braced_quoted(body_str).unwrap_or_default(),
                },
                "link" => nodes::NodeType::Link {
                    url: extract_braced_field(body_str, "url=").unwrap_or_default(),
                    title: None,
                },
                "image" => nodes::NodeType::Image {
                    source: extract_braced_field(body_str, "src=").unwrap_or_default(),
                    alt: String::new(),
                    width: None,
                    height: None,
                    placement: nodes::FloatPlacement::Here,
                },
                "footnote" => nodes::NodeType::Footnote {
                    content: extract_braced_quoted(body_str).unwrap_or_default(),
                },
                "figure" => nodes::NodeType::Figure {
                    placement: nodes::FloatPlacement::Here,
                },
                "caption" => nodes::NodeType::Caption,
                "code-block" => nodes::NodeType::CodeBlock {
                    language: None,
                    content: String::new(),
                },
                "toc" => nodes::NodeType::TableOfContents { max_depth: 3 },
                "hr" => nodes::NodeType::ThematicBreak,
                "page-break" => nodes::NodeType::PageBreak,
                "bold" => nodes::NodeType::Bold,
                "italic" => nodes::NodeType::Italic,
                "mono" => nodes::NodeType::Mono,
                "underline" => nodes::NodeType::Underline,
                "strike" => nodes::NodeType::Strikethrough,
                "smallcaps" => nodes::NodeType::SmallCaps,
                "styled" => nodes::NodeType::Styled {
                    style_name: extract_braced_field(body_str, "style=").unwrap_or_default(),
                },
                "group" => nodes::NodeType::Group,
                "math" => nodes::NodeType::MathInline {
                    content: extract_braced_quoted(body_str).unwrap_or_default(),
                },
                "table" => nodes::NodeType::Table {
                    col_specs: Vec::new(),
                    num_cols: 0,
                    caption: None,
                    column_widths: Vec::new(),
                    header_row: false,
                },
                "table-row" => nodes::NodeType::TableRow { is_header: false },
                "table-cell" => nodes::NodeType::TableCell {
                    colspan: 1,
                    rowspan: 1,
                },
                "footnote-block" => nodes::NodeType::FootnoteBlock,
                "endnote" => nodes::NodeType::Endnote {
                    content: extract_braced_quoted(body_str).unwrap_or_default(),
                },
                "comment" => nodes::NodeType::Comment {
                    author: extract_braced_field(body_str, "author=")
                        .unwrap_or_else(|| "Anonymous".into()),
                    content: extract_braced_quoted(body_str).unwrap_or_default(),
                },
                "br" => nodes::NodeType::LineBreak,
                _ => continue,
            };

            let mut node = nodes::Node::new(id, node_type);
            if let Some(pid) = parent_id {
                node = node.with_parent(pid);
            }
            if let Some(l) = label {
                node.label = Some(l);
            }
            if let Some(s) = style {
                node.style = Some(s);
            }
            if let Some(c) = counter {
                node.counter = Some(c);
            }

            if let Some(ref l) = node.label {
                let category = if node.is_heading() {
                    annotations::LabelCategory::Section
                } else if matches!(node.node_type, nodes::NodeType::MathBlock { .. }) {
                    annotations::LabelCategory::Equation
                } else {
                    annotations::LabelCategory::Custom
                };
                module.annotations.add_label(l.clone(), id, category);
            }

            module.body.push(node);
        }
    }

    Ok(module)
}

fn extract_quoted(line: &str, prefix: &str) -> Option<String> {
    let start = line.find(prefix)?;
    let rest = line[start + prefix.len()..].trim_start();
    let inner = rest.strip_prefix('"')?;
    let end = inner.find('"')?;
    Some(inner[..end].to_string())
}

fn extract_attr_num(attrs: &str, key: &str) -> Option<u32> {
    let start = attrs.find(key)?;
    let rest = &attrs[start + key.len()..];
    let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    num_str.parse().ok()
}

fn extract_attr_quoted(attrs: &str, key: &str) -> Option<String> {
    let start = attrs.find(key)?;
    let rest = attrs[start + key.len()..].trim_start();
    let inner = rest.strip_prefix('"')?;
    let end = inner.find('"')?;
    Some(inner[..end].to_string())
}

fn parse_node_line(line: &str) -> Option<(&str, &str, &str)> {
    if !line.starts_with('@') {
        return None;
    }
    let tag_end = line[1..].find(|c: char| c.is_whitespace() || c == '[' || c == '{')?;
    let tag = &line[1..tag_end + 1];
    let rest = line[tag_end + 1..].trim_start();
    let (attrs, body) = if rest.starts_with('[') {
        let bracket_end = rest.find(']')?;
        let attrs = &rest[1..bracket_end];
        let rest2 = rest[bracket_end + 1..].trim_start();
        let body = if rest2.starts_with('{') {
            let brace_end = rest2.rfind('}')?;
            &rest2[1..brace_end]
        } else {
            ""
        };
        (attrs, body)
    } else if rest.starts_with('{') {
        let brace_end = rest.rfind('}')?;
        ("", &rest[1..brace_end])
    } else {
        ("", "")
    };
    Some((tag, attrs, body))
}

fn extract_braced_quoted(s: &str) -> Option<String> {
    let s = s.trim();
    let inner = s.strip_prefix('"')?;
    let end = inner.find('"')?;
    Some(inner[..end].to_string())
}

fn extract_braced_field(s: &str, key: &str) -> Option<String> {
    let start = s.find(key)?;
    let rest = s[start + key.len()..].trim_start();
    let inner = rest.strip_prefix('"')?;
    let end = inner.find('"')?;
    Some(inner[..end].to_string())
}

/// Serialize a module to text format.
pub fn module_to_text(module: &SIRModuleV2) -> String {
    let mut out = String::new();

    // Header comment
    out.push_str(&format!(
        ";; ldir-ir v{}.{}.{}\n",
        module.header.version.0, module.header.version.1, module.header.version.2
    ));
    if let Some(ref fmt) = module.header.source_format {
        out.push_str(&format!(";; source: {}\n", fmt));
    }
    out.push('\n');

    // Metadata
    out.push_str("@meta {\n");
    if let Some(ref title) = module.metadata.title {
        out.push_str(&format!("  title = {:?}\n", title));
    }
    if let Some(ref author) = module.metadata.author {
        out.push_str(&format!("  author = {:?}\n", author));
    }
    out.push_str(&format!("  language = {:?}\n", module.metadata.language));
    if let Some(ref cls) = module.metadata.document_class {
        out.push_str(&format!("  class = {:?}\n", cls));
    }
    out.push_str("}\n\n");

    // Fonts
    for font in &module.resources.fonts {
        out.push_str(&format!(
            "@font {:?} {{ family = {:?}, weight = {:?} }}\n",
            font.name,
            font.family,
            match font.weight {
                crate::sir::v2::resources::FontWeight::Regular => "regular",
                crate::sir::v2::resources::FontWeight::Bold => "bold",
                crate::sir::v2::resources::FontWeight::Light => "light",
                crate::sir::v2::resources::FontWeight::Thin => "thin",
                _ => "regular",
            }
        ));
    }
    if !module.resources.fonts.is_empty() {
        out.push('\n');
    }

    // Counters
    for counter in &module.resources.counters {
        out.push_str(&format!(
            "@counter {:?} {{ format = {:?} }}\n",
            counter.name,
            match &counter.format {
                crate::sir::v2::resources::CounterFormat::Arabic => "arabic",
                crate::sir::v2::resources::CounterFormat::RomanLower => "roman-lower",
                crate::sir::v2::resources::CounterFormat::Custom(s) => s.as_str(),
                _ => "arabic",
            }
        ));
    }
    if !module.resources.counters.is_empty() {
        out.push('\n');
    }

    // Styles
    for style in &module.styles.styles {
        out.push_str(&format!("@style {:?} {{\n", style.name));
        if let Some(ref parent) = style.parent {
            out.push_str(&format!("  parent = {:?}\n", parent));
        }
        out.push_str("}\n\n");
    }

    // Body nodes
    out.push_str("@body {\n");
    for node in module.body.iter() {
        emit_node(&mut out, node, 1);
    }
    out.push_str("}\n");

    out
}

fn emit_node(out: &mut String, node: &crate::sir::v2::nodes::Node, indent: usize) {
    let pad = "  ".repeat(indent);
    let (tag, extra) = node_tag(node);

    let mut attrs = Vec::new();
    attrs.push(format!("id={}", node.id));
    if let Some(pid) = node.parent_id {
        attrs.push(format!("parent={}", pid));
    }
    if let Some(ref label) = node.label {
        attrs.push(format!("label={:?}", label));
    }
    if let Some(ref style) = node.style {
        attrs.push(format!("style={:?}", style));
    }

    out.push_str(&format!(
        "{}@{} [{}] {}\n",
        pad,
        tag,
        attrs.join(", "),
        extra
    ));

    for _child_id in &node.child_ids {
        // Children are emitted at higher indent when we have access to them
        // For now, just reference
    }
}

fn node_tag(node: &crate::sir::v2::nodes::Node) -> (&'static str, String) {
    match &node.node_type {
        crate::sir::v2::nodes::NodeType::Document => ("document", "{}".into()),
        crate::sir::v2::nodes::NodeType::Section => ("section", "{}".into()),
        crate::sir::v2::nodes::NodeType::Paragraph => ("paragraph", "{}".into()),
        crate::sir::v2::nodes::NodeType::Text { content } => {
            ("text", format!("{{ {:?} }}", content))
        }
        crate::sir::v2::nodes::NodeType::MathBlock { numbered, .. } => {
            ("equation", format!("{{ numbered={} }}", numbered))
        }
        crate::sir::v2::nodes::NodeType::List { ordered, .. } => {
            ("list", format!("{{ ordered={} }}", ordered))
        }
        crate::sir::v2::nodes::NodeType::Image { source, .. } => {
            ("image", format!("{{ src={:?} }}", source))
        }
        crate::sir::v2::nodes::NodeType::Link { url, .. } => {
            ("link", format!("{{ url={:?} }}", url))
        }
        crate::sir::v2::nodes::NodeType::Footnote { content } => {
            ("footnote", format!("{{ {:?} }}", content))
        }
        crate::sir::v2::nodes::NodeType::Figure { .. } => ("figure", "{}".into()),
        crate::sir::v2::nodes::NodeType::CodeBlock { language, .. } => (
            "code-block",
            format!("{{ lang={:?} }}", language.as_deref().unwrap_or("")),
        ),
        crate::sir::v2::nodes::NodeType::BlockQuote => ("blockquote", "{}".into()),
        crate::sir::v2::nodes::NodeType::TableOfContents { max_depth } => {
            ("toc", format!("{{ depth={} }}", max_depth))
        }
        crate::sir::v2::nodes::NodeType::ThematicBreak => ("hr", "{}".into()),
        crate::sir::v2::nodes::NodeType::PageBreak => ("page-break", "{}".into()),
        crate::sir::v2::nodes::NodeType::Caption => ("caption", "{}".into()),
        crate::sir::v2::nodes::NodeType::Styled { style_name } => {
            ("styled", format!("{{ style={:?} }}", style_name))
        }
        crate::sir::v2::nodes::NodeType::Bold => ("bold", "{}".into()),
        crate::sir::v2::nodes::NodeType::Italic => ("italic", "{}".into()),
        crate::sir::v2::nodes::NodeType::Mono => ("mono", "{}".into()),
        crate::sir::v2::nodes::NodeType::Group => ("group", "{}".into()),
        crate::sir::v2::nodes::NodeType::Chapter => ("chapter", "{}".into()),
        crate::sir::v2::nodes::NodeType::Part => ("part", "{}".into()),
        crate::sir::v2::nodes::NodeType::Subsection => ("subsection", "{}".into()),
        crate::sir::v2::nodes::NodeType::Subsubsection => ("subsubsection", "{}".into()),
        crate::sir::v2::nodes::NodeType::ListItem => ("list-item", "{}".into()),
        crate::sir::v2::nodes::NodeType::MathInline { content } => {
            ("math", format!("{{ {:?} }}", content))
        }
        crate::sir::v2::nodes::NodeType::LineBreak => ("br", "{}".into()),
        crate::sir::v2::nodes::NodeType::Underline => ("underline", "{}".into()),
        crate::sir::v2::nodes::NodeType::Strikethrough => ("strike", "{}".into()),
        crate::sir::v2::nodes::NodeType::SmallCaps => ("smallcaps", "{}".into()),
        crate::sir::v2::nodes::NodeType::Table { .. } => ("table", "{}".into()),
        crate::sir::v2::nodes::NodeType::TableRow { .. } => ("table-row", "{}".into()),
        crate::sir::v2::nodes::NodeType::TableCell { .. } => ("table-cell", "{}".into()),
        crate::sir::v2::nodes::NodeType::FootnoteBlock => ("footnote-block", "{}".into()),
        crate::sir::v2::nodes::NodeType::Endnote { content } => {
            ("endnote", format!("{{ {:?} }}", content))
        }
        crate::sir::v2::nodes::NodeType::Comment { author, content } => (
            "comment",
            format!("{{ author={:?} {:?} }}", author, content),
        ),
        crate::sir::v2::nodes::NodeType::Citation { keys, .. } => {
            ("citation", format!("{{ keys={:?} }}", keys))
        }
        crate::sir::v2::nodes::NodeType::Reference { label } => {
            ("reference", format!("{{ label={:?} }}", label))
        }
        crate::sir::v2::nodes::NodeType::Label { name } => {
            ("label", format!("{{ name={:?} }}", name))
        }
        crate::sir::v2::nodes::NodeType::TrackedInsert {
            author,
            date,
            revision_id,
        } => (
            "tracked_insert",
            format!(
                "{{ author={:?} date={:?} rev={} }}",
                author, date, revision_id
            ),
        ),
        crate::sir::v2::nodes::NodeType::TrackedDelete {
            author,
            date,
            revision_id,
        } => (
            "tracked_delete",
            format!(
                "{{ author={:?} date={:?} rev={} }}",
                author, date, revision_id
            ),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sir::v2::nodes::{Node, NodeType};

    #[test]
    fn test_text_empty_module() {
        let m = SIRModuleV2::new();
        let text = module_to_text(&m);
        assert!(text.contains(";; ldir-ir v2.0.0"));
        assert!(text.contains("@meta {"));
        assert!(text.contains("@body {"));
    }

    #[test]
    fn test_text_with_metadata() {
        let mut m = SIRModuleV2::from_source("markdown", "readme.md");
        m.metadata.title = Some("Hello World".to_string());
        m.metadata.author = Some("Test".to_string());
        m.metadata.document_class = Some("article".to_string());

        let text = module_to_text(&m);
        assert!(text.contains(";; source: markdown"));
        assert!(text.contains("title = \"Hello World\""));
        assert!(text.contains("author = \"Test\""));
        assert!(text.contains("class = \"article\""));
    }

    #[test]
    fn test_text_with_nodes() {
        let mut m = SIRModuleV2::new();
        m.body
            .push(Node::new(1, NodeType::Section).with_label("sec:intro"));
        m.body
            .push(Node::new(2, NodeType::Paragraph).with_parent(1));
        m.body.push(
            Node::new(
                3,
                NodeType::Text {
                    content: "Hello".to_string(),
                },
            )
            .with_parent(2),
        );

        let text = module_to_text(&m);
        assert!(text.contains("@section [id=1, label=\"sec:intro\"]"));
        assert!(text.contains("@paragraph [id=2, parent=1]"));
        assert!(text.contains("@text [id=3, parent=2] { \"Hello\" }"));
    }

    #[test]
    fn test_text_to_module_basic() -> Result<(), Box<dyn std::error::Error>> {
        let text = r#";; ldir-ir v2.0.0
@meta { title = "Test" }
@section [id=1] { }
@paragraph [id=2, parent=1] { }
@text [id=3, parent=2] { "Hello" }
"#;
        let m = text_to_module(text)?;
        assert_eq!(m.body.len(), 3);
        assert!(m.body.get(1).is_some());
        assert!(m.body.get(2).is_some());
        assert!(m.body.get(3).is_some());
        Ok(())
    }

    #[test]
    fn test_text_to_module_with_style() -> Result<(), Box<dyn std::error::Error>> {
        let text = r#";; ldir-ir v2.0.0
@style "heading" { parent = "body" }
@section [id=1, style="heading"] { }
"#;
        let m = text_to_module(text)?;
        assert_eq!(m.body.len(), 1);
        assert_eq!(m.styles.styles.len(), 1);
        let node = m.body.get(1).ok_or("no node at id 1")?;
        assert_eq!(node.style.as_deref(), Some("heading"));
        Ok(())
    }

    #[test]
    fn test_text_to_module_with_label() -> Result<(), Box<dyn std::error::Error>> {
        let text = r#";; ldir-ir v2.0.0
@section [id=1, label="sec:intro"] { }
"#;
        let m = text_to_module(text)?;
        assert!(m.annotations.find_label("sec:intro").is_some());
        Ok(())
    }

    #[test]
    fn test_text_to_module_with_counter() -> Result<(), Box<dyn std::error::Error>> {
        let text = r#";; ldir-ir v2.0.0
@counter "section" { format = "arabic" }
@section [id=1, counter="section"] { }
"#;
        let m = text_to_module(text)?;
        assert_eq!(m.resources.counters.len(), 1);
        let node = m.body.get(1).ok_or("no node at id 1")?;
        assert_eq!(node.counter.as_deref(), Some("section"));
        Ok(())
    }

    #[test]
    fn test_text_to_module_with_font() -> Result<(), Box<dyn std::error::Error>> {
        let text = r#";; ldir-ir v2.0.0
@font "body" { family = "Inter", weight = "regular" }
"#;
        let m = text_to_module(text)?;
        assert_eq!(m.resources.fonts.len(), 1);
        assert_eq!(m.resources.fonts[0].name, "body");
        Ok(())
    }

    #[test]
    fn test_text_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let mut m = SIRModuleV2::new();
        m.metadata.title = Some("Roundtrip Test".to_string());
        m.body
            .push(Node::new(1, NodeType::Section).with_label("sec:a"));
        m.body
            .push(Node::new(2, NodeType::Paragraph).with_parent(1));
        m.body.push(
            Node::new(
                3,
                NodeType::Text {
                    content: "Hi".to_string(),
                },
            )
            .with_parent(2),
        );

        let text = module_to_text(&m);
        let m2 = text_to_module(&text)?;
        assert_eq!(m2.body.len(), 3);
        let node = m2.body.get(1).ok_or("no node at id 1")?;
        assert_eq!(node.label.as_deref(), Some("sec:a"));
        Ok(())
    }
}
