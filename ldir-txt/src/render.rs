use ldir_ir::sir::v2::module::SIRModuleV2;
use ldir_ir::sir::v2::nodes::*;

/// Plain text rendering options.
#[derive(Debug, Clone)]
pub struct TextOptions {
    pub include_toc: bool,
    pub indent_size: usize,
    pub max_line_width: usize,
    pub underline_headings: bool,
}

impl Default for TextOptions {
    fn default() -> Self {
        Self {
            include_toc: true,
            indent_size: 2,
            max_line_width: 80,
            underline_headings: true,
        }
    }
}

/// Renders S-IR v2 modules to plain text.
pub struct TextRenderer {
    options: TextOptions,
    heading_counter: [u32; 6],
    list_counters: Vec<u32>,
}

impl Default for TextRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl TextRenderer {
    pub fn new() -> Self {
        Self::with_options(TextOptions::default())
    }

    pub fn with_options(options: TextOptions) -> Self {
        Self {
            options,
            heading_counter: [0; 6],
            list_counters: Vec::new(),
        }
    }

    pub fn render(&mut self, module: &SIRModuleV2) -> String {
        let mut out = String::new();

        if let Some(ref title) = module.metadata.title {
            out.push_str(title);
            out.push('\n');
            if self.options.underline_headings {
                out.push_str(&"=".repeat(title.len()));
                out.push('\n');
            }
            out.push('\n');
        }

        if self.options.include_toc {
            self.emit_toc(&mut out, module);
        }

        for &root_id in module.body.roots() {
            if let Some(root) = module.body.get(root_id) {
                self.render_node(&mut out, module, root, 0);
            }
        }

        out
    }

    fn emit_toc(&self, out: &mut String, module: &SIRModuleV2) {
        let headings = module.headings();
        if headings.is_empty() {
            return;
        }

        out.push_str("Table of Contents\n");
        out.push_str(&"-".repeat(18));
        out.push('\n');

        let mut counters = [0u32; 6];
        for node in &headings {
            if let Some(level) = node.heading_level() {
                let idx = level as usize;
                if idx < 6 {
                    counters[idx] += 1;
                    for c in &mut counters[(idx + 1)..] {
                        *c = 0;
                    }
                }
            }
            let num = format_heading_number(&counters);
            let text = module.body.collect_text(node.id);
            let indent = "  ".repeat(node.heading_level().unwrap_or(0) as usize);
            if num.is_empty() {
                out.push_str(&format!("{}{}\n", indent, text));
            } else {
                out.push_str(&format!("{}{} {}\n", indent, num, text));
            }
        }
        out.push('\n');
    }

    fn render_node(
        &mut self,
        out: &mut String,
        module: &SIRModuleV2,
        node: &Node,
        depth: usize,
    ) {
        match &node.node_type {
            NodeType::Document => {
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_node(out, module, child, depth);
                    }
                }
            }

