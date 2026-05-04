use clap::Parser;
use std::fs;
use std::path::PathBuf;

/// LDIR IR Validator — check S-IR v2 modules for well-formedness.
#[derive(Parser)]
#[command(name = "ldir-validate", version, about)]
struct Cli {
    /// Input file (binary S-IR or .ldir text)
    input: PathBuf,

    /// Output format
    #[arg(short = 'f', long, default_value = "text")]
    format: String,
}

#[derive(Debug, Clone)]
struct ValidationError {
    kind: String,
    message: String,
}

fn main() {
    let cli = Cli::parse();

    let bytes = fs::read(&cli.input).unwrap_or_else(|e| {
        eprintln!(
            "[ldir-validate] Error reading {}: {}",
            cli.input.display(),
            e
        );
        std::process::exit(1);
    });

    let module = load_module(&bytes, cli.input.as_path());
    let errors = validate(&module);

    if cli.format == "text" {
        if errors.is_empty() {
            println!("OK: {} is valid", cli.input.display());
        } else {
            for e in &errors {
                println!("ERROR [{}]: {}", e.kind, e.message);
            }
            std::process::exit(1);
        }
    } else if cli.format == "json" {
        let out = if errors.is_empty() {
            serde_json::json!({ "valid": true, "errors": [] })
        } else {
            serde_json::json!({
                "valid": false,
                "errors": errors.iter().map(|e| {
                    serde_json::json!({ "kind": e.kind, "message": e.message })
                }).collect::<Vec<_>>()
            })
        };
        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
        if !errors.is_empty() {
            std::process::exit(1);
        }
    } else {
        eprintln!("[ldir-validate] Unknown format: {}", cli.format);
        std::process::exit(1);
    }
}

fn load_module(bytes: &[u8], path: &std::path::Path) -> ldir_ir::sir::v2::SIRModuleV2 {
    if let Ok(m) = ldir_ir::sir::v2::deserialize_module(bytes) {
        return m;
    }
    if let Ok(text) = std::str::from_utf8(bytes)
        && let Ok(m) = crate::parser::parse_ldir_text(text)
    {
        return m;
    }
    eprintln!(
        "[ldir-validate] Error: {} is not valid binary S-IR or .ldir text",
        path.display()
    );
    std::process::exit(1);
}

fn validate(module: &ldir_ir::sir::v2::SIRModuleV2) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    validate_unique_ids(module, &mut errors);
    validate_parent_refs(module, &mut errors);
    validate_no_cycles(module, &mut errors);
    validate_labels(module, &mut errors);
    validate_style_refs(module, &mut errors);
    validate_counter_refs(module, &mut errors);

    errors
}

fn validate_unique_ids(module: &ldir_ir::sir::v2::SIRModuleV2, errors: &mut Vec<ValidationError>) {
    let mut seen = std::collections::HashSet::new();
    for node in module.body.iter() {
        if !seen.insert(node.id) {
            errors.push(ValidationError {
                kind: "duplicate-id".into(),
                message: format!("duplicate node id: {}", node.id),
            });
        }
    }
}

fn validate_parent_refs(module: &ldir_ir::sir::v2::SIRModuleV2, errors: &mut Vec<ValidationError>) {
    let ids: std::collections::HashSet<u32> = module.body.iter().map(|n| n.id).collect();
    for node in module.body.iter() {
        if let Some(pid) = node.parent_id
            && !ids.contains(&pid)
        {
            errors.push(ValidationError {
                kind: "invalid-parent".into(),
                message: format!("node {} references non-existent parent {}", node.id, pid),
            });
        }
        for &cid in &node.child_ids {
            if !ids.contains(&cid) {
                errors.push(ValidationError {
                    kind: "invalid-child".into(),
                    message: format!("node {} references non-existent child {}", node.id, cid),
                });
            }
        }
    }
}

