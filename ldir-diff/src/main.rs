use clap::Parser;
use std::fs;
use std::path::PathBuf;

/// LDIR IR Structural Diff — compare two S-IR modules semantically.
#[derive(Parser)]
#[command(name = "ldir-diff", version, about)]
struct Cli {
    /// First (old) S-IR file
    old: PathBuf,

    /// Second (new) S-IR file
    new: PathBuf,

    /// Output format
    #[arg(short = 'f', long, default_value = "text")]
    format: String,
}

fn main() {
    let cli = Cli::parse();

    let old_bytes = fs::read(&cli.old).unwrap_or_else(|e| {
        eprintln!("[ldir-diff] Error reading {}: {}", cli.old.display(), e);
        std::process::exit(1);
    });
    let new_bytes = fs::read(&cli.new).unwrap_or_else(|e| {
        eprintln!("[ldir-diff] Error reading {}: {}", cli.new.display(), e);
        std::process::exit(1);
    });

    let old_module = load_module(&old_bytes, cli.old.as_path());
    let new_module = load_module(&new_bytes, cli.new.as_path());

    let diff = compute_diff(&old_module, &new_module);

    if cli.format == "text" {
        print_diff_text(&diff);
    } else if cli.format == "json" {
        println!(
            "{}",
            serde_json::to_string_pretty(&diff).unwrap_or_default()
        );
    } else if cli.format == "count" {
        print_diff_count(&diff);
    } else {
        eprintln!("[ldir-diff] Unknown format: {}", cli.format);
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
        "[ldir-diff] Error: {} is not valid binary S-IR or .ldir text",
        path.display()
    );
    std::process::exit(1);
}