            NodeType::Part | NodeType::Chapter | NodeType::Section
            | NodeType::Subsection | NodeType::Subsubsection => {
                let level = node.heading_level().unwrap_or(2);
                if level > 0 && (level as usize) < 6 {
                    self.heading_counter[level as usize] += 1;
                    for i in (level as usize + 1)..6 {
                        self.heading_counter[i] = 0;
                    }
                }
                let num = format_heading_number(&self.heading_counter);
                let text = module.body.collect_text(node.id);

                let prefix = match level {
                    0 => "",
                    1 => "# ",
                    2 => "## ",
                    3 => "### ",
                    4 => "#### ",
                    _ => "##### ",
                };

                out.push_str(prefix);
                if !num.is_empty() {
                    out.push_str(&num);
                    out.push(' ');
                }
                out.push_str(&text);
                out.push('\n');

                if self.options.underline_headings {
                    let content_len = prefix.len()
                        + if num.is_empty() { 0 } else { num.len() + 1 }
                        + text.len();
                    let ch = if level <= 1 { '=' } else { '-' };
                    out.push_str(&String::from(ch).repeat(content_len));
                    out.push('\n');
                }
                out.push('\n');

                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_node(out, module, child, depth);
                    }
                }
            }

            NodeType::Paragraph => {
                let text = self.collect_inline_text(module, node);
                out.push_str(&text);
                out.push_str("\n\n");
            }

            NodeType::Text { content } => {
                out.push_str(content);
            }

            NodeType::Bold => {
                out.push_str(&self.collect_inline_text(module, node));
            }

            NodeType::Italic => {
                out.push_str(&self.collect_inline_text(module, node));
            }

            NodeType::Mono => {
                out.push('`');
                out.push_str(&self.collect_inline_text(module, node));
                out.push('`');
            }

            NodeType::Underline => {
                out.push_str(&self.collect_inline_text(module, node));
            }

            NodeType::Strikethrough => {
                out.push_str(&self.collect_inline_text(module, node));
            }

            NodeType::SmallCaps => {
                out.push_str(&self.collect_inline_text(module, node));
            }

            NodeType::Link { url, .. } => {
                let text = self.collect_inline_text(module, node);
                out.push_str(&text);
                out.push_str(&format!(" ({})", url));
            }

            NodeType::Image { alt, .. } => {
                out.push_str(&format!("[Image: {}]", alt));
            }

            NodeType::MathInline { content } => {
                out.push_str(&format!("${}$", content));
            }

            NodeType::LineBreak => {
                out.push('\n');
            }

            NodeType::MathBlock { numbered, .. } => {
                let text = module.body.collect_text(node.id);
                out.push_str("  ");
                out.push_str(&text);
                if *numbered {
                    self.heading_counter[0] += 1;
                    out.push_str(&format!("  ({})", self.heading_counter[0]));
                }
                out.push_str("\n\n");
            }

            NodeType::List { ordered, start, .. } => {
                let initial = start.unwrap_or(if *ordered { 1 } else { 0 });
                self.list_counters.push(initial);
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_node(out, module, child, depth + 1);
                    }
                }
                self.list_counters.pop();
            }

            NodeType::ListItem => {
                let indent = "  ".repeat(depth.max(1));
                if let Some(counter) = self.list_counters.last_mut() {
                    if *counter == 0 {
                        out.push_str(&format!("{}- ", indent));
                    } else {
                        out.push_str(&format!("{}{}. ", indent, counter));
                        *counter += 1;
                    }
                } else {
                    out.push_str(&format!("{}- ", indent));
                }
                let text = self.collect_inline_text(module, node);
                out.push_str(&text);
                out.push('\n');
            }

            NodeType::BlockQuote => {
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_block_quote_node(out, module, child, depth);
                    }
                }
                out.push('\n');
            }

            NodeType::CodeBlock { language } => {
                let text = module.body.collect_text(node.id);
                if let Some(lang) = language {
                    out.push_str(&format!("```{}", lang));
                } else {
                    out.push_str("```");
                }
                out.push('\n');
                out.push_str(&text);
                out.push('\n');
                out.push_str("```\n\n");
            }

            NodeType::Table { .. } => {
                self.render_table(out, module, node);
            }

            NodeType::Figure { .. } => {
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        if let NodeType::Caption = &child.node_type {
                            let text = module.body.collect_text(child.id);
                            out.push_str(&format!("[Figure] {}\n\n", text));
                        } else {
                            self.render_node(out, module, child, depth);
                        }
                    }
                }
            }

            NodeType::Caption => {}

            NodeType::Footnote { content } => {
                out.push_str(&format!("[^{}]", content));
            }

            NodeType::FootnoteBlock => {
                let mut fn_num = 0u32;
                for node in module.body.iter() {
                    if let NodeType::Footnote { content } = &node.node_type {
                        fn_num += 1;
                        out.push_str(&format!("[{}] {}\n", fn_num, content));
                    }
                }
                out.push('\n');
            }

            NodeType::TableOfContents { .. } => {}

            NodeType::ThematicBreak => {
                out.push_str(&"-".repeat(self.options.max_line_width));
                out.push_str("\n\n");
            }

            NodeType::PageBreak => {}

            NodeType::Styled { .. } => {
                out.push_str(&self.collect_inline_text(module, node));
            }

            NodeType::Group => {
                out.push_str(&self.collect_inline_text(module, node));
            }

            NodeType::TableRow { .. } | NodeType::TableCell { .. } => {}

            NodeType::Citation { keys, .. } => {
                for key in keys {
                    out.push_str(&format!("[{}]", key));
                }
            }
        }
    }

    fn render_block_quote_node(
        &mut self,
        out: &mut String,
        module: &SIRModuleV2,
        node: &Node,
        depth: usize,
    ) {
        match &node.node_type {
            NodeType::Paragraph => {
                let text = self.collect_inline_text(module, node);
                for line in text.lines() {
                    out.push_str("> ");
                    out.push_str(line);
                    out.push('\n');
                }
                out.push('\n');
            }
            _ => {
                self.render_node(out, module, node, depth);
            }
        }
    }

    fn render_table(&mut self, out: &mut String, module: &SIRModuleV2, node: &Node) {
        let mut rows: Vec<Vec<String>> = Vec::new();
        let mut col_widths: Vec<usize> = Vec::new();
        let mut is_header = false;

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
            for (col_idx, cell) in row.iter().enumerate() {
                let width = col_widths.get(col_idx).copied().unwrap_or(cell.len());
                out.push_str(&format!("  {:<width$}", cell, width = width));
            }
            out.push('\n');

            if row_idx == 0 && is_header && rows.len() > 1 {
                for &w in &col_widths {
                    out.push_str(&format!("  {:->width$}", "", width = w));
                }
                out.push('\n');
            }
        }
        out.push('\n');
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

    fn collect_inline_text_recursive(
        &self,
        module: &SIRModuleV2,
        node: &Node,
        out: &mut String,
    ) {
        match &node.node_type {
            NodeType::Text { content } => {
                out.push_str(content);
            }
            NodeType::Link { url, .. } => {
                let text = self.collect_inline_text(module, node);
                out.push_str(&format!("{} ({})", text, url));
            }
            NodeType::MathInline { content } => {
                out.push_str(&format!("${}$", content));
            }
            NodeType::Image { alt, .. } => {
                out.push_str(&format!("[Image: {}]", alt));
            }
            NodeType::Footnote { content } => {
                out.push_str(&format!("[^{}]", content));
            }
            NodeType::Bold
            | NodeType::Italic
            | NodeType::Underline
            | NodeType::Strikethrough
            | NodeType::SmallCaps
            | NodeType::Styled { .. }
            | NodeType::Group => {
                let text = self.collect_inline_text(module, node);
                out.push_str(&text);
            }
            NodeType::Mono => {
                out.push('`');
                let text = self.collect_inline_text(module, node);
                out.push_str(&text);
                out.push('`');
            }
            NodeType::LineBreak => {
                out.push('\n');
            }
            _ => {
                let text = self.collect_inline_text(module, node);
                out.push_str(&text);
            }
        }
    }
}

