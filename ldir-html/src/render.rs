use ldir_ir::sir::v2::module::SIRModuleV2;
use ldir_ir::sir::v2::nodes::*;

/// HTML rendering options.
#[derive(Debug, Clone)]
pub struct HtmlOptions {
    pub include_toc: bool,
    pub include_styles: bool,
    pub math_format: MathFormat,
    pub indent: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum MathFormat {
    MathML,
    LaTeX,
    Text,
}

impl Default for HtmlOptions {
    fn default() -> Self {
        Self {
            include_toc: true,
            include_styles: true,
            math_format: MathFormat::MathML,
            indent: 2,
        }
    }
}

/// Renders S-IR v2 modules to HTML5.
pub struct HtmlRenderer {
    options: HtmlOptions,
    heading_counter: [u32; 6],
    equation_counter: u32,
    figure_counter: u32,
    footnote_counter: u32,
}

impl HtmlRenderer {
    pub fn new() -> Self {
        Self::with_options(HtmlOptions::default())
    }

    pub fn with_options(options: HtmlOptions) -> Self {
        Self {
            options,
            heading_counter: [0; 6],
            equation_counter: 0,
            figure_counter: 0,
            footnote_counter: 0,
        }
    }

    /// Render the module to a complete HTML5 document string.
    pub fn render(&mut self, module: &SIRModuleV2) -> String {
        let mut html = String::new();
        let ind = self.options.indent;

        // DOCTYPE and html
        html.push_str("<!DOCTYPE html>\n<html");
        if module.metadata.language != "en" {
            html.push_str(&format!(
                " lang=\"{}\"",
                escape_html(&module.metadata.language)
            ));
        }
        html.push_str(">\n");

        // <head>
        html.push_str(&" ".repeat(ind));
        html.push_str("<head>\n");
        html.push_str(&" ".repeat(ind * 2));
        html.push_str("<meta charset=\"utf-8\">\n");
        html.push_str(&" ".repeat(ind * 2));
        html.push_str(
            "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
        );
        if let Some(ref title) = module.metadata.title {
            html.push_str(&" ".repeat(ind * 2));
            html.push_str(&format!("<title>{}</title>\n", escape_html(title)));
        }
        if self.options.include_styles {
            self.emit_styles(&mut html, ind * 2);
        }
        html.push_str(&" ".repeat(ind));
        html.push_str("</head>\n\n");

        // <body>
        html.push_str(&" ".repeat(ind));
        html.push_str("<body>\n");

        // TOC
        if self.options.include_toc {
            self.emit_toc(&mut html, module, ind + 1);
        }

        // Document content
        for &root_id in module.body.roots() {
            if let Some(root) = module.body.get(root_id) {
                self.render_node(&mut html, module, root, ind + 1);
            }
        }

        html.push_str(&" ".repeat(ind));
        html.push_str("</body>\n");
        html.push_str("</html>\n");

        html
    }