#[derive(Debug, Clone, serde::Serialize)]
struct DiffResult {
    metadata_changes: Vec<String>,
    nodes_added: Vec<u32>,
    nodes_removed: Vec<u32>,
    nodes_modified: Vec<NodeChange>,
    labels_added: Vec<String>,
    labels_removed: Vec<String>,
    styles_added: Vec<String>,
    styles_removed: Vec<String>,
    counters_added: Vec<String>,
    counters_removed: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct NodeChange {
    id: u32,
    field: String,
    old_value: String,
    new_value: String,
}

fn compute_diff(
    old: &ldir_ir::sir::v2::SIRModuleV2,
    new: &ldir_ir::sir::v2::SIRModuleV2,
) -> DiffResult {
    let mut diff = DiffResult {
        metadata_changes: Vec::new(),
        nodes_added: Vec::new(),
        nodes_removed: Vec::new(),
        nodes_modified: Vec::new(),
        labels_added: Vec::new(),
        labels_removed: Vec::new(),
        styles_added: Vec::new(),
        styles_removed: Vec::new(),
        counters_added: Vec::new(),
        counters_removed: Vec::new(),
    };

    if old.metadata.title != new.metadata.title {
        diff.metadata_changes.push(format!(
            "title: {:?} -> {:?}",
            old.metadata.title, new.metadata.title
        ));
    }
    if old.metadata.author != new.metadata.author {
        diff.metadata_changes.push(format!(
            "author: {:?} -> {:?}",
            old.metadata.author, new.metadata.author
        ));
    }
    if old.metadata.language != new.metadata.language {
        diff.metadata_changes.push(format!(
            "language: {:?} -> {:?}",
            old.metadata.language, new.metadata.language
        ));
    }
    if old.metadata.document_class != new.metadata.document_class {
        diff.metadata_changes.push(format!(
            "class: {:?} -> {:?}",
            old.metadata.document_class, new.metadata.document_class
        ));
    }

    let old_ids: std::collections::HashSet<u32> = old.body.iter().map(|n| n.id).collect();
    let new_ids: std::collections::HashSet<u32> = new.body.iter().map(|n| n.id).collect();

    for id in new_ids.difference(&old_ids) {
        diff.nodes_added.push(*id);
    }
    for id in old_ids.difference(&new_ids) {
        diff.nodes_removed.push(*id);
    }

    for id in old_ids.intersection(&new_ids) {
        let Some(old_node) = old.body.get(*id) else {
            continue;
        };
        let Some(new_node) = new.body.get(*id) else {
            continue;
        };

        if old_node.node_type != new_node.node_type {
            diff.nodes_modified.push(NodeChange {
                id: *id,
                field: "type".into(),
                old_value: format!("{:?}", old_node.node_type),
                new_value: format!("{:?}", new_node.node_type),
            });
        }
        if old_node.parent_id != new_node.parent_id {
            diff.nodes_modified.push(NodeChange {
                id: *id,
                field: "parent".into(),
                old_value: format!("{:?}", old_node.parent_id),
                new_value: format!("{:?}", new_node.parent_id),
            });
        }
        if old_node.label != new_node.label {
            diff.nodes_modified.push(NodeChange {
                id: *id,
                field: "label".into(),
                old_value: format!("{:?}", old_node.label),
                new_value: format!("{:?}", new_node.label),
            });
        }
        if old_node.style != new_node.style {
            diff.nodes_modified.push(NodeChange {
                id: *id,
                field: "style".into(),
                old_value: format!("{:?}", old_node.style),
                new_value: format!("{:?}", new_node.style),
            });
        }
        if old_node.child_ids != new_node.child_ids {
            diff.nodes_modified.push(NodeChange {
                id: *id,
                field: "children".into(),
                old_value: format!("{:?}", old_node.child_ids),
                new_value: format!("{:?}", new_node.child_ids),
            });
        }
    }

    let old_labels: std::collections::HashSet<_> = old.annotations.labels.keys().cloned().collect();
    let new_labels: std::collections::HashSet<_> = new.annotations.labels.keys().cloned().collect();
    for l in new_labels.difference(&old_labels) {
        diff.labels_added.push(l.clone());
    }
    for l in old_labels.difference(&new_labels) {
        diff.labels_removed.push(l.clone());
    }

    let old_styles: std::collections::HashSet<_> =
        old.styles.styles.iter().map(|s| s.name.clone()).collect();
    let new_styles: std::collections::HashSet<_> =
        new.styles.styles.iter().map(|s| s.name.clone()).collect();
    for s in new_styles.difference(&old_styles) {
        diff.styles_added.push(s.clone());
    }
    for s in old_styles.difference(&new_styles) {
        diff.styles_removed.push(s.clone());
    }

    let old_counters: std::collections::HashSet<_> = old
        .resources
        .counters
        .iter()
        .map(|c| c.name.clone())
        .collect();
    let new_counters: std::collections::HashSet<_> = new
        .resources
        .counters
        .iter()
        .map(|c| c.name.clone())
        .collect();
    for c in new_counters.difference(&old_counters) {
        diff.counters_added.push(c.clone());
    }
    for c in old_counters.difference(&new_counters) {
        diff.counters_removed.push(c.clone());
    }

    diff.nodes_added.sort();
    diff.nodes_removed.sort();
    diff.labels_added.sort();
    diff.labels_removed.sort();
    diff.styles_added.sort();
    diff.styles_removed.sort();
    diff.counters_added.sort();
    diff.counters_removed.sort();

    diff
}

fn print_diff_text(diff: &DiffResult) {
    let mut has_changes = false;

    if !diff.metadata_changes.is_empty() {
        has_changes = true;
        println!("=== Metadata Changes ===");
        for c in &diff.metadata_changes {
            println!("  {}", c);
        }
        println!();
    }

    if !diff.styles_added.is_empty() || !diff.styles_removed.is_empty() {
        has_changes = true;
        println!("=== Style Changes ===");
        for s in &diff.styles_added {
            println!("  + style: {}", s);
        }
        for s in &diff.styles_removed {
            println!("  - style: {}", s);
        }
        println!();
    }

    if !diff.counters_added.is_empty() || !diff.counters_removed.is_empty() {
        has_changes = true;
        println!("=== Counter Changes ===");
        for c in &diff.counters_added {
            println!("  + counter: {}", c);
        }
        for c in &diff.counters_removed {
            println!("  - counter: {}", c);
        }
        println!();
    }

    if !diff.labels_added.is_empty() || !diff.labels_removed.is_empty() {
        has_changes = true;
        println!("=== Label Changes ===");
        for l in &diff.labels_added {
            println!("  + label: {}", l);
        }
        for l in &diff.labels_removed {
            println!("  - label: {}", l);
        }
        println!();
    }

    if !diff.nodes_added.is_empty() {
        has_changes = true;
        println!("=== Nodes Added ({}) ===", diff.nodes_added.len());
        for &id in &diff.nodes_added {
            println!("  + node {}", id);
        }
        println!();
    }

    if !diff.nodes_removed.is_empty() {
        has_changes = true;
        println!("=== Nodes Removed ({}) ===", diff.nodes_removed.len());
        for &id in &diff.nodes_removed {
            println!("  - node {}", id);
        }
        println!();
    }

    if !diff.nodes_modified.is_empty() {
        has_changes = true;
        println!("=== Nodes Modified ({}) ===", diff.nodes_modified.len());
        for c in &diff.nodes_modified {
            println!(
                "  ~ node {} .{}: {} -> {}",
                c.id, c.field, c.old_value, c.new_value
            );
        }
        println!();
    }

    if !has_changes {
        println!("No differences found.");
    }
}

fn print_diff_count(diff: &DiffResult) {
    println!("metadata:  {}", diff.metadata_changes.len());
    println!(
        "added:     {} nodes, {} labels, {} styles, {} counters",
        diff.nodes_added.len(),
        diff.labels_added.len(),
        diff.styles_added.len(),
        diff.counters_added.len()
    );
    println!(
        "removed:   {} nodes, {} labels, {} styles, {} counters",
        diff.nodes_removed.len(),
        diff.labels_removed.len(),
        diff.styles_removed.len(),
        diff.counters_removed.len()
    );
    println!("modified:  {} nodes", diff.nodes_modified.len());
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
                    },
                    "footnote" => nodes::NodeType::Footnote {
                        content: extract_braced_quoted(body_str).unwrap_or_default(),
                    },
                    "figure" => nodes::NodeType::Figure {
                        placement: nodes::FloatPlacement::Here,
                    },
                    "caption" => nodes::NodeType::Caption,
                    "code-block" => nodes::NodeType::CodeBlock { language: None },
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

