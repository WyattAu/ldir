use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    ControlSequence(String),
    BraceOpen,
    BraceClose,
    BracketOpen,
    BracketClose,
    DollarSign,
    DoubleDollar,
    Comment(String),
    Ampersand,
    Caret,
    Underscore,
    LineBreak,
    Tilde,
    Hash,
    Text(String),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::ControlSequence(name) => write!(f, "\\{name}"),
            Token::BraceOpen => write!(f, "{{"),
            Token::BraceClose => write!(f, "}}"),
            Token::BracketOpen => write!(f, "["),
            Token::BracketClose => write!(f, "]"),
            Token::DollarSign => write!(f, "$"),
            Token::DoubleDollar => write!(f, "$$"),
            Token::Comment(text) => write!(f, "%{text}"),
            Token::Ampersand => write!(f, "&"),
            Token::Caret => write!(f, "^"),
            Token::Underscore => write!(f, "_"),
            Token::LineBreak => write!(f, "\\\\"),
            Token::Tilde => write!(f, "~"),
            Token::Hash => write!(f, "#"),
            Token::Text(text) => write!(f, "{text}"),
        }
    }
}

fn is_letter(c: char) -> bool {
    c.is_ascii_alphabetic()
}

pub struct TeXLexer<'a> {
    input: &'a str,
    #[allow(dead_code)]
    pos: usize,
}

