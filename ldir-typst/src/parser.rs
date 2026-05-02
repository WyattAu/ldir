//! Recursive descent parser for Typst → S-IR v2.

use ldir_ir::sir::v2::SIRModuleV2;
use ldir_ir::sir::v2::annotations::LabelCategory;
use ldir_ir::sir::v2::nodes::{ColSpec, ColumnAlign, ListType, Node, NodeType};

pub fn parse_typst(text: &str) -> SIRModuleV2 {
    let tokens = tokenize(text);
    let mut parser = TypstParser::new(tokens);
    parser.parse_document()
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Heading(u8),
    Text(String),
    Star,
    Underscore,
    Backtick,
    Dollar,
    Minus,
    Plus,
    Dot,
    Hash,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LAngle,
    RAngle,
    Colon,
    Comma,
    Equals,
    AtSign,
    Newline,
    Eof,
}

fn tokenize(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let len = chars.len();
    let mut at_line_start = true;

    while i < len {
        let ch = chars[i];

        if at_line_start && ch == '=' {
            let mut count: u8 = 0;
            while i < len && chars[i] == '=' && count < 4 {
                count += 1;
                i += 1;
            }
            if count > 0 {
                tokens.push(Token::Heading(count));
                at_line_start = false;
                continue;
            }
        }

        at_line_start = false;

        match ch {
            '/' if i + 1 < len && chars[i + 1] == '/' => {
                while i < len && chars[i] != '\n' {
                    i += 1;
                }
            }
            '*' => {
                tokens.push(Token::Star);
                i += 1;
            }
            '_' => {
                tokens.push(Token::Underscore);
                i += 1;
            }
            '`' => {
                tokens.push(Token::Backtick);
                i += 1;
            }
            '$' => {
                tokens.push(Token::Dollar);
                i += 1;
            }
            '-' => {
                tokens.push(Token::Minus);
                i += 1;
            }
            '+' => {
                tokens.push(Token::Plus);
                i += 1;
            }
            '.' => {
                tokens.push(Token::Dot);
                i += 1;
            }
            '#' => {
                tokens.push(Token::Hash);
                i += 1;
            }
            '(' => {
                tokens.push(Token::LParen);
                i += 1;
            }
            ')' => {
                tokens.push(Token::RParen);
                i += 1;
            }
            '[' => {
                tokens.push(Token::LBracket);
                i += 1;
            }
            ']' => {
                tokens.push(Token::RBracket);
                i += 1;
            }
            '<' => {
                tokens.push(Token::LAngle);
                i += 1;
            }
            '>' => {
                tokens.push(Token::RAngle);
                i += 1;
            }
            ':' => {
                tokens.push(Token::Colon);
                i += 1;
            }
            ',' => {
                tokens.push(Token::Comma);
                i += 1;
            }
            '=' => {
                tokens.push(Token::Equals);
                i += 1;
            }
            '@' => {
                tokens.push(Token::AtSign);
                i += 1;
            }
            '\n' => {
                tokens.push(Token::Newline);
                i += 1;
                at_line_start = true;
            }
            c if c.is_whitespace() => {
                i += 1;
            }
            '"' => {
                let mut buf = String::new();
                i += 1;
                while i < len && chars[i] != '"' {
                    buf.push(chars[i]);
                    i += 1;
                }
                if i < len {
                    i += 1;
                }
                if !buf.is_empty() {
                    tokens.push(Token::Text(buf));
                }
            }
            _ => {
                let mut buf = String::new();
                while i < len {
                    let c = chars[i];
                    if c.is_whitespace()
                        || matches!(
                            c,
                            '*' | '_'
                                | '`'
                                | '$'
                                | '-'
                                | '+'
                                | '.'
                                | '#'
                                | '('
                                | ')'
                                | '['
                                | ']'
                                | '<'
                                | '>'
                                | ':'
                                | ','
                                | '='
                                | '/'
                                | '@'
                                | '"'
                        )
                    {
                        break;
                    }
                    buf.push(c);
                    i += 1;
                }
                if !buf.is_empty() {
                    tokens.push(Token::Text(buf));
                }
            }
        }
    }

    tokens.push(Token::Eof);
    tokens
}