    #[test]
    fn test_identical_modules() {
        let text = "@section [id=1] { }\n@paragraph [id=2, parent=1] { }\n";
        let old = make_module(text);
        let new = make_module(text);
        let diff = compute_diff(&old, &new);
        assert!(diff.metadata_changes.is_empty());
        assert!(diff.nodes_added.is_empty());
        assert!(diff.nodes_removed.is_empty());
        assert!(diff.nodes_modified.is_empty());
    }

    #[test]
    fn test_added_node() {
        let old = make_module("@section [id=1] { }\n");
        let new = make_module("@section [id=1] { }\n@paragraph [id=2, parent=1] { }\n");
        let diff = compute_diff(&old, &new);
        assert_eq!(diff.nodes_added, vec![2]);
        assert!(diff.nodes_removed.is_empty());
    }

    #[test]
    fn test_removed_node() {
        let old = make_module("@section [id=1] { }\n@paragraph [id=2, parent=1] { }\n");
        let new = make_module("@section [id=1] { }\n");
        let diff = compute_diff(&old, &new);
        assert!(diff.nodes_added.is_empty());
        assert_eq!(diff.nodes_removed, vec![2]);
    }

    #[test]
    fn test_modified_node() {
        let old = make_module("@text [id=1] { \"Hello\" }\n@text [id=2] { \"World\" }\n");
        let new = make_module("@text [id=1] { \"Hello\" }\n@text [id=2] { \"Changed\" }\n");
        let diff = compute_diff(&old, &new);
        assert_eq!(diff.nodes_modified.len(), 1);
        assert_eq!(diff.nodes_modified[0].id, 2);
        assert_eq!(diff.nodes_modified[0].field, "type");
    }

    #[test]
    fn test_metadata_change() {
        let mut old = make_module("@section [id=1] { }\n");
        old.metadata.title = Some("V1".into());
        let mut new = make_module("@section [id=1] { }\n");
        new.metadata.title = Some("V2".into());
        let diff = compute_diff(&old, &new);
        assert!(!diff.metadata_changes.is_empty());
        assert!(diff.metadata_changes[0].contains("title"));
    }