    fn emit_styles(&self, html: &mut String, ind: usize) {
        let pad = " ".repeat(ind);
        html.push_str(&pad);
        html.push_str("<style>\n");
        let rules = [
            "body { font-family: serif; max-width: 720px; margin: 0 auto; padding: 2em; line-height: 1.6; color: #1a1a1a; }",
            "h1, h2, h3, h4, h5, h6 { margin-top: 1.5em; margin-bottom: 0.5em; line-height: 1.2; }",
            "h1 { font-size: 2em; } h2 { font-size: 1.5em; } h3 { font-size: 1.25em; } h4 { font-size: 1.1em; }",
            "p { margin: 0.8em 0; text-align: justify; }",
            "blockquote { border-left: 3px solid #ccc; margin-left: 0; padding-left: 1em; color: #555; }",
            "pre { background: #f5f5f5; padding: 1em; overflow-x: auto; border-radius: 4px; }",
            "code { font-family: monospace; background: #f0f0f0; padding: 0.2em 0.4em; border-radius: 2px; font-size: 0.9em; }",
            "pre code { background: none; padding: 0; }",
            "table { border-collapse: collapse; width: 100%; margin: 1em 0; }",
            "th, td { border: 1px solid #ddd; padding: 0.5em 0.8em; text-align: left; }",
            "th { background: #f5f5f5; font-weight: bold; }",
            "img { max-width: 100%; height: auto; }",
            "a { color: #0066cc; text-decoration: none; }",
            "a:hover { text-decoration: underline; }",
            ".math-display { display: block; text-align: center; margin: 1em 0; padding: 0.5em; overflow-x: auto; }",
            ".toc { background: #f9f9f9; border: 1px solid #eee; padding: 1em 1.5em; margin-bottom: 2em; border-radius: 4px; }",
            ".toc h2 { margin-top: 0; font-size: 1.2em; }",
            ".toc ul { list-style: none; padding-left: 0; }",
            ".toc ul ul { padding-left: 1.5em; }",
            ".toc a { color: #333; }",
            ".footnote-ref { font-size: 0.8em; vertical-align: super; }",
            ".footnotes { font-size: 0.9em; border-top: 1px solid #ccc; margin-top: 2em; padding-top: 1em; }",
            ".footnotes li { margin-bottom: 0.3em; }",
            ".figure { margin: 1em 0; text-align: center; }",
            ".figure img { display: block; margin: 0 auto; }",
            ".caption { font-size: 0.9em; color: #555; margin-top: 0.3em; }",
            ".eq-number { float: right; }",
        ];
        for rule in &rules {
            html.push_str(&pad);
            html.push_str("  ");
            html.push_str(rule);
            html.push('\n');
        }
        html.push_str(&pad);
        html.push_str("</style>\n");
    }

    fn emit_toc(&self, html: &mut String, module: &SIRModuleV2, ind: usize) {
        let headings = module.headings();
        if headings.is_empty() {
            return;
        }

        // Compute heading numbers for the TOC
        let mut counters = [0u32; 6];
        let heading_numbers: Vec<String> = headings
            .iter()
            .map(|node| {
                if let Some(level) = node.heading_level() {
                    let idx = level as usize;
                    if idx < 6 {
                        counters[idx] += 1;
                        for c in &mut counters[(idx + 1)..] {
                            *c = 0;
                        }
                    }
                }
                format_heading_number(&counters)
            })
            .collect();

        let pad = " ".repeat(ind);
        html.push_str(&pad);
        html.push_str("<nav class=\"toc\">\n");
        html.push_str(&" ".repeat(ind + 1));
        html.push_str("<h2>Table of Contents</h2>\n");
        html.push_str(&" ".repeat(ind + 1));
        html.push_str("<ul>\n");

        for (node, num) in headings.iter().zip(heading_numbers.iter()) {
            let text = module.body.collect_text(node.id);
            let fallback_id = format!("heading-{}", node.id);
            let id = node.label.as_deref().unwrap_or(&fallback_id);
            html.push_str(&" ".repeat(ind + 1));
            html.push_str(&format!(
                "<li><a href=\"#{}\">{} {}</a></li>\n",
                id,
                num,
                escape_html(&text)
            ));
        }

        html.push_str(&" ".repeat(ind + 1));
        html.push_str("</ul>\n");
        html.push_str(&pad);
        html.push_str("</nav>\n\n");
    }

    fn render_node(&mut self, html: &mut String, module: &SIRModuleV2, node: &Node, ind: usize) {
        let pad = " ".repeat(ind);

        match &node.node_type {
            NodeType::Document => {
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_node(html, module, child, ind);
                    }
                }
            }

