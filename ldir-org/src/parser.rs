//! Recursive descent parser for Org-mode → S-IR v2.

use ldir_ir::sir::v2::SIRModuleV2;
use ldir_ir::sir::v2::nodes::{ColSpec, ColumnAlign, FloatPlacement, ListType, Node, NodeType};

pub fn parse_org(text: &str) -> SIRModuleV2 {
    let lines: Vec<&str> = text.lines().collect();
    let mut parser = OrgParser::new(lines);
    parser.parse_document()
}

struct OrgParser {
    lines: Vec<String>,
    pos: usize,
    next_id: u32,
    _in_paragraph: bool,
}

impl OrgParser {
    fn new(lines: Vec<&str>) -> Self {
        let lines: Vec<String> = lines.into_iter().map(|l| l.to_string()).collect();
        Self {
            lines,
            pos: 0,
            next_id: 1,
            _in_paragraph: false,
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

    fn count_heading_stars(line: &str) -> Option<u8> {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || !trimmed.starts_with('*') {
            return None;
        }
        let mut count = 0u8;
        for ch in trimmed.chars() {
            if ch == '*' {
                count += 1;
            } else {
                break;
            }
        }
        let rest = &trimmed[count as usize..];
        if count >= 1 && (rest.starts_with(' ') || rest.is_empty()) {
            Some(count)
        } else {
            None
        }
    }

    fn is_metadata_line(line: &str) -> bool {
        let trimmed = line.trim();
        if !trimmed.starts_with("#+") {
            return false;
        }
        let rest = &trimmed[2..];
        let key_end = rest.find(':');
        if let Some(end) = key_end {
            if end == 0 {
                return false;
            }
            let key = &rest[..end];
            key.chars().all(|c| c.is_ascii_uppercase() || c == '_')
        } else {
            false
        }
    }

    fn parse_metadata_key_value(line: &str) -> Option<(String, String)> {
        let trimmed = line.trim();
        let rest = &trimmed[2..];
        if let Some(colon_pos) = rest.find(':') {
            let key = rest[..colon_pos].to_string();
            let val = rest[colon_pos + 1..].trim().to_string();
            if !key.is_empty() {
                return Some((key, val));
            }
        }
        None
    }

    fn is_comment_line(line: &str) -> bool {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return false;
        }
        if Self::is_metadata_line(line) {
            return false;
        }
        trimmed.starts_with("# ")
    }

    fn is_unordered_list_item(line: &str) -> bool {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("- ") && !trimmed.starts_with("+ ") {
            return false;
        }
        if Self::count_heading_stars(line).is_some() {
            return false;
        }
        true
    }

    fn is_ordered_list_item(line: &str) -> bool {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            return false;
        }
        let mut chars = trimmed.chars();
        if !chars.next().is_some_and(|c| c.is_ascii_digit()) {
            return false;
        }
        if !chars.next().is_some_and(|c| c == '.' || c == ')') {
            return false;
        }
        chars.next() == Some(' ')
    }

    fn is_horizontal_rule(line: &str) -> bool {
        let trimmed = line.trim();
        if trimmed.len() < 5 {
            return false;
        }
        let Some(first) = trimmed.chars().next() else {
            return false;
        };
        matches!(first, '-' | '_' | '*') && trimmed.chars().all(|c| c == first)
    }

    fn is_table_line(line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with('|') && trimmed.contains('|') && trimmed.len() > 1
    }

