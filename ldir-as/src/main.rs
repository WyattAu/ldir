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
        if line.is_empty() || line.starts_with(";;") { continue; }

        if line.starts_with("@meta") { continue; }
        if line.starts_with("@body") { continue; }

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

            if id >= next_id { next_id = id + 1; }

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
                "table-cell" => nodes::NodeType::TableCell { colspan: 1, rowspan: 1 },
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
    if !line.starts_with('@') { return None; }

    let tag_end = line[1..].find(|c: char| c.is_whitespace() || c == '[' || c == '{')?;
    let tag = &line[1..tag_end+1];

    let rest = line[tag_end+1..].trim_start();

    let (attrs, body) = if rest.starts_with('[') {
        let bracket_end = rest.find(']')?;
        let attrs = &rest[1..bracket_end];
        let rest2 = rest[bracket_end+1..].trim_start();
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