            NodeType::Part
            | NodeType::Chapter
            | NodeType::Section
            | NodeType::Subsection
            | NodeType::Subsubsection => {
                let level = node.heading_level().unwrap_or(2);
                let tag = match level {
                    0 | 1 => "h1",
                    2 => "h2",
                    3 => "h3",
                    4 => "h4",
                    _ => "h5",
                };
                if level > 0 && (level as usize) < 6 {
                    self.heading_counter[level as usize] += 1;
                    for i in (level as usize + 1)..6 {
                        self.heading_counter[i] = 0;
                    }
                }
                let fallback_id = format!("heading-{}", node.id);
                let id = node.label.as_deref().unwrap_or(&fallback_id);
                let text = module.body.collect_text(node.id);
                html.push_str(&pad);
                html.push_str(&format!(
                    "<{} id=\"{}\">{} {}</{}>\n",
                    tag,
                    id,
                    format_heading_number(&self.heading_counter),
                    escape_html(&text),
                    tag
                ));
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_node(html, module, child, ind);
                    }
                }
            }

            NodeType::Paragraph => {
                html.push_str(&pad);
                html.push_str("<p>");
                self.render_children_inline(html, module, node);
                html.push_str("</p>\n");
            }

            NodeType::Text { content } => {
                html.push_str(&escape_html(content));
            }

            NodeType::Bold => {
                html.push_str("<strong>");
                self.render_children_inline(html, module, node);
                html.push_str("</strong>");
            }

            NodeType::Italic => {
                html.push_str("<em>");
                self.render_children_inline(html, module, node);
                html.push_str("</em>");
            }

            NodeType::Mono => {
                html.push_str("<code>");
                self.render_children_inline(html, module, node);
                html.push_str("</code>");
            }

            NodeType::Underline => {
                html.push_str("<u>");
                self.render_children_inline(html, module, node);
                html.push_str("</u>");
            }

            NodeType::Strikethrough => {
                html.push_str("<s>");
                self.render_children_inline(html, module, node);
                html.push_str("</s>");
            }

            NodeType::SmallCaps => {
                html.push_str("<span style=\"font-variant: small-caps;\">");
                self.render_children_inline(html, module, node);
                html.push_str("</span>");
            }

            NodeType::Link { url, title } => {
                html.push_str("<a");
                html.push_str(&format!(" href=\"{}\"", escape_html(url)));
                if let Some(t) = title {
                    html.push_str(&format!(" title=\"{}\"", escape_html(t)));
                }
                html.push('>');
                self.render_children_inline(html, module, node);
                html.push_str("</a>");
            }

            NodeType::Image { source, alt, .. } => {
                html.push_str("<img");
                html.push_str(&format!(" src=\"{}\"", escape_html(source)));
                html.push_str(&format!(" alt=\"{}\"", escape_html(alt)));
                html.push('>');
            }

            NodeType::MathInline { content } => match self.options.math_format {
                MathFormat::MathML => {
                    html.push_str("<math>");
                    html.push_str(&escape_html(content));
                    html.push_str("</math>");
                }
                MathFormat::LaTeX => {
                    html.push_str("<code class=\"math\">");
                    html.push_str(&escape_html(content));
                    html.push_str("</code>");
                }
                MathFormat::Text => {
                    html.push_str(&format!("[{}]", escape_html(content)));
                }
            },

            NodeType::MathBlock { numbered, .. } => {
                self.equation_counter += 1;
                let text = module.body.collect_text(node.id);
                html.push_str(&pad);
                html.push_str("<div class=\"math-display\">");
                if let Some(ref label) = node.label {
                    html.push_str(&format!("<a id=\"{}\"></a>", label));
                }
                match self.options.math_format {
                    MathFormat::MathML => {
                        html.push_str("<math display=\"block\">");
                        html.push_str(&escape_html(&text));
                        html.push_str("</math>");
                    }
                    MathFormat::LaTeX => {
                        html.push_str("<code class=\"math\">");
                        html.push_str(&escape_html(&text));
                        html.push_str("</code>");
                    }
                    MathFormat::Text => {
                        html.push_str(&escape_html(&text));
                    }
                }
                if *numbered {
                    html.push_str(&format!(
                        "<span class=\"eq-number\">({})</span>",
                        self.equation_counter
                    ));
                }
                html.push_str("</div>\n");
            }

            NodeType::List { ordered, .. } => {
                let tag = if *ordered { "ol" } else { "ul" };
                html.push_str(&pad);
                html.push_str(&format!("<{}>\n", tag));
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_node(html, module, child, ind + 1);
                    }
                }
                html.push_str(&pad);
                html.push_str(&format!("</{}>\n", tag));
            }

            NodeType::ListItem => {
                html.push_str(&pad);
                html.push_str("<li>");
                self.render_children_inline(html, module, node);
                html.push_str("</li>\n");
            }

            NodeType::BlockQuote => {
                html.push_str(&pad);
                html.push_str("<blockquote>\n");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_node(html, module, child, ind + 1);
                    }
                }
                html.push_str(&pad);
                html.push_str("</blockquote>\n");
            }

            NodeType::CodeBlock { language } => {
                let text = module.body.collect_text(node.id);
                html.push_str(&pad);
                html.push_str("<pre><code");
                if let Some(lang) = language {
                    html.push_str(&format!(" class=\"language-{}\"", escape_html(lang)));
                }
                html.push('>');
                html.push_str(&escape_html(&text));
                html.push_str("</code></pre>\n");
            }

            NodeType::Table { .. } => {
                html.push_str(&pad);
                html.push_str("<table>\n");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_node(html, module, child, ind + 1);
                    }
                }
                html.push_str(&pad);
                html.push_str("</table>\n");
            }

            NodeType::TableRow { is_header } => {
                html.push_str(&pad);
                html.push_str("<tr>\n");
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_node_inner(html, module, child, ind + 1, *is_header);
                    }
                }
                html.push_str(&pad);
                html.push_str("</tr>\n");
            }

            NodeType::TableCell { .. } => {
                let text = module.body.collect_text(node.id);
                html.push_str(&pad);
                html.push_str("<td>");
                html.push_str(&escape_html(&text));
                html.push_str("</td>\n");
            }

            NodeType::Figure { .. } => {
                self.figure_counter += 1;
                html.push_str(&pad);
                html.push_str("<div class=\"figure\">\n");
                if let Some(ref label) = node.label {
                    html.push_str(&" ".repeat(ind + 1));
                    html.push_str(&format!("<a id=\"{}\"></a>\n", label));
                }
                for &child_id in &node.child_ids {
                    if let Some(child) = module.body.get(child_id) {
                        self.render_node(html, module, child, ind + 1);
                    }
                }
                html.push_str(&pad);
                html.push_str("</div>\n");
            }

            NodeType::Caption => {
                let text = module.body.collect_text(node.id);
                html.push_str(&" ".repeat(ind));
                html.push_str(&format!(
                    "<p class=\"caption\">Figure {}: {}</p>\n",
                    self.figure_counter,
                    escape_html(&text)
                ));
            }

            NodeType::Footnote { .. } => {
                self.footnote_counter += 1;
                html.push_str(&format!(
                    "<sup class=\"footnote-ref\"><a href=\"#fn-{}\">[{}]</a></sup>",
                    self.footnote_counter, self.footnote_counter
                ));
            }

            NodeType::FootnoteBlock => {
                html.push_str(&pad);
                html.push_str("<div class=\"footnotes\">\n");
                html.push_str(&" ".repeat(ind + 1));
                html.push_str("<ol>\n");
                let mut fn_num = 0u32;
                for node in module.body.iter() {
                    if let NodeType::Footnote { content } = &node.node_type {
                        fn_num += 1;
                        html.push_str(&" ".repeat(ind + 2));
                        html.push_str(&format!(
                            "<li id=\"fn-{}\">{} <a href=\"#fn-{}-back\">\u{21a9}</a></li>\n",
                            fn_num,
                            escape_html(content),
                            fn_num
                        ));
                    }
                }
                html.push_str(&" ".repeat(ind + 1));
                html.push_str("</ol>\n");
                html.push_str(&pad);
                html.push_str("</div>\n");
            }

            NodeType::TableOfContents { .. } => {}

            NodeType::ThematicBreak => {
                html.push_str(&pad);
                html.push_str("<hr>\n");
            }

            NodeType::PageBreak => {}

            NodeType::Styled { style_name } => {
                html.push_str(&format!("<span class=\"{}\">", escape_html(style_name)));
                self.render_children_inline(html, module, node);
                html.push_str("</span>");
            }

            NodeType::Group => {
                self.render_children_inline(html, module, node);
            }

            NodeType::LineBreak => {
                html.push_str("<br>");
            }

            NodeType::Citation { keys, .. } => {
                for key in keys {
                    html.push_str(&format!("[{}]", escape_html(key)));
                }
            }
        }
    }

    fn render_node_inner(
        &mut self,
        html: &mut String,
        module: &SIRModuleV2,
        node: &Node,
        ind: usize,
        is_header_row: bool,
    ) {
        match &node.node_type {
            NodeType::TableCell { .. } => {
                let text = module.body.collect_text(node.id);
                let tag = if is_header_row { "th" } else { "td" };
                let pad = " ".repeat(ind);
                html.push_str(&pad);
                html.push_str(&format!("<{}>", tag));
                html.push_str(&escape_html(&text));
                html.push_str(&format!("</{}>\n", tag));
            }
            _ => {
                self.render_node(html, module, node, ind);
            }
        }
    }

    fn render_children_inline(&mut self, html: &mut String, module: &SIRModuleV2, node: &Node) {
        for &child_id in &node.child_ids {
            if let Some(child) = module.body.get(child_id) {
                self.render_node(html, module, child, 0);
            }
        }
    }
}

fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
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

impl Default for HtmlRenderer {
    fn default() -> Self {
        Self::new()
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
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(0).unwrap().add_child(3);
        m.body.get_mut(1).unwrap().add_child(2);
        m.body.get_mut(3).unwrap().add_child(4);
        m
    }

    #[test]
    fn test_basic_html_output() {
        let m = make_section_module();
        let html = HtmlRenderer::new().render(&m);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<html"));
        assert!(html.contains("<head>"));
        assert!(html.contains("<title>Test Document</title>"));
        assert!(html.contains("<body>"));
        assert!(html.contains("Hello, world!"));
        assert!(html.contains("</html>"));
    }

    #[test]
    fn test_heading_rendered() {
        let m = make_section_module();
        let html = HtmlRenderer::new().render(&m);
        assert!(html.contains("<h2"));
        assert!(html.contains("Introduction"));
    }

    #[test]
    fn test_heading_id_from_label() {
        let m = make_section_module();
        let html = HtmlRenderer::new().render(&m);
        assert!(html.contains("id=\"sec:intro\""));
    }

    #[test]
    fn test_toc_generated() {
        let m = make_section_module();
        let html = HtmlRenderer::new().render(&m);
        assert!(html.contains("class=\"toc\""));
        assert!(html.contains("Table of Contents"));
    }

