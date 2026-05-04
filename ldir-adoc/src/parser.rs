//! Recursive descent parser for Asciidoc → S-IR v2.

use ldir_ir::sir::v2::SIRModuleV2;
use ldir_ir::sir::v2::nodes::{ColSpec, ColumnAlign, ListType, Node, NodeType};

pub fn parse_asciidoc(text: &str) -> SIRModuleV2 {
    let lines: Vec<&str> = text.lines().collect();
    let mut parser = AsciidocParser::new(lines);
    parser.parse_document()
}

struct AsciidocParser {
    lines: Vec<String>,
    pos: usize,
    next_id: u32,
}

impl AsciidocParser {
    fn new(lines: Vec<&str>) -> Self {
        let lines: Vec<String> = lines.into_iter().map(|l| l.to_string()).collect();
        Self {
            lines,
            pos: 0,
            next_id: 1,
        }
    }

    fn peek(&self) -> Option<&str> {
        self.lines.get(self.pos).map(|s| s.as_str())
    }

    fn advance(&mut self) -> Option<String> {
        if self.pos < self.lines.len() {
            let line = self.lines[self.pos].clone();
            self.pos += 1;
            Some(line)
        } else {
            None
        }
    }

    fn at_end(&self) -> bool {
        self.pos >= self.lines.len()
    }

    fn gen_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn skip_blank(&mut self) {
        while let Some(line) = self.peek() {
            if line.trim().is_empty() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn count_heading_level(line: &str) -> Option<u8> {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            return None;
        }
        let mut count = 0u8;
        for ch in trimmed.chars() {
            if ch == '=' {
                count += 1;
            } else {
                break;
            }
        }
        if (1..=6).contains(&count) {
            let rest = &trimmed[count as usize..];
            if rest.starts_with(' ') || rest.is_empty() {
                return Some(count);
            }
        }
        None
    }

    fn is_attribute_line(line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with(':') && trimmed.contains(": ") && !trimmed.starts_with("//")
    }

    fn parse_attribute(line: &str) -> Option<(String, String)> {
        let trimmed = line.trim().trim_start_matches(':');
        if let Some(pos) = trimmed.find(": ") {
            let key = trimmed[..pos].trim().to_string();
            let val = trimmed[pos + 2..].trim().to_string();
            if !key.is_empty() {
                return Some((key, val));
            }
        }
        None
    }

    fn is_comment_line(line: &str) -> bool {
        line.trim_start().starts_with("//")
    }

    fn is_ordered_list_item(line: &str) -> bool {
        let trimmed = line.trim_start();
        trimmed.starts_with(". ")
            || (trimmed.starts_with('.')
                && trimmed.len() > 1
                && trimmed.chars().nth(1).is_some_and(|c| c.is_ascii_digit()))
    }

    fn is_unordered_list_item(line: &str) -> bool {
        let trimmed = line.trim_start();
        trimmed.starts_with("* ")
            || (trimmed.starts_with('*')
                && trimmed.len() > 1
                && trimmed.chars().nth(1).is_some_and(|c| c != '*'))
    }

    fn is_code_block_delimiter(line: &str) -> bool {
        let trimmed = line.trim();
        trimmed == "----"
            || trimmed == "...."
            || trimmed == "===="
            || trimmed == "~~~~"
            || trimmed == "----+"
            || trimmed == "----"
    }

    fn is_literal_block_delimiter(line: &str) -> bool {
        let trimmed = line.trim();
        trimmed == "----" || trimmed == "...."
    }

    fn is_table_delimiter(line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with("|===") || trimmed == "|==="
    }

    fn is_table_end(line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.ends_with("===") || trimmed == "|==="
    }

    fn admonition_type(line: &str) -> Option<&'static str> {
        let trimmed = line.trim();
        for kw in &["NOTE:", "TIP:", "WARNING:", "CAUTION:", "IMPORTANT:"] {
            if trimmed.starts_with(kw) {
                let rest = &trimmed[kw.len()..];
                if rest.is_empty() || rest.starts_with(' ') {
                    return Some(&kw[..kw.len() - 1]);
                }
            }
        }
        None
    }

