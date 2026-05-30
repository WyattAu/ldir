use ldir_ir::sir::v2::module::SIRModuleV2;
use ldir_ir::sir::v2::nodes::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JupyterError {
    #[error("JSON serialization failed: {0}")]
    JsonError(#[from] serde_json::Error),
}

#[derive(Serialize, Deserialize)]
pub struct Notebook {
    pub nbformat: u32,
    #[serde(rename = "nbformat_minor")]
    pub nbformat_minor: u32,
    pub metadata: NotebookMetadata,
    pub cells: Vec<Cell>,
}

#[derive(Serialize, Deserialize)]
pub struct NotebookMetadata {
    pub kernelspec: Kernelspec,
    #[serde(rename = "language_info")]
    pub language_info: LanguageInfo,
}

#[derive(Serialize, Deserialize)]
pub struct Kernelspec {
    #[serde(rename = "display_name")]
    pub display_name: String,
    pub language: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct LanguageInfo {
    pub name: String,
    pub version: String,
}

#[derive(Serialize, Deserialize)]
pub struct Cell {
    #[serde(rename = "cell_type")]
    pub cell_type: String,
    pub metadata: serde_json::Value,
    pub source: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(dead_code)]
    pub execution_count: Option<serde_json::Value>,
}

pub fn sir_to_notebook(module: &SIRModuleV2) -> Result<String, JupyterError> {
    let mut converter = JupyterConverter::new();
    let notebook = converter.convert(module);
    serde_json::to_string_pretty(&notebook).map_err(JupyterError::from)
}

struct JupyterConverter {
    heading_counter: [u32; 6],
}

impl JupyterConverter {
    fn new() -> Self {
        Self {
            heading_counter: [0; 6],
        }
    }

    fn convert(&mut self, module: &SIRModuleV2) -> Notebook {
        let mut cells = Vec::new();

        for &root_id in module.body.roots() {
            if let Some(root) = module.body.get(root_id) {
                self.convert_node(&mut cells, module, root);
            }
        }

        Notebook {
            nbformat: 4,
            nbformat_minor: 5,
            metadata: NotebookMetadata {
                kernelspec: Kernelspec {
                    display_name: "Python 3".to_string(),
                    language: "python".to_string(),
                    name: "python3".to_string(),
                },
                language_info: LanguageInfo {
                    name: "python".to_string(),
                    version: "3.11.0".to_string(),
                },
            },
            cells,
        }
    }

    fn convert_node(&mut self, cells: &mut Vec<Cell>, module: &SIRModuleV2, node: &Node) {
        match &node.node_type {
            NodeType::Document => {
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.convert_node(cells, module, child);
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
                let md_level = (level + 1).min(6) as usize;
                let text = module.body.collect_text(node.id);
                let heading_line = format!("{} {}\n", "#".repeat(md_level), text);
                cells.push(Cell {
                    cell_type: "markdown".to_string(),
                    metadata: serde_json::Value::Object(serde_json::Map::new()),
                    source: vec![heading_line, "\n".to_string()],
                    outputs: None,
                    execution_count: None,
                });

                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.convert_node(cells, module, child);
                    }
                }
            }

            NodeType::Paragraph => {
                let text = self.collect_inline_text(module, node);
                if !text.is_empty() {
                    cells.push(Cell {
                        cell_type: "markdown".to_string(),
                        metadata: serde_json::Value::Object(serde_json::Map::new()),
                        source: vec![format!("{}\n", text)],
                        outputs: None,
                        execution_count: None,
                    });
                }
            }

            NodeType::CodeBlock { language, content } => {
                let is_executable = language
                    .as_deref()
                    .is_some_and(|l| l == "python" || l == "r");
                let code = if content.is_empty() {
                    module.body.collect_text(node.id)
                } else {
                    content.clone()
                };

                if is_executable {
                    cells.push(Cell {
                        cell_type: "code".to_string(),
                        metadata: serde_json::Value::Object(serde_json::Map::new()),
                        source: split_source_lines(&code),
                        outputs: Some(Vec::new()),
                        execution_count: Some(serde_json::Value::Null),
                    });
                } else {
                    let mut source_lines = Vec::new();
                    if let Some(lang) = language {
                        source_lines.push(format!("```{}\n", lang));
                    } else {
                        source_lines.push("```\n".to_string());
                    }
                    for line in code.lines() {
                        source_lines.push(format!("{}\n", line));
                    }
                    source_lines.push("```\n".to_string());
                    cells.push(Cell {
                        cell_type: "markdown".to_string(),
                        metadata: serde_json::Value::Object(serde_json::Map::new()),
                        source: source_lines,
                        outputs: None,
                        execution_count: None,
                    });
                }
            }

            NodeType::List { ordered, .. } => {
                let mut md = String::new();
                let mut counter: u32 = 1;
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id)
                        && let NodeType::ListItem = &child.node_type
                    {
                        let text = self.collect_inline_text(module, child);
                        if *ordered {
                            md.push_str(&format!("{}. {}\n", counter, text));
                            counter += 1;
                        } else {
                            md.push_str(&format!("- {}\n", text));
                        }
                    }
                }
                if !md.is_empty() {
                    cells.push(Cell {
                        cell_type: "markdown".to_string(),
                        metadata: serde_json::Value::Object(serde_json::Map::new()),
                        source: vec![md, "\n".to_string()],
                        outputs: None,
                        execution_count: None,
                    });
                }
            }

            NodeType::Table { .. } => {
                let mut md = String::new();
                let mut is_header = false;
                let mut col_widths: Vec<usize> = Vec::new();
                let mut rows: Vec<Vec<String>> = Vec::new();

                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        if let NodeType::TableRow { is_header: hdr } = &child.node_type {
                            is_header = *hdr;
                        }
                        let mut cells: Vec<String> = Vec::new();
                        for &cell_id in &child.child_ids {
                            if let Some(cell_node) = module.body.get(cell_id) {
                                let text = module.body.collect_text(cell_node.id);
                                let idx = cells.len();
                                if idx >= col_widths.len() {
                                    col_widths.push(text.len());
                                } else {
                                    col_widths[idx] = col_widths[idx].max(text.len());
                                }
                                cells.push(text);
                            }
                        }
                        rows.push(cells);
                    }
                }

                for (row_idx, row) in rows.iter().enumerate() {
                    md.push('|');
                    for (col_idx, cell) in row.iter().enumerate() {
                        let width = col_widths.get(col_idx).copied().unwrap_or(cell.len());
                        md.push_str(&format!(" {:width$} |", cell, width = width));
                    }
                    md.push('\n');

                    if row_idx == 0 && is_header && rows.len() > 1 {
                        md.push('|');
                        for &w in &col_widths {
                            md.push_str(&format!(" {:->width$} |", "", width = w));
                        }
                        md.push('\n');
                    }
                }
                md.push('\n');

                cells.push(Cell {
                    cell_type: "markdown".to_string(),
                    metadata: serde_json::Value::Object(serde_json::Map::new()),
                    source: vec![md],
                    outputs: None,
                    execution_count: None,
                });
            }

            NodeType::BlockQuote => {
                let mut md = String::new();
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id)
                        && let NodeType::Paragraph = &child.node_type
                    {
                        let text = self.collect_inline_text(module, child);
                        for line in text.lines() {
                            md.push_str(&format!("> {}\n", line));
                        }
                    }
                }
                if !md.is_empty() {
                    cells.push(Cell {
                        cell_type: "markdown".to_string(),
                        metadata: serde_json::Value::Object(serde_json::Map::new()),
                        source: vec![md, "\n".to_string()],
                        outputs: None,
                        execution_count: None,
                    });
                }
            }

            NodeType::MathBlock { .. } => {
                let text = module.body.collect_text(node.id);
                cells.push(Cell {
                    cell_type: "markdown".to_string(),
                    metadata: serde_json::Value::Object(serde_json::Map::new()),
                    source: vec![format!("$$\n{}\n$$\n", text)],
                    outputs: None,
                    execution_count: None,
                });
            }

            NodeType::ThematicBreak => {
                cells.push(Cell {
                    cell_type: "markdown".to_string(),
                    metadata: serde_json::Value::Object(serde_json::Map::new()),
                    source: vec!["---\n".to_string(), "\n".to_string()],
                    outputs: None,
                    execution_count: None,
                });
            }

            NodeType::Image { source, alt, .. } => {
                cells.push(Cell {
                    cell_type: "markdown".to_string(),
                    metadata: serde_json::Value::Object(serde_json::Map::new()),
                    source: vec![format!("![{}]({})\n", alt, source)],
                    outputs: None,
                    execution_count: None,
                });
            }

            _ => {
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.convert_node(cells, module, child);
                    }
                }
            }
        }
    }

    fn collect_inline_text(&self, module: &SIRModuleV2, node: &Node) -> String {
        let mut text = String::new();
        for &child_id in &node.child_ids {
            if let Some(child) = module.body.get(child_id) {
                self.collect_inline_text_recursive(module, child, &mut text);
            }
        }
        text
    }

    fn collect_inline_text_recursive(&self, module: &SIRModuleV2, node: &Node, out: &mut String) {
        match &node.node_type {
            NodeType::Text { content } => {
                out.push_str(content);
            }
            NodeType::Bold => {
                out.push_str("**");
                let inner = self.collect_inline_text(module, node);
                out.push_str(&inner);
                out.push_str("**");
            }
            NodeType::Italic => {
                out.push('*');
                let inner = self.collect_inline_text(module, node);
                out.push_str(&inner);
                out.push('*');
            }
            NodeType::Mono => {
                out.push('`');
                let inner = self.collect_inline_text(module, node);
                out.push_str(&inner);
                out.push('`');
            }
            NodeType::Strikethrough => {
                out.push_str("~~");
                let inner = self.collect_inline_text(module, node);
                out.push_str(&inner);
                out.push_str("~~");
            }
            NodeType::Link { url, .. } => {
                let text = self.collect_inline_text(module, node);
                if text == *url {
                    out.push_str(&format!("<{}>", url));
                } else {
                    out.push_str(&format!("[{}]({})", text, url));
                }
            }
            NodeType::MathInline { content } => {
                out.push('$');
                out.push_str(content);
                out.push('$');
            }
            NodeType::LineBreak => {
                out.push_str("  \n");
            }
            NodeType::Image { alt, source, .. } => {
                out.push_str(&format!("![{}]({})", alt, source));
            }
            NodeType::Footnote { content } => {
                out.push_str(&format!("[^{}]", content));
            }
            _ => {
                let inner = self.collect_inline_text(module, node);
                out.push_str(&inner);
            }
        }
    }
}