fn format_heading_number(counters: &[u32; 6]) -> String {
    let parts: Vec<String> = counters[1..]
        .iter()
        .filter(|&&c| c > 0)
        .map(|c| c.to_string())
        .collect();
    if parts.is_empty() {
        String::new()
    } else {
        format!("{}.", parts.join("."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_section_module() -> SIRModuleV2 {
        let mut m = SIRModuleV2::new();
        m.metadata.title = Some("Test Document".into());
        m.body.push(Node::new(0, NodeType::Document));
        m.body.push(
            Node::new(1, NodeType::Section)
                .with_parent(0)
                .with_label("sec:intro"),
        );
        m.body.push(
            Node::new(2, NodeType::Text { content: "Introduction".into() }).with_parent(1),
        );
        m.body.push(Node::new(3, NodeType::Paragraph).with_parent(0));
        m.body.push(
            Node::new(4, NodeType::Text { content: "Hello, world!".into() }).with_parent(3),
        );
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(0).unwrap().add_child(3);
        m.body.get_mut(1).unwrap().add_child(2);
        m.body.get_mut(3).unwrap().add_child(4);
        m
    }

    #[test]
    fn test_basic_text_output() {
        let m = make_section_module();
        let text = TextRenderer::new().render(&m);
        assert!(text.contains("Test Document"));
        assert!(text.contains("Introduction"));
        assert!(text.contains("Hello, world!"));
    }

    #[test]
    fn test_toc_generated() {
        let m = make_section_module();
        let text = TextRenderer::new().render(&m);
        assert!(text.contains("Table of Contents"));
        assert!(text.contains("1. Introduction"));
    }

    #[test]
    fn test_no_toc_when_disabled() {
        let m = make_section_module();
        let text = TextRenderer::with_options(TextOptions {
            include_toc: false,
            ..Default::default()
        })
        .render(&m);
        assert!(!text.contains("Table of Contents"));
    }

    #[test]
    fn test_heading_prefixes() {
        let m = make_section_module();
        let text = TextRenderer::new().render(&m);
        assert!(text.contains("## 1. Introduction"));
    }

    #[test]
    fn test_unordered_list() {
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
            Node::new(3, NodeType::Text { content: "item 1".into() }).with_parent(2),
        );
        m.body.push(Node::new(4, NodeType::ListItem).with_parent(1));
        m.body.push(
            Node::new(5, NodeType::Text { content: "item 2".into() }).with_parent(4),
        );
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(1).unwrap().add_child(2);
        m.body.get_mut(1).unwrap().add_child(4);
        m.body.get_mut(2).unwrap().add_child(3);
        m.body.get_mut(4).unwrap().add_child(5);

        let text = TextRenderer::new().render(&m);
        assert!(text.contains("- item 1"));
        assert!(text.contains("- item 2"));
    }

    #[test]
    fn test_ordered_list() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body.push(
            Node::new(
                1,
                NodeType::List {
                    list_type: ListType::Ordered,
                    ordered: true,
                    start: None,
                },
            )
            .with_parent(0),
        );
        m.body.push(Node::new(2, NodeType::ListItem).with_parent(1));
        m.body.push(
            Node::new(3, NodeType::Text { content: "first".into() }).with_parent(2),
        );
        m.body.push(Node::new(4, NodeType::ListItem).with_parent(1));
        m.body.push(
            Node::new(5, NodeType::Text { content: "second".into() }).with_parent(4),
        );
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(1).unwrap().add_child(2);
        m.body.get_mut(1).unwrap().add_child(4);
        m.body.get_mut(2).unwrap().add_child(3);
        m.body.get_mut(4).unwrap().add_child(5);

        let text = TextRenderer::new().render(&m);
        assert!(text.contains("1. first"));
        assert!(text.contains("2. second"));
    }

    #[test]
    fn test_link_rendered() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body.push(Node::new(1, NodeType::Paragraph).with_parent(0));
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
            Node::new(3, NodeType::Text { content: "link".into() }).with_parent(2),
        );
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(1).unwrap().add_child(2);
        m.body.get_mut(2).unwrap().add_child(3);

        let text = TextRenderer::new().render(&m);
        assert!(text.contains("link (https://example.com)"));
    }

    #[test]
    fn test_code_block() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body.push(
            Node::new(
                1,
                NodeType::CodeBlock {
                    language: Some("rust".into()),
                },
            )
            .with_parent(0),
        );
        m.body.push(
            Node::new(2, NodeType::Text { content: "fn main() {}".into() }).with_parent(1),
        );
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(1).unwrap().add_child(2);

        let text = TextRenderer::new().render(&m);
        assert!(text.contains("```rust"));
        assert!(text.contains("fn main()"));
        assert!(text.contains("```"));
    }

    #[test]
    fn test_math_inline() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body.push(Node::new(1, NodeType::Paragraph).with_parent(0));
        m.body.push(
            Node::new(
                2,
                NodeType::MathInline {
                    content: "x^2 + y^2".into(),
                },
            )
            .with_parent(1),
        );
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(1).unwrap().add_child(2);

        let text = TextRenderer::new().render(&m);
        assert!(text.contains("$x^2 + y^2$"));
    }

    #[test]
    fn test_thematic_break() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body.push(Node::new(1, NodeType::ThematicBreak).with_parent(0));
        m.body.get_mut(0).unwrap().add_child(1);

        let text = TextRenderer::new().render(&m);
        assert!(text.contains(&"-".repeat(80)));
    }

    #[test]
    fn test_blockquote() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body.push(Node::new(1, NodeType::BlockQuote).with_parent(0));
        m.body.push(Node::new(2, NodeType::Paragraph).with_parent(1));
        m.body.push(
            Node::new(3, NodeType::Text { content: "A quote".into() }).with_parent(2),
        );
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(1).unwrap().add_child(2);
        m.body.get_mut(2).unwrap().add_child(3);

        let text = TextRenderer::new().render(&m);
        assert!(text.contains("> A quote"));
    }

    #[test]
    fn test_inline_styles() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body.push(Node::new(1, NodeType::Paragraph).with_parent(0));
        m.body.push(Node::new(2, NodeType::Bold).with_parent(1));
        m.body.push(
            Node::new(3, NodeType::Text { content: "bold".into() }).with_parent(2),
        );
        m.body.push(
            Node::new(4, NodeType::Text { content: " and ".into() }).with_parent(1),
        );
        m.body.push(Node::new(5, NodeType::Mono).with_parent(1));
        m.body.push(
            Node::new(6, NodeType::Text { content: "code".into() }).with_parent(5),
        );
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(1).unwrap().add_child(2);
        m.body.get_mut(1).unwrap().add_child(4);
        m.body.get_mut(1).unwrap().add_child(5);
        m.body.get_mut(2).unwrap().add_child(3);
        m.body.get_mut(5).unwrap().add_child(6);

        let text = TextRenderer::new().render(&m);
        assert!(text.contains("bold and `code`"));
    }

    #[test]
    fn test_table_rendered() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body.push(
            Node::new(
                1,
                NodeType::Table {
                    col_specs: vec![],
                    num_cols: 2,
                },
            )
            .with_parent(0),
        );
        m.body.push(
            Node::new(2, NodeType::TableRow { is_header: true }).with_parent(1),
        );
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
            Node::new(4, NodeType::Text { content: "Name".into() }).with_parent(3),
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
            Node::new(6, NodeType::Text { content: "Value".into() }).with_parent(5),
        );
        m.body.push(
            Node::new(7, NodeType::TableRow { is_header: false }).with_parent(1),
        );
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
            Node::new(9, NodeType::Text { content: "A".into() }).with_parent(8),
        );
        m.body.push(
            Node::new(
                10,
                NodeType::TableCell {
                    colspan: 1,
                    rowspan: 1,
                },
            )
            .with_parent(7),
        );
        m.body.push(
            Node::new(11, NodeType::Text { content: "1".into() }).with_parent(10),
        );
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(1).unwrap().add_child(2);
        m.body.get_mut(1).unwrap().add_child(7);
        m.body.get_mut(2).unwrap().add_child(3);
        m.body.get_mut(2).unwrap().add_child(5);
        m.body.get_mut(3).unwrap().add_child(4);
        m.body.get_mut(5).unwrap().add_child(6);
        m.body.get_mut(7).unwrap().add_child(8);
        m.body.get_mut(7).unwrap().add_child(10);
        m.body.get_mut(8).unwrap().add_child(9);
        m.body.get_mut(10).unwrap().add_child(11);

        let text = TextRenderer::new().render(&m);
        assert!(text.contains("Name"));
        assert!(text.contains("Value"));
        assert!(text.contains("  A"));
        assert!(text.contains("  1"));
    }

    #[test]
    fn test_empty_module() {
        let m = SIRModuleV2::new();
        let text = TextRenderer::new().render(&m);
        assert!(text.is_empty());
    }

    #[test]
    fn test_no_html_entities() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body.push(Node::new(1, NodeType::Paragraph).with_parent(0));
        m.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "a < b & c > d".into(),
                },
            )
            .with_parent(1),
        );
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(1).unwrap().add_child(2);

        let text = TextRenderer::new().render(&m);
        assert!(text.contains("a < b & c > d"));
        assert!(!text.contains("&lt;"));
        assert!(!text.contains("&amp;"));
    }

    #[test]
    fn test_heading_numbering() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body.push(Node::new(1, NodeType::Section).with_parent(0));
        m.body.push(
            Node::new(2, NodeType::Text { content: "First".into() }).with_parent(1),
        );
        m.body.push(Node::new(3, NodeType::Section).with_parent(0));
        m.body.push(
            Node::new(4, NodeType::Text { content: "Second".into() }).with_parent(3),
        );
        m.body.push(Node::new(5, NodeType::Subsection).with_parent(0));
        m.body.push(
            Node::new(6, NodeType::Text { content: "Sub".into() }).with_parent(5),
        );
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(0).unwrap().add_child(3);
        m.body.get_mut(0).unwrap().add_child(5);
        m.body.get_mut(1).unwrap().add_child(2);
        m.body.get_mut(3).unwrap().add_child(4);
        m.body.get_mut(5).unwrap().add_child(6);

        let text = TextRenderer::new().render(&m);
        assert!(text.contains("## 1. First"));
        assert!(text.contains("## 2. Second"));
        assert!(text.contains("### 2.1. Sub"));
    }

    #[test]
    fn test_image_alt_text() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body.push(Node::new(1, NodeType::Paragraph).with_parent(0));
        m.body.push(
            Node::new(
                2,
                NodeType::Image {
                    source: "photo.png".into(),
                    alt: "A photo".into(),
                    width: None,
                    height: None,
                },
            )
            .with_parent(1),
        );
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(1).unwrap().add_child(2);

        let text = TextRenderer::new().render(&m);
        assert!(text.contains("[Image: A photo]"));
    }

    #[test]
    fn test_default_options() {
        let opts = TextOptions::default();
        assert!(opts.include_toc);
        assert_eq!(opts.indent_size, 2);
        assert_eq!(opts.max_line_width, 80);
        assert!(opts.underline_headings);
    }

    #[test]
    fn test_format_heading_number() {
        let c1 = [0, 1, 0, 0, 0, 0];
        assert_eq!(format_heading_number(&c1), "1.");
        let c2 = [0, 2, 1, 0, 0, 0];
        assert_eq!(format_heading_number(&c2), "2.1.");
        let c3 = [0, 0, 0, 0, 0, 0];
        assert_eq!(format_heading_number(&c3), "");
    }
}
