use clap::Parser;
use std::fs;
use std::path::PathBuf;

/// LDIR IR Assembler — converts .ldir text to binary S-IR.
#[derive(Parser)]
#[command(name = "ldir-as", version, about)]
struct Cli {
    /// Input file (.ldir text format)
    input: PathBuf,

    /// Output file (default: stdout as binary)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    let text = fs::read_to_string(&cli.input).unwrap_or_else(|e| {
        eprintln!("[ldir-as] Error reading {}: {}", cli.input.display(), e);
        std::process::exit(1);
    });

    let module = match parse_ldir_text(&text) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("[ldir-as] Parse error: {}", e);
            std::process::exit(1);
        }
    };

    let bytes = ldir_ir::sir::v2::serialize_module(&module);

    match cli.output {
        Some(ref out) => {
            fs::write(out, &bytes).unwrap_or_else(|e| {
                eprintln!("[ldir-as] Error writing {}: {}", out.display(), e);
                std::process::exit(1);
            });
            eprintln!("[ldir-as] Wrote {} bytes to {}", bytes.len(), out.display());
        }
        None => {
            use std::io::Write;
            std::io::stdout().write_all(&bytes).unwrap_or_else(|e| {
                eprintln!("[ldir-as] Error writing stdout: {}", e);
                std::process::exit(1);
            });
        }
    }
}