fn split_source_lines(code: &str) -> Vec<String> {
    let lines: Vec<&str> = code.lines().collect();
    let mut result = Vec::with_capacity(lines.len());
    for (i, line) in lines.iter().enumerate() {
        if i < lines.len() - 1 || code.ends_with('\n') {
            result.push(format!("{}\n", line));
        } else {
            result.push(line.to_string());
        }
    }
    if result.is_empty() {
        result.push("\n".to_string());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_doc_module() -> SIRModuleV2 {
        let mut m = SIRModuleV2::new();
        m.metadata.title = Some("Test Document".into());
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
    fn test_basic_notebook_generation() {
        let m = make_doc_module();
        let json = sir_to_notebook(&m).expect("conversion succeeded");
        let nb: serde_json::Value = serde_json::from_str(&json).expect("valid json");

        assert_eq!(nb["nbformat"], 4);
        assert_eq!(nb["nbformat_minor"], 5);
        assert_eq!(nb["metadata"]["kernelspec"]["name"], "python3");
        assert_eq!(nb["metadata"]["language_info"]["name"], "python");
        assert_eq!(nb["metadata"]["language_info"]["version"], "3.11.0");
    }

    #[test]
    fn test_markdown_cells() {
        let m = make_doc_module();
        let json = sir_to_notebook(&m).expect("conversion succeeded");
        let nb: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        let cells = nb["cells"].as_array().expect("cells array");

        assert!(!cells.is_empty());
        let header_cell = &cells[0];
        assert_eq!(header_cell["cell_type"], "markdown");
        let source = header_cell["source"].as_array().expect("source");
        assert!(source[0].as_str().expect("str").starts_with("### "));

        let para_cell = &cells[1];
        assert_eq!(para_cell["cell_type"], "markdown");
    }

    #[test]
    fn test_code_cells_python() {
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

        let json = sir_to_notebook(&m).expect("conversion succeeded");
        let nb: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        let cells = nb["cells"].as_array().expect("cells array");

        assert_eq!(cells.len(), 1);
        assert_eq!(cells[0]["cell_type"], "code");
        assert!(!cells[0]["outputs"].is_null());
        assert_eq!(cells[0]["execution_count"], serde_json::Value::Null);
        let source = cells[0]["source"].as_array().expect("source");
        assert!(
            source
                .iter()
                .any(|s| s.as_str().expect("str").contains("print('hello')"))
        );
    }

    #[test]
    fn test_code_cells_r() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body.push(
            Node::new(
                1,
                NodeType::CodeBlock {
                    language: Some("r".into()),
                    content: "x <- 1".into(),
                },
            )
            .with_parent(0),
        );
        if let Some(n) = m.body.get_mut(0) {
            n.add_child(1);
        }

        let json = sir_to_notebook(&m).expect("conversion succeeded");
        let nb: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        let cells = nb["cells"].as_array().expect("cells array");

        assert_eq!(cells[0]["cell_type"], "code");
    }

    #[test]
    fn test_code_cells_other_language() {
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

        let json = sir_to_notebook(&m).expect("conversion succeeded");
        let nb: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        let cells = nb["cells"].as_array().expect("cells array");

        assert_eq!(cells[0]["cell_type"], "markdown");
        let source = cells[0]["source"].as_array().expect("source");
        assert!(
            source
                .iter()
                .any(|s| s.as_str().expect("str").contains("```rust"))
        );
    }

    #[test]
    fn test_notebook_metadata_structure() {
        let m = SIRModuleV2::new();
        let json = sir_to_notebook(&m).expect("conversion succeeded");
        let nb: Notebook = serde_json::from_str(&json).expect("valid notebook");

        assert_eq!(nb.nbformat, 4);
        assert_eq!(nb.nbformat_minor, 5);
        assert_eq!(nb.metadata.kernelspec.display_name, "Python 3");
        assert_eq!(nb.metadata.kernelspec.language, "python");
        assert_eq!(nb.metadata.kernelspec.name, "python3");
        assert_eq!(nb.metadata.language_info.name, "python");
        assert_eq!(nb.metadata.language_info.version, "3.11.0");
    }
}
