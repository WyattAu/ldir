use ldir_ir::sir::v2::metadata::Dimension;
use ldir_ir::sir::v2::module::SIRModuleV2;
use ldir_ir::sir::v2::nodes::*;

enum HtmlToken {
    OpenTag {
        tag: String,
        attrs: Vec<(String, String)>,
        self_closing: bool,
    },
    CloseTag {
        tag: String,
    },
    Text {
        content: String,
    },
    Comment,
}

struct DomNode {
    tag: String,
    attrs: Vec<(String, String)>,
    children: Vec<DomNode>,
    text: Option<String>,
}

fn attr(attrs: &[(String, String)], name: &str) -> Option<String> {
    attrs
        .iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.clone())
}

fn parse_html_dimension(s: &str) -> Option<Dimension> {
    let s = s.trim();
    if let Some(v) = s
        .strip_suffix("px")
        .and_then(|v| v.trim().parse::<f64>().ok())
    {
        return Some(Dimension::Pt(v));
    }
    if let Some(v) = s
        .strip_suffix('%')
        .and_then(|v| v.trim().parse::<f64>().ok())
    {
        return Some(Dimension::Percent(v));
    }
    if let Some(v) = s
        .strip_suffix("pt")
        .and_then(|v| v.trim().parse::<f64>().ok())
    {
        return Some(Dimension::Pt(v));
    }
    if let Some(v) = s
        .strip_suffix("mm")
        .and_then(|v| v.trim().parse::<f64>().ok())
    {
        return Some(Dimension::Mm(v));
    }
    if let Some(v) = s
        .strip_suffix("cm")
        .and_then(|v| v.trim().parse::<f64>().ok())
    {
        return Some(Dimension::Cm(v));
    }
    if let Some(v) = s
        .strip_suffix("in")
        .and_then(|v| v.trim().parse::<f64>().ok())
    {
        return Some(Dimension::In(v));
    }
    s.parse::<f64>().ok().map(Dimension::Pt)
}

fn extract_width_from_style(style: &str) -> Option<f64> {
    let style_lower = style.to_ascii_lowercase();
    let start = style_lower.find("width:")?;
    let after = &style_lower[start + 6..];
    let end = after.find(';').unwrap_or(after.len());
    let val = after[..end].trim();
    let val = val.strip_suffix("px").unwrap_or(val).trim();
    val.parse::<f64>().ok()
}