fn parse_ldir_text(text: &str) -> Result<ldir_ir::sir::v2::SIRModuleV2, String> {
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
            let id = extract_attr_num(attrs_str, "id=").unwrap_or(next_id);
            let parent_id = extract_attr_num(attrs_str, "parent=");
            let label = extract_attr_quoted(attrs_str, "label=");
            let style = extract_attr_quoted(attrs_str, "style=");

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
                "bold" => nodes::NodeType::Bold,
                "italic" => nodes::NodeType::Italic,
                "mono" => nodes::NodeType::Mono,
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
                },
                "table-row" => nodes::NodeType::TableRow { is_header: false },
                "table-cell" => nodes::NodeType::TableCell {
                    colspan: 1,
                    rowspan: 1,
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

#[cfg(test)]
mod tests {
    use super::*;

    // === Parser helper function tests ===

    #[test]
    fn test_extract_quoted_basic() {
        assert_eq!(
            extract_quoted(r#"@font "DejaVu" family = "Sans""#, "font "),
            Some("DejaVu".into())
        );
    }

    #[test]
    fn test_extract_quoted_not_found() {
        assert_eq!(extract_quoted("@font bar", "font "), None);
    }

    #[test]
    fn test_extract_quoted_missing_prefix() {
        assert_eq!(extract_quoted("@font", "font "), None);
    }

    #[test]
    fn test_extract_attr_num_found() {
        assert_eq!(extract_attr_num("id=42 parent=1", "id="), Some(42));
    }

    #[test]
    fn test_extract_attr_num_not_found() {
        assert_eq!(extract_attr_num("id=42", "parent="), None);
    }

    #[test]
    fn test_extract_attr_num_zero() {
        assert_eq!(extract_attr_num("id=0", "id="), Some(0));
    }

    #[test]
    fn test_extract_attr_num_trailing_chars() {
        // "id=42," should still parse 42 (stops at non-digit)
        assert_eq!(extract_attr_num("id=42,", "id="), Some(42));
    }

    #[test]
    fn test_extract_attr_quoted_found() {
        assert_eq!(
            extract_attr_quoted(r#"label="sec:intro""#, "label="),
            Some("sec:intro".into())
        );
    }

    #[test]
    fn test_extract_attr_quoted_not_found() {
        assert_eq!(extract_attr_quoted("id=1", "label="), None);
    }

    #[test]
    fn test_parse_node_line_basic() {
        let result = parse_node_line("@document [id=0] { }");
        assert_eq!(result, Some(("document", "id=0", " ")));
    }

    #[test]
    fn test_parse_node_line_with_body() {
        let result = parse_node_line(r#"@text [id=1] { "Hello" }"#);
        assert_eq!(result, Some(("text", "id=1", r#" "Hello" "#)));
    }

    #[test]
    fn test_parse_node_line_no_attrs() {
        let result = parse_node_line("@hr { }");
        assert_eq!(result, Some(("hr", "", " ")));
    }

    #[test]
    fn test_parse_node_line_no_at_sign() {
        assert_eq!(parse_node_line("document [id=0] { }"), None);
    }

    #[test]
    fn test_parse_node_line_no_bracket_or_brace() {
        // "@document" with no whitespace/brackets after tag — tag_end find returns None
        assert_eq!(parse_node_line("@document"), None);
    }

    #[test]
    fn test_extract_braced_quoted_found() {
        assert_eq!(extract_braced_quoted(r#""Hello""#), Some("Hello".into()));
    }

    #[test]
    fn test_extract_braced_quoted_with_spaces() {
        assert_eq!(
            extract_braced_quoted(r#"  "Hello"  "#),
            Some("Hello".into())
        );
    }

    #[test]
    fn test_extract_braced_quoted_not_found() {
        assert_eq!(extract_braced_quoted("Hello"), None);
    }

    #[test]
    fn test_extract_braced_field_found() {
        assert_eq!(
            extract_braced_field(r#"url="https://example.com""#, "url="),
            Some("https://example.com".into())
        );
    }

    #[test]
    fn test_extract_braced_field_not_found() {
        assert_eq!(
            extract_braced_field(r#"url="https://example.com""#, "src="),
            None
        );
    }

    // === Integration tests: parse_ldir_text ===

    #[test]
    fn test_parse_empty_text() {
        let module = parse_ldir_text("").unwrap();
        assert!(module.body.is_empty());
    }

    #[test]
    fn test_parse_comments_skipped() {
        let module = parse_ldir_text(";; comment\n;; another").unwrap();
        assert!(module.body.is_empty());
    }

    #[test]
    fn test_parse_meta_and_body_skipped() {
        let module = parse_ldir_text("@meta title=\"Test\"\n@body\n").unwrap();
        assert!(module.body.is_empty());
    }

    #[test]
    fn test_parse_document_node() {
        let module = parse_ldir_text("@document [id=0] { }\n").unwrap();
        assert_eq!(module.body.len(), 1);
        assert_eq!(module.body.get(0).unwrap().id, 0);
    }

    #[test]
    fn test_parse_text_node() {
        let module = parse_ldir_text(r#"@text [id=1] { "Hello" }"#).unwrap();
        assert_eq!(module.body.len(), 1);
        let node = module.body.get(1).unwrap();
        assert_eq!(node.id, 1);
    }

    #[test]
    fn test_parse_parent_child() {
        let text = "@section [id=1] { }\n@paragraph [id=2, parent=1] { }\n";
        let module = parse_ldir_text(text).unwrap();
        assert_eq!(module.body.len(), 2);
        assert_eq!(module.body.get(2).unwrap().parent_id, Some(1));
    }

    #[test]
    fn test_parse_label() {
        let text = r#"@section [id=1, label="sec:intro"] { }"#;
        let module = parse_ldir_text(text).unwrap();
        assert_eq!(module.body.get(1).unwrap().label, Some("sec:intro".into()));
        assert!(module.annotations.labels.contains_key("sec:intro"));
    }

    #[test]
    fn test_parse_font_decl() {
        let text = r#"@font "main" family = "DejaVu""#;
        let module = parse_ldir_text(text).unwrap();
        assert_eq!(module.resources.fonts.len(), 1);
        assert_eq!(module.resources.fonts[0].name, "main");
        assert_eq!(module.resources.fonts[0].family, "DejaVu");
    }

    #[test]
    fn test_parse_style_decl() {
        let text = r#"@style "heading""#;
        let module = parse_ldir_text(text).unwrap();
        assert_eq!(module.styles.styles.len(), 1);
        assert_eq!(module.styles.styles[0].name, "heading");
    }

    #[test]
    fn test_parse_counter_decl() {
        let text = r#"@counter "figure""#;
        let module = parse_ldir_text(text).unwrap();
        assert_eq!(module.resources.counters.len(), 1);
        assert_eq!(module.resources.counters[0].name, "figure");
    }

    #[test]
    fn test_parse_auto_increment_id() {
        let text = "@section { }\n@paragraph { }\n@text { }\n";
        let module = parse_ldir_text(text).unwrap();
        assert_eq!(module.body.len(), 3);
        // Auto-increment should produce 0, 1, 2
        let ids: Vec<u32> = module.body.iter().map(|n| n.id).collect();
        assert_eq!(ids, vec![0, 1, 2]);
    }

    #[test]
    fn test_parse_mixed_nodes() {
        let text = r#"
@document [id=0] { }
@chapter [id=1, parent=0] { }
@section [id=2, parent=1] { }
@paragraph [id=3, parent=2] { }
@text [id=4, parent=3] { "Hello" }
@bold [id=5, parent=4] { }
@text [id=6, parent=5] { "world" }
@equation [id=7, parent=3, label="eq:test"] { numbered=true }
@code-block [id=8, parent=3] { }
@list [id=9, parent=3] { }
@list-item [id=10, parent=9] { }
@blockquote [id=11, parent=3] { }
@table [id=12, parent=3] { }
@table-row [id=13, parent=12] { }
@table-cell [id=14, parent=13] { }
@toc [id=15, parent=0] { }
@hr [id=16, parent=3] { }
@page-break [id=17, parent=3] { }
@image [id=18, parent=3] { src="fig.png" }
@link [id=19, parent=3] { url="https://example.com" }
@footnote [id=20, parent=3] { "A note" }
@figure [id=21, parent=3] { }
@caption [id=22, parent=21] { }
@group [id=23, parent=3] { }
@math [id=24, parent=3] { "x^2" }
@mono [id=25, parent=4] { }
@italic [id=26, parent=4] { }
@styled [id=27, parent=4] { style="emphasis" }
@part [id=28, parent=0] { }
@subsection [id=29, parent=2] { }
@subsubsection [id=30, parent=29] { }
@footnote-block [id=31, parent=0] { }
"#;
        let module = parse_ldir_text(text).unwrap();
        // Should parse the vast majority of node types; at least 30 out of 32
        assert!(
            module.body.len() >= 30,
            "expected >= 30 nodes, got {}",
            module.body.len()
        );
    }

    // === Round-trip test: assemble → disassemble → reassemble ===

    #[test]
    fn test_roundtrip_simple() {
        let text = "@section [id=1] { }\n@paragraph [id=2, parent=1] { }\n";
        let module = parse_ldir_text(text).unwrap();
        let bytes = ldir_ir::sir::v2::serialize_module(&module);
        let restored = ldir_ir::sir::v2::deserialize_module(&bytes).unwrap();
        assert_eq!(restored.body.len(), 2);
        assert_eq!(restored.body.get(1).unwrap().id, 1);
        assert_eq!(restored.body.get(2).unwrap().id, 2);
    }

    #[test]
    fn test_roundtrip_with_metadata() {
        let mut module = parse_ldir_text("@section [id=1] { }\n").unwrap();
        module.metadata.title = Some("Test Doc".into());
        module.metadata.author = Some("Author".into());
        let bytes = ldir_ir::sir::v2::serialize_module(&module);
        let restored = ldir_ir::sir::v2::deserialize_module(&bytes).unwrap();
        assert_eq!(restored.metadata.title, Some("Test Doc".into()));
        assert_eq!(restored.metadata.author, Some("Author".into()));
    }

    #[test]
    fn test_roundtrip_with_labels() {
        let text = r#"@section [id=1, label="sec:intro"] { }"#;
        let module = parse_ldir_text(text).unwrap();
        let bytes = ldir_ir::sir::v2::serialize_module(&module);
        let restored = ldir_ir::sir::v2::deserialize_module(&bytes).unwrap();
        assert!(restored.annotations.labels.contains_key("sec:intro"));
    }

    #[test]
    fn test_text_to_text_format() {
        let text = "@section [id=1] { }\n@text [id=2, parent=1] { }\n";
        let module = parse_ldir_text(text).unwrap();
        let text_out = ldir_ir::sir::v2::module_to_text(&module);
        // The text output should contain recognizable node definitions
        assert!(text_out.contains("section") || text_out.contains("Section"));
        assert!(text_out.contains("text") || text_out.contains("Text"));
    }

    #[test]
    fn test_serialize_empty_module() {
        let module = parse_ldir_text("").unwrap();
        let bytes = ldir_ir::sir::v2::serialize_module(&module);
        let restored = ldir_ir::sir::v2::deserialize_module(&bytes).unwrap();
        assert!(restored.body.is_empty());
    }
}