    fn parse_inline_content(text: &str) -> Vec<NodeType> {
        let mut nodes = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;
        let len = chars.len();

        while i < len {
            let ch = chars[i];

            if ch == '\\' && i + 1 < len {
                i += 1;
                let mut buf = String::new();
                buf.push(chars[i]);
                i += 1;
                nodes.push(NodeType::Text { content: buf });
                continue;
            }

            if ch == '*' {
                let end = Self::find_closing_delim(&chars, i + 1, '*');
                if let Some(end_idx) = end {
                    let content: String = chars[i + 1..end_idx].iter().collect();
                    let trimmed = content.trim().to_string();
                    if !trimmed.is_empty() {
                        nodes.push(NodeType::Bold);
                        nodes.push(NodeType::Text { content: trimmed });
                    }
                    i = end_idx + 1;
                    continue;
                }
            }

            if ch == '_' {
                let end = Self::find_closing_delim(&chars, i + 1, '_');
                if let Some(end_idx) = end {
                    let content: String = chars[i + 1..end_idx].iter().collect();
                    let trimmed = content.trim().to_string();
                    if !trimmed.is_empty() {
                        nodes.push(NodeType::Italic);
                        nodes.push(NodeType::Text { content: trimmed });
                    }
                    i = end_idx + 1;
                    continue;
                }
            }

            if ch == '`' {
                let end = Self::find_closing_delim(&chars, i + 1, '`');
                if let Some(end_idx) = end {
                    let content: String = chars[i + 1..end_idx].iter().collect();
                    let trimmed = content.trim().to_string();
                    if !trimmed.is_empty() {
                        nodes.push(NodeType::Mono);
                        nodes.push(NodeType::Text { content: trimmed });
                    }
                    i = end_idx + 1;
                    continue;
                }
            }

            if ch == '$' {
                let end = Self::find_closing_delim(&chars, i + 1, '$');
                if let Some(end_idx) = end {
                    let content: String = chars[i + 1..end_idx].iter().collect();
                    if !content.is_empty() {
                        nodes.push(NodeType::MathInline { content });
                    }
                    i = end_idx + 1;
                    continue;
                }
            }

            if ch == 'h' && i + 11 < len {
                let rest: String = chars[i..i + 11].iter().collect();
                if rest == "footnote:[" {
                    let start = i + 11;
                    if let Some(end) = chars[start..].iter().position(|&c| c == ']') {
                        let content: String = chars[start..start + end].iter().collect();
                        if !content.is_empty() {
                            nodes.push(NodeType::Footnote { content });
                        }
                        i = start + end + 1;
                        continue;
                    }
                }
            }

            if ch == '[' && i + 1 < len && chars[i + 1] == '[' {
                let start = i + 2;
                if let Some(mid) = chars[start..].iter().position(|&c| c == ']') {
                    if mid + start + 1 < len && chars[mid + start + 1] == ']' {
                        let content: String = chars[start..start + mid].iter().collect();
                        if !content.is_empty() {
                            nodes.push(NodeType::Text {
                                content: format!("[[{}]]", content),
                            });
                        }
                        i = start + mid + 2;
                        continue;
                    }
                }
            }

            if ch == 'i' && i + 6 < len {
                let rest: String = chars[i..i + 7].iter().collect();
                if rest == "image::" {
                    i += 7;
                    let mut path = String::new();
                    while i < len && chars[i] != '[' {
                        path.push(chars[i]);
                        i += 1;
                    }
                    let mut alt = String::new();
                    if i < len && chars[i] == '[' {
                        i += 1;
                        while i < len && chars[i] != ']' {
                            alt.push(chars[i]);
                            i += 1;
                        }
                        if i < len {
                            i += 1;
                        }
                    }
                    let alt_text = if alt.is_empty() {
                        path.clone()
                    } else {
                        alt.split(',').next().unwrap_or("").trim().to_string()
                    };
                    path = path.trim().trim_end_matches(':').to_string();
                    nodes.push(NodeType::Image {
                        source: path,
                        alt: alt_text,
                        width: None,
                        height: None,
                    });
                    continue;
                }
            }

            if ch == 'l' && i + 4 < len {
                let rest: String = chars[i..i + 5].iter().collect();
                if rest == "link:" {
                    i += 5;
                    let mut url = String::new();
                    while i < len && chars[i] != '[' {
                        url.push(chars[i]);
                        i += 1;
                    }
                    let mut link_text = String::new();
                    if i < len && chars[i] == '[' {
                        i += 1;
                        while i < len && chars[i] != ']' {
                            link_text.push(chars[i]);
                            i += 1;
                        }
                        if i < len {
                            i += 1;
                        }
                    }
                    if !url.is_empty() {
                        nodes.push(NodeType::Link {
                            url,
                            title: if link_text.is_empty() {
                                None
                            } else {
                                Some(link_text)
                            },
                        });
                    }
                    continue;
                }
            }

            if ch == 'h' && i + 3 < len {
                let rest: String = chars[i..i + 4].iter().collect();
                if rest == "http" {
                    let mut url = String::new();
                    while i < len && chars[i] != '[' && !chars[i].is_whitespace() {
                        url.push(chars[i]);
                        i += 1;
                    }
                    let mut link_text = String::new();
                    if i < len && chars[i] == '[' {
                        i += 1;
                        while i < len && chars[i] != ']' {
                            link_text.push(chars[i]);
                            i += 1;
                        }
                        if i < len {
                            i += 1;
                        }
                    }
                    if !url.is_empty() {
                        nodes.push(NodeType::Link {
                            url,
                            title: if link_text.is_empty() {
                                None
                            } else {
                                Some(link_text)
                            },
                        });
                    }
                    continue;
                }
            }

            if ch == 'l' && i + 4 < len {
                let rest: String = chars[i..i + 5].iter().collect();
                if rest == "link:" {
                    i += 5;
                    let mut url = String::new();
                    while i < len && chars[i] != '[' {
                        url.push(chars[i]);
                        i += 1;
                    }
                    let mut link_text = String::new();
                    if i < len && chars[i] == '[' {
                        i += 1;
                        while i < len && chars[i] != ']' {
                            link_text.push(chars[i]);
                            i += 1;
                        }
                        if i < len {
                            i += 1;
                        }
                    }
                    if !url.is_empty() {
                        nodes.push(NodeType::Link {
                            url,
                            title: if link_text.is_empty() {
                                None
                            } else {
                                Some(link_text)
                            },
                        });
                    }
                    continue;
                }
            }

            if ch == 'f' && i + 9 < len {
                let rest: String = chars[i..i + 10].iter().collect();
                if rest == "footnote:[" {
                    let start = i + 11;
                    if let Some(end) = chars[start..].iter().position(|&c| c == ']') {
                        let content: String = chars[start..start + end].iter().collect();
                        if !content.is_empty() {
                            nodes.push(NodeType::Footnote { content });
                        }
                        i = start + end + 1;
                        continue;
                    }
                }
            }

            if ch == 'i' && i + 6 < len {
                let rest: String = chars[i..i + 7].iter().collect();
                if rest == "image::" {
                    i += 7;
                    let mut path = String::new();
                    while i < len && chars[i] != '[' {
                        path.push(chars[i]);
                        i += 1;
                    }
                    let mut alt = String::new();
                    if i < len && chars[i] == '[' {
                        i += 1;
                        while i < len && chars[i] != ']' {
                            alt.push(chars[i]);
                            i += 1;
                        }
                        if i < len {
                            i += 1;
                        }
                    }
                    let alt_text = if alt.is_empty() {
                        path.clone()
                    } else {
                        alt.split(',').next().unwrap_or("").trim().to_string()
                    };
                    path = path.trim().to_string();
                    nodes.push(NodeType::Image {
                        source: path,
                        alt: alt_text,
                        width: None,
                        height: None,
                    });
                    continue;
                }
            }

            if ch.is_whitespace() {
                i += 1;
                continue;
            }

            let mut buf = String::new();
            while i < len {
                let c = chars[i];
                if c == '*' || c == '_' || c == '`' || c == '$' || c == '[' || c == ']' || c == '\\'
                {
                    break;
                }
                if c == 'h' && i + 3 < len {
                    let r: String = chars[i..i + 4].iter().collect();
                    if r == "http" {
                        break;
                    }
                }
                if c == 'l' && i + 4 < len {
                    let r: String = chars[i..i + 5].iter().collect();
                    if r == "link:" {
                        break;
                    }
                }
                if c == 'f' && i + 9 < len {
                    let r: String = chars[i..i + 10].iter().collect();
                    if r == "footnote:[" {
                        break;
                    }
                }
                if c == 'i' && i + 6 < len {
                    let r: String = chars[i..i + 7].iter().collect();
                    if r == "image::" {
                        break;
                    }
                }
                buf.push(c);
                i += 1;
            }
            let trimmed = buf.trim().to_string();
            if !trimmed.is_empty() {
                nodes.push(NodeType::Text { content: trimmed });
            } else if i < len {
                nodes.push(NodeType::Text {
                    content: chars[i].to_string(),
                });
                i += 1;
            }
        }

        nodes
    }