    #[test]
    fn test_label_added() {
        let old = make_module("@section [id=1] { }\n");
        let new = make_module(r#"@section [id=1, label="sec:intro"] { }"#);
        let diff = compute_diff(&old, &new);
        assert!(diff.labels_added.contains(&"sec:intro".to_string()));
        assert!(diff.labels_removed.is_empty());
    }

    #[test]
    fn test_label_removed() {
        let old = make_module(r#"@section [id=1, label="sec:intro"] { }"#);
        let new = make_module("@section [id=1] { }\n");
        let diff = compute_diff(&old, &new);
        assert!(diff.labels_removed.contains(&"sec:intro".to_string()));
        assert!(diff.labels_added.is_empty());
    }

    #[test]
    fn test_style_added() {
        let old = make_module("@section [id=1] { }\n");
        let new_text = r#"@section [id=1] { }"#;
        let mut new = make_module(new_text);
        new.styles.styles.push(ldir_ir::sir::v2::styles::StyleDecl {
            name: "emphasis".into(),
            parent: None,
            properties: ldir_ir::sir::v2::styles::StyleProperties::default(),
        });
        let diff = compute_diff(&old, &new);
        assert!(diff.styles_added.contains(&"emphasis".to_string()));
    }

    #[test]
    fn test_counter_added() {
        let old = make_module("@section [id=1] { }\n");
        let new_text = "@section [id=1] { }\n";
        let mut new = make_module(new_text);
        new.resources.counters.push(ldir_ir::sir::v2::resources::CounterDecl {
            name: "figure".into(),
            format: ldir_ir::sir::v2::resources::CounterFormat::Arabic,
            reset_scope: ldir_ir::sir::v2::resources::CounterReset::PerSection,
        });
        let diff = compute_diff(&old, &new);
        assert!(diff.counters_added.contains(&"figure".to_string()));
    }

    #[test]
    fn test_parent_change() {
        let old = make_module("@section [id=1] { }\n@paragraph [id=2, parent=1] { }\n");
        let new = make_module("@section [id=1] { }\n@section [id=2] { }\n");
        let diff = compute_diff(&old, &new);
        let parent_changes: Vec<_> = diff
            .nodes_modified
            .iter()
            .filter(|c| c.field == "parent")
            .collect();
        assert_eq!(parent_changes.len(), 1);
        assert_eq!(parent_changes[0].id, 2);
    }

    #[test]
    fn test_multiple_adds_and_removes() {
        let old = make_module("@section [id=1] { }\n@paragraph [id=2, parent=1] { }\n");
        let new = make_module(
            "@section [id=1] { }\n@text [id=3, parent=1] { }\n@bold [id=4, parent=3] { }\n",
        );
        let diff = compute_diff(&old, &new);
        assert!(diff.nodes_added.contains(&3));
        assert!(diff.nodes_added.contains(&4));
        assert!(diff.nodes_removed.contains(&2));
    }

    #[test]
    fn test_diff_serializable_to_json() {
        let old = make_module("@section [id=1] { }\n");
        let new = make_module("@section [id=1] { }\n@paragraph [id=2, parent=1] { }\n");
        let diff = compute_diff(&old, &new);
        let json = serde_json::to_string(&diff);
        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("nodes_added"));
    }

    #[test]
    fn test_empty_modules() {
        let old = make_module("");
        let new = make_module("");
        let diff = compute_diff(&old, &new);
        assert!(diff.metadata_changes.is_empty());
        assert!(diff.nodes_added.is_empty());
        assert!(diff.nodes_removed.is_empty());
        assert!(diff.nodes_modified.is_empty());
        assert!(diff.labels_added.is_empty());
        assert!(diff.labels_removed.is_empty());
        assert!(diff.styles_added.is_empty());
        assert!(diff.styles_removed.is_empty());
        assert!(diff.counters_added.is_empty());
        assert!(diff.counters_removed.is_empty());
    }

    #[test]
    fn test_metadata_author_change() {
        let mut old = make_module("@section [id=1] { }\n");
        old.metadata.author = Some("Alice".into());
        let mut new = make_module("@section [id=1] { }\n");
        new.metadata.author = Some("Bob".into());
        let diff = compute_diff(&old, &new);
        assert!(diff.metadata_changes.iter().any(|c| c.contains("author")));
    }

    #[test]
    fn test_metadata_language_change() {
        let mut old = make_module("@section [id=1] { }\n");
        old.metadata.language = "en".into();
        let mut new = make_module("@section [id=1] { }\n");
        new.metadata.language = "ja".into();
        let diff = compute_diff(&old, &new);
        assert!(diff.metadata_changes.iter().any(|c| c.contains("language")));
    }
}