fn validate_no_cycles(module: &ldir_ir::sir::v2::SIRModuleV2, errors: &mut Vec<ValidationError>) {
    let ids: std::collections::HashSet<u32> = module.body.iter().map(|n| n.id).collect();

    for &start_id in &ids {
        let mut visited = std::collections::HashSet::new();
        let mut current = Some(start_id);
        while let Some(id) = current {
            if id == start_id && !visited.is_empty() {
                errors.push(ValidationError {
                    kind: "cycle".into(),
                    message: format!("parent cycle detected involving node {}", start_id),
                });
                break;
            }
            if !visited.insert(id) {
                break;
            }
            current = module.body.get(id).and_then(|n| n.parent_id);
        }
    }
}

fn validate_labels(module: &ldir_ir::sir::v2::SIRModuleV2, errors: &mut Vec<ValidationError>) {
    let ids: std::collections::HashSet<u32> = module.body.iter().map(|n| n.id).collect();
    for (label, info) in &module.annotations.labels {
        if !ids.contains(&info.node_id) {
            errors.push(ValidationError {
                kind: "dangling-label".into(),
                message: format!(
                    "label \"{}\" references non-existent node {}",
                    label, info.node_id
                ),
            });
        }
    }
    for r in &module.annotations.refs {
        if module.annotations.find_label(&r.label).is_none() {
            errors.push(ValidationError {
                kind: "dangling-ref".into(),
                message: format!(
                    "ref node {} references non-existent label \"{}\"",
                    r.ref_node_id, r.label
                ),
            });
        }
    }
}

fn validate_style_refs(module: &ldir_ir::sir::v2::SIRModuleV2, errors: &mut Vec<ValidationError>) {
    let style_names: std::collections::HashSet<&str> = module
        .styles
        .styles
        .iter()
        .map(|s| s.name.as_str())
        .collect();

    for style_decl in &module.styles.styles {
        if let Some(ref parent) = style_decl.parent
            && !style_names.contains(parent.as_str())
        {
            errors.push(ValidationError {
                kind: "invalid-style-parent".into(),
                message: format!(
                    "style \"{}\" references non-existent parent style \"{}\"",
                    style_decl.name, parent
                ),
            });
        }
    }

    for node in module.body.iter() {
        if let Some(ref style) = node.style
            && !style_names.contains(style.as_str())
        {
            errors.push(ValidationError {
                kind: "invalid-style-ref".into(),
                message: format!(
                    "node {} references non-existent style \"{}\"",
                    node.id, style
                ),
            });
        }
        if let ldir_ir::sir::v2::nodes::NodeType::Styled { style_name } = &node.node_type
            && !style_names.contains(style_name.as_str())
        {
            errors.push(ValidationError {
                kind: "invalid-style-ref".into(),
                message: format!(
                    "styled node {} references non-existent style \"{}\"",
                    node.id, style_name
                ),
            });
        }
    }
}

fn validate_counter_refs(
    module: &ldir_ir::sir::v2::SIRModuleV2,
    errors: &mut Vec<ValidationError>,
) {
    let counter_names: std::collections::HashSet<&str> = module
        .resources
        .counters
        .iter()
        .map(|c| c.name.as_str())
        .collect();

    for node in module.body.iter() {
        if let Some(ref counter) = node.counter
            && !counter_names.contains(counter.as_str())
        {
            errors.push(ValidationError {
                kind: "invalid-counter-ref".into(),
                message: format!(
                    "node {} references non-existent counter \"{}\"",
                    node.id, counter
                ),
            });
        }
    }
}