impl<'a> TeXLexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = self.input.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            let c = chars[i];

            match c {
                '\\' => {
                    if i + 1 < len && chars[i + 1] == '\\' {
                        tokens.push(Token::LineBreak);
                        i += 2;
                    } else {
                        i += 1;
                        if i < len {
                            let next = chars[i];
                            if is_letter(next) {
                                let mut name = String::new();
                                while i < len && is_letter(chars[i]) {
                                    name.push(chars[i]);
                                    i += 1;
                                }
                                tokens.push(Token::ControlSequence(name));
                            } else {
                                i += 1;
                                let tok = match next {
                                    '{' => Token::BraceOpen,
                                    '}' => Token::BraceClose,
                                    '$' => Token::DollarSign,
                                    '%' => Token::Text("%".into()),
                                    '&' => Token::Ampersand,
                                    '#' => Token::Hash,
                                    '_' => Token::Underscore,
                                    '^' => Token::Caret,
                                    '~' => Token::Tilde,
                                    ' ' => Token::Text(" ".into()),
                                    '\n' | '\r' => Token::Text(" ".into()),
                                    _ => Token::Text(next.to_string()),
                                };
                                tokens.push(tok);
                            }
                        }
                    }
                }
                '{' => {
                    tokens.push(Token::BraceOpen);
                    i += 1;
                }
                '}' => {
                    tokens.push(Token::BraceClose);
                    i += 1;
                }
                '[' => {
                    tokens.push(Token::BracketOpen);
                    i += 1;
                }
                ']' => {
                    tokens.push(Token::BracketClose);
                    i += 1;
                }
                '$' => {
                    if i + 1 < len && chars[i + 1] == '$' {
                        tokens.push(Token::DoubleDollar);
                        i += 2;
                    } else {
                        tokens.push(Token::DollarSign);
                        i += 1;
                    }
                }
                '%' => {
                    i += 1;
                    let mut comment = String::new();
                    while i < len && chars[i] != '\n' {
                        comment.push(chars[i]);
                        i += 1;
                    }
                    if i < len && chars[i] == '\n' {
                        i += 1;
                    }
                    tokens.push(Token::Comment(comment));
                }
                '&' => {
                    tokens.push(Token::Ampersand);
                    i += 1;
                }
                '^' => {
                    tokens.push(Token::Caret);
                    i += 1;
                }
                '_' => {
                    tokens.push(Token::Underscore);
                    i += 1;
                }
                '~' => {
                    tokens.push(Token::Tilde);
                    i += 1;
                }
                '#' => {
                    tokens.push(Token::Hash);
                    i += 1;
                }
                '\n' => {
                    i += 1;
                    if i < len && chars[i] == '\n' {
                        tokens.push(Token::Text("\n\n".into()));
                        i += 1;
                        while i < len && chars[i] == '\n' {
                            i += 1;
                        }
                    } else {
                        tokens.push(Token::Text(" ".into()));
                    }
                }
                ' ' | '\t' => {
                    tokens.push(Token::Text(" ".into()));
                    i += 1;
                    while i < len && (chars[i] == ' ' || chars[i] == '\t') {
                        i += 1;
                    }
                }
                _ => {
                    let mut text = String::new();
                    while i < len {
                        let ch = chars[i];
                        if ch == '\\'
                            || ch == '{'
                            || ch == '}'
                            || ch == '['
                            || ch == ']'
                            || ch == '$'
                            || ch == '%'
                            || ch == '&'
                            || ch == '^'
                            || ch == '_'
                            || ch == '~'
                            || ch == '#'
                            || ch == '\n'
                        {
                            break;
                        }
                        if ch == ' ' || ch == '\t' {
                            break;
                        }
                        text.push(ch);
                        i += 1;
                    }
                    if !text.is_empty() {
                        tokens.push(Token::Text(text));
                    }
                }
            }
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let tokens = TeXLexer::new("").tokenize();
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_control_sequence() {
        let tokens = TeXLexer::new(r"\section{Hello}").tokenize();
        assert_eq!(tokens[0], Token::ControlSequence("section".into()));
        assert_eq!(tokens[1], Token::BraceOpen);
        assert_eq!(tokens[2], Token::Text("Hello".into()));
        assert_eq!(tokens[3], Token::BraceClose);
    }

    #[test]
    fn test_escaped_braces() {
        let tokens = TeXLexer::new(r"\{ \}").tokenize();
        assert_eq!(tokens[0], Token::BraceOpen);
        assert_eq!(tokens[1], Token::Text(" ".into()));
        assert_eq!(tokens[2], Token::BraceClose);
    }

    #[test]
    fn test_double_backslash() {
        let tokens = TeXLexer::new(r"\\newline").tokenize();
        assert_eq!(tokens[0], Token::LineBreak);
        assert_eq!(tokens[1], Token::Text("newline".into()));
    }

    #[test]
    fn test_control_space() {
        let tokens = TeXLexer::new(r"\ ").tokenize();
        assert_eq!(tokens[0], Token::Text(" ".into()));
    }

    #[test]
    fn test_dollar_signs() {
        let tokens = TeXLexer::new(r"$x$ and $$y$$").tokenize();
        assert_eq!(tokens[0], Token::DollarSign);
        assert_eq!(tokens[1], Token::Text("x".into()));
        assert_eq!(tokens[2], Token::DollarSign);
        assert_eq!(tokens[3], Token::Text(" ".into()));
        assert_eq!(tokens[4], Token::Text("and".into()));
        assert_eq!(tokens[5], Token::Text(" ".into()));
        assert_eq!(tokens[6], Token::DoubleDollar);
        assert_eq!(tokens[7], Token::Text("y".into()));
        assert_eq!(tokens[8], Token::DoubleDollar);
    }

    #[test]
    fn test_comment() {
        let tokens = TeXLexer::new("hello% this is a comment\nworld").tokenize();
        assert_eq!(tokens[0], Token::Text("hello".into()));
        assert_eq!(tokens[1], Token::Comment(" this is a comment".into()));
        assert_eq!(tokens[2], Token::Text("world".into()));
    }

    #[test]
    fn test_paragraph_break() {
        let tokens = TeXLexer::new("one\n\ntwo").tokenize();
        assert_eq!(tokens[0], Token::Text("one".into()));
        assert_eq!(tokens[1], Token::Text("\n\n".into()));
        assert_eq!(tokens[2], Token::Text("two".into()));
    }

    #[test]
    fn test_special_chars() {
        let tokens = TeXLexer::new("&^_~#").tokenize();
        assert_eq!(tokens[0], Token::Ampersand);
        assert_eq!(tokens[1], Token::Caret);
        assert_eq!(tokens[2], Token::Underscore);
        assert_eq!(tokens[3], Token::Tilde);
        assert_eq!(tokens[4], Token::Hash);
    }
}
