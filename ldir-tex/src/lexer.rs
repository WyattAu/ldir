use std::fmt;

use ldir_ir::sir::v2::SourceSpan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpannedToken {
    pub token: Token,
    pub span: SourceSpan,
}

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
}

impl<'a> TeXLexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input }
    }

    #[cfg(test)]
    pub fn tokenize(&mut self) -> Vec<Token> {
        self.tokenize_with_spans()
            .into_iter()
            .map(|st| st.token)
            .collect()
    }

    pub fn tokenize_with_spans(&mut self) -> Vec<SpannedToken> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = self.input.chars().collect();
        let byte_offsets: Vec<u32> = {
            let mut v = Vec::with_capacity(chars.len());
            let mut off = 0u32;
            for &c in &chars {
                v.push(off);
                off += c.len_utf8() as u32;
            }
            v
        };
        let total_bytes = self.input.len() as u32;
        let len = chars.len();
        let mut i = 0;
        let mut line = 1u32;
        let mut col = 1u32;

        let end_byte = |end_i: usize| -> u32 {
            if end_i < len {
                byte_offsets[end_i]
            } else {
                total_bytes
            }
        };

        let make_span = |s_line: u32, s_col: u32, s_byte: u32, e_i: usize| -> SourceSpan {
            let e_byte = end_byte(e_i);
            SourceSpan::new(s_line, s_col, s_byte, e_byte - s_byte)
        };

        while i < len {
            let start_i = i;
            let start_line = line;
            let start_col = col;
            let start_byte = byte_offsets[i];
            let c = chars[i];

            match c {
                '\\' => {
                    if i + 1 < len && chars[i + 1] == '\\' {
                        let span = make_span(start_line, start_col, start_byte, i + 2);
                        tokens.push(SpannedToken {
                            token: Token::LineBreak,
                            span,
                        });
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
                                let span = make_span(start_line, start_col, start_byte, i);
                                tokens.push(SpannedToken {
                                    token: Token::ControlSequence(name),
                                    span,
                                });
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
                                let span = make_span(start_line, start_col, start_byte, i);
                                tokens.push(SpannedToken { token: tok, span });
                            }
                        }
                    }
                }
                '{' => {
                    let span = make_span(start_line, start_col, start_byte, i + 1);
                    tokens.push(SpannedToken {
                        token: Token::BraceOpen,
                        span,
                    });
                    i += 1;
                }
                '}' => {
                    let span = make_span(start_line, start_col, start_byte, i + 1);
                    tokens.push(SpannedToken {
                        token: Token::BraceClose,
                        span,
                    });
                    i += 1;
                }
                '[' => {
                    let span = make_span(start_line, start_col, start_byte, i + 1);
                    tokens.push(SpannedToken {
                        token: Token::BracketOpen,
                        span,
                    });
                    i += 1;
                }
                ']' => {
                    let span = make_span(start_line, start_col, start_byte, i + 1);
                    tokens.push(SpannedToken {
                        token: Token::BracketClose,
                        span,
                    });
                    i += 1;
                }
                '$' => {
                    if i + 1 < len && chars[i + 1] == '$' {
                        let span = make_span(start_line, start_col, start_byte, i + 2);
                        tokens.push(SpannedToken {
                            token: Token::DoubleDollar,
                            span,
                        });
                        i += 2;
                    } else {
                        let span = make_span(start_line, start_col, start_byte, i + 1);
                        tokens.push(SpannedToken {
                            token: Token::DollarSign,
                            span,
                        });
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
                    let span = make_span(start_line, start_col, start_byte, i);
                    tokens.push(SpannedToken {
                        token: Token::Comment(comment),
                        span,
                    });
                }
                '&' => {
                    let span = make_span(start_line, start_col, start_byte, i + 1);
                    tokens.push(SpannedToken {
                        token: Token::Ampersand,
                        span,
                    });
                    i += 1;
                }
                '^' => {
                    let span = make_span(start_line, start_col, start_byte, i + 1);
                    tokens.push(SpannedToken {
                        token: Token::Caret,
                        span,
                    });
                    i += 1;
                }
                '_' => {
                    let span = make_span(start_line, start_col, start_byte, i + 1);
                    tokens.push(SpannedToken {
                        token: Token::Underscore,
                        span,
                    });
                    i += 1;
                }
                '~' => {
                    let span = make_span(start_line, start_col, start_byte, i + 1);
                    tokens.push(SpannedToken {
                        token: Token::Tilde,
                        span,
                    });
                    i += 1;
                }
                '#' => {
                    let span = make_span(start_line, start_col, start_byte, i + 1);
                    tokens.push(SpannedToken {
                        token: Token::Hash,
                        span,
                    });
                    i += 1;
                }
                '\n' => {
                    i += 1;
                    if i < len && chars[i] == '\n' {
                        let span = make_span(start_line, start_col, start_byte, i + 1);
                        i += 1;
                        while i < len && chars[i] == '\n' {
                            i += 1;
                        }
                        tokens.push(SpannedToken {
                            token: Token::Text("\n\n".into()),
                            span,
                        });
                    } else {
                        let span = make_span(start_line, start_col, start_byte, i);
                        tokens.push(SpannedToken {
                            token: Token::Text(" ".into()),
                            span,
                        });
                    }
                }
                ' ' | '\t' => {
                    i += 1;
                    while i < len && (chars[i] == ' ' || chars[i] == '\t') {
                        i += 1;
                    }
                    let span = make_span(start_line, start_col, start_byte, i);
                    tokens.push(SpannedToken {
                        token: Token::Text(" ".into()),
                        span,
                    });
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
                        let span = make_span(start_line, start_col, start_byte, i);
                        tokens.push(SpannedToken {
                            token: Token::Text(text),
                            span,
                        });
                    }
                }
            }

            for ch in &chars[start_i..i] {
                if *ch == '\n' {
                    line += 1;
                    col = 1;
                } else {
                    col += 1;
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