    fn find_closing_delim(chars: &[char], start: usize, delim: char) -> Option<usize> {
        let mut i = start;
        while i < chars.len() {
            if chars[i] == delim {
                return Some(i);
            }
            i += 1;
        }
        None
    }

    fn add_children(
        &mut self,
        module: &mut SIRModuleV2,
        parent_id: u32,
        child_nodes: Vec<NodeType>,
    ) {
        for node_type in child_nodes {
            let child_id = self.gen_id();
            module
                .body
                .push(Node::new(child_id, node_type).with_parent(parent_id));
            if let Some(parent) = module.body.get_mut(parent_id) {
                parent.add_child(child_id);
            }
        }
    }

    fn parse_document(&mut self) -> SIRModuleV2 {
        let mut module = SIRModuleV2::from_source("asciidoc", "<input>");
        let doc_id = self.gen_id();
        module.body.push(Node::new(doc_id, NodeType::Document));

        let mut title_set = false;
        let mut author_set = false;

        if let Some(line) = self.peek() {
            if let Some(level) = Self::count_heading_level(line) {
                if level == 1 {
                    let inline = Self::parse_inline_content(line.trim_start_matches('=').trim());
                    if let Some(NodeType::Text { content }) = inline.first() {
                        module.metadata.title = Some(content.clone());
                    }
                    title_set = true;
                    self.advance();
                }
            }
        }

        while !self.at_end() {
            if let Some(line) = self.peek() {
                let trimmed = line.trim();

                if trimmed.is_empty() {
                    self.advance();
                    continue;
                }

                if Self::is_comment_line(line) {
                    self.advance();
                    continue;
                }

                if Self::is_attribute_line(line) {
                    if let Some((key, val)) = Self::parse_attribute(line) {
                        match key.as_str() {
                            "subject" | "description" => {
                                module.metadata.subject = Some(val);
                            }
                            "lang" => {
                                module.metadata.language = val;
                            }
                            "author" if !author_set => {
                                module.metadata.author = Some(val);
                                author_set = true;
                            }
                            _ => {}
                        }
                    }
                    self.advance();
                    continue;
                }

                if !title_set {
                    if let Some(level) = Self::count_heading_level(line) {
                        if level == 1 {
                            let inline =
                                Self::parse_inline_content(line.trim_start_matches('=').trim());
                            if let Some(NodeType::Text { content }) = inline.first() {
                                module.metadata.title = Some(content.clone());
                            }
                            title_set = true;
                            self.advance();
                            continue;
                        }
                    }
                }

                break;
            } else {
                break;
            }
        }

        self.skip_blank();

        while !self.at_end() {
            if let Some(line) = self.peek() {
                let trimmed = line.trim();

                if trimmed.is_empty() {
                    self.advance();
                    continue;
                }

                if Self::is_comment_line(line) {
                    self.advance();
                    continue;
                }

                if Self::is_attribute_line(line) {
                    if let Some((key, val)) = Self::parse_attribute(line) {
                        match key.as_str() {
                            "subject" | "description" => {
                                module.metadata.subject = Some(val);
                            }
                            "lang" => {
                                module.metadata.language = val;
                            }
                            _ => {}
                        }
                    }
                    self.advance();
                    continue;
                }

                if let Some(level) = Self::count_heading_level(line) {
                    let heading_text = line.trim_start_matches('=').trim();
                    let inline_nodes = Self::parse_inline_content(heading_text);
                    self.advance();

                    let heading_type = match level {
                        1 => NodeType::Chapter,
                        2 => NodeType::Section,
                        3 => NodeType::Subsection,
                        4..=6 => NodeType::Subsubsection,
                        _ => NodeType::Paragraph,
                    };

                    let heading_id = self.gen_id();
                    let mut heading_node = Node::new(heading_id, heading_type).with_parent(doc_id);
                    if let Some(NodeType::Text { content }) = inline_nodes.first() {
                        heading_node.counter = Some(content.clone());
                    }
                    module.body.push(heading_node);
                    if let Some(doc) = module.body.get_mut(doc_id) {
                        doc.add_child(heading_id);
                    }

                    self.skip_blank();
                    continue;
                }

                if let Some(admon_type) = Self::admonition_type(line) {
                    let content = line.trim().splitn(2, ':').nth(1).unwrap_or("").trim();
                    let inline_nodes = Self::parse_inline_content(content);
                    self.advance();

                    let admon_id = self.gen_id();
                    let style_name = admon_type.to_string();
                    module.body.push(
                        Node::new(admon_id, NodeType::BlockQuote)
                            .with_parent(doc_id)
                            .with_style(style_name),
                    );
                    self.add_children(&mut module, admon_id, inline_nodes);
                    if let Some(doc) = module.body.get_mut(doc_id) {
                        doc.add_child(admon_id);
                    }
                    self.skip_blank();
                    continue;
                }

                if Self::is_unordered_list_item(line) {
                    let list_id = self.gen_id();
                    module.body.push(
                        Node::new(
                            list_id,
                            NodeType::List {
                                list_type: ListType::Unordered,
                                ordered: false,
                                start: None,
                            },
                        )
                        .with_parent(doc_id),
                    );

                    while !self.at_end() {
                        if let Some(l) = self.peek() {
                            if !Self::is_unordered_list_item(l) {
                                break;
                            }
                        } else {
                            break;
                        }
                        let Some(item_text) = self.advance() else {
                            break;
                        };
                        let content = item_text.trim_start_matches('*').trim();
                        let inline_nodes = Self::parse_inline_content(content);
                        let item_id = self.gen_id();
                        module
                            .body
                            .push(Node::new(item_id, NodeType::ListItem).with_parent(list_id));
                        self.add_children(&mut module, item_id, inline_nodes);
                        if let Some(list) = module.body.get_mut(list_id) {
                            list.add_child(item_id);
                        }
                    }

                    if let Some(doc) = module.body.get_mut(doc_id) {
                        doc.add_child(list_id);
                    }
                    self.skip_blank();
                    continue;
                }

                if Self::is_ordered_list_item(line) {
                    let list_id = self.gen_id();
                    module.body.push(
                        Node::new(
                            list_id,
                            NodeType::List {
                                list_type: ListType::Ordered,
                                ordered: true,
                                start: None,
                            },
                        )
                        .with_parent(doc_id),
                    );

                    while !self.at_end() {
                        if let Some(l) = self.peek() {
                            if !Self::is_ordered_list_item(l) {
                                break;
                            }
                        } else {
                            break;
                        }
                        let Some(item_text) = self.advance() else {
                            break;
                        };
                        let content = item_text.trim_start_matches('.').trim();
                        let inline_nodes = Self::parse_inline_content(content);
                        let item_id = self.gen_id();
                        module
                            .body
                            .push(Node::new(item_id, NodeType::ListItem).with_parent(list_id));
                        self.add_children(&mut module, item_id, inline_nodes);
                        if let Some(list) = module.body.get_mut(list_id) {
                            list.add_child(item_id);
                        }
                    }

                    if let Some(doc) = module.body.get_mut(doc_id) {
                        doc.add_child(list_id);
                    }
                    self.skip_blank();
                    continue;
                }

                if trimmed.starts_with("[source") || trimmed.starts_with("[code") {
                    let lang = if let Some(start) = trimmed.find(',') {
                        let lang_str = trimmed[start + 1..].trim_end_matches(']').trim();
                        if lang_str.is_empty() {
                            None
                        } else {
                            Some(lang_str.to_string())
                        }
                    } else {
                        None
                    };
                    self.advance();
                    if let Some(delimiter) = self.advance() {
                        if delimiter.trim() == "----" {
                            let mut code_content = String::new();
                            while let Some(code_line) = self.advance() {
                                if code_line.trim() == "----" {
                                    break;
                                }
                                if !code_content.is_empty() {
                                    code_content.push('\n');
                                }
                                code_content.push_str(&code_line);
                            }
                            let cb_id = self.gen_id();
                            module.body.push(
                                Node::new(cb_id, NodeType::CodeBlock { language: lang })
                                    .with_parent(doc_id),
                            );
                            if !code_content.is_empty() {
                                let text_id = self.gen_id();
                                module.body.push(
                                    Node::new(
                                        text_id,
                                        NodeType::Text {
                                            content: code_content,
                                        },
                                    )
                                    .with_parent(cb_id),
                                );
                                if let Some(cb) = module.body.get_mut(cb_id) {
                                    cb.add_child(text_id);
                                }
                            }
                            if let Some(doc) = module.body.get_mut(doc_id) {
                                doc.add_child(cb_id);
                            }
                            self.skip_blank();
                            continue;
                        }
                    }
                    continue;
                }

                if trimmed == "----" || trimmed == "...." {
                    let mut literal_content = String::new();
                    let delimiter = trimmed.to_string();
                    self.advance();
                    while let Some(lit_line) = self.advance() {
                        if lit_line.trim() == delimiter {
                            break;
                        }
                        if !literal_content.is_empty() {
                            literal_content.push('\n');
                        }
                        literal_content.push_str(&lit_line);
                    }
                    let cb_id = self.gen_id();
                    module.body.push(
                        Node::new(cb_id, NodeType::CodeBlock { language: None })
                            .with_parent(doc_id),
                    );
                    if !literal_content.is_empty() {
                        let text_id = self.gen_id();
                        module.body.push(
                            Node::new(
                                text_id,
                                NodeType::Text {
                                    content: literal_content,
                                },
                            )
                            .with_parent(cb_id),
                        );
                        if let Some(cb) = module.body.get_mut(cb_id) {
                            cb.add_child(text_id);
                        }
                    }
                    if let Some(doc) = module.body.get_mut(doc_id) {
                        doc.add_child(cb_id);
                    }
                    self.skip_blank();
                    continue;
                }

                if trimmed.starts_with("[quote]") {
                    self.advance();
                    let mut quote_content = String::new();
                    while let Some(ql) = self.peek() {
                        let qt = ql.trim();
                        if qt.is_empty() || qt.starts_with("----") || qt.starts_with('[') {
                            break;
                        }
                        if !quote_content.is_empty() {
                            quote_content.push(' ');
                        }
                        quote_content.push_str(qt);
                        self.advance();
                    }
                    let inline_nodes = Self::parse_inline_content(&quote_content);
                    let bq_id = self.gen_id();
                    module
                        .body
                        .push(Node::new(bq_id, NodeType::BlockQuote).with_parent(doc_id));
                    self.add_children(&mut module, bq_id, inline_nodes);
                    if let Some(doc) = module.body.get_mut(doc_id) {
                        doc.add_child(bq_id);
                    }
                    self.skip_blank();
                    continue;
                }

                if Self::is_table_delimiter(line) {
                    self.advance();
                    let mut rows: Vec<Vec<String>> = Vec::new();
                    let mut num_cols = 0usize;

                    while let Some(tl) = self.peek() {
                        let tt = tl.trim();
                        if Self::is_table_end(tt) {
                            self.advance();
                            break;
                        }
                        if tt.is_empty() {
                            self.advance();
                            continue;
                        }
                        if tt.starts_with('|') {
                            let cells: Vec<String> = tt
                                .trim_start_matches('|')
                                .split('|')
                                .map(|c| c.trim().to_string())
                                .filter(|c| !c.is_empty())
                                .collect();
                            num_cols = num_cols.max(cells.len());
                            if !cells.is_empty() {
                                rows.push(cells);
                            }
                        }
                        self.advance();
                    }

                    if !rows.is_empty() {
                        let table_id = self.gen_id();
                        let col_specs: Vec<ColSpec> = (0..num_cols)
                            .map(|_| ColSpec {
                                align: ColumnAlign::Left,
                                width: None,
                            })
                            .collect();
                        module.body.push(
                            Node::new(
                                table_id,
                                NodeType::Table {
                                    col_specs,
                                    num_cols,
                                },
                            )
                            .with_parent(doc_id),
                        );

                        for (row_idx, row) in rows.iter().enumerate() {
                            let is_header = row_idx == 0;
                            let row_id = self.gen_id();
                            module.body.push(
                                Node::new(row_id, NodeType::TableRow { is_header })
                                    .with_parent(table_id),
                            );
                            if let Some(tbl) = module.body.get_mut(table_id) {
                                tbl.add_child(row_id);
                            }

                            for cell_text in row {
                                let tc_id = self.gen_id();
                                module.body.push(
                                    Node::new(
                                        tc_id,
                                        NodeType::TableCell {
                                            colspan: 1,
                                            rowspan: 1,
                                        },
                                    )
                                    .with_parent(row_id),
                                );
                                if let Some(row_node) = module.body.get_mut(row_id) {
                                    row_node.add_child(tc_id);
                                }
                                let text_id = self.gen_id();
                                module.body.push(
                                    Node::new(
                                        text_id,
                                        NodeType::Text {
                                            content: cell_text.clone(),
                                        },
                                    )
                                    .with_parent(tc_id),
                                );
                                if let Some(tc) = module.body.get_mut(tc_id) {
                                    tc.add_child(text_id);
                                }
                            }
                        }

                        if let Some(doc) = module.body.get_mut(doc_id) {
                            doc.add_child(table_id);
                        }
                    }
                    self.skip_blank();
                    continue;
                }

                let mut para_text = String::new();
                while let Some(pl) = self.peek() {
                    let pt = pl.trim();
                    if pt.is_empty() {
                        break;
                    }
                    if Self::count_heading_level(pl).is_some()
                        || Self::is_unordered_list_item(pl)
                        || Self::is_ordered_list_item(pl)
                        || Self::is_comment_line(pl)
                        || Self::is_attribute_line(pl)
                        || pt == "----"
                        || pt == "...."
                        || Self::is_table_delimiter(pl)
                        || pt.starts_with("[source")
                        || pt.starts_with("[code")
                        || pt.starts_with("[quote]")
                        || Self::admonition_type(pl).is_some()
                    {
                        break;
                    }
                    if !para_text.is_empty() {
                        para_text.push(' ');
                    }
                    para_text.push_str(pt);
                    self.advance();
                }

                let para_text = para_text.trim().to_string();
                if !para_text.is_empty() {
                    let inline_nodes = Self::parse_inline_content(&para_text);
                    let para_id = self.gen_id();
                    module
                        .body
                        .push(Node::new(para_id, NodeType::Paragraph).with_parent(doc_id));
                    self.add_children(&mut module, para_id, inline_nodes);
                    if let Some(doc) = module.body.get_mut(doc_id) {
                        doc.add_child(para_id);
                    }
                }
            } else {
                break;
            }
        }

        module
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldir_ir::sir::v2::nodes::NodeType;

    fn find_nodes(module: &SIRModuleV2, pred: impl Fn(&NodeType) -> bool) -> Vec<&Node> {
        module.body.find_by_type(pred)
    }

    #[test]
    fn test_title() {
        let module = parse_asciidoc("= Document Title\n");
        assert_eq!(module.metadata.title.as_deref(), Some("Document Title"));
    }

    #[test]
    fn test_heading_section() {
        let module = parse_asciidoc("== Section Title\n");
        let sections = find_nodes(&module, |nt| matches!(nt, NodeType::Section));
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].counter.as_deref(), Some("Section Title"));
    }

    #[test]
    fn test_heading_subsection() {
        let module = parse_asciidoc("=== Subsection Title\n");
        let subs = find_nodes(&module, |nt| matches!(nt, NodeType::Subsection));
        assert_eq!(subs.len(), 1);
    }

    #[test]
    fn test_bold() {
        let module = parse_asciidoc("This is *bold* text.\n");
        let bolds = find_nodes(&module, |nt| matches!(nt, NodeType::Bold));
        assert_eq!(bolds.len(), 1);
    }

    #[test]
    fn test_italic() {
        let module = parse_asciidoc("This is _italic_ text.\n");
        let italics = find_nodes(&module, |nt| matches!(nt, NodeType::Italic));
        assert_eq!(italics.len(), 1);
    }

    #[test]
    fn test_mono() {
        let module = parse_asciidoc("This is `code` text.\n");
        let monos = find_nodes(&module, |nt| matches!(nt, NodeType::Mono));
        assert_eq!(monos.len(), 1);
    }

    #[test]
    fn test_paragraph() {
        let module = parse_asciidoc("Hello world.\n");
        let paras = find_nodes(&module, |nt| matches!(nt, NodeType::Paragraph));
        assert_eq!(paras.len(), 1);
    }

    #[test]
    fn test_unordered_list() {
        let module = parse_asciidoc("* Item one\n* Item two\n");
        let lists = find_nodes(&module, |nt| {
            matches!(nt, NodeType::List { ordered: false, .. })
        });
        assert_eq!(lists.len(), 1);
        let items = find_nodes(&module, |nt| matches!(nt, NodeType::ListItem));
        assert!(items.len() >= 2);
    }

    #[test]
    fn test_ordered_list() {
        let module = parse_asciidoc(". First item\n. Second item\n");
        let lists = find_nodes(&module, |nt| {
            matches!(nt, NodeType::List { ordered: true, .. })
        });
        assert_eq!(lists.len(), 1);
    }

    #[test]
    fn test_code_block() {
        let module = parse_asciidoc("[source,rust]\n----\nfn main() {}\n----\n");
        let cbs = find_nodes(
            &module,
            |nt| matches!(nt, NodeType::CodeBlock { language: Some(lang) } if lang == "rust"),
        );
        assert_eq!(cbs.len(), 1);
    }

    #[test]
    fn test_literal_block() {
        let module = parse_asciidoc("----\nSome literal text.\n----\n");
        let cbs = find_nodes(&module, |nt| {
            matches!(nt, NodeType::CodeBlock { language: None })
        });
        assert_eq!(cbs.len(), 1);
    }

    #[test]
    fn test_block_quote() {
        let module = parse_asciidoc("[quote]\nA block quote.\n");
        let quotes = find_nodes(&module, |nt| matches!(nt, NodeType::BlockQuote));
        assert_eq!(quotes.len(), 1);
    }

    #[test]
    fn test_admonition_note() {
        let module = parse_asciidoc("NOTE: This is a note.\n");
        let quotes = find_nodes(&module, |nt| matches!(nt, NodeType::BlockQuote));
        assert_eq!(quotes.len(), 1);
        assert_eq!(quotes[0].style.as_deref(), Some("NOTE"));
    }

    #[test]
    fn test_admonition_tip() {
        let module = parse_asciidoc("TIP: Helpful tip here.\n");
        let quotes = find_nodes(&module, |nt| matches!(nt, NodeType::BlockQuote { .. }));
        assert_eq!(quotes.len(), 1);
        assert_eq!(quotes[0].style.as_deref(), Some("TIP"));
    }

    #[test]
    fn test_table() {
        let module = parse_asciidoc("|===\n| Header 1 | Header 2\n| Cell 1 | Cell 2\n|===\n");
        let tables = find_nodes(&module, |nt| matches!(nt, NodeType::Table { .. }));
        assert_eq!(tables.len(), 1);
        if let NodeType::Table { num_cols, .. } = &tables[0].node_type {
            assert_eq!(*num_cols, 2);
        }
        let rows = find_nodes(&module, |nt| matches!(nt, NodeType::TableRow { .. }));
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn test_math_inline() {
        let module = parse_asciidoc("The equation $x^2 + y^2$ is here.\n");
        let math = find_nodes(&module, |nt| matches!(nt, NodeType::MathInline { .. }));
        assert_eq!(math.len(), 1);
        if let NodeType::MathInline { content } = &math[0].node_type {
            assert!(content.contains("x^2"));
        }
    }

    #[test]
    fn test_footnote() {
        let module = parse_asciidoc("Some text.footnote:[A footnote note.]\n");
        let fns = find_nodes(&module, |nt| matches!(nt, NodeType::Footnote { .. }));
        assert_eq!(fns.len(), 1);
    }

    #[test]
    fn test_comment_stripped() {
        let module = parse_asciidoc("// This is a comment\nHello\n");
        let texts = find_nodes(&module, |nt| matches!(nt, NodeType::Text { .. }));
        assert!(!texts.iter().any(
            |n| matches!(&n.node_type, NodeType::Text { content } if content.contains("comment"))
        ));
    }

    #[test]
    fn test_attribute() {
        let module = parse_asciidoc(":lang: en\n:subject: My doc\n\nHello\n");
        assert_eq!(module.metadata.language, "en");
        assert_eq!(module.metadata.subject.as_deref(), Some("My doc"));
    }

    #[test]
    fn test_link_inline() {
        let module = parse_asciidoc("Visit https://example.com[Link text] now.\n");
        let links = find_nodes(&module, |nt| matches!(nt, NodeType::Link { .. }));
        assert_eq!(links.len(), 1);
        if let NodeType::Link { url, title, .. } = &links[0].node_type {
            assert!(url.contains("example.com"));
            assert_eq!(title.as_deref(), Some("Link text"));
        }
    }

    #[test]
    fn test_image() {
        let module = parse_asciidoc("image::photo.png[Alt text]\n");
        let imgs = find_nodes(&module, |nt| matches!(nt, NodeType::Image { .. }));
        assert_eq!(imgs.len(), 1);
        if let NodeType::Image { source, alt, .. } = &imgs[0].node_type {
            assert_eq!(source, "photo.png");
            assert_eq!(alt, "Alt text");
        }
    }

    #[test]
    fn test_empty_document() {
        let module = parse_asciidoc("");
        let docs = find_nodes(&module, |nt| matches!(nt, NodeType::Document));
        assert_eq!(docs.len(), 1);
    }

    #[test]
    fn test_source_format() {
        let module = parse_asciidoc("= Title\n");
        assert_eq!(module.header.source_format.as_deref(), Some("asciidoc"));
    }

    #[test]
    fn test_full_document() {
        let input = r#"= Document Title
:subject: Test doc
:lang: en

== Section One

This is a paragraph with *bold* and _italic_ and `code`.

. Ordered item one
. Ordered item two

* Unordered A
* Unordered B

[source,rust]
----
fn main() { println!("hello"); }
----

NOTE: This is a note.

A link: https://example.com[Click here]

Inline math: $x^2 + y^2$

footnote:[A footnote reference.]

|===
| H1 | H2
| C1 | C2
|===
"#;
        let module = parse_asciidoc(input);
        assert_eq!(module.metadata.title.as_deref(), Some("Document Title"));
        assert_eq!(module.metadata.language, "en");
        let sections = find_nodes(&module, |nt| matches!(nt, NodeType::Section));
        assert_eq!(sections.len(), 1);
        let bolds = find_nodes(&module, |nt| matches!(nt, NodeType::Bold));
        assert_eq!(bolds.len(), 1);
        let italics = find_nodes(&module, |nt| matches!(nt, NodeType::Italic));
        assert_eq!(italics.len(), 1);
        let monos = find_nodes(&module, |nt| matches!(nt, NodeType::Mono));
        assert!(monos.len() >= 1);
        let math = find_nodes(&module, |nt| matches!(nt, NodeType::MathInline { .. }));
        assert_eq!(math.len(), 1);
        let footnotes = find_nodes(&module, |nt| matches!(nt, NodeType::Footnote { .. }));
        assert_eq!(footnotes.len(), 1);
        let tables = find_nodes(&module, |nt| matches!(nt, NodeType::Table { .. }));
        assert_eq!(tables.len(), 1);
    }
}