    #[test]
    fn test_no_toc_when_disabled() {
        let m = make_section_module();
        let html = HtmlRenderer::with_options(HtmlOptions {
            include_toc: false,
            ..Default::default()
        })
        .render(&m);
        assert!(!html.contains("class=\"toc\""));
    }

    #[test]
    fn test_inline_styles() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::Paragraph).with_parent(0));
        m.body.push(Node::new(2, NodeType::Bold).with_parent(1));
        m.body.push(
            Node::new(
                3,
                NodeType::Text {
                    content: "bold".into(),
                },
            )
            .with_parent(2),
        );
        m.body.push(
            Node::new(
                4,
                NodeType::Text {
                    content: " and ".into(),
                },
            )
            .with_parent(1),
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
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(1).unwrap().add_child(2);
        m.body.get_mut(1).unwrap().add_child(4);
        m.body.get_mut(1).unwrap().add_child(5);
        m.body.get_mut(2).unwrap().add_child(3);
        m.body.get_mut(5).unwrap().add_child(6);

        let html = HtmlRenderer::new().render(&m);
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<em>italic</em>"));
    }

    #[test]
    fn test_html_escaping() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::Paragraph).with_parent(0));
        m.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "<script>alert('xss')</script>".into(),
                },
            )
            .with_parent(1),
        );
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(1).unwrap().add_child(2);

        let html = HtmlRenderer::new().render(&m);
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_link_rendered() {
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
                    content: "link".into(),
                },
            )
            .with_parent(2),
        );
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(1).unwrap().add_child(2);
        m.body.get_mut(2).unwrap().add_child(3);

        let html = HtmlRenderer::new().render(&m);
        assert!(html.contains("<a href=\"https://example.com\""));
        assert!(html.contains("title=\"Example\""));
    }

    #[test]
    fn test_list_rendered() {
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
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(1).unwrap().add_child(2);
        m.body.get_mut(1).unwrap().add_child(4);
        m.body.get_mut(2).unwrap().add_child(3);
        m.body.get_mut(4).unwrap().add_child(5);

        let html = HtmlRenderer::new().render(&m);
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>item 1</li>"));
        assert!(html.contains("<li>item 2</li>"));
        assert!(html.contains("</ul>"));
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
            Node::new(
                2,
                NodeType::Text {
                    content: "fn main() {}".into(),
                },
            )
            .with_parent(1),
        );
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(1).unwrap().add_child(2);

        let html = HtmlRenderer::new().render(&m);
        assert!(html.contains("<pre><code"));
        assert!(html.contains("language-rust"));
        assert!(html.contains("fn main()"));
    }

    #[test]
    fn test_blockquote() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::BlockQuote).with_parent(0));
        m.body
            .push(Node::new(2, NodeType::Paragraph).with_parent(1));
        m.body.push(
            Node::new(
                3,
                NodeType::Text {
                    content: "A quote".into(),
                },
            )
            .with_parent(2),
        );
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(1).unwrap().add_child(2);
        m.body.get_mut(2).unwrap().add_child(3);

        let html = HtmlRenderer::new().render(&m);
        assert!(html.contains("<blockquote>"));
        assert!(html.contains("A quote"));
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
                    content: "Header".into(),
                },
            )
            .with_parent(3),
        );
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(1).unwrap().add_child(2);
        m.body.get_mut(2).unwrap().add_child(3);
        m.body.get_mut(3).unwrap().add_child(4);

        let html = HtmlRenderer::new().render(&m);
        assert!(html.contains("<table>"));
        assert!(html.contains("<tr>"));
        assert!(html.contains("<th>Header</th>"));
    }

    #[test]
    fn test_thematic_break() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::ThematicBreak).with_parent(0));
        m.body.get_mut(0).unwrap().add_child(1);

        let html = HtmlRenderer::new().render(&m);
        assert!(html.contains("<hr>"));
    }

    #[test]
    fn test_math_inline() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::Paragraph).with_parent(0));
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

        let html = HtmlRenderer::new().render(&m);
        assert!(html.contains("<math>"));
        assert!(html.contains("x^2 + y^2"));
    }

    #[test]
    fn test_equation_numbered() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body.push(
            Node::new(
                1,
                NodeType::MathBlock {
                    math_type: MathType::Equation,
                    numbered: true,
                },
            )
            .with_parent(0)
            .with_label("eq:1"),
        );
        m.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "E = mc^2".into(),
                },
            )
            .with_parent(1),
        );
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(1).unwrap().add_child(2);

        let html = HtmlRenderer::new().render(&m);
        assert!(html.contains("class=\"math-display\""));
        assert!(html.contains("class=\"eq-number\""));
        assert!(html.contains("(1)"));
    }

    #[test]
    fn test_escape_html_function() {
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html("a & b"), "a &amp; b");
        assert_eq!(escape_html("\"quotes\""), "&quot;quotes&quot;");
    }

    #[test]
    fn test_without_styles() {
        let m = make_section_module();
        let html = HtmlRenderer::with_options(HtmlOptions {
            include_styles: false,
            ..Default::default()
        })
        .render(&m);
        assert!(!html.contains("<style>"));
    }

    #[test]
    fn test_lang_attribute_non_english() {
        let mut m = make_section_module();
        m.metadata.language = "ja".into();
        let html = HtmlRenderer::new().render(&m);
        assert!(html.contains("lang=\"ja\""));
    }

    #[test]
    fn test_no_lang_attribute_for_english() {
        let m = make_section_module();
        let html = HtmlRenderer::new().render(&m);
        assert!(!html.contains("lang="));
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
            Node::new(
                3,
                NodeType::Text {
                    content: "first".into(),
                },
            )
            .with_parent(2),
        );
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(1).unwrap().add_child(2);
        m.body.get_mut(2).unwrap().add_child(3);

        let html = HtmlRenderer::new().render(&m);
        assert!(html.contains("<ol>"));
        assert!(html.contains("</ol>"));
        assert!(html.contains("<li>first</li>"));
    }

    #[test]
    fn test_image_rendered() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::Paragraph).with_parent(0));
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

        let html = HtmlRenderer::new().render(&m);
        assert!(html.contains("src=\"photo.png\""));
        assert!(html.contains("alt=\"A photo\""));
    }

    #[test]
    fn test_figure_with_caption() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body.push(
            Node::new(
                1,
                NodeType::Figure {
                    placement: FloatPlacement::Here,
                },
            )
            .with_parent(0)
            .with_label("fig:demo"),
        );
        m.body.push(
            Node::new(
                2,
                NodeType::Image {
                    source: "diagram.png".into(),
                    alt: "Diagram".into(),
                    width: None,
                    height: None,
                },
            )
            .with_parent(1),
        );
        m.body.push(Node::new(3, NodeType::Caption).with_parent(1));
        m.body.push(
            Node::new(
                4,
                NodeType::Text {
                    content: "A diagram".into(),
                },
            )
            .with_parent(3),
        );
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(1).unwrap().add_child(2);
        m.body.get_mut(1).unwrap().add_child(3);
        m.body.get_mut(3).unwrap().add_child(4);

        let html = HtmlRenderer::new().render(&m);
        assert!(html.contains("class=\"figure\""));
        assert!(html.contains("id=\"fig:demo\""));
        assert!(html.contains("Figure 1: A diagram"));
    }

    #[test]
    fn test_footnote_rendered() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::Paragraph).with_parent(0));
        m.body.push(
            Node::new(
                2,
                NodeType::Footnote {
                    content: "A footnote".into(),
                },
            )
            .with_parent(1),
        );
        m.body
            .push(Node::new(3, NodeType::FootnoteBlock).with_parent(0));
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(0).unwrap().add_child(3);
        m.body.get_mut(1).unwrap().add_child(2);

        let html = HtmlRenderer::new().render(&m);
        assert!(html.contains("class=\"footnote-ref\""));
        assert!(html.contains("[1]"));
        assert!(html.contains("class=\"footnotes\""));
        assert!(html.contains("id=\"fn-1\""));
        assert!(html.contains("A footnote"));
    }

    #[test]
    fn test_line_break() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::Paragraph).with_parent(0));
        m.body
            .push(Node::new(2, NodeType::LineBreak).with_parent(1));
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(1).unwrap().add_child(2);

        let html = HtmlRenderer::new().render(&m);
        assert!(html.contains("<br>"));
    }

    #[test]
    fn test_math_latex_format() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::Paragraph).with_parent(0));
        m.body.push(
            Node::new(
                2,
                NodeType::MathInline {
                    content: "\\alpha".into(),
                },
            )
            .with_parent(1),
        );
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(1).unwrap().add_child(2);

        let html = HtmlRenderer::with_options(HtmlOptions {
            math_format: MathFormat::LaTeX,
            ..Default::default()
        })
        .render(&m);
        assert!(html.contains("<code class=\"math\">"));
        assert!(html.contains("\\alpha"));
        assert!(!html.contains("<math>"));
    }

    #[test]
    fn test_math_text_format() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::Paragraph).with_parent(0));
        m.body.push(
            Node::new(
                2,
                NodeType::MathInline {
                    content: "E=mc^2".into(),
                },
            )
            .with_parent(1),
        );
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(1).unwrap().add_child(2);

        let html = HtmlRenderer::with_options(HtmlOptions {
            math_format: MathFormat::Text,
            ..Default::default()
        })
        .render(&m);
        assert!(html.contains("[E=mc^2]"));
        assert!(!html.contains("<math>"));
    }

    #[test]
    fn test_styled_node() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::Paragraph).with_parent(0));
        m.body.push(
            Node::new(
                2,
                NodeType::Styled {
                    style_name: "highlight".into(),
                },
            )
            .with_parent(1),
        );
        m.body.push(
            Node::new(
                3,
                NodeType::Text {
                    content: "styled".into(),
                },
            )
            .with_parent(2),
        );
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(1).unwrap().add_child(2);
        m.body.get_mut(2).unwrap().add_child(3);

        let html = HtmlRenderer::new().render(&m);
        assert!(html.contains("<span class=\"highlight\">styled</span>"));
    }

    #[test]
    fn test_mono_inline() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::Paragraph).with_parent(0));
        m.body.push(Node::new(2, NodeType::Mono).with_parent(1));
        m.body.push(
            Node::new(
                3,
                NodeType::Text {
                    content: "code".into(),
                },
            )
            .with_parent(2),
        );
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(1).unwrap().add_child(2);
        m.body.get_mut(2).unwrap().add_child(3);

        let html = HtmlRenderer::new().render(&m);
        assert!(html.contains("<code>code</code>"));
    }

    #[test]
    fn test_underline_inline() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::Paragraph).with_parent(0));
        m.body
            .push(Node::new(2, NodeType::Underline).with_parent(1));
        m.body.push(
            Node::new(
                3,
                NodeType::Text {
                    content: "under".into(),
                },
            )
            .with_parent(2),
        );
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(1).unwrap().add_child(2);
        m.body.get_mut(2).unwrap().add_child(3);

        let html = HtmlRenderer::new().render(&m);
        assert!(html.contains("<u>under</u>"));
    }

    #[test]
    fn test_strikethrough_inline() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::Paragraph).with_parent(0));
        m.body
            .push(Node::new(2, NodeType::Strikethrough).with_parent(1));
        m.body.push(
            Node::new(
                3,
                NodeType::Text {
                    content: "del".into(),
                },
            )
            .with_parent(2),
        );
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(1).unwrap().add_child(2);
        m.body.get_mut(2).unwrap().add_child(3);

        let html = HtmlRenderer::new().render(&m);
        assert!(html.contains("<s>del</s>"));
    }

    #[test]
    fn test_heading_numbering() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body.push(Node::new(1, NodeType::Section).with_parent(0));
        m.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "First".into(),
                },
            )
            .with_parent(1),
        );
        m.body.push(Node::new(3, NodeType::Section).with_parent(0));
        m.body.push(
            Node::new(
                4,
                NodeType::Text {
                    content: "Second".into(),
                },
            )
            .with_parent(3),
        );
        m.body
            .push(Node::new(5, NodeType::Subsection).with_parent(0));
        m.body.push(
            Node::new(
                6,
                NodeType::Text {
                    content: "Sub".into(),
                },
            )
            .with_parent(5),
        );
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(0).unwrap().add_child(3);
        m.body.get_mut(0).unwrap().add_child(5);
        m.body.get_mut(1).unwrap().add_child(2);
        m.body.get_mut(3).unwrap().add_child(4);
        m.body.get_mut(5).unwrap().add_child(6);

        let html = HtmlRenderer::new().render(&m);
        assert!(html.contains("1. First"));
        assert!(html.contains("2. Second"));
        assert!(html.contains("2.1. Sub"));
    }

    #[test]
    fn test_table_header_row_uses_th() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body.push(
            Node::new(
                1,
                NodeType::Table {
                    col_specs: vec![],
                    num_cols: 1,
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
                    content: "H".into(),
                },
            )
            .with_parent(3),
        );
        m.body
            .push(Node::new(5, NodeType::TableRow { is_header: false }).with_parent(1));
        m.body.push(
            Node::new(
                6,
                NodeType::TableCell {
                    colspan: 1,
                    rowspan: 1,
                },
            )
            .with_parent(5),
        );
        m.body.push(
            Node::new(
                7,
                NodeType::Text {
                    content: "D".into(),
                },
            )
            .with_parent(6),
        );
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(1).unwrap().add_child(2);
        m.body.get_mut(1).unwrap().add_child(5);
        m.body.get_mut(2).unwrap().add_child(3);
        m.body.get_mut(3).unwrap().add_child(4);
        m.body.get_mut(5).unwrap().add_child(6);
        m.body.get_mut(6).unwrap().add_child(7);

        let html = HtmlRenderer::new().render(&m);
        assert!(html.contains("<th>H</th>"));
        assert!(html.contains("<td>D</td>"));
    }

    #[test]
    fn test_code_block_no_language() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::CodeBlock { language: None }).with_parent(0));
        m.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "text".into(),
                },
            )
            .with_parent(1),
        );
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(1).unwrap().add_child(2);

        let html = HtmlRenderer::new().render(&m);
        assert!(html.contains("<pre><code>"));
        assert!(!html.contains("language-"));
    }

    #[test]
    fn test_group_node() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(0, NodeType::Document));
        m.body
            .push(Node::new(1, NodeType::Paragraph).with_parent(0));
        m.body.push(Node::new(2, NodeType::Group).with_parent(1));
        m.body.push(
            Node::new(
                3,
                NodeType::Text {
                    content: "inside".into(),
                },
            )
            .with_parent(2),
        );
        m.body.get_mut(0).unwrap().add_child(1);
        m.body.get_mut(1).unwrap().add_child(2);
        m.body.get_mut(2).unwrap().add_child(3);

        let html = HtmlRenderer::new().render(&m);
        assert!(html.contains("inside"));
        assert!(!html.contains("<group>"));
    }
}
