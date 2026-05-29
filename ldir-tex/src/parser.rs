use ldir_ir::sir::{
    BlockType, ROOT_SENTINEL, SIRDocument, SIRInstruction, SIROpcode, SourceSpan, StyleModifier,
};

use crate::lexer::{SpannedToken, Token};

pub struct TeXParser<'a> {
    tokens: &'a [SpannedToken],
    pos: usize,
    doc: SIRDocument,
    next_id: u32,
    in_math: bool,
    footnote_counter: u32,
    current_span: Option<SourceSpan>,
    warnings: Vec<(SourceSpan, String)>,
}

impl<'a> TeXParser<'a> {
    pub fn new(tokens: &'a [SpannedToken]) -> Self {
        Self {
            tokens,
            pos: 0,
            doc: SIRDocument::new(),
            next_id: 0,
            in_math: false,
            footnote_counter: 0,
            current_span: None,
            warnings: Vec::new(),
        }
    }

    pub fn parse(mut self) -> (SIRDocument, Vec<(SourceSpan, String)>) {
        let root_id = self.next_entity_id();
        let payload_offset = self.doc.payload_mut().append(&[BlockType::Document as u8]);
        self.push_instr(SIRInstruction::new(
            SIROpcode::PushBlock,
            root_id,
            ROOT_SENTINEL,
            payload_offset,
        ));

        self.skip_preamble();
        self.parse_body(root_id);

        let warnings = std::mem::take(&mut self.warnings);
        (std::mem::take(&mut self.doc), warnings)
    }