fn decode_entities(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'&' {
            let semi = s[i..].find(';');
            if let Some(pos) = semi {
                let entity = &s[i..i + pos + 1];
                let decoded = match entity {
                    "&amp;" => Some('&'),
                    "&lt;" => Some('<'),
                    "&gt;" => Some('>'),
                    "&quot;" => Some('"'),
                    "&apos;" => Some('\''),
                    "&nbsp;" => Some('\u{00A0}'),
                    "&mdash;" => Some('—'),
                    "&ndash;" => Some('–'),
                    "&copy;" => Some('©'),
                    "&reg;" => Some('®'),
                    "&trade;" => Some('™'),
                    "&hellip;" => Some('…'),
                    "&laquo;" => Some('«'),
                    "&raquo;" => Some('»'),
                    "&bull;" => Some('•'),
                    "&middot;" => Some('·'),
                    _ => {
                        if let Some(hex_str) =
                            entity.strip_prefix("&#x").and_then(|e| e.strip_suffix(';'))
                        {
                            if let Ok(code) = u32::from_str_radix(hex_str, 16) {
                                char::from_u32(code)
                            } else {
                                None
                            }
                        } else if let Some(dec_str) =
                            entity.strip_prefix("&#").and_then(|e| e.strip_suffix(';'))
                        {
                            if let Ok(code) = dec_str.parse::<u32>() {
                                char::from_u32(code)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                };
                match decoded {
                    Some(c) => {
                        out.push(c);
                        i += pos + 1;
                    }
                    None => {
                        out.push_str(entity);
                        i += pos + 1;
                    }
                }
            } else {
                out.push('&');
                i += 1;
            }
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

fn tokenize(input: &str) -> Vec<HtmlToken> {
    let mut tokens = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'<' {
            if i + 4 <= bytes.len() && &input[i..i + 4] == "<!--" {
                if let Some(end) = input[i..].find("-->") {
                    tokens.push(HtmlToken::Comment);
                    i += end + 3;
                } else {
                    i = bytes.len();
                }
                continue;
            }

            if i + 9 <= bytes.len() && input[i..i + 9].eq_ignore_ascii_case("<!doctype") {
                if let Some(end) = input[i..].find('>') {
                    i += end + 1;
                } else {
                    i = bytes.len();
                }
                continue;
            }

            if i + 1 < bytes.len() && bytes[i + 1] == b'?' {
                if let Some(end) = input[i..].find("?>") {
                    i += end + 2;
                } else {
                    i = bytes.len();
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
            while i < bytes.len() && bytes[i].is_ascii_alphanumeric() {
                i += 1;
            }
            let tag_name = input[tag_start..i].to_ascii_lowercase();

            if tag_name.is_empty() {
                tokens.push(HtmlToken::Text {
                    content: "<".to_string(),
                });
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
                let attr_name = input[attr_name_start..i].to_ascii_lowercase();

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
                        while i < bytes.len()
                            && bytes[i] != b'>'
                            && bytes[i] != b' '
                            && bytes[i] != b'/'
                        {
                            i += 1;
                        }
                        input[val_start..i].to_string()
                    };
                    attrs.push((attr_name, attr_value));
                } else {
                    attrs.push((attr_name, String::new()));
                }
            }

            if is_closing {
                tokens.push(HtmlToken::CloseTag { tag: tag_name });
            } else {
                tokens.push(HtmlToken::OpenTag {
                    tag: tag_name,
                    attrs,
                    self_closing,
                });
            }
        } else {
            let text_start = i;
            while i < bytes.len() && bytes[i] != b'<' {
                i += 1;
            }
            let raw = &input[text_start..i];
            let decoded = decode_entities(raw);
            if !decoded.is_empty() {
                tokens.push(HtmlToken::Text { content: decoded });
            }
        }
    }

    tokens
}

fn build_dom(tokens: &[HtmlToken]) -> DomNode {
    let root = DomNode {
        tag: String::new(),
        attrs: Vec::new(),
        children: Vec::new(),
        text: None,
    };
    let mut stack: Vec<DomNode> = vec![root];

    for token in tokens {
        match token {
            HtmlToken::OpenTag {
                tag,
                attrs,
                self_closing,
            } => {
                let void = matches!(
                    tag.as_str(),
                    "br" | "hr"
                        | "img"
                        | "input"
                        | "meta"
                        | "link"
                        | "area"
                        | "base"
                        | "col"
                        | "embed"
                        | "source"
                        | "track"
                        | "wbr"
                );
                let node = DomNode {
                    tag: tag.clone(),
                    attrs: attrs.clone(),
                    children: Vec::new(),
                    text: None,
                };
                if !*self_closing && !void {
                    stack.push(node);
                } else {
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(node);
                    }
                }
            }
            HtmlToken::CloseTag { tag } => {
                let skip_tags = ["head", "style", "script", "noscript"];
                if skip_tags.contains(&tag.as_str()) {
                    while let Some(current) = stack.last() {
                        if current.tag == *tag {
                            let popped = stack.pop();
                            if let Some(popped_node) = popped
                                && let Some(parent) = stack.last_mut()
                            {
                                parent.children.push(popped_node);
                            }
                            break;
                        }
                        let popped = stack.pop();
                        if stack.len() <= 1 {
                            if let Some(n) = popped {
                                stack[0].children.push(n);
                            }
                            break;
                        }
                    }
                    continue;
                }

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
            HtmlToken::Text { content } => {
                if let Some(current) = stack.last_mut() {
                    current
                        .text
                        .get_or_insert_with(String::new)
                        .push_str(content);
                }
            }
            HtmlToken::Comment => {}
        }
    }

    while stack.len() > 1 {
        let popped = stack.pop();
        let parent = stack.last_mut();
        if let (Some(popped), Some(parent)) = (popped, parent) {
            parent.children.push(popped);
        }
    }

    stack
        .pop()
        .unwrap_or_else(|| unreachable!("stack always contains root element"))
}

struct Converter {
    module: SIRModuleV2,
    next_id: u32,
}

impl Converter {
    fn new() -> Self {
        let module = SIRModuleV2::from_source("html", "");
        Self { module, next_id: 0 }
    }

    fn alloc_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn convert_dom(&mut self, dom: &DomNode) {
        let body = Self::find_body(dom);
        let children: &[DomNode] = match &body {
            Some(node) => &node.children,
            None => &dom.children,
        };

        if let Some(lang) = body.as_ref().and_then(|b| attr(&b.attrs, "lang")) {
            self.module.metadata.language = lang;
        } else {
            for child in &dom.children {
                if child.tag == "html" {
                    if let Some(lang) = attr(&child.attrs, "lang") {
                        self.module.metadata.language = lang;
                    }
                    break;
                }
            }
        }

        for child in children {
            self.convert_node(child, None);
        }
    }

    fn find_body(dom: &DomNode) -> Option<&DomNode> {
        for child in &dom.children {
            if child.tag == "html" {
                for html_child in &child.children {
                    if html_child.tag == "body" {
                        return Some(html_child);
                    }
                }
                return None;
            }
            if child.tag == "body" {
                return Some(child);
            }
        }
        None
    }

    fn convert_node(&mut self, dom: &DomNode, parent_id: Option<u32>) -> Option<u32> {
        let skip_tags = ["head", "style", "script", "noscript", "html", "!doctype"];
        if skip_tags.contains(&dom.tag.as_str()) {
            for child in &dom.children {
                self.convert_node(child, parent_id);
            }
            return None;
        }

        let tag = dom.tag.as_str();

        if matches!(tag, "meta" | "link" | "title") {
            return None;
        }

        let node_type = match tag {
            "h1" => Some(NodeType::Chapter),
            "h2" => Some(NodeType::Section),
            "h3" => Some(NodeType::Subsection),
            "h4" => Some(NodeType::Subsubsection),
            "h5" | "h6" => Some(NodeType::Subsubsection),
            "p" => Some(NodeType::Paragraph),
            "strong" | "b" => Some(NodeType::Bold),
            "em" | "i" => Some(NodeType::Italic),
            "code" => Some(NodeType::Mono),
            "u" => Some(NodeType::Underline),
            "s" | "del" | "strike" => Some(NodeType::Strikethrough),
            "a" => {
                let url = attr(&dom.attrs, "href").unwrap_or_default();
                let title = attr(&dom.attrs, "title");
                Some(NodeType::Link { url, title })
            }
            "img" => {
                let source = attr(&dom.attrs, "src").unwrap_or_default();
                let alt = attr(&dom.attrs, "alt").unwrap_or_default();
                let width = attr(&dom.attrs, "width").and_then(|v| parse_html_dimension(&v));
                let height = attr(&dom.attrs, "height").and_then(|v| parse_html_dimension(&v));
                Some(NodeType::Image {
                    source,
                    alt,
                    width,
                    height,
                    placement: FloatPlacement::Here,
                })
            }
            "ul" => Some(NodeType::List {
                list_type: ListType::Unordered,
                ordered: false,
                start: None,
            }),
            "ol" => Some(NodeType::List {
                list_type: ListType::Ordered,
                ordered: true,
                start: attr(&dom.attrs, "start").and_then(|s| s.parse::<u32>().ok()),
            }),
            "li" => Some(NodeType::ListItem),
            "dl" => Some(NodeType::List {
                list_type: ListType::Description,
                ordered: false,
                start: None,
            }),
            "dt" => Some(NodeType::Bold),
            "dd" => Some(NodeType::Group),
            "colgroup" | "col" => None,
            "blockquote" => Some(NodeType::BlockQuote),
            "pre" => Some(NodeType::CodeBlock {
                language: None,
                content: String::new(),
            }),
            "table" => Some(NodeType::Table {
                col_specs: Vec::new(),
                num_cols: 0,
                caption: None,
                column_widths: Vec::new(),
                header_row: false,
            }),
            "tr" => {
                let is_header = dom.children.iter().any(|c| c.tag == "th");
                Some(NodeType::TableRow { is_header })
            }
            "th" | "td" => {
                let colspan = attr(&dom.attrs, "colspan")
                    .and_then(|v| v.parse::<u8>().ok())
                    .unwrap_or(1);
                let rowspan = attr(&dom.attrs, "rowspan")
                    .and_then(|v| v.parse::<u8>().ok())
                    .unwrap_or(1);
                Some(NodeType::TableCell { colspan, rowspan })
            }
            "hr" => Some(NodeType::ThematicBreak),
            "br" => Some(NodeType::LineBreak),
            "div" => Some(NodeType::Group),
            "span" => {
                if let Some(class) = attr(&dom.attrs, "class") {
                    let first_class = class.split_whitespace().next().unwrap_or("").to_string();
                    if !first_class.is_empty() {
                        Some(NodeType::Styled {
                            style_name: first_class,
                        })
                    } else {
                        Some(NodeType::Group)
                    }
                } else {
                    Some(NodeType::Group)
                }
            }
            "sup" => Some(NodeType::Group),
            "sub" => Some(NodeType::Group),
            "math" | "mtext" => Some(NodeType::MathInline {
                content: dom.text.clone().unwrap_or_default(),
            }),
            "figcaption" => Some(NodeType::Caption),
            "figure" => Some(NodeType::Figure {
                placement: FloatPlacement::Here,
            }),
            _ => Some(NodeType::Group),
        };

        let node_type = node_type?;

        let id = self.alloc_id();
        let mut node = Node::new(id, node_type);

        if let Some(pid) = parent_id {
            node.parent_id = Some(pid);
        }

        if let Some(label) = attr(&dom.attrs, "id") {
            node.label = Some(label);
        }

        if let Some(class) = attr(&dom.attrs, "class") {
            let first = class.split_whitespace().next().unwrap_or("").to_string();
            if !first.is_empty() && node.label.is_none() {
                node.style = Some(first);
            }
        }

        let emitted_id = self.module.body.push(node);

        if let Some(pid) = parent_id
            && let Some(parent) = self.module.body.get_mut(pid)
        {
            parent.add_child(emitted_id);
        }

        if tag == "pre" {
            if let Some(inner) = dom.children.first()
                && inner.tag == "code"
            {
                let lang = attr(&inner.attrs, "class")
                    .and_then(|c| {
                        c.split_whitespace()
                            .find(|cls| cls.starts_with("language-"))
                            .map(|cls| cls.strip_prefix("language-").unwrap_or(cls).to_string())
                    })
                    .or_else(|| attr(&inner.attrs, "lang"));
                if let Some(pre_node) = self.module.body.get_mut(emitted_id) {
                    pre_node.node_type = NodeType::CodeBlock {
                        language: lang,
                        content: String::new(),
                    };
                }

                if let Some(text) = &inner.text {
                    let text_id = self.alloc_id();
                    self.module.body.push(
                        Node::new(
                            text_id,
                            NodeType::Text {
                                content: text.clone(),
                            },
                        )
                        .with_parent(emitted_id),
                    );
                    if let Some(pre_node) = self.module.body.get_mut(emitted_id) {
                        pre_node.add_child(text_id);
                    }
                }
                return Some(emitted_id);
            }

            if let Some(text) = &dom.text {
                let text_id = self.alloc_id();
                self.module.body.push(
                    Node::new(
                        text_id,
                        NodeType::Text {
                            content: text.clone(),
                        },
                    )
                    .with_parent(emitted_id),
                );
                if let Some(pre_node) = self.module.body.get_mut(emitted_id) {
                    pre_node.add_child(text_id);
                }
            }
            return Some(emitted_id);
        }

        if tag == "img" {
            return Some(emitted_id);
        }

        if tag == "table" {
            let mut col_widths = Vec::new();
            for child in &dom.children {
                if child.tag == "colgroup" {
                    for col in &child.children {
                        if col.tag == "col" {
                            if let Some(w) =
                                attr(&col.attrs, "width").and_then(|v| v.parse::<f64>().ok())
                            {
                                col_widths.push(w);
                            } else if let Some(style) = attr(&col.attrs, "style")
                                && let Some(w) = extract_width_from_style(&style)
                            {
                                col_widths.push(w);
                            }
                        }
                    }
                } else if child.tag == "col" {
                    if let Some(w) = attr(&child.attrs, "width").and_then(|v| v.parse::<f64>().ok())
                    {
                        col_widths.push(w);
                    } else if let Some(style) = attr(&child.attrs, "style")
                        && let Some(w) = extract_width_from_style(&style)
                    {
                        col_widths.push(w);
                    }
                }
            }
            if !col_widths.is_empty()
                && let Some(table_node) = self.module.body.get_mut(emitted_id)
                && let NodeType::Table {
                    column_widths,
                    num_cols,
                    ..
                } = &mut table_node.node_type
            {
                *num_cols = col_widths.len();
                *column_widths = col_widths;
            }
        }

        if tag == "br" || tag == "hr" {
            return Some(emitted_id);
        }

        if tag == "math" || tag == "mtext" {
            return Some(emitted_id);
        }

        if let Some(text) = &dom.text
            && !text.is_empty()
        {
            let text_id = self.alloc_id();
            self.module.body.push(
                Node::new(
                    text_id,
                    NodeType::Text {
                        content: text.clone(),
                    },
                )
                .with_parent(emitted_id),
            );
            if let Some(this_node) = self.module.body.get_mut(emitted_id) {
                this_node.add_child(text_id);
            }
        }

        for child in &dom.children {
            self.convert_node(child, Some(emitted_id));
        }

        Some(emitted_id)
    }
}

pub fn parse_html(html: &str) -> SIRModuleV2 {
    let tokens = tokenize(html);
    let dom = build_dom(&tokens);
    let mut converter = Converter::new();
    converter.convert_dom(&dom);
    converter.module
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find_node_by_type(module: &SIRModuleV2, pred: fn(&NodeType) -> bool) -> Option<&Node> {
        module.body.iter().find(|n| pred(&n.node_type))
    }

    fn find_nodes_by_type(module: &SIRModuleV2, pred: fn(&NodeType) -> bool) -> Vec<&Node> {
        module.body.iter().filter(|n| pred(&n.node_type)).collect()
    }

    #[test]
    fn test_simple_paragraph() {
        let module = parse_html("<p>Hello</p>");
        assert!(find_node_by_type(&module, |nt| matches!(nt, NodeType::Paragraph)).is_some());
        assert!(
            find_node_by_type(
                &module,
                |nt| matches!(nt, NodeType::Text { content } if content == "Hello")
            )
            .is_some()
        );
    }

    #[test]
    fn test_heading() {
        let module = parse_html("<h1>Title</h1>");
        assert!(find_node_by_type(&module, |nt| matches!(nt, NodeType::Chapter)).is_some());
        assert!(
            find_node_by_type(
                &module,
                |nt| matches!(nt, NodeType::Text { content } if content == "Title")
            )
            .is_some()
        );
    }

    #[test]
    fn test_heading_levels() {
        let module = parse_html("<h1>A</h1><h2>B</h2><h3>C</h3><h4>D</h4><h5>E</h5><h6>F</h6>");
        assert!(find_node_by_type(&module, |nt| matches!(nt, NodeType::Chapter)).is_some());
        assert!(find_node_by_type(&module, |nt| matches!(nt, NodeType::Section)).is_some());
        assert!(find_node_by_type(&module, |nt| matches!(nt, NodeType::Subsection)).is_some());
        assert!(find_node_by_type(&module, |nt| matches!(nt, NodeType::Subsubsection)).is_some());
    }

    #[test]
    fn test_bold_italic() {
        let module = parse_html("<p><strong>bold</strong> <em>italic</em></p>");
        assert!(find_node_by_type(&module, |nt| matches!(nt, NodeType::Bold)).is_some());
        assert!(find_node_by_type(&module, |nt| matches!(nt, NodeType::Italic)).is_some());
    }

    #[test]
    fn test_link() {
        let module = parse_html(r#"<a href="https://example.com">click</a>"#);
        let link = find_node_by_type(&module, |nt| matches!(nt, NodeType::Link { .. }));
        assert!(link.is_some());
        if let Some(Node {
            node_type: NodeType::Link { url, .. },
            ..
        }) = link
        {
            assert_eq!(url, "https://example.com");
        }
    }

    #[test]
    fn test_image() {
        let module = parse_html(r#"<img src="photo.png" alt="a photo">"#);
        let img = find_node_by_type(&module, |nt| matches!(nt, NodeType::Image { .. }));
        assert!(img.is_some());
        if let Some(Node {
            node_type: NodeType::Image { source, alt, .. },
            ..
        }) = img
        {
            assert_eq!(source, "photo.png");
            assert_eq!(alt, "a photo");
        }
    }

    #[test]
    fn test_unordered_list() {
        let module = parse_html("<ul><li>one</li><li>two</li></ul>");
        let list = find_node_by_type(&module, |nt| {
            matches!(nt, NodeType::List { ordered: false, .. })
        });
        assert!(list.is_some());
        let items = find_nodes_by_type(&module, |nt| matches!(nt, NodeType::ListItem));
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_ordered_list() {
        let module = parse_html("<ol><li>first</li></ol>");
        let list = find_node_by_type(&module, |nt| {
            matches!(nt, NodeType::List { ordered: true, .. })
        });
        assert!(list.is_some());
    }

    #[test]
    fn test_blockquote() {
        let module = parse_html("<blockquote>quoted text</blockquote>");
        assert!(find_node_by_type(&module, |nt| matches!(nt, NodeType::BlockQuote)).is_some());
        assert!(
            find_node_by_type(
                &module,
                |nt| matches!(nt, NodeType::Text { content } if content == "quoted text")
            )
            .is_some()
        );
    }

    #[test]
    fn test_code_block() {
        let module = parse_html("<pre><code>let x = 1;</code></pre>");
        let cb = find_node_by_type(&module, |nt| matches!(nt, NodeType::CodeBlock { .. }));
        assert!(cb.is_some());
    }

    #[test]
    fn test_code_block_with_language() {
        let module = parse_html(r#"<pre><code class="language-rust">fn main() {}</code></pre>"#);
        let cb = find_node_by_type(
            &module,
            |nt| matches!(nt, NodeType::CodeBlock { language: Some(lang), .. } if lang == "rust"),
        );
        assert!(cb.is_some());
    }

    #[test]
    fn test_table() {
        let module = parse_html("<table><tr><td>A</td></tr></table>");
        assert!(find_node_by_type(&module, |nt| matches!(nt, NodeType::Table { .. })).is_some());
        assert!(find_node_by_type(&module, |nt| matches!(nt, NodeType::TableRow { .. })).is_some());
        assert!(
            find_node_by_type(&module, |nt| matches!(nt, NodeType::TableCell { .. })).is_some()
        );
    }

    #[test]
    fn test_table_with_header() {
        let module =
            parse_html("<table><tr><th>H1</th><th>H2</th></tr><tr><td>D</td></tr></table>");
        let header_rows: Vec<_> = module
            .body
            .iter()
            .filter(|n| matches!(n.node_type, NodeType::TableRow { is_header: true }))
            .collect();
        assert_eq!(header_rows.len(), 1);
        let data_rows: Vec<_> = module
            .body
            .iter()
            .filter(|n| matches!(n.node_type, NodeType::TableRow { is_header: false }))
            .collect();
        assert_eq!(data_rows.len(), 1);
    }

    #[test]
    fn test_nested_formatting() {
        let module = parse_html("<p><strong><em>bold italic</em></strong></p>");
        assert!(find_node_by_type(&module, |nt| matches!(nt, NodeType::Bold)).is_some());
        assert!(find_node_by_type(&module, |nt| matches!(nt, NodeType::Italic)).is_some());
        assert!(
            find_node_by_type(
                &module,
                |nt| matches!(nt, NodeType::Text { content } if content == "bold italic")
            )
            .is_some()
        );
    }

    #[test]
    fn test_comment_ignored() {
        let module = parse_html("<!-- this is a comment --><p>Hello</p>");
        assert!(find_node_by_type(&module, |nt| matches!(nt, NodeType::Paragraph)).is_some());
        let _total = module.body.iter().count();
        let group_count = module
            .body
            .iter()
            .filter(|n| matches!(n.node_type, NodeType::Group))
            .count();
        assert_eq!(group_count, 0, "comments should produce no nodes");
    }

    #[test]
    fn test_head_skipped() {
        let module = parse_html(
            "<html><head><title>Test</title><style>body{}</style></head><body><p>content</p></body></html>",
        );
        assert!(find_node_by_type(&module, |nt| matches!(nt, NodeType::Paragraph)).is_some());
        let has_title = module
            .body
            .iter()
            .any(|n| matches!(&n.node_type, NodeType::Text { content } if content == "Test"));
        assert!(!has_title, "title inside head should be skipped");
    }

    #[test]
    fn test_entity_decoding() {
        let module = parse_html("<p>&amp; &lt; &gt; &quot;</p>");
        let text = module.body.iter().find_map(|n| match &n.node_type {
            NodeType::Text { content } => Some(content.clone()),
            _ => None,
        });
        assert!(text.is_some());
        let text = text.unwrap();
        assert!(text.contains('&'), "should contain decoded &");
        assert!(text.contains('<'), "should contain decoded <");
        assert!(text.contains('>'), "should contain decoded >");
        assert!(text.contains('"'), "should contain decoded \"");
    }

    #[test]
    fn test_numeric_entity() {
        let module = parse_html("<p>&#65;&#x42;</p>");
        let text = module.body.iter().find_map(|n| match &n.node_type {
            NodeType::Text { content } => Some(content.clone()),
            _ => None,
        });
        let text = text.unwrap();
        assert!(text.contains('A'), "should decode &#65; to A");
        assert!(text.contains('B'), "should decode &#x42; to B");
    }

    #[test]
    fn test_id_to_label() {
        let module = parse_html(r#"<h1 id="intro">Introduction</h1>"#);
        let heading = find_node_by_type(&module, |nt| matches!(nt, NodeType::Chapter));
        assert!(heading.is_some());
        assert_eq!(heading.unwrap().label.as_deref(), Some("intro"));
    }

    #[test]
    fn test_class_to_style() {
        let module = parse_html(r#"<span class="red">text</span>"#);
        let styled = find_node_by_type(&module, |nt| matches!(nt, NodeType::Styled { .. }));
        assert!(styled.is_some());
        if let Some(Node {
            node_type: NodeType::Styled { style_name },
            ..
        }) = styled
        {
            assert_eq!(style_name, "red");
        }
    }

    #[test]
    fn test_self_closing_tags() {
        let module = parse_html("<br/><hr/>");
        assert!(find_node_by_type(&module, |nt| matches!(nt, NodeType::LineBreak)).is_some());
        assert!(find_node_by_type(&module, |nt| matches!(nt, NodeType::ThematicBreak)).is_some());
    }

    #[test]
    fn test_full_document() {
        let module = parse_html(
            r#"<!DOCTYPE html>
<html>
<head>
<title>Test</title>
</head>
<body>
<h1>Hello</h1>
<p>World</p>
</body>
</html>"#,
        );
        assert!(find_node_by_type(&module, |nt| matches!(nt, NodeType::Chapter)).is_some());
        assert!(find_node_by_type(&module, |nt| matches!(nt, NodeType::Paragraph)).is_some());
    }

    #[test]
    fn test_hr_tag() {
        let module = parse_html("<hr>");
        assert!(find_node_by_type(&module, |nt| matches!(nt, NodeType::ThematicBreak)).is_some());
    }

    #[test]
    fn test_inline_code() {
        let module = parse_html("<code>var x = 1;</code>");
        assert!(find_node_by_type(&module, |nt| matches!(nt, NodeType::Mono)).is_some());
        assert!(
            find_node_by_type(
                &module,
                |nt| matches!(nt, NodeType::Text { content } if content == "var x = 1;")
            )
            .is_some()
        );
    }

    #[test]
    fn test_math_element() {
        let module = parse_html("<math>x^2 + y^2 = z^2</math>");
        let math = find_node_by_type(&module, |nt| matches!(nt, NodeType::MathInline { .. }));
        assert!(math.is_some());
        if let Some(Node {
            node_type: NodeType::MathInline { content },
            ..
        }) = math
        {
            assert_eq!(content, "x^2 + y^2 = z^2");
        }
    }

    #[test]
    fn test_underline() {
        let module = parse_html("<u>underlined</u>");
        assert!(find_node_by_type(&module, |nt| matches!(nt, NodeType::Underline)).is_some());
    }

    #[test]
    fn test_strikethrough() {
        let module = parse_html("<s>deleted</s><del>removed</del><strike>gone</strike>");
        let strikes = find_nodes_by_type(&module, |nt| matches!(nt, NodeType::Strikethrough));
        assert_eq!(strikes.len(), 3);
    }

    #[test]
    fn test_figure_and_caption() {
        let module = parse_html(
            "<figure><img src=\"fig.png\" alt=\"Figure\"><figcaption>Caption text</figcaption></figure>",
        );
        assert!(find_node_by_type(&module, |nt| matches!(nt, NodeType::Figure { .. })).is_some());
        assert!(find_node_by_type(&module, |nt| matches!(nt, NodeType::Caption)).is_some());
    }

    #[test]
    fn test_colspan_rowspan() {
        let module = parse_html(r#"<table><tr><td colspan="2" rowspan="3">span</td></tr></table>"#);
        let cell = find_node_by_type(&module, |nt| {
            matches!(
                nt,
                NodeType::TableCell {
                    colspan: 2,
                    rowspan: 3
                }
            )
        });
        assert!(cell.is_some());
    }

    #[test]
    fn test_lang_attribute() {
        let module = parse_html(r#"<html lang="fr"><body><p>Bonjour</p></body></html>"#);
        assert_eq!(module.metadata.language, "fr");
    }

    #[test]
    fn test_empty_input() {
        let module = parse_html("");
        assert!(module.body.is_empty());
    }

    #[test]
    fn test_deeply_nested() {
        let module = parse_html("<div><div><div><p>deep</p></div></div></div>");
        assert!(find_node_by_type(&module, |nt| matches!(nt, NodeType::Paragraph)).is_some());
        assert!(
            find_node_by_type(
                &module,
                |nt| matches!(nt, NodeType::Text { content } if content == "deep")
            )
            .is_some()
        );
    }

    #[test]
    fn test_mixed_inline_content() {
        let module = parse_html("<p>Hello <strong>world</strong> and <em>everyone</em></p>");
        let bold = find_node_by_type(&module, |nt| matches!(nt, NodeType::Bold));
        assert!(bold.is_some());
        let italic = find_node_by_type(&module, |nt| matches!(nt, NodeType::Italic));
        assert!(italic.is_some());
    }

    #[test]
    fn test_source_format_tracked() {
        let module = parse_html("<p>test</p>");
        assert_eq!(module.header.source_format.as_deref(), Some("html"));
    }
}