mod parser {
    pub fn parse_ldir_text(text: &str) -> Result<ldir_ir::sir::v2::SIRModuleV2, String> {
        use ldir_ir::sir::v2::*;

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
                    module.styles.styles.push(styles::StyleDecl {
                        name,
                        parent: None,
                        properties: styles::StyleProperties::default(),
                    });
                }
                continue;
            }

            if let Some((tag, attrs_str, body_str)) = parse_node_line(line) {
                let id = extract_attr_num(attrs_str, "id=").unwrap_or_else(|| {
                    let v = next_id;
                    next_id += 1;
                    v
                });
                let parent_id = extract_attr_num(attrs_str, "parent=");
                let label = extract_attr_quoted(attrs_str, "label=");
                let style = extract_attr_quoted(attrs_str, "style=");
                if id >= next_id {
                    next_id = id + 1;
                }

                let node_type = match tag {
                    "document" => nodes::NodeType::Document,
                    "section" => nodes::NodeType::Section,
                    "paragraph" => nodes::NodeType::Paragraph,
                    "text" => nodes::NodeType::Text {
                        content: extract_braced_quoted(body_str).unwrap_or_default(),
                    },
                    "equation" => nodes::NodeType::MathBlock {
                        math_type: nodes::MathType::Equation,
                        numbered: body_str.contains("numbered=true"),
                    },
                    "bold" => nodes::NodeType::Bold,
                    "italic" => nodes::NodeType::Italic,
                    "mono" => nodes::NodeType::Mono,
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
                    "list" => nodes::NodeType::List {
                        list_type: nodes::ListType::Unordered,
                        ordered: false,
                        start: None,
                    },
                    "list-item" => nodes::NodeType::ListItem,
                    "blockquote" => nodes::NodeType::BlockQuote,
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
                    "group" => nodes::NodeType::Group,
                    "math" => nodes::NodeType::MathInline {
                        content: extract_braced_quoted(body_str).unwrap_or_default(),
                    },
                    "chapter" => nodes::NodeType::Chapter,
                    "part" => nodes::NodeType::Part,
                    "subsection" => nodes::NodeType::Subsection,
                    "subsubsection" => nodes::NodeType::Subsubsection,
                    "styled" => nodes::NodeType::Styled {
                        style_name: extract_braced_field(body_str, "style=").unwrap_or_default(),
                    },
                    "footnote-block" => nodes::NodeType::FootnoteBlock,
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

                if let Some(ref l) = node.label {
                    let cat = if node.is_heading() {
                        annotations::LabelCategory::Section
                    } else if matches!(node.node_type, nodes::NodeType::MathBlock { .. }) {
                        annotations::LabelCategory::Equation
                    } else {
                        annotations::LabelCategory::Custom
                    };
                    module.annotations.add_label(l.clone(), id, cat);
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
                let be = rest2.rfind('}')?;
                &rest2[1..be]
            } else {
                ""
            };
            (attrs, body)
        } else if rest.starts_with('{') {
            let be = rest.rfind('}')?;
            ("", &rest[1..be])
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_module(text: &str) -> ldir_ir::sir::v2::SIRModuleV2 {
        crate::parser::parse_ldir_text(text).unwrap()
    }

    fn make_module_direct(
        nodes: Vec<ldir_ir::sir::v2::nodes::Node>,
    ) -> ldir_ir::sir::v2::SIRModuleV2 {
        let mut m = ldir_ir::sir::v2::SIRModuleV2::new();
        for n in nodes {
            m.body.push(n);
        }
        m
    }

    #[test]
    fn test_valid_module() {
        let m = make_module("@section [id=1] { }\n@paragraph [id=2, parent=1] { }\n");
        let errors = validate(&m);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_duplicate_ids() {
        use ldir_ir::sir::v2::nodes::*;
        let m = make_module_direct(vec![
            Node::new(1, NodeType::Section),
            Node::new(1, NodeType::Paragraph),
        ]);
        let errors = validate(&m);
        assert!(errors.iter().any(|e| e.kind == "duplicate-id"));
    }

    #[test]
    fn test_invalid_parent() {
        use ldir_ir::sir::v2::nodes::*;
        let m = make_module_direct(vec![Node::new(1, NodeType::Paragraph).with_parent(99)]);
        let errors = validate(&m);
        assert!(errors.iter().any(|e| e.kind == "invalid-parent"));
    }

    #[test]
    fn test_cycle_detection() {
        use ldir_ir::sir::v2::nodes::*;
        let m = make_module_direct(vec![
            Node::new(1, NodeType::Section).with_parent(2),
            Node::new(2, NodeType::Section).with_parent(1),
        ]);
        let errors = validate(&m);
        assert!(errors.iter().any(|e| e.kind == "cycle"));
    }

    #[test]
    fn test_dangling_label() {
        let mut m = ldir_ir::sir::v2::SIRModuleV2::new();
        m.annotations.add_label(
            "sec:ghost".into(),
            999,
            ldir_ir::sir::v2::annotations::LabelCategory::Section,
        );
        let errors = validate(&m);
        assert!(errors.iter().any(|e| e.kind == "dangling-label"));
    }
}