    fn is_table_separator(line: &str) -> bool {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
            return false;
        }
        let inner = trimmed.trim_start_matches('|').trim_end_matches('|');
        inner.chars().all(|c| c == '-' || c == '+') && !inner.is_empty()
    }

    fn is_block_start(line: &str) -> Option<(String, Option<String>)> {
        let trimmed = line.trim();
        if !trimmed.starts_with("#+BEGIN_") {
            return None;
        }
        let rest = &trimmed[8..];
        let space_pos = rest.find(' ');
        let block_type = if let Some(pos) = space_pos {
            rest[..pos].to_string()
        } else {
            rest.to_string()
        };

        let block_arg = space_pos.map(|pos| rest[pos + 1..].trim().to_string());
        Some((block_type, block_arg))
    }

    fn is_block_end(line: &str, block_type: &str) -> bool {
        let trimmed = line.trim();
        trimmed == format!("#+END_{}", block_type)
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
                nodes.push(NodeType::Text {
                    content: chars[i].to_string(),
                });
                i += 1;
                continue;
            }

            if ch == '*' && i > 0 {
                let prev = chars[i - 1];
                if prev.is_whitespace() || prev == '(' {
                    let end = Self::find_closing_inline(&chars, i + 1, '*');
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
            }

            if ch == '/' {
                let end = Self::find_closing_inline(&chars, i + 1, '/');
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

            if ch == '~' {
                let end = Self::find_closing_inline(&chars, i + 1, '~');
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

            if ch == '+' {
                let end = Self::find_closing_inline(&chars, i + 1, '+');
                if let Some(end_idx) = end {
                    let content: String = chars[i + 1..end_idx].iter().collect();
                    let trimmed = content.trim().to_string();
                    if !trimmed.is_empty() {
                        nodes.push(NodeType::Strikethrough);
                        nodes.push(NodeType::Text { content: trimmed });
                    }
                    i = end_idx + 1;
                    continue;
                }
            }

            if ch == '_' {
                let end = Self::find_closing_inline(&chars, i + 1, '_');
                if let Some(end_idx) = end {
                    let content: String = chars[i + 1..end_idx].iter().collect();
                    let trimmed = content.trim().to_string();
                    if !trimmed.is_empty() {
                        nodes.push(NodeType::Underline);
                        nodes.push(NodeType::Text { content: trimmed });
                    }
                    i = end_idx + 1;
                    continue;
                }
            }

            if ch == '[' && i + 1 < len && chars[i + 1] == '[' {
                let start = i + 2;
                let mut close1 = None;
                for j in start..len {
                    if chars[j] == ']' && j + 1 < len && chars[j + 1] == ']' {
                        close1 = Some(j);
                        break;
                    }
                }
                if let Some(close_pos) = close1 {
                    let link_content: String = chars[start..close_pos].iter().collect();
                    let parts: Vec<&str> = link_content.splitn(2, "][").collect();
                    if parts.len() == 2 {
                        let target = parts[0].trim();
                        let desc = parts[1].trim();
                        if target.starts_with("file:") {
                            let path = target.trim_start_matches("file:");
                            nodes.push(NodeType::Image {
                                source: path.to_string(),
                                alt: desc.to_string(),
                                width: None,
                                height: None,
                                placement: FloatPlacement::Here,
                            });
                        } else {
                            nodes.push(NodeType::Link {
                                url: target.to_string(),
                                title: Some(desc.to_string()),
                            });
                        }
                    } else if parts.len() == 1 {
                        let target = parts[0].trim();
                        if target.starts_with("file:") {
                            let path = target.trim_start_matches("file:");
                            nodes.push(NodeType::Image {
                                source: path.to_string(),
                                alt: path.to_string(),
                                width: None,
                                height: None,
                                placement: FloatPlacement::Here,
                            });
                        } else {
                            nodes.push(NodeType::Link {
                                url: target.to_string(),
                                title: None,
                            });
                        }
                    }
                    i = close_pos + 2;
                    continue;
                }
            }

            if ch == '$' {
                let end = Self::find_closing_inline(&chars, i + 1, '$');
                if let Some(end_idx) = end {
                    let content: String = chars[i + 1..end_idx].iter().collect();
                    if !content.is_empty() {
                        nodes.push(NodeType::MathInline { content });
                    }
                    i = end_idx + 1;
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
                if matches!(c, '*' | '/' | '~' | '+' | '_' | '$' | '[' | '\\' | '\n') {
                    break;
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

    fn find_closing_inline(chars: &[char], start: usize, delim: char) -> Option<usize> {
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
        let mut module = SIRModuleV2::from_source("org", "<input>");
        let doc_id = self.gen_id();
        module.body.push(Node::new(doc_id, NodeType::Document));

        self.skip_blank();

        while !self.at_end() {
            if let Some(line) = self.peek() {
                let trimmed = line.trim();

                if trimmed.is_empty() {
                    self.advance();
                    continue;
                }

                if Self::is_metadata_line(line) {
                    if let Some((key, val)) = Self::parse_metadata_key_value(line) {
                        match key.as_str() {
                            "TITLE" => {
                                module.metadata.title = Some(val);
                            }
                            "AUTHOR" => {
                                module.metadata.author = Some(val);
                            }
                            "LANGUAGE" => {
                                module.metadata.language = val;
                            }
                            "DESCRIPTION" | "SUBJECT" => {
                                module.metadata.subject = Some(val);
                            }
                            _ => {}
                        }
                    }
                    self.advance();
                    continue;
                }

                if Self::is_comment_line(line) {
                    self.advance();
                    continue;
                }

                if Self::is_horizontal_rule(line) {
                    let hr_id = self.gen_id();
                    module
                        .body
                        .push(Node::new(hr_id, NodeType::ThematicBreak).with_parent(doc_id));
                    if let Some(doc) = module.body.get_mut(doc_id) {
                        doc.add_child(hr_id);
                    }
                    self.advance();
                    self.skip_blank();
                    continue;
                }

                if let Some(level) = Self::count_heading_stars(line) {
                    let heading_text = line.trim_start_matches('*').trim();
                    let todo_keyword = if heading_text.starts_with("TODO ") {
                        Some("TODO")
                    } else if heading_text.starts_with("DONE ") {
                        Some("DONE")
                    } else {
                        None
                    };
                    let text_content = if let Some(kw) = todo_keyword {
                        heading_text[kw.len()..].trim()
                    } else {
                        heading_text
                    };
                    let inline_nodes = Self::parse_inline_content(text_content);
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
                    if let Some(kw) = todo_keyword {
                        heading_node.style = Some(kw.to_string());
                    }
                    module.body.push(heading_node);
                    if let Some(doc) = module.body.get_mut(doc_id) {
                        doc.add_child(heading_id);
                    }

                    self.skip_blank();
                    continue;
                }

                if let Some((block_type, block_arg)) = Self::is_block_start(line) {
                    self.advance();
                    let mut block_content = String::new();
                    while let Some(bl) = self.peek() {
                        if Self::is_block_end(bl, &block_type) {
                            self.advance();
                            break;
                        }
                        if !block_content.is_empty() {
                            block_content.push('\n');
                        }
                        block_content.push_str(bl.trim());
                        self.advance();
                    }

                    match block_type.as_str() {
                        "QUOTE" => {
                            let inline_nodes = Self::parse_inline_content(&block_content);
                            let bq_id = self.gen_id();
                            module
                                .body
                                .push(Node::new(bq_id, NodeType::BlockQuote).with_parent(doc_id));
                            self.add_children(&mut module, bq_id, inline_nodes);
                            if let Some(doc) = module.body.get_mut(doc_id) {
                                doc.add_child(bq_id);
                            }
                        }
                        "SRC" => {
                            let lang = block_arg.clone();
                            let cb_id = self.gen_id();
                            module.body.push(
                                Node::new(
                                    cb_id,
                                    NodeType::CodeBlock {
                                        language: lang,
                                        content: String::new(),
                                    },
                                )
                                .with_parent(doc_id),
                            );
                            if !block_content.is_empty() {
                                let text_id = self.gen_id();
                                module.body.push(
                                    Node::new(
                                        text_id,
                                        NodeType::Text {
                                            content: block_content,
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
                        }
                        "EXAMPLE" => {
                            let cb_id = self.gen_id();
                            module.body.push(
                                Node::new(
                                    cb_id,
                                    NodeType::CodeBlock {
                                        language: None,
                                        content: String::new(),
                                    },
                                )
                                .with_parent(doc_id),
                            );
                            if !block_content.is_empty() {
                                let text_id = self.gen_id();
                                module.body.push(
                                    Node::new(
                                        text_id,
                                        NodeType::Text {
                                            content: block_content,
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
                        }
                        "EXPORT" => {
                            let fmt = block_arg.as_deref().unwrap_or("");
                            let inline_nodes = Self::parse_inline_content(&block_content);
                            let group_id = self.gen_id();
                            module.body.push(
                                Node::new(group_id, NodeType::Group)
                                    .with_parent(doc_id)
                                    .with_style(format!("export:{}", fmt)),
                            );
                            self.add_children(&mut module, group_id, inline_nodes);
                            if let Some(doc) = module.body.get_mut(doc_id) {
                                doc.add_child(group_id);
                            }
                        }
                        _ => {
                            let inline_nodes = Self::parse_inline_content(&block_content);
                            let group_id = self.gen_id();
                            module
                                .body
                                .push(Node::new(group_id, NodeType::Group).with_parent(doc_id));
                            self.add_children(&mut module, group_id, inline_nodes);
                            if let Some(doc) = module.body.get_mut(doc_id) {
                                doc.add_child(group_id);
                            }
                        }
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
                        let content = item_text
                            .trim_start_matches('-')
                            .trim_start_matches('+')
                            .trim();
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
                        let trimmed = item_text.trim_start();
                        let dot_pos = trimmed.find(['.', ')']).unwrap_or(0);
                        let content = trimmed[dot_pos + 1..].trim();
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

                if Self::is_table_line(line) {
                    let mut rows: Vec<Vec<String>> = Vec::new();
                    let mut num_cols = 0usize;
                    let mut has_separator = false;

                    while let Some(tl) = self.peek() {
                        let tt = tl.trim();
                        if tt.is_empty() {
                            break;
                        }
                        if !Self::is_table_line(tt) && !Self::is_table_separator(tt) {
                            break;
                        }
                        if Self::is_table_separator(tt) {
                            has_separator = true;
                            self.advance();
                            continue;
                        }
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
                                    caption: None,
                                    column_widths: Vec::new(),
                                    header_row: false,
                                },
                            )
                            .with_parent(doc_id),
                        );

                        for (row_idx, row) in rows.iter().enumerate() {
                            let is_header = row_idx == 0 && has_separator;
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
                    if Self::count_heading_stars(pl).is_some()
                        || Self::is_metadata_line(pl)
                        || Self::is_comment_line(pl)
                        || Self::is_horizontal_rule(pl)
                        || Self::is_unordered_list_item(pl)
                        || Self::is_ordered_list_item(pl)
                        || Self::is_table_line(pl)
                        || Self::is_block_start(pl).is_some()
                        || pt.starts_with("[fn:")
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
    fn test_metadata_title() {
        let module = parse_org("#+TITLE: My Document\n");
        assert_eq!(module.metadata.title.as_deref(), Some("My Document"));
    }

    #[test]
    fn test_metadata_author() {
        let module = parse_org("#+AUTHOR: Jane Doe\n");
        assert_eq!(module.metadata.author.as_deref(), Some("Jane Doe"));
    }

    #[test]
    fn test_metadata_language() {
        let module = parse_org("#+LANGUAGE: en\n");
        assert_eq!(module.metadata.language, "en");
    }

    #[test]
    fn test_heading_level_1() {
        let module = parse_org("* Heading Level 1\n");
        let chapters = find_nodes(&module, |nt| matches!(nt, NodeType::Chapter));
        assert_eq!(chapters.len(), 1);
        assert_eq!(chapters[0].counter.as_deref(), Some("Heading Level 1"));
    }

    #[test]
    fn test_heading_level_2() {
        let module = parse_org("** Heading Level 2\n");
        let sections = find_nodes(&module, |nt| matches!(nt, NodeType::Section));
        assert_eq!(sections.len(), 1);
    }

    #[test]
    fn test_heading_level_3() {
        let module = parse_org("*** Heading Level 3\n");
        let subs = find_nodes(&module, |nt| matches!(nt, NodeType::Subsection));
        assert_eq!(subs.len(), 1);
    }

    #[test]
    fn test_bold() {
        let module = parse_org("This is *bold* text.\n");
        let bolds = find_nodes(&module, |nt| matches!(nt, NodeType::Bold));
        assert_eq!(bolds.len(), 1);
    }

    #[test]
    fn test_italic() {
        let module = parse_org("This is /italic/ text.\n");
        let italics = find_nodes(&module, |nt| matches!(nt, NodeType::Italic));
        assert_eq!(italics.len(), 1);
    }

    #[test]
    fn test_code_inline() {
        let module = parse_org("This is ~code~ text.\n");
        let monos = find_nodes(&module, |nt| matches!(nt, NodeType::Mono));
        assert_eq!(monos.len(), 1);
    }

    #[test]
    fn test_strikethrough() {
        let module = parse_org("This is +strikethrough+ text.\n");
        let strikes = find_nodes(&module, |nt| matches!(nt, NodeType::Strikethrough));
        assert_eq!(strikes.len(), 1);
    }

    #[test]
    fn test_underline() {
        let module = parse_org("This is _underline_ text.\n");
        let underlines = find_nodes(&module, |nt| matches!(nt, NodeType::Underline));
        assert_eq!(underlines.len(), 1);
    }

    #[test]
    fn test_paragraph() {
        let module = parse_org("Hello world.\n");
        let paras = find_nodes(&module, |nt| matches!(nt, NodeType::Paragraph));
        assert_eq!(paras.len(), 1);
    }

    #[test]
    fn test_unordered_list() {
        let module = parse_org("- Item one\n- Item two\n");
        let lists = find_nodes(&module, |nt| {
            matches!(nt, NodeType::List { ordered: false, .. })
        });
        assert_eq!(lists.len(), 1);
        let items = find_nodes(&module, |nt| matches!(nt, NodeType::ListItem));
        assert!(items.len() >= 2);
    }

    #[test]
    fn test_ordered_list() {
        let module = parse_org("1. First item\n2. Second item\n");
        let lists = find_nodes(&module, |nt| {
            matches!(nt, NodeType::List { ordered: true, .. })
        });
        assert_eq!(lists.len(), 1);
    }

    #[test]
    fn test_source_block() {
        let module = parse_org("#+BEGIN_SRC rust\nfn main() {}\n#+END_SRC\n");
        let cbs = find_nodes(
            &module,
            |nt| matches!(nt, NodeType::CodeBlock { language: Some(lang), .. } if lang == "rust"),
        );
        assert_eq!(cbs.len(), 1);
    }

    #[test]
    fn test_quote_block() {
        let module = parse_org("#+BEGIN_QUOTE\nA block quote.\n#+END_QUOTE\n");
        let quotes = find_nodes(&module, |nt| matches!(nt, NodeType::BlockQuote));
        assert_eq!(quotes.len(), 1);
    }

    #[test]
    fn test_table() {
        let module = parse_org(
            "| Header 1 | Header 2 |\n|----------+----------|\n| Cell 1   | Cell 2   |\n",
        );
        let tables = find_nodes(&module, |nt| matches!(nt, NodeType::Table { .. }));
        assert_eq!(tables.len(), 1);
        if let NodeType::Table { num_cols, .. } = &tables[0].node_type {
            assert_eq!(*num_cols, 2);
        }
        let header_rows = find_nodes(&module, |nt| {
            matches!(nt, NodeType::TableRow { is_header: true })
        });
        assert_eq!(header_rows.len(), 1);
    }

    #[test]
    fn test_horizontal_rule() {
        let module = parse_org("-----\n");
        let hrs = find_nodes(&module, |nt| matches!(nt, NodeType::ThematicBreak));
        assert_eq!(hrs.len(), 1);
    }

    #[test]
    fn test_todo_item() {
        let module = parse_org("* TODO A task item\n");
        let chapters = find_nodes(&module, |nt| matches!(nt, NodeType::Chapter));
        assert_eq!(chapters.len(), 1);
        assert_eq!(chapters[0].style.as_deref(), Some("TODO"));
    }

    #[test]
    fn test_done_item() {
        let module = parse_org("* DONE A completed item\n");
        let chapters = find_nodes(&module, |nt| matches!(nt, NodeType::Chapter));
        assert_eq!(chapters.len(), 1);
        assert_eq!(chapters[0].style.as_deref(), Some("DONE"));
    }

    #[test]
    fn test_link() {
        let module = parse_org("[[https://example.com][Link text]]\n");
        let links = find_nodes(&module, |nt| matches!(nt, NodeType::Link { .. }));
        assert_eq!(links.len(), 1);
        if let NodeType::Link { url, title, .. } = &links[0].node_type {
            assert_eq!(url, "https://example.com");
            assert_eq!(title.as_deref(), Some("Link text"));
        }
    }

    #[test]
    fn test_image() {
        let module = parse_org("[[file:photo.png][Alt text]]\n");
        let imgs = find_nodes(&module, |nt| matches!(nt, NodeType::Image { .. }));
        assert_eq!(imgs.len(), 1);
        if let NodeType::Image { source, alt, .. } = &imgs[0].node_type {
            assert_eq!(source, "photo.png");
            assert_eq!(alt, "Alt text");
        }
    }

    #[test]
    fn test_comment_stripped() {
        let module = parse_org("# This is a comment\nHello\n");
        let texts = find_nodes(&module, |nt| matches!(nt, NodeType::Text { .. }));
        assert!(!texts.iter().any(
            |n| matches!(&n.node_type, NodeType::Text { content } if content.contains("comment"))
        ));
        assert!(texts.iter().any(
            |n| matches!(&n.node_type, NodeType::Text { content } if content.contains("Hello"))
        ));
    }

    #[test]
    fn test_export_block() {
        let module = parse_org("#+BEGIN_EXPORT html\n<h1>Raw HTML</h1>\n#+END_EXPORT\n");
        let groups = find_nodes(&module, |nt| matches!(nt, NodeType::Group { .. }));
        assert_eq!(groups.len(), 1);
        assert!(
            groups[0]
                .style
                .as_deref()
                .unwrap_or("")
                .starts_with("export:")
        );
    }

    #[test]
    fn test_empty_document() {
        let module = parse_org("");
        let docs = find_nodes(&module, |nt| matches!(nt, NodeType::Document));
        assert_eq!(docs.len(), 1);
    }

    #[test]
    fn test_source_format() {
        let module = parse_org("#+TITLE: Test\n");
        assert_eq!(module.header.source_format.as_deref(), Some("org"));
    }

    #[test]
    fn test_full_document() {
        let input = r#"#+TITLE: Document Title
#+AUTHOR: Author Name
#+LANGUAGE: en

* Heading Level 1

This is a paragraph with *bold*, /italic/, ~code~, +strike+, and _underline_.

- Unordered item
- Another item

1. Ordered item
2. Another item

#+BEGIN_QUOTE
A block quote.
#+END_QUOTE

#+BEGIN_SRC rust
fn main() {}
#+END_SRC

| Header 1 | Header 2 |
|----------+----------|
| Cell 1   | Cell 2   |

-----

* TODO A task item
* DONE A completed item

[[https://example.com][Link text]]
[[file:photo.png][Alt text]]
"#;
        let module = parse_org(input);
        assert_eq!(module.metadata.title.as_deref(), Some("Document Title"));
        assert_eq!(module.metadata.author.as_deref(), Some("Author Name"));
        let chapters = find_nodes(&module, |nt| matches!(nt, NodeType::Chapter));
        assert_eq!(chapters.len(), 3);
        let bolds = find_nodes(&module, |nt| matches!(nt, NodeType::Bold));
        assert_eq!(bolds.len(), 1);
        let italics = find_nodes(&module, |nt| matches!(nt, NodeType::Italic));
        assert_eq!(italics.len(), 1);
        let monos = find_nodes(&module, |nt| matches!(nt, NodeType::Mono));
        assert_eq!(monos.len(), 1);
        let strikes = find_nodes(&module, |nt| matches!(nt, NodeType::Strikethrough));
        assert_eq!(strikes.len(), 1);
        let underlines = find_nodes(&module, |nt| matches!(nt, NodeType::Underline));
        assert_eq!(underlines.len(), 1);
        let quotes = find_nodes(&module, |nt| matches!(nt, NodeType::BlockQuote));
        assert_eq!(quotes.len(), 1);
        let cbs = find_nodes(&module, |nt| matches!(nt, NodeType::CodeBlock { .. }));
        assert_eq!(cbs.len(), 1);
        let tables = find_nodes(&module, |nt| matches!(nt, NodeType::Table { .. }));
        assert_eq!(tables.len(), 1);
        let hrs = find_nodes(&module, |nt| matches!(nt, NodeType::ThematicBreak));
        assert_eq!(hrs.len(), 1);
        let links = find_nodes(&module, |nt| matches!(nt, NodeType::Link { .. }));
        assert_eq!(links.len(), 1);
        let imgs = find_nodes(&module, |nt| matches!(nt, NodeType::Image { .. }));
        assert_eq!(imgs.len(), 1);
    }
}