    fn next_entity_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|st| &st.token)
    }

    fn advance(&mut self) -> Option<&Token> {
        self.current_span = self.tokens.get(self.pos).map(|st| st.span);
        let tok = self.tokens.get(self.pos).map(|st| &st.token);
        self.pos += 1;
        tok
    }

    fn token_at(&self, idx: usize) -> Option<&Token> {
        self.tokens.get(idx).map(|st| &st.token)
    }

    fn push_instr(&mut self, instr: SIRInstruction) {
        self.doc.push(instr);
        self.doc.source_spans.push(self.current_span);
    }

    fn push_instr_with_payload(&mut self, instr: SIRInstruction, payload: &[u8]) {
        self.doc.push_with_payload(instr, payload);
        self.doc.source_spans.push(self.current_span);
    }

    fn skip_preamble(&mut self) {
        while let Some(tok) = self.peek() {
            match tok {
                Token::ControlSequence(name)
                    if matches!(
                        name.as_str(),
                        "documentclass" | "usepackage" | "title" | "author" | "date" | "maketitle"
                    ) =>
                {
                    self.advance();
                    self.skip_group();
                    if let Some(Token::BracketOpen) = self.peek() {
                        self.skip_brackets();
                    }
                }
                Token::ControlSequence(name) if name == "begin" => {
                    if self.is_begin_env("document") {
                        self.advance();
                        self.skip_group();
                        return;
                    }
                    break;
                }
                Token::Comment(_) => {
                    self.advance();
                }
                Token::Text(t) if t.trim().is_empty() => {
                    self.advance();
                }
                _ => break,
            }
        }
    }

    fn is_begin_env(&self, env_name: &str) -> bool {
        if let Some(Token::ControlSequence(name)) = self.token_at(self.pos)
            && name == "begin"
            && let Some(Token::BraceOpen) = self.token_at(self.pos + 1)
            && let Some(Token::Text(env)) = self.token_at(self.pos + 2)
        {
            return env == env_name;
        }
        false
    }

    fn skip_group(&mut self) {
        if let Some(&Token::BraceOpen) = self.peek() {
            self.advance();
            let mut depth = 1;
            while depth > 0 {
                if let Some(tok) = self.advance() {
                    match tok {
                        Token::BraceOpen => depth += 1,
                        Token::BraceClose => depth -= 1,
                        _ => {}
                    }
                } else {
                    break;
                }
            }
        }
    }

    fn skip_brackets(&mut self) {
        if let Some(&Token::BracketOpen) = self.peek() {
            self.advance();
            while let Some(tok) = self.peek() {
                if matches!(tok, Token::BracketClose) {
                    self.advance();
                    return;
                }
                self.advance();
            }
        }
    }

    fn parse_group_content(&mut self) -> String {
        let mut result = String::new();
        if let Some(&Token::BraceOpen) = self.peek() {
            self.advance();
            let mut depth = 1;
            let in_math = self.in_math;
            while depth > 0 {
                let tok = self.advance().cloned();
                match tok {
                    Some(Token::BraceOpen) => {
                        depth += 1;
                        result.push('{');
                    }
                    Some(Token::BraceClose) => {
                        depth -= 1;
                        if depth > 0 {
                            result.push('}');
                        }
                    }
                    Some(Token::ControlSequence(name)) => {
                        if in_math {
                            if let Some(sym) = Self::substitute_math_cmd(&name) {
                                result.push_str(sym);
                            } else {
                                result.push_str(&name);
                            }
                        } else {
                            result.push_str(&format!("\\{name}"));
                        }
                    }
                    Some(Token::Text(t)) => {
                        result.push_str(&t);
                    }
                    Some(Token::DollarSign) => {
                        result.push('$');
                    }
                    Some(Token::Caret) => {
                        result.push('^');
                    }
                    Some(Token::Underscore) => {
                        result.push('_');
                    }
                    Some(Token::Tilde) => {
                        result.push('\u{00A0}');
                    }
                    Some(Token::LineBreak) => {
                        result.push('\n');
                    }
                    Some(_) | None => {}
                }
            }
        }
        result.trim().to_string()
    }

    fn parse_optional(&mut self) -> Option<String> {
        if let Some(&Token::BracketOpen) = self.peek() {
            self.advance();
            let mut content = String::new();
            while let Some(tok) = self.peek() {
                if matches!(tok, Token::BracketClose) {
                    self.advance();
                    return Some(content.trim().to_string());
                }
                match tok {
                    Token::Text(t) => content.push_str(t),
                    Token::ControlSequence(name) => content.push_str(&format!("\\{name}")),
                    _ => content.push_str(&tok.to_string()),
                }
                self.advance();
            }
            Some(content.trim().to_string())
        } else {
            None
        }
    }

    fn parse_body(&mut self, parent_id: u32) {
        let mut text_buffer = String::new();

        while let Some(tok) = self.peek().cloned() {
            match &tok {
                Token::ControlSequence(name) => match name.as_str() {
                    "end" => {
                        if self.is_end_env("document") {
                            self.consume_end_env("document");
                            break;
                        }
                        self.flush_paragraph(&mut text_buffer, parent_id);
                        self.advance();
                        self.skip_group();
                    }
                    "begin" => {
                        self.flush_paragraph(&mut text_buffer, parent_id);
                        self.advance();
                        let env_name = self.parse_group_content();
                        self.parse_environment(&env_name, parent_id);
                    }
                    "documentclass" | "usepackage" | "title" | "author" | "date" | "maketitle" => {
                        self.flush_paragraph(&mut text_buffer, parent_id);
                        self.advance();
                        self.skip_group();
                        self.parse_optional();
                    }
                    "includegraphics" => {
                        self.flush_paragraph(&mut text_buffer, parent_id);
                        self.advance();
                        self.parse_optional();
                        let path = self.parse_group_content();
                        self.emit_block(BlockType::Image, parent_id, None, &path);
                    }
                    "section" | "subsection" | "subsubsection" | "paragraph" => {
                        self.flush_paragraph(&mut text_buffer, parent_id);
                        let level = match name.as_str() {
                            "section" => 1u32,
                            "subsection" => 2,
                            "subsubsection" => 3,
                            "paragraph" => 4,
                            _ => 1,
                        };
                        self.advance();
                        let title = self.parse_group_content();
                        self.emit_heading(parent_id, level, &title);
                    }
                    "label" => {
                        self.advance();
                        let key = self.parse_group_content();
                        text_buffer.push_str(&format!("\\label{{{}}}", key));
                    }
                    "ref" => {
                        self.advance();
                        let key = self.parse_group_content();
                        text_buffer.push_str(&format!("\\ref{{{}}}", key));
                    }
                    "eqref" => {
                        self.advance();
                        let key = self.parse_group_content();
                        text_buffer.push_str(&format!("\\eqref{{{}}}", key));
                    }
                    "textbf" => {
                        self.advance();
                        let content = self.parse_group_content();
                        self.emit_styled_text(
                            parent_id,
                            &mut text_buffer,
                            StyleModifier::BOLD_STYLE,
                            &content,
                        );
                    }
                    "textit" => {
                        self.advance();
                        let content = self.parse_group_content();
                        self.emit_styled_text(
                            parent_id,
                            &mut text_buffer,
                            StyleModifier::ITALIC_STYLE,
                            &content,
                        );
                    }
                    "texttt" => {
                        self.advance();
                        let content = self.parse_group_content();
                        self.emit_styled_text(
                            parent_id,
                            &mut text_buffer,
                            StyleModifier::MONO_STYLE,
                            &content,
                        );
                    }
                    "emph" => {
                        self.advance();
                        let content = self.parse_group_content();
                        self.emit_styled_text(
                            parent_id,
                            &mut text_buffer,
                            StyleModifier::ITALIC_STYLE,
                            &content,
                        );
                    }
                    "url" => {
                        self.advance();
                        let url = self.parse_group_content();
                        self.flush_paragraph(&mut text_buffer, parent_id);
                        self.emit_link(parent_id, &url);
                        let content_id = self.next_entity_id();
                        self.push_instr_with_payload(
                            SIRInstruction::new(SIROpcode::SetContent, content_id, parent_id, 0),
                            url.as_bytes(),
                        );
                    }
                    "href" => {
                        self.advance();
                        let url = self.parse_group_content();
                        let text = self.parse_group_content();
                        self.flush_paragraph(&mut text_buffer, parent_id);
                        self.emit_link(parent_id, &url);
                        let content_id = self.next_entity_id();
                        self.push_instr_with_payload(
                            SIRInstruction::new(SIROpcode::SetContent, content_id, parent_id, 0),
                            text.as_bytes(),
                        );
                    }
                    "par" => {
                        self.flush_paragraph(&mut text_buffer, parent_id);
                        self.advance();
                    }
                    "footnote" => {
                        self.advance();
                        let content = self.parse_group_content();
                        self.footnote_counter += 1;
                        let num = self.footnote_counter;
                        let mark = format!("\\fnmark{{{}}}", num);
                        text_buffer.push_str(&mark);
                        self.doc.footnotes.push((num, content));
                    }
                    _ => {
                        if self.in_math {
                            if let Some(sym) = Self::substitute_math_cmd(name) {
                                text_buffer.push_str(sym);
                            } else {
                                text_buffer.push_str(&format!("\\{name}"));
                            }
                        } else {
                            text_buffer.push_str(&format!("\\{name}"));
                        }
                        self.advance();
                    }
                },
                Token::DollarSign => {
                    self.flush_paragraph(&mut text_buffer, parent_id);
                    self.advance();
                    let math_text = self.parse_inline_math();
                    self.emit_inline_math(parent_id, &math_text);
                }
                Token::DoubleDollar => {
                    self.flush_paragraph(&mut text_buffer, parent_id);
                    self.advance();
                    let math_text = self.parse_display_math_double_dollar();
                    self.emit_block(BlockType::Math, parent_id, None, &math_text);
                }
                Token::Text(t) => {
                    if t == "\n\n" {
                        self.flush_paragraph(&mut text_buffer, parent_id);
                    } else {
                        text_buffer.push_str(t);
                    }
                    self.advance();
                }
                Token::Comment(_) => {
                    self.advance();
                }
                Token::LineBreak => {
                    text_buffer.push('\n');
                    self.advance();
                }
                Token::Tilde => {
                    text_buffer.push('\u{00A0}');
                    self.advance();
                }
                Token::Caret => {
                    text_buffer.push('^');
                    self.advance();
                }
                Token::Underscore => {
                    text_buffer.push('_');
                    self.advance();
                }
                Token::BraceOpen | Token::BraceClose => {
                    self.advance();
                }
                Token::BracketOpen | Token::BracketClose => {
                    self.advance();
                }
                Token::Ampersand | Token::Hash => {
                    self.advance();
                }
            }
        }

        self.flush_paragraph(&mut text_buffer, parent_id);
    }

    fn flush_paragraph(&mut self, buffer: &mut String, parent_id: u32) {
        let trimmed = buffer.trim().to_string();
        if !trimmed.is_empty() {
            self.emit_block(BlockType::Paragraph, parent_id, None, &trimmed);
        }
        buffer.clear();
    }

    fn push_math_cmd(&mut self, math: &mut String) {
        let name = match self.peek() {
            Some(Token::ControlSequence(n)) => n.clone(),
            _ => return,
        };
        match name.as_str() {
            "begin" | "end" => {
                math.push_str(&format!("\\{name}"));
                self.advance();
                if let Some(&Token::BraceOpen) = self.peek() {
                    self.advance();
                    math.push('{');
                    if let Some(Token::Text(env)) = self.peek().cloned() {
                        math.push_str(&env);
                        self.advance();
                    }
                    if let Some(&Token::BraceClose) = self.peek() {
                        self.advance();
                        math.push('}');
                    }
                }
            }
            "left" | "right" => {
                math.push_str(&format!("\\{name}"));
                self.advance();
                if let Some(tok) = self.peek().cloned() {
                    match tok {
                        Token::BraceOpen => {
                            math.push('{');
                            self.advance();
                        }
                        Token::BraceClose => {
                            math.push('}');
                            self.advance();
                        }
                        Token::Text(t) => {
                            math.push_str(&t);
                            self.advance();
                        }
                        _ => {}
                    }
                }
            }
            "text" => {
                math.push_str("\\text");
                self.advance();
                if let Some(&Token::BraceOpen) = self.peek() {
                    let content = self.parse_group_content();
                    math.push_str(&format!("{{{}}}", content));
                }
            }
            "frac" => {
                self.advance();
                let num = self.parse_group_content();
                let den = self.parse_group_content();
                math.push_str(&format!("{num}/{den}"));
            }
            "sqrt" => {
                self.advance();
                let content = self.parse_group_content();
                math.push('\u{221A}');
                math.push_str(&content);
            }
            "label" => {
                self.advance();
                let key = self.parse_group_content();
                math.push_str(&format!("\\label{{{}}}", key));
            }
            _ => {
                if let Some(sym) = Self::substitute_math_cmd(&name) {
                    math.push_str(sym);
                } else {
                    math.push_str(&format!("\\{name}"));
                }
                self.advance();
            }
        }
    }

    fn parse_inline_math(&mut self) -> String {
        self.in_math = true;
        let mut math = String::new();
        while let Some(tok) = self.peek() {
            match tok {
                Token::DollarSign => {
                    self.advance();
                    break;
                }
                Token::ControlSequence(_) => {
                    self.push_math_cmd(&mut math);
                }
                Token::Text(t) => {
                    math.push_str(t);
                    self.advance();
                }
                Token::Caret => {
                    math.push('^');
                    self.advance();
                }
                Token::Underscore => {
                    math.push('_');
                    self.advance();
                }
                Token::BraceOpen => {
                    math.push('{');
                    self.advance();
                }
                Token::BraceClose => {
                    math.push('}');
                    self.advance();
                }
                Token::Ampersand => {
                    math.push_str(" & ");
                    self.advance();
                }
                Token::LineBreak => {
                    math.push_str(" \\\\ ");
                    self.advance();
                }
                _ => {
                    self.advance();
                }
            }
        }
        self.in_math = false;
        math.trim().to_string()
    }

    fn parse_display_math_double_dollar(&mut self) -> String {
        self.in_math = true;
        let mut math = String::new();
        while let Some(tok) = self.peek() {
            match tok {
                Token::DoubleDollar => {
                    self.advance();
                    break;
                }
                Token::ControlSequence(_) => {
                    self.push_math_cmd(&mut math);
                }
                Token::Text(t) => {
                    math.push_str(t);
                    self.advance();
                }
                Token::Caret => {
                    math.push('^');
                    self.advance();
                }
                Token::Underscore => {
                    math.push('_');
                    self.advance();
                }
                Token::BraceOpen => {
                    math.push('{');
                    self.advance();
                }
                Token::BraceClose => {
                    math.push('}');
                    self.advance();
                }
                Token::Ampersand => {
                    math.push_str(" & ");
                    self.advance();
                }
                Token::LineBreak => {
                    math.push_str(" \\\\ ");
                    self.advance();
                }
                _ => {
                    self.advance();
                }
            }
        }
        self.in_math = false;
        math.trim().to_string()
    }

    fn parse_environment(&mut self, name: &str, parent_id: u32) {
        match name {
            "itemize" | "enumerate" => {
                self.parse_list_env(parent_id);
            }
            "quote" | "abstract" => {
                let content = self.collect_env_content(name);
                self.emit_block(BlockType::BlockQuote, parent_id, None, &content);
            }
            "verbatim" => {
                let content = self.collect_verbatim_content();
                self.emit_block(BlockType::Code, parent_id, None, &content);
            }
            "equation" => {
                let math = self.collect_math_env_content("equation");
                self.emit_math_block(BlockType::Math, parent_id, true, &math);
            }
            "equation*" => {
                let math = self.collect_math_env_content("equation");
                self.emit_math_block(BlockType::Math, parent_id, false, &math);
            }
            "align" => {
                let math = self.collect_math_env_content("align");
                self.emit_math_block(BlockType::Math, parent_id, false, &math);
            }
            "align*" => {
                let math = self.collect_math_env_content("align");
                self.emit_math_block(BlockType::Math, parent_id, false, &math);
            }
            "figure" => {
                self.parse_figure_env(parent_id);
            }
            "table" => {
                self.parse_table_env(parent_id);
            }
            "tabular" => {
                self.parse_tabular_env(parent_id);
            }
            _ => {
                let span = self
                    .tokens
                    .get(self.pos)
                    .map(|st| st.span)
                    .unwrap_or_else(SourceSpan::unknown);
                self.warnings
                    .push((span, format!("unknown environment: \\begin{{{}}}", name)));
                self.skip_to_end(name);
            }
        }
    }

    fn is_end_env(&self, env_name: &str) -> bool {
        if let Some(Token::ControlSequence(name)) = self.token_at(self.pos)
            && name == "end"
            && let Some(Token::BraceOpen) = self.token_at(self.pos + 1)
            && let Some(Token::Text(env)) = self.token_at(self.pos + 2)
        {
            return env == env_name;
        }
        false
    }

    fn consume_end_env(&mut self, _env_name: &str) {
        self.advance(); // \end
        self.skip_group(); // {env_name}
    }

    fn parse_list_env(&mut self, parent_id: u32) {
        self.parse_list_env_inner(parent_id, 0);
    }

    fn parse_list_env_inner(&mut self, parent_id: u32, depth: usize) {
        let list_id = self.emit_block(BlockType::List, parent_id, None, "");
        let items = self.collect_list_items(depth);
        let content = items.join("\n");
        if !content.is_empty() {
            let content_id = self.next_entity_id();
            self.push_instr_with_payload(
                SIRInstruction::new(SIROpcode::SetContent, content_id, list_id, 0),
                content.as_bytes(),
            );
        }
    }

    fn collect_list_items(&mut self, depth: usize) -> Vec<String> {
        let mut item_buffer = String::new();
        let mut items: Vec<String> = Vec::new();

        while let Some(tok) = self.peek().cloned() {
            match &tok {
                Token::ControlSequence(name) if name == "end" => {
                    if self.is_end_env("itemize") || self.is_end_env("enumerate") {
                        if !item_buffer.trim().is_empty() {
                            items.push(Self::indent_text(item_buffer.trim(), depth));
                        }
                        self.consume_end_env("itemize");
                        return items;
                    }
                    item_buffer.push_str(&format!("\\{name}"));
                    self.advance();
                }
                Token::ControlSequence(name) if name == "item" => {
                    if !item_buffer.trim().is_empty() {
                        items.push(Self::indent_text(item_buffer.trim(), depth));
                        item_buffer.clear();
                    }
                    self.advance();
                    if let Some(&Token::BracketOpen) = self.peek() {
                        self.parse_optional();
                    }
                }
                Token::ControlSequence(name) if name == "begin" => {
                    if self.is_begin_env("itemize") || self.is_begin_env("enumerate") {
                        if !item_buffer.trim().is_empty() {
                            items.push(Self::indent_text(item_buffer.trim(), depth));
                            item_buffer.clear();
                        }
                        self.advance();
                        let _env = self.parse_group_content();
                        let nested = self.collect_list_items(depth + 1);
                        items.extend(nested);
                    } else {
                        item_buffer.push_str(&format!("\\{name}"));
                        self.advance();
                    }
                }
                Token::ControlSequence(name) => {
                    item_buffer.push_str(&format!("\\{name}"));
                    self.advance();
                }
                Token::Text(t) => {
                    item_buffer.push_str(t);
                    self.advance();
                }
                Token::Comment(_) => {
                    self.advance();
                }
                Token::LineBreak => {
                    item_buffer.push(' ');
                    self.advance();
                }
                Token::Tilde => {
                    item_buffer.push('\u{00A0}');
                    self.advance();
                }
                _ => {
                    self.advance();
                }
            }
        }

        items
    }

    fn indent_text(text: &str, depth: usize) -> String {
        if depth == 0 {
            text.to_string()
        } else {
            format!("{}{}", "  ".repeat(depth), text)
        }
    }

    fn parse_figure_env(&mut self, parent_id: u32) {
        let fig_id = self.emit_block(BlockType::Figure, parent_id, None, "");

        while let Some(tok) = self.peek().cloned() {
            match &tok {
                Token::ControlSequence(name) if name == "end" => {
                    if self.is_end_env("figure") {
                        self.consume_end_env("figure");
                        break;
                    }
                    self.advance();
                    self.skip_group();
                }
                Token::ControlSequence(name) if name == "includegraphics" => {
                    self.advance();
                    self.parse_optional();
                    let path = self.parse_group_content();
                    self.emit_block(BlockType::Image, fig_id, None, &path);
                }
                Token::ControlSequence(name) if name == "caption" => {
                    self.advance();
                    let caption = self.parse_group_content();
                    if !caption.is_empty() {
                        let content_id = self.next_entity_id();
                        self.push_instr_with_payload(
                            SIRInstruction::new(SIROpcode::SetContent, content_id, fig_id, 0),
                            caption.as_bytes(),
                        );
                    }
                }
                Token::ControlSequence(name) if name == "centering" => {
                    self.advance();
                }
                Token::ControlSequence(_) => {
                    self.advance();
                    self.skip_group();
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn parse_table_env(&mut self, parent_id: u32) {
        let table_id = self.emit_block(BlockType::Table, parent_id, None, "");

        while let Some(tok) = self.peek().cloned() {
            match &tok {
                Token::ControlSequence(name) if name == "end" => {
                    if self.is_end_env("table") {
                        self.consume_end_env("table");
                        break;
                    }
                    self.advance();
                    self.skip_group();
                }
                Token::ControlSequence(name) if name == "caption" => {
                    self.advance();
                    let caption = self.parse_group_content();
                    if !caption.is_empty() {
                        let content_id = self.next_entity_id();
                        self.push_instr_with_payload(
                            SIRInstruction::new(SIROpcode::SetContent, content_id, table_id, 0),
                            caption.as_bytes(),
                        );
                    }
                }
                Token::ControlSequence(name) if name == "centering" => {
                    self.advance();
                }
                Token::ControlSequence(name) if name == "begin" => {
                    if self.is_begin_env("tabular") {
                        self.advance();
                        self.parse_group_content();
                        self.parse_tabular_as_children(table_id);
                    } else {
                        self.advance();
                        self.skip_group();
                    }
                }
                Token::ControlSequence(_) => {
                    self.advance();
                    self.skip_group();
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn parse_tabular_as_children(&mut self, table_id: u32) {
        let preamble = if let Some(&Token::BraceOpen) = self.peek() {
            self.parse_group_content()
        } else {
            String::new()
        };

        let alignments: Vec<char> = preamble
            .chars()
            .filter(|c| matches!(c, 'l' | 'c' | 'r' | 'p'))
            .collect();

        let mut rows: Vec<Vec<String>> = Vec::new();
        let mut current_row: Vec<String> = Vec::new();
        let mut current_cell = String::new();
        let mut next_is_header = false;

        while let Some(tok) = self.peek().cloned() {
            match &tok {
                Token::ControlSequence(name) if name == "end" => {
                    if self.is_end_env("tabular") {
                        if !current_cell.trim().is_empty() {
                            current_row.push(current_cell.trim().to_string());
                        }
                        if !current_row.is_empty() {
                            rows.push(current_row);
                        }
                        self.consume_end_env("tabular");
                        break;
                    }
                    if name == "hline"
                        || name == "toprule"
                        || name == "midrule"
                        || name == "bottomrule"
                    {
                        if !current_cell.trim().is_empty() {
                            current_row.push(current_cell.trim().to_string());
                        }
                        if !current_row.is_empty() {
                            rows.push(current_row);
                        }
                        current_row = Vec::new();
                        current_cell.clear();
                        next_is_header = true;
                    } else {
                        current_cell.push_str(&format!("\\{name}"));
                    }
                    self.advance();
                }
                Token::ControlSequence(name) => {
                    current_cell.push_str(&format!("\\{name}"));
                    self.advance();
                }
                Token::Ampersand => {
                    current_row.push(current_cell.trim().to_string());
                    current_cell.clear();
                    self.advance();
                }
                Token::Text(t) => {
                    if t.contains("\\\\") {
                        let parts: Vec<&str> = t.split("\\\\").collect();
                        for (i, part) in parts.iter().enumerate() {
                            if i > 0 {
                                current_row.push(current_cell.trim().to_string());
                                if !current_row.is_empty() {
                                    rows.push(std::mem::take(&mut current_row));
                                }
                                current_cell.clear();
                            }
                            current_cell.push_str(part);
                        }
                    } else {
                        current_cell.push_str(t);
                    }
                    self.advance();
                }
                Token::LineBreak => {
                    current_cell.push(' ');
                    self.advance();
                }
                Token::Tilde => {
                    current_cell.push('\u{00A0}');
                    self.advance();
                }
                Token::Comment(_) => {
                    self.advance();
                }
                _ => {
                    self.advance();
                }
            }
        }

        let num_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);

        for (row_idx, row_cells) in rows.iter().enumerate() {
            let is_header = next_is_header && row_idx == 0;
            let row_id = self.emit_block(
                BlockType::TableRow,
                table_id,
                Some(&[if is_header { 1u32 } else { 0u32 }]),
                "",
            );
            for cell_text in row_cells {
                self.emit_block(BlockType::TableCell, row_id, None, cell_text);
            }
        }

        if num_cols > 0 {
            let num_cols_id = self.next_entity_id();
            self.push_instr_with_payload(
                SIRInstruction::new(SIROpcode::SetContent, num_cols_id, table_id, 0),
                num_cols.to_string().as_bytes(),
            );
        }

        if !alignments.is_empty() {
            let align_str: String = alignments.iter().collect();
            let align_id = self.next_entity_id();
            self.push_instr_with_payload(
                SIRInstruction::new(SIROpcode::SetContent, align_id, table_id, 0),
                align_str.as_bytes(),
            );
        }
    }

    fn collect_env_content(&mut self, env_name: &str) -> String {
        let mut content = String::new();
        while let Some(tok) = self.peek() {
            match tok {
                Token::ControlSequence(name) if name == "end" => {
                    if self.is_end_env(env_name) {
                        self.consume_end_env(env_name);
                        break;
                    }
                    content.push_str(&format!("\\{name}"));
                    self.advance();
                }
                Token::ControlSequence(name) => {
                    content.push_str(&format!("\\{name}"));
                    self.advance();
                }
                Token::Text(t) => {
                    content.push_str(t);
                    self.advance();
                }
                Token::LineBreak => {
                    content.push(' ');
                    self.advance();
                }
                Token::Tilde => {
                    content.push('\u{00A0}');
                    self.advance();
                }
                Token::Comment(_) => {
                    self.advance();
                }
                _ => {
                    self.advance();
                }
            }
        }
        content.trim().to_string()
    }

    fn parse_tabular_env(&mut self, parent_id: u32) {
        let table_id = self.emit_block(BlockType::Table, parent_id, None, "");
        self.parse_tabular_as_children(table_id);
    }

    fn collect_verbatim_content(&mut self) -> String {
        let mut content = String::new();
        while let Some(tok) = self.peek() {
            match tok {
                Token::ControlSequence(name) if name == "end" => {
                    if self.is_end_env("verbatim") {
                        self.consume_end_env("verbatim");
                        break;
                    }
                    content.push_str(&format!("\\{name}"));
                    self.advance();
                }
                Token::Text(t) => {
                    content.push_str(t);
                    self.advance();
                }
                Token::LineBreak => {
                    content.push('\n');
                    self.advance();
                }
                _ => {
                    content.push_str(&tok.to_string());
                    self.advance();
                }
            }
        }
        content
    }

    fn collect_math_env_content(&mut self, env_name: &str) -> String {
        self.in_math = true;
        let mut math = String::new();
        while let Some(tok) = self.peek() {
            match tok {
                Token::ControlSequence(name) if name == "end" => {
                    if self.is_end_env(env_name) {
                        self.consume_end_env(env_name);
                        break;
                    }
                    math.push_str(&format!("\\{name}"));
                    self.advance();
                }
                Token::ControlSequence(_) => {
                    self.push_math_cmd(&mut math);
                }
                Token::Text(t) => {
                    math.push_str(t);
                    self.advance();
                }
                Token::Caret => {
                    math.push('^');
                    self.advance();
                }
                Token::Underscore => {
                    math.push('_');
                    self.advance();
                }
                Token::BraceOpen => {
                    math.push('{');
                    self.advance();
                }
                Token::BraceClose => {
                    math.push('}');
                    self.advance();
                }
                Token::Ampersand => {
                    math.push_str(" & ");
                    self.advance();
                }
                Token::LineBreak => {
                    math.push_str(" \\\\ ");
                    self.advance();
                }
                _ => {
                    self.advance();
                }
            }
        }
        self.in_math = false;
        math.trim().to_string()
    }

    fn skip_to_end(&mut self, env_name: &str) {
        let mut depth = 1;
        while let Some(tok) = self.peek() {
            match tok {
                Token::ControlSequence(name) if name == "begin" => {
                    self.advance();
                    self.skip_group();
                    depth += 1;
                }
                Token::ControlSequence(name) if name == "end" => {
                    if self.is_end_env(env_name) {
                        depth -= 1;
                        if depth == 0 {
                            self.consume_end_env(env_name);
                            return;
                        }
                    }
                    self.advance();
                    self.skip_group();
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn substitute_math_cmd(cmd: &str) -> Option<&'static str> {
        match cmd {
            "alpha" => Some("α"),
            "beta" => Some("β"),
            "gamma" => Some("γ"),
            "delta" => Some("δ"),
            "epsilon" => Some("ε"),
            "zeta" => Some("ζ"),
            "eta" => Some("η"),
            "theta" => Some("θ"),
            "iota" => Some("ι"),
            "kappa" => Some("κ"),
            "lambda" => Some("λ"),
            "mu" => Some("μ"),
            "nu" => Some("ν"),
            "xi" => Some("ξ"),
            "pi" => Some("π"),
            "rho" => Some("ρ"),
            "sigma" => Some("σ"),
            "tau" => Some("τ"),
            "upsilon" => Some("υ"),
            "phi" => Some("φ"),
            "chi" => Some("χ"),
            "psi" => Some("ψ"),
            "omega" => Some("ω"),
            "Gamma" => Some("Γ"),
            "Delta" => Some("Δ"),
            "Theta" => Some("Θ"),
            "Lambda" => Some("Λ"),
            "Xi" => Some("Ξ"),
            "Pi" => Some("Π"),
            "Sigma" => Some("Σ"),
            "Phi" => Some("Φ"),
            "Psi" => Some("Ψ"),
            "Omega" => Some("Ω"),
            "sum" => Some("∑"),
            "prod" => Some("∏"),
            "int" => Some("∫"),
            "infty" => Some("∞"),
            "partial" => Some("∂"),
            "nabla" => Some("∇"),
            "leq" => Some("≤"),
            "geq" => Some("≥"),
            "neq" => Some("≠"),
            "approx" => Some("≈"),
            "times" => Some("×"),
            "div" => Some("÷"),
            "pm" => Some("±"),
            "mp" => Some("∓"),
            "cdot" => Some("·"),
            "ldots" => Some("…"),
            "cdots" => Some("⋯"),
            _ => None,
        }
    }

    fn emit_block(
        &mut self,
        block_type: BlockType,
        parent_id: u32,
        extra: Option<&[u32]>,
        content: &str,
    ) -> u32 {
        let block_id = self.next_entity_id();
        let mut payload = vec![block_type as u8];
        if let Some(extra) = extra {
            for &val in extra {
                payload.extend_from_slice(&val.to_le_bytes());
            }
        }
        let payload_offset = self.doc.payload_mut().append(&payload);
        self.push_instr(SIRInstruction::new(
            SIROpcode::PushBlock,
            block_id,
            parent_id,
            payload_offset,
        ));
        if !content.is_empty() {
            let content_id = self.next_entity_id();
            self.push_instr_with_payload(
                SIRInstruction::new(SIROpcode::SetContent, content_id, block_id, 0),
                content.as_bytes(),
            );
        }
        block_id
    }

    fn emit_math_block(
        &mut self,
        block_type: BlockType,
        parent_id: u32,
        numbered: bool,
        content: &str,
    ) -> u32 {
        let block_id = self.next_entity_id();
        let mut payload = vec![block_type as u8];
        payload.push(if numbered { 1u8 } else { 0u8 });
        let payload_offset = self.doc.payload_mut().append(&payload);
        self.push_instr(SIRInstruction::new(
            SIROpcode::PushBlock,
            block_id,
            parent_id,
            payload_offset,
        ));
        if !content.is_empty() {
            let content_id = self.next_entity_id();
            self.push_instr_with_payload(
                SIRInstruction::new(SIROpcode::SetContent, content_id, block_id, 0),
                content.as_bytes(),
            );
        }
        block_id
    }

    fn emit_heading(&mut self, parent_id: u32, level: u32, title: &str) {
        self.emit_block(BlockType::Heading, parent_id, Some(&[level]), title);
    }

    fn emit_styled_text(
        &mut self,
        parent_id: u32,
        text_buffer: &mut String,
        modifier: StyleModifier,
        content: &str,
    ) {
        self.flush_paragraph(text_buffer, parent_id);
        let para_id = self.emit_block(BlockType::Paragraph, parent_id, None, content);
        let _ = self.emit_style(para_id, modifier, true);
        let _ = self.emit_style(para_id, modifier, false);
    }

    fn emit_style(&mut self, parent_id: u32, modifier: StyleModifier, is_push: bool) -> u32 {
        let id = self.next_entity_id();
        let packed = if is_push {
            StyleModifier::push(modifier)
        } else {
            StyleModifier::pop()
        };
        self.push_instr(SIRInstruction::new(
            SIROpcode::ApplyStyle,
            id,
            parent_id,
            packed,
        ));
        id
    }

    fn emit_link(&mut self, parent_id: u32, url: &str) -> u32 {
        let id = self.next_entity_id();
        let mut url_bytes = url.to_string().into_bytes();
        url_bytes.push(0);
        self.push_instr_with_payload(
            SIRInstruction::new(SIROpcode::LinkData, id, parent_id, 0),
            &url_bytes,
        );
        id
    }

    fn emit_inline_math(&mut self, parent_id: u32, math_text: &str) {
        let para_id = self.emit_block(BlockType::Paragraph, parent_id, None, math_text);
        let _ = self.emit_style(para_id, StyleModifier::MONO_STYLE, true);
        let _ = self.emit_style(para_id, StyleModifier::MONO_STYLE, false);
    }
}