struct TypstParser {
    tokens: Vec<Token>,
    pos: usize,
    next_id: u32,
}

impl TypstParser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            next_id: 1,
        }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof);
        self.pos += 1;
        tok
    }

    fn at(&self, t: &Token) -> bool {
        self.peek() == t
    }

    fn gen_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn skip_newlines(&mut self) {
        while self.at(&Token::Newline) {
            self.advance();
        }
    }

    fn collect_text_until(&mut self, terminator: &dyn Fn(&Token) -> bool) -> String {
        let mut buf = String::new();
        loop {
            match self.peek().clone() {
                t if terminator(&t) => break,
                Token::Text(s) => {
                    if !buf.is_empty() {
                        let last = buf.chars().last().unwrap_or(' ');
                        if !last.is_ascii_punctuation() {
                            buf.push(' ');
                        }
                    }
                    buf.push_str(&s);
                    self.advance();
                }
                Token::Newline => {
                    self.advance();
                }
                Token::Star => {
                    self.advance();
                    buf.push('*');
                }
                Token::Underscore => {
                    self.advance();
                    buf.push('_');
                }
                Token::Backtick => {
                    self.advance();
                    buf.push('`');
                }
                Token::Dollar => {
                    self.advance();
                    buf.push('$');
                }
                Token::Minus => {
                    self.advance();
                    buf.push('-');
                }
                Token::AtSign => {
                    self.advance();
                    buf.push('@');
                }
                Token::Colon => {
                    self.advance();
                    buf.push(':');
                }
                Token::Equals => {
                    self.advance();
                    buf.push('=');
                }
                Token::LParen => {
                    self.advance();
                    buf.push('(');
                }
                Token::RParen => {
                    self.advance();
                    buf.push(')');
                }
                Token::Comma => {
                    self.advance();
                    buf.push(',');
                }
                Token::Dot => {
                    self.advance();
                    buf.push('.');
                }
                Token::Plus => {
                    self.advance();
                    buf.push('+');
                }
                Token::Hash => {
                    self.advance();
                    buf.push('#');
                }
                _ => {
                    break;
                }
            }
        }
        buf.trim().to_string()
    }

    fn collect_inline_content(&mut self, terminators: &[Token]) -> Vec<NodeType> {
        let mut nodes = Vec::new();
        loop {
            if terminators.contains(self.peek()) {
                break;
            }
            match self.peek().clone() {
                Token::Eof | Token::Newline => {
                    self.advance();
                    if nodes.is_empty() {
                        break;
                    }
                    if !terminators.contains(&Token::Newline) {
                        continue;
                    }
                    break;
                }
                Token::Text(s) => {
                    self.advance();
                    if !s.is_empty() {
                        nodes.push(NodeType::Text { content: s });
                    }
                }
                Token::Star => {
                    self.advance();
                    let inner = self.collect_text_until(&|t| t == &Token::Star);
                    if let Token::Star = self.peek() {
                        self.advance();
                    }
                    if !inner.is_empty() {
                        nodes.push(NodeType::Bold);
                        nodes.push(NodeType::Text { content: inner });
                    }
                }
                Token::Underscore => {
                    self.advance();
                    let inner = self.collect_text_until(&|t| t == &Token::Underscore);
                    if let Token::Underscore = self.peek() {
                        self.advance();
                    }
                    if !inner.is_empty() {
                        nodes.push(NodeType::Italic);
                        nodes.push(NodeType::Text { content: inner });
                    }
                }
                Token::Backtick => {
                    self.advance();
                    let inner = self.collect_text_until(&|t| t == &Token::Backtick);
                    if let Token::Backtick = self.peek() {
                        self.advance();
                    }
                    if !inner.is_empty() {
                        nodes.push(NodeType::Mono);
                        nodes.push(NodeType::Text { content: inner });
                    }
                }
                Token::Dollar => {
                    self.advance();
                    let math_content = self.collect_text_until(&|t| t == &Token::Dollar);
                    if let Token::Dollar = self.peek() {
                        self.advance();
                    }
                    if !math_content.is_empty() {
                        nodes.push(NodeType::MathInline {
                            content: math_content,
                        });
                    }
                }
                Token::Hash => {
                    self.advance();
                    if let Token::Text(func_name) = self.peek().clone() {
                        let name = func_name.clone();
                        self.advance();
                        match name.as_str() {
                            "footnote" => {
                                if self.at(&Token::LBracket) {
                                    self.advance();
                                }
                                let content = self.collect_text_until(&|t| t == &Token::RBracket);
                                if self.at(&Token::RBracket) {
                                    self.advance();
                                }
                                nodes.push(NodeType::Footnote { content });
                            }
                            "link" => {
                                let mut url = String::new();
                                if self.at(&Token::LParen) {
                                    self.advance();
                                    if let Token::Text(s) = self.advance() {
                                        url = s;
                                    }
                                    if self.at(&Token::RParen) {
                                        self.advance();
                                    }
                                }
                                if self.at(&Token::LBracket) {
                                    self.advance();
                                }
                                let _text = self.collect_text_until(&|t| t == &Token::RBracket);
                                if self.at(&Token::RBracket) {
                                    self.advance();
                                }
                                if !url.is_empty() {
                                    nodes.push(NodeType::Link { url, title: None });
                                }
                            }
                            "ref" => {
                                let _label = self.collect_text_until(&|t| {
                                    matches!(t, Token::RAngle | Token::Newline | Token::Eof)
                                });
                            }
                            _ => {
                                nodes.push(NodeType::Text { content: name });
                            }
                        }
                    }
                }
                Token::AtSign => {
                    self.advance();
                    let label = self.collect_text_until(&|t| {
                        matches!(
                            t,
                            Token::Text(_)
                                | Token::Newline
                                | Token::Eof
                                | Token::Star
                                | Token::Underscore
                                | Token::Backtick
                        )
                    });
                    if !label.is_empty() {
                        nodes.push(NodeType::Text {
                            content: format!("@{}", label),
                        });
                    }
                }
                Token::LAngle => {
                    break;
                }
                _ => {
                    self.advance();
                }
            }
        }
        nodes
    }

    fn parse_document(&mut self) -> SIRModuleV2 {
        let mut module = SIRModuleV2::from_source("typst", "<input>");
        let doc_id = self.gen_id();
        let doc_node = Node::new(doc_id, NodeType::Document);
        module.body.push(doc_node);

        self.skip_newlines();

        loop {
            if self.at(&Token::Eof) {
                break;
            }

            match self.peek().clone() {
                Token::Heading(level) => {
                    self.advance();
                    let inline_nodes = self.collect_inline_content(&[Token::Newline, Token::Eof]);
                    self.skip_newlines();
                    let heading_type = match level {
                        1 => NodeType::Section,
                        2 => NodeType::Subsection,
                        3 => NodeType::Subsubsection,
                        _ => NodeType::Paragraph,
                    };
                    let heading_id = self.gen_id();
                    let mut heading_node = Node::new(heading_id, heading_type).with_parent(doc_id);
                    if let Some(NodeType::Text { content }) = inline_nodes.first() {
                        heading_node.counter = Some(content.clone());
                    }
                    module.body.push(heading_node);

                    let mut label = None;
                    if self.at(&Token::LAngle) {
                        self.advance();
                        label = Some(self.collect_text_until(&|t| t == &Token::RAngle));
                        if self.at(&Token::RAngle) {
                            self.advance();
                        }
                    }

                    if let Some(ref lbl) = label {
                        module.annotations.add_label(
                            lbl.clone(),
                            heading_id,
                            LabelCategory::Section,
                        );
                        if let Some(h) = module.body.get_mut(heading_id) {
                            h.label = Some(lbl.clone());
                        }
                    }

                    if let Some(doc) = module.body.get_mut(doc_id) {
                        doc.add_child(heading_id);
                    }
                }
                Token::Hash => {
                    self.advance();
                    if let Token::Text(func_name) = self.peek().clone() {
                        let name = func_name.clone();
                        self.advance();
                        match name.as_str() {
                            "set" | "show" | "text" => {
                                self.skip_set_args();
                                self.skip_newlines();
                            }
                            "quote" => {
                                if self.at(&Token::LBracket) {
                                    self.advance();
                                }
                                let inline_nodes = self.collect_inline_content(&[Token::RBracket]);
                                if self.at(&Token::RBracket) {
                                    self.advance();
                                }
                                self.skip_newlines();

                                let bq_id = self.gen_id();
                                module.body.push(
                                    Node::new(bq_id, NodeType::BlockQuote).with_parent(doc_id),
                                );
                                if let Some(doc) = module.body.get_mut(doc_id) {
                                    doc.add_child(bq_id);
                                }

                                for node_type in &inline_nodes {
                                    let child_id = self.gen_id();
                                    module.body.push(
                                        Node::new(child_id, node_type.clone()).with_parent(bq_id),
                                    );
                                    if let Some(bq) = module.body.get_mut(bq_id) {
                                        bq.add_child(child_id);
                                    }
                                }
                            }
                            "footnote" => {
                                if self.at(&Token::LBracket) {
                                    self.advance();
                                }
                                let content = self.collect_text_until(&|t| t == &Token::RBracket);
                                if self.at(&Token::RBracket) {
                                    self.advance();
                                }
                                self.skip_newlines();
                                let fn_id = self.gen_id();
                                module.body.push(
                                    Node::new(fn_id, NodeType::Footnote { content })
                                        .with_parent(doc_id),
                                );
                                if let Some(doc) = module.body.get_mut(doc_id) {
                                    doc.add_child(fn_id);
                                }
                            }
                            "link" => {
                                let mut url = String::new();
                                if self.at(&Token::LParen) {
                                    self.advance();
                                    if let Token::Text(s) = self.advance() {
                                        url = s;
                                    }
                                    if self.at(&Token::RParen) {
                                        self.advance();
                                    }
                                }
                                if self.at(&Token::LBracket) {
                                    self.advance();
                                }
                                let _text = self.collect_text_until(&|t| t == &Token::RBracket);
                                if self.at(&Token::RBracket) {
                                    self.advance();
                                }
                                self.skip_newlines();
                                if !url.is_empty() {
                                    let link_id = self.gen_id();
                                    module.body.push(
                                        Node::new(link_id, NodeType::Link { url, title: None })
                                            .with_parent(doc_id),
                                    );
                                    if let Some(doc) = module.body.get_mut(doc_id) {
                                        doc.add_child(link_id);
                                    }
                                }
                            }
                            "figure" => {
                                self.skip_figure_args();
                                self.skip_newlines();
                                if self.at(&Token::LAngle) {
                                    self.advance();
                                    let _label = self.collect_text_until(&|t| t == &Token::RAngle);
                                    if self.at(&Token::RAngle) {
                                        self.advance();
                                    }
                                }
                                let fig_id = self.gen_id();
                                module.body.push(
                                    Node::new(
                                        fig_id,
                                        NodeType::Figure {
                                            placement:
                                                ldir_ir::sir::v2::nodes::FloatPlacement::Here,
                                        },
                                    )
                                    .with_parent(doc_id),
                                );
                                if let Some(doc) = module.body.get_mut(doc_id) {
                                    doc.add_child(fig_id);
                                }
                            }
                            "table" => {
                                let (num_cols, cells) = self.parse_table_args();
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

                                let mut cell_idx = 0;
                                let mut row_id = self.gen_id();
                                module.body.push(
                                    Node::new(row_id, NodeType::TableRow { is_header: false })
                                        .with_parent(table_id),
                                );
                                if let Some(tbl) = module.body.get_mut(table_id) {
                                    tbl.add_child(row_id);
                                }

                                for cell_text in &cells {
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
                                    if let Some(row) = module.body.get_mut(row_id) {
                                        row.add_child(tc_id);
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
                                    cell_idx += 1;
                                    if cell_idx % num_cols == 0 && cell_idx < cells.len() {
                                        row_id = self.gen_id();
                                        module.body.push(
                                            Node::new(
                                                row_id,
                                                NodeType::TableRow { is_header: false },
                                            )
                                            .with_parent(table_id),
                                        );
                                        if let Some(tbl) = module.body.get_mut(table_id) {
                                            tbl.add_child(row_id);
                                        }
                                    }
                                }

                                if let Some(doc) = module.body.get_mut(doc_id) {
                                    doc.add_child(table_id);
                                }
                                self.skip_newlines();
                            }
                            _ => {
                                self.skip_newlines();
                            }
                        }
                    } else {
                        self.skip_newlines();
                    }
                }
                Token::Minus => {
                    self.advance();
                    let inline_nodes = self.collect_inline_content(&[Token::Newline, Token::Eof]);
                    self.skip_newlines();

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

                    let item_id = self.gen_id();
                    module
                        .body
                        .push(Node::new(item_id, NodeType::ListItem).with_parent(list_id));
                    if let Some(list) = module.body.get_mut(list_id) {
                        list.add_child(item_id);
                    }

                    for node_type in &inline_nodes {
                        let child_id = self.gen_id();
                        module
                            .body
                            .push(Node::new(child_id, node_type.clone()).with_parent(item_id));
                        if let Some(item) = module.body.get_mut(item_id) {
                            item.add_child(child_id);
                        }
                    }

                    while self.at(&Token::Minus) {
                        self.advance();
                        let item_inline =
                            self.collect_inline_content(&[Token::Newline, Token::Eof]);
                        self.skip_newlines();
                        let sub_item_id = self.gen_id();
                        module
                            .body
                            .push(Node::new(sub_item_id, NodeType::ListItem).with_parent(list_id));
                        if let Some(list) = module.body.get_mut(list_id) {
                            list.add_child(sub_item_id);
                        }
                        for node_type in &item_inline {
                            let child_id = self.gen_id();
                            module.body.push(
                                Node::new(child_id, node_type.clone()).with_parent(sub_item_id),
                            );
                            if let Some(item) = module.body.get_mut(sub_item_id) {
                                item.add_child(child_id);
                            }
                        }
                    }

                    if let Some(doc) = module.body.get_mut(doc_id) {
                        doc.add_child(list_id);
                    }
                }
                Token::Plus => {
                    self.advance();
                    let inline_nodes = self.collect_inline_content(&[Token::Newline, Token::Eof]);
                    self.skip_newlines();

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

                    let item_id = self.gen_id();
                    module
                        .body
                        .push(Node::new(item_id, NodeType::ListItem).with_parent(list_id));
                    if let Some(list) = module.body.get_mut(list_id) {
                        list.add_child(item_id);
                    }

                    for node_type in &inline_nodes {
                        let child_id = self.gen_id();
                        module
                            .body
                            .push(Node::new(child_id, node_type.clone()).with_parent(item_id));
                        if let Some(item) = module.body.get_mut(item_id) {
                            item.add_child(child_id);
                        }
                    }

                    while self.at(&Token::Plus) {
                        self.advance();
                        let item_inline =
                            self.collect_inline_content(&[Token::Newline, Token::Eof]);
                        self.skip_newlines();
                        let sub_item_id = self.gen_id();
                        module
                            .body
                            .push(Node::new(sub_item_id, NodeType::ListItem).with_parent(list_id));
                        if let Some(list) = module.body.get_mut(list_id) {
                            list.add_child(sub_item_id);
                        }
                        for node_type in &item_inline {
                            let child_id = self.gen_id();
                            module.body.push(
                                Node::new(child_id, node_type.clone()).with_parent(sub_item_id),
                            );
                            if let Some(item) = module.body.get_mut(sub_item_id) {
                                item.add_child(child_id);
                            }
                        }
                    }

                    if let Some(doc) = module.body.get_mut(doc_id) {
                        doc.add_child(list_id);
                    }
                }
                Token::Newline => {
                    self.advance();
                    self.skip_newlines();
                }
                Token::Text(_) => {
                    let inline_nodes = self.collect_inline_content(&[Token::Newline, Token::Eof]);

                    if self.at(&Token::LAngle) {
                        self.advance();
                        self.collect_text_until(&|t| t == &Token::RAngle);
                        if self.at(&Token::RAngle) {
                            self.advance();
                        }
                    }
                    self.skip_newlines();

                    if inline_nodes.is_empty() {
                        continue;
                    }

                    let para_id = self.gen_id();
                    module
                        .body
                        .push(Node::new(para_id, NodeType::Paragraph).with_parent(doc_id));
                    if let Some(doc) = module.body.get_mut(doc_id) {
                        doc.add_child(para_id);
                    }

                    for node_type in &inline_nodes {
                        let child_id = self.gen_id();
                        module
                            .body
                            .push(Node::new(child_id, node_type.clone()).with_parent(para_id));
                        if let Some(para) = module.body.get_mut(para_id) {
                            para.add_child(child_id);
                        }
                    }
                }
                _ => {
                    self.advance();
                }
            }
        }

        module
    }

    fn skip_set_args(&mut self) {
        let mut depth: u32 = 0;
        loop {
            match self.peek() {
                Token::LParen => {
                    depth += 1;
                    self.advance();
                }
                Token::RParen => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                    self.advance();
                }
                Token::Newline | Token::Eof => {
                    break;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn skip_figure_args(&mut self) {
        let mut depth: u32 = 0;
        loop {
            match self.peek() {
                Token::LParen => {
                    depth += 1;
                    self.advance();
                }
                Token::RParen => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                    self.advance();
                }
                Token::Newline | Token::Eof => {
                    break;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn parse_table_args(&mut self) -> (usize, Vec<String>) {
        let mut num_cols: usize = 1;
        let mut cells: Vec<String> = Vec::new();

        if self.at(&Token::LParen) {
            self.advance();

            loop {
                if self.at(&Token::RParen) || self.at(&Token::Eof) {
                    break;
                }

                if let Token::Text(name) = self.peek()
                    && name == "columns"
                {
                    self.advance();
                    if self.at(&Token::Colon) {
                        self.advance();
                    }
                    if let Token::Text(val) = self.peek() {
                        num_cols = val.parse().unwrap_or(1);
                        self.advance();
                    }
                    if self.at(&Token::Comma) {
                        self.advance();
                    }
                    continue;
                }

                if self.at(&Token::LBracket) {
                    self.advance();
                    let content = self.collect_text_until(&|t| t == &Token::RBracket);
                    if self.at(&Token::RBracket) {
                        self.advance();
                    }
                    cells.push(content);
                    if self.at(&Token::Comma) {
                        self.advance();
                    }
                    continue;
                }

                self.advance();
            }

            if self.at(&Token::RParen) {
                self.advance();
            }
        }

        (num_cols, cells)
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
    fn test_heading_level_1() {
        let module = parse_typst("= Introduction\n");
        let sections = find_nodes(&module, |nt| matches!(nt, NodeType::Section));
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].counter.as_deref(), Some("Introduction"));
    }

    #[test]
    fn test_heading_level_2() {
        let module = parse_typst("== Background\n");
        let subs = find_nodes(&module, |nt| matches!(nt, NodeType::Subsection));
        assert_eq!(subs.len(), 1);
        assert_eq!(subs[0].counter.as_deref(), Some("Background"));
    }

    #[test]
    fn test_heading_level_3() {
        let module = parse_typst("=== Methods\n");
        let subsubs = find_nodes(&module, |nt| matches!(nt, NodeType::Subsubsection));
        assert_eq!(subsubs.len(), 1);
        assert_eq!(subsubs[0].counter.as_deref(), Some("Methods"));
    }

    #[test]
    fn test_bold() {
        let module = parse_typst("This is *bold* text.\n");
        let bolds = find_nodes(&module, |nt| matches!(nt, NodeType::Bold));
        assert_eq!(bolds.len(), 1);
    }

    #[test]
    fn test_italic() {
        let module = parse_typst("This is _italic_ text.\n");
        let italics = find_nodes(&module, |nt| matches!(nt, NodeType::Italic));
        assert_eq!(italics.len(), 1);
    }

    #[test]
    fn test_mono() {
        let module = parse_typst("This is `code` text.\n");
        let monos = find_nodes(&module, |nt| matches!(nt, NodeType::Mono));
        assert_eq!(monos.len(), 1);
    }

    #[test]
    fn test_paragraph() {
        let module = parse_typst("Hello world.\n");
        let paras = find_nodes(&module, |nt| matches!(nt, NodeType::Paragraph));
        assert_eq!(paras.len(), 1);
        let texts = find_nodes(&module, |nt| matches!(nt, NodeType::Text { .. }));
        assert!(texts.iter().any(
            |n| matches!(&n.node_type, NodeType::Text { content } if content.contains("Hello"))
        ));
    }

    #[test]
    fn test_unordered_list() {
        let module = parse_typst("- Item one\n- Item two\n");
        let lists = find_nodes(&module, |nt| {
            matches!(nt, NodeType::List { ordered: false, .. })
        });
        assert_eq!(lists.len(), 1);
        let items = find_nodes(&module, |nt| matches!(nt, NodeType::ListItem));
        assert!(items.len() >= 2);
    }

    #[test]
    fn test_ordered_list() {
        let module = parse_typst("+ First item\n+ Second item\n");
        let lists = find_nodes(&module, |nt| {
            matches!(nt, NodeType::List { ordered: true, .. })
        });
        assert_eq!(lists.len(), 1);
        let items = find_nodes(&module, |nt| matches!(nt, NodeType::ListItem));
        assert!(items.len() >= 2);
    }

    #[test]
    fn test_math_inline() {
        let module = parse_typst("The equation is $x^2 + y^2 = z^2$ here.\n");
        let math = find_nodes(&module, |nt| matches!(nt, NodeType::MathInline { .. }));
        assert_eq!(math.len(), 1);
        if let NodeType::MathInline { content } = &math[0].node_type {
            assert!(content.contains("x^2"));
        }
    }

    #[test]
    fn test_comment_stripped() {
        let module = parse_typst("// This is a comment\nHello\n");
        let texts = find_nodes(&module, |nt| matches!(nt, NodeType::Text { .. }));
        assert!(!texts.iter().any(
            |n| matches!(&n.node_type, NodeType::Text { content } if content.contains("comment"))
        ));
        assert!(texts.iter().any(
            |n| matches!(&n.node_type, NodeType::Text { content } if content.contains("Hello"))
        ));
    }

    #[test]
    fn test_quote_block() {
        let module = parse_typst("#quote[A notable quote.]\n");
        let quotes = find_nodes(&module, |nt| matches!(nt, NodeType::BlockQuote));
        assert_eq!(quotes.len(), 1);
    }

    #[test]
    fn test_footnote() {
        let module = parse_typst("#footnote[This is a footnote.]\n");
        let fns = find_nodes(&module, |nt| matches!(nt, NodeType::Footnote { .. }));
        assert_eq!(fns.len(), 1);
        if let NodeType::Footnote { content } = &fns[0].node_type {
            assert!(content.contains("footnote"));
        }
    }

    #[test]
    fn test_link() {
        let module = parse_typst("#link(\"https://example.com\")[click here]\n");
        let links = find_nodes(&module, |nt| matches!(nt, NodeType::Link { .. }));
        assert_eq!(links.len(), 1);
        if let NodeType::Link { url, .. } = &links[0].node_type {
            assert_eq!(url, "https://example.com");
        }
    }

    #[test]
    fn test_empty_document() {
        let module = parse_typst("");
        let docs = find_nodes(&module, |nt| matches!(nt, NodeType::Document));
        assert_eq!(docs.len(), 1);
        assert_eq!(module.body.len(), 1);
    }

    #[test]
    fn test_multiple_paragraphs() {
        let module = parse_typst("First paragraph.\n\nSecond paragraph.\n");
        let paras = find_nodes(&module, |nt| matches!(nt, NodeType::Paragraph));
        assert_eq!(paras.len(), 2);
    }

    #[test]
    fn test_complex_document() {
        let input = "= Title\n\nSome *bold* text.\n\n- Item\n- Item two\n";
        let module = parse_typst(input);
        let sections = find_nodes(&module, |nt| matches!(nt, NodeType::Section));
        assert_eq!(sections.len(), 1);
        let bolds = find_nodes(&module, |nt| matches!(nt, NodeType::Bold));
        assert_eq!(bolds.len(), 1);
        let lists = find_nodes(&module, |nt| matches!(nt, NodeType::List { .. }));
        assert_eq!(lists.len(), 1);
    }

    #[test]
    fn test_nested_formatting() {
        let module = parse_typst("*bold _italic_* text\n");
        let bolds = find_nodes(&module, |nt| matches!(nt, NodeType::Bold));
        assert_eq!(bolds.len(), 1);
        let italics = find_nodes(&module, |nt| matches!(nt, NodeType::Italic));
        assert_eq!(italics.len(), 1);
    }

    #[test]
    fn test_set_rules_skipped() {
        let module = parse_typst("#set page(paper: \"a4\")\n\nHello\n");
        let paras = find_nodes(&module, |nt| matches!(nt, NodeType::Paragraph));
        assert_eq!(paras.len(), 1);
        let texts = find_nodes(
            &module,
            |nt| matches!(nt, NodeType::Text { content } if content.contains("Hello")),
        );
        assert_eq!(texts.len(), 1);
    }

    #[test]
    fn test_table() {
        let module = parse_typst("#table(\n  columns: 2,\n  [A], [B],\n  [C], [D],\n)\n");
        let tables = find_nodes(&module, |nt| matches!(nt, NodeType::Table { .. }));
        assert_eq!(tables.len(), 1);
        if let NodeType::Table { num_cols, .. } = &tables[0].node_type {
            assert_eq!(*num_cols, 2);
        }
        let rows = find_nodes(&module, |nt| matches!(nt, NodeType::TableRow { .. }));
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn test_figure() {
        let module =
            parse_typst("#figure(\n  image(\"photo.png\"),\n  caption: Photo,\n) <fig:photo>\n");
        let figs = find_nodes(&module, |nt| matches!(nt, NodeType::Figure { .. }));
        assert_eq!(figs.len(), 1);
    }

    #[test]
    fn test_heading_with_label() {
        let module = parse_typst("= Introduction <sec:intro>\n");
        let sections = find_nodes(&module, |nt| matches!(nt, NodeType::Section));
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].label.as_deref(), Some("sec:intro"));
        assert!(module.annotations.find_label("sec:intro").is_some());
    }

    #[test]
    fn test_tokenizer_heading() {
        let tokens = tokenize("== Hello\n");
        assert_eq!(tokens[0], Token::Heading(2));
        assert_eq!(tokens[1], Token::Text("Hello".to_string()));
    }

    #[test]
    fn test_tokenizer_comment() {
        let tokens = tokenize("hello // comment\nworld");
        let texts: Vec<_> = tokens
            .iter()
            .filter(|t| matches!(t, Token::Text(_)))
            .collect();
        assert_eq!(texts.len(), 2);
    }

    #[test]
    fn test_tokenizer_dollar() {
        let tokens = tokenize("$x^2$");
        assert_eq!(tokens[0], Token::Dollar);
        assert_eq!(tokens[1], Token::Text("x^2".to_string()));
        assert_eq!(tokens[2], Token::Dollar);
    }

    #[test]
    fn test_full_document_structure() {
        let input = r#"= Document Title

This is a *bold* and _italic_ paragraph with `code`.

== Section One

Some text with $x^2 + y^2$ math.

- First item
- Second item

// This is a comment

#quote[A notable quote.]

#footnote[A small note.]

== Section Two

More content here.
"#;
        let module = parse_typst(input);
        let docs = find_nodes(&module, |nt| matches!(nt, NodeType::Document));
        assert_eq!(docs.len(), 1);
        let sections = find_nodes(&module, |nt| matches!(nt, NodeType::Section));
        assert_eq!(sections.len(), 1);
        let subs = find_nodes(&module, |nt| matches!(nt, NodeType::Subsection));
        assert_eq!(subs.len(), 2);
        let bolds = find_nodes(&module, |nt| matches!(nt, NodeType::Bold));
        assert_eq!(bolds.len(), 1);
        let italics = find_nodes(&module, |nt| matches!(nt, NodeType::Italic));
        assert_eq!(italics.len(), 1);
        let monos = find_nodes(&module, |nt| matches!(nt, NodeType::Mono));
        assert_eq!(monos.len(), 1);
        let math = find_nodes(&module, |nt| matches!(nt, NodeType::MathInline { .. }));
        assert_eq!(math.len(), 1);
        let lists = find_nodes(&module, |nt| matches!(nt, NodeType::List { .. }));
        assert_eq!(lists.len(), 1);
        let quotes = find_nodes(&module, |nt| matches!(nt, NodeType::BlockQuote));
        assert_eq!(quotes.len(), 1);
        let fns = find_nodes(&module, |nt| matches!(nt, NodeType::Footnote { .. }));
        assert_eq!(fns.len(), 1);
        assert_eq!(module.header.source_format.as_deref(), Some("typst"));
    }
}
