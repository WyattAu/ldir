//! Math layout engine (Phase 2 + Phase 6A: Advanced Math Environments).
//!
//! Parses math content strings into tokens, builds a layout tree,
//! and produces positioned glyph commands for display and inline math.
//!
//! Supports: subscripts (`_`), superscripts (`^`), fractions (`\frac{}{}`),
//! radicals (`\sqrt{}`), big operators (`\sum`, `\int`, `\prod`),
//! binary operators, delimiters, `\begin{cases}`, matrix environments,
//! and stretchy delimiters (`\left`/`\right`).

use std::sync::Arc;

use crate::fp266::Fp266;
use crate::shaping::ShapedGlyph;

#[derive(Debug, Clone, PartialEq, Eq)]
enum MathToken {
    Text(String),
    Subscript(String),
    Superscript(String),
    Fraction(String, String),
    Radical(Box<MathToken>),
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    BigOp(String),
    BinOp(String),
    BeginCases,
    EndCases,
    BeginMatrix {
        left: Option<char>,
        right: Option<char>,
    },
    EndMatrix,
    LeftDelim(char),
    RightDelim(char),
    Ampersand,
    RowSeparator,
}

#[derive(Debug, Clone)]
enum MathNode {
    Text(String),
    Subscript {
        base: String,
        sub: String,
    },
    Superscript {
        base: String,
        sup: String,
    },
    Fraction {
        num: String,
        den: String,
    },
    Radical {
        inner: Box<MathToken>,
    },
    BigOp(String),
    BinOp(String),
    Delimiter(char),
    Cases {
        rows: Vec<Vec<Vec<MathNode>>>,
    },
    Matrix {
        rows: Vec<Vec<Vec<MathNode>>>,
        delimiters: (Option<char>, Option<char>),
    },
    StretchyGroup {
        left: Option<char>,
        content: Vec<MathNode>,
        right: Option<char>,
    },
}

#[derive(Debug, Clone, Default)]
struct MathBox {
    width: Fp266,
    height: Fp266,
    depth: Fp266,
}

impl MathBox {
    fn zero() -> Self {
        Self {
            width: Fp266::ZERO,
            height: Fp266::ZERO,
            depth: Fp266::ZERO,
        }
    }
}

pub struct PositionedGlyph {
    pub glyph_id: i32,
    pub x: Fp266,
    pub y: Fp266,
    pub advance: Fp266,
}

fn parse_braced_arg(chars: &[char], pos: &mut usize) -> String {
    if *pos >= chars.len() || chars[*pos] != '{' {
        return String::new();
    }
    *pos += 1;
    let start = *pos;
    let mut depth = 1usize;
    while *pos < chars.len() && depth > 0 {
        if chars[*pos] == '{' {
            depth += 1;
        } else if chars[*pos] == '}' {
            depth -= 1;
        }
        if depth > 0 {
            *pos += 1;
        }
    }
    let result: String = chars[start..*pos].iter().collect();
    if *pos < chars.len() {
        *pos += 1;
    }
    result
}

fn parse_single_arg(chars: &[char], pos: &mut usize) -> String {
    if *pos >= chars.len() {
        return String::new();
    }
    if chars[*pos] == '{' {
        return parse_braced_arg(chars, pos);
    }
    let ch = chars[*pos];
    *pos += 1;
    ch.to_string()
}

fn tokenize_math(content: &str) -> Vec<MathToken> {
    let chars: Vec<char> = content.chars().collect();
    let mut tokens = Vec::new();
    let mut pos = 0;
    let len = chars.len();

    while pos < len {
        if chars[pos].is_whitespace() {
            pos += 1;
            continue;
        }

        if chars[pos] == '\\' {
            if pos + 1 < len && chars[pos + 1] == '\\' {
                tokens.push(MathToken::RowSeparator);
                pos += 2;
                continue;
            }
            let cmd_start = pos + 1;
            pos = cmd_start;
            while pos < len && chars[pos].is_alphabetic() {
                pos += 1;
            }
            let cmd: String = chars[cmd_start..pos].iter().collect();

            match cmd.as_str() {
                "frac" => {
                    let num = parse_braced_arg(&chars, &mut pos);
                    let den = parse_braced_arg(&chars, &mut pos);
                    tokens.push(MathToken::Fraction(num, den));
                }
                "sqrt" => {
                    let inner = parse_braced_arg(&chars, &mut pos);
                    if inner.is_empty() {
                        if pos < len {
                            let ch = chars[pos];
                            pos += 1;
                            tokens.push(MathToken::Radical(Box::new(MathToken::Text(
                                ch.to_string(),
                            ))));
                        }
                    } else {
                        let inner_tokens = tokenize_math(&inner);
                        if inner_tokens.len() == 1 {
                            tokens.push(MathToken::Radical(Box::new(
                                inner_tokens.into_iter().next().unwrap(),
                            )));
                        } else {
                            tokens.push(MathToken::Radical(Box::new(MathToken::Text(inner))));
                        }
                    }
                }
                "sum" | "int" | "prod" => {
                    tokens.push(MathToken::BigOp(cmd));
                }
                "left" => {
                    if pos < len {
                        let delim = chars[pos];
                        pos += 1;
                        tokens.push(MathToken::LeftDelim(delim));
                    }
                }
                "right" => {
                    if pos < len {
                        let delim = chars[pos];
                        pos += 1;
                        tokens.push(MathToken::RightDelim(delim));
                    }
                }
                "begin" => {
                    if pos < len && chars[pos] == '{' {
                        pos += 1;
                        let env_start = pos;
                        while pos < len && chars[pos] != '}' {
                            pos += 1;
                        }
                        let env: String = chars[env_start..pos].iter().collect();
                        if pos < len {
                            pos += 1;
                        }
                        match env.as_str() {
                            "cases" => tokens.push(MathToken::BeginCases),
                            "matrix" => tokens.push(MathToken::BeginMatrix {
                                left: None,
                                right: None,
                            }),
                            "pmatrix" => tokens.push(MathToken::BeginMatrix {
                                left: Some('('),
                                right: Some(')'),
                            }),
                            "bmatrix" => tokens.push(MathToken::BeginMatrix {
                                left: Some('['),
                                right: Some(']'),
                            }),
                            "vmatrix" => tokens.push(MathToken::BeginMatrix {
                                left: Some('|'),
                                right: Some('|'),
                            }),
                            "Vmatrix" => tokens.push(MathToken::BeginMatrix {
                                left: Some('\u{2016}'),
                                right: Some('\u{2016}'),
                            }),
                            _ => {
                                tokens.push(MathToken::Text(format!("\\begin{{{env}}}")));
                            }
                        }
                    } else {
                        tokens.push(MathToken::Text("\\begin".to_string()));
                    }
                }
                "end" => {
                    if pos < len && chars[pos] == '{' {
                        pos += 1;
                        let env_start = pos;
                        while pos < len && chars[pos] != '}' {
                            pos += 1;
                        }
                        let env: String = chars[env_start..pos].iter().collect();
                        if pos < len {
                            pos += 1;
                        }
                        match env.as_str() {
                            "cases" => tokens.push(MathToken::EndCases),
                            "matrix" | "pmatrix" | "bmatrix" | "vmatrix" | "Vmatrix" => {
                                tokens.push(MathToken::EndMatrix)
                            }
                            _ => {
                                tokens.push(MathToken::Text(format!("\\end{{{env}}}")));
                            }
                        }
                    } else {
                        tokens.push(MathToken::Text("\\end".to_string()));
                    }
                }
                "text" => {
                    let content = parse_braced_arg(&chars, &mut pos);
                    tokens.push(MathToken::Text(content));
                }
                "pm" => tokens.push(MathToken::BinOp("±".to_string())),
                "times" => tokens.push(MathToken::BinOp("×".to_string())),
                "div" => tokens.push(MathToken::BinOp("÷".to_string())),
                "leq" => tokens.push(MathToken::BinOp("≤".to_string())),
                "geq" => tokens.push(MathToken::BinOp("≥".to_string())),
                "neq" => tokens.push(MathToken::BinOp("≠".to_string())),
                "approx" => tokens.push(MathToken::BinOp("≈".to_string())),
                "infty" => tokens.push(MathToken::Text("∞".to_string())),
                _ => {
                    let display = match cmd.as_str() {
                        "alpha" => "α",
                        "beta" => "β",
                        "gamma" => "γ",
                        "delta" => "δ",
                        "epsilon" => "ε",
                        "zeta" => "ζ",
                        "eta" => "η",
                        "theta" => "θ",
                        "iota" => "ι",
                        "kappa" => "κ",
                        "lambda" => "λ",
                        "mu" => "μ",
                        "nu" => "ν",
                        "xi" => "ξ",
                        "pi" => "π",
                        "rho" => "ρ",
                        "sigma" => "σ",
                        "tau" => "τ",
                        "upsilon" => "υ",
                        "phi" => "φ",
                        "chi" => "χ",
                        "psi" => "ψ",
                        "omega" => "ω",
                        "Gamma" => "Γ",
                        "Delta" => "Δ",
                        "Theta" => "Θ",
                        "Lambda" => "Λ",
                        "Xi" => "Ξ",
                        "Pi" => "Π",
                        "Sigma" => "Σ",
                        "Phi" => "Φ",
                        "Psi" => "Ψ",
                        "Omega" => "Ω",
                        other => other,
                    };
                    if display == cmd.as_str() {
                        tokens.push(MathToken::Text(format!("\\{}", cmd)));
                    } else {
                        tokens.push(MathToken::Text(display.to_string()));
                    }
                }
            }
        } else if chars[pos] == '^' {
            pos += 1;
            let sup_arg = parse_single_arg(&chars, &mut pos);
            tokens.push(MathToken::Superscript(sup_arg));
        } else if chars[pos] == '_' {
            pos += 1;
            let sub_arg = parse_single_arg(&chars, &mut pos);
            tokens.push(MathToken::Subscript(sub_arg));
        } else if chars[pos] == '(' {
            tokens.push(MathToken::LeftParen);
            pos += 1;
        } else if chars[pos] == ')' {
            tokens.push(MathToken::RightParen);
            pos += 1;
        } else if chars[pos] == '[' {
            tokens.push(MathToken::LeftBracket);
            pos += 1;
        } else if chars[pos] == ']' {
            tokens.push(MathToken::RightBracket);
            pos += 1;
        } else if chars[pos] == '&' {
            tokens.push(MathToken::Ampersand);
            pos += 1;
        } else if "+-×÷±=<>≤≥≠≈".contains(chars[pos]) {
            tokens.push(MathToken::BinOp(chars[pos].to_string()));
            pos += 1;
        } else {
            let start = pos;
            while pos < len
                && !chars[pos].is_whitespace()
                && chars[pos] != '^'
                && chars[pos] != '_'
                && chars[pos] != '\\'
                && chars[pos] != '('
                && chars[pos] != ')'
                && chars[pos] != '['
                && chars[pos] != ']'
                && chars[pos] != '&'
                && !"+-×÷±=<>≤≥≠≈".contains(chars[pos])
            {
                pos += 1;
            }
            let text: String = chars[start..pos].iter().collect();
            if !text.is_empty() {
                tokens.push(MathToken::Text(text));
            }
        }
    }

    tokens
}

fn shape_text_run(
    text: &str,
    font_size: Fp266,
    font_data: Option<&[u8]>,
) -> (Vec<PositionedGlyph>, MathBox) {
    if text.is_empty() {
        return (Vec::new(), MathBox::zero());
    }

    let raw_glyphs: Vec<ShapedGlyph> = if let Some(data) = font_data {
        crate::shaping::shape_text(data, text, font_size).glyphs
    } else {
        crate::shaping::fast_path::shape_unicode_basic(&[], text, font_size, 0).glyphs
    };

    let mut cursor_x = Fp266::ZERO;
    let mut glyphs = Vec::with_capacity(raw_glyphs.len());
    for g in &raw_glyphs {
        let x = cursor_x + g.x_offset;
        let y = g.y_offset;
        glyphs.push(PositionedGlyph {
            glyph_id: g.glyph_id as i32,
            x,
            y,
            advance: g.advance,
        });
        cursor_x += g.advance;
    }

    let total_width = cursor_x;
    let height = Fp266::from_frac(font_size.to_int() * 8, 10);
    let depth = Fp266::from_frac(font_size.to_int() * 2, 10);

    (
        glyphs,
        MathBox {
            width: total_width,
            height,
            depth,
        },
    )
}

struct LayoutState {
    glyphs: Vec<PositionedGlyph>,
    cursor_x: Fp266,
    base_y: Fp266,
    font_size: Fp266,
    font_data: Option<Arc<Vec<u8>>>,
    pending_base: String,
}

impl LayoutState {
    fn new(font_size: Fp266, base_y: Fp266, font_data: Option<Arc<Vec<u8>>>) -> Self {
        Self {
            glyphs: Vec::new(),
            cursor_x: Fp266::ZERO,
            base_y,
            font_size,
            font_data,
            pending_base: String::new(),
        }
    }

    fn font_data_ref(&self) -> Option<&[u8]> {
        self.font_data.as_deref().map(|v| v.as_slice())
    }

    fn measure_state(&self) -> Self {
        Self::new(self.font_size, Fp266::ZERO, self.font_data.clone())
    }

    fn emit_text(&mut self, text: &str) -> MathBox {
        let (mut glyphs, box_info) = shape_text_run(text, self.font_size, self.font_data_ref());
        for g in &mut glyphs {
            g.x += self.cursor_x;
            g.y = self.base_y;
        }
        let width = box_info.width;
        self.cursor_x += width;
        self.glyphs.extend(glyphs);
        self.pending_base = text.to_string();
        box_info
    }

    fn emit_superscript(&mut self, base: &str, sup: &str) -> MathBox {
        let base_box = self.emit_text(base);
        let sup_size = Fp266::from_frac(self.font_size.to_int() * 7, 10);
        let saved_font_size = self.font_size;
        let saved_x = self.cursor_x;

        self.font_size = sup_size;
        let sup_y = self.base_y + Fp266::from_frac(saved_font_size.to_int() * 6, 10);
        let saved_base_y = self.base_y;
        self.base_y = sup_y;
        self.cursor_x = saved_x + base_box.width + Fp266::from_int(1);

        let sup_box = self.emit_text(sup);

        self.font_size = saved_font_size;
        self.base_y = saved_base_y;
        self.cursor_x = saved_x;

        let width = base_box.width + Fp266::from_int(1) + sup_box.width;
        let height = base_box.height + Fp266::from_frac(saved_font_size.to_int() * 4, 10);

        MathBox {
            width,
            height,
            depth: base_box.depth,
        }
    }

    fn emit_subscript(&mut self, base: &str, sub: &str) -> MathBox {
        let base_box = self.emit_text(base);
        let sub_size = Fp266::from_frac(self.font_size.to_int() * 7, 10);
        let saved_font_size = self.font_size;
        let saved_x = self.cursor_x;

        self.font_size = sub_size;
        let sub_y = self.base_y + Fp266::from_frac(saved_font_size.to_int() * 2, 10);
        let saved_base_y = self.base_y;
        self.base_y = sub_y;
        self.cursor_x = saved_x + base_box.width + Fp266::from_int(1);

        let sub_box = self.emit_text(sub);

        self.font_size = saved_font_size;
        self.base_y = saved_base_y;
        self.cursor_x = saved_x;

        let width = base_box.width + Fp266::from_int(1) + sub_box.width;
        let depth = base_box.depth + Fp266::from_frac(saved_font_size.to_int() * 2, 10);

        MathBox {
            width,
            height: base_box.height,
            depth,
        }
    }

    fn emit_fraction(&mut self, numerator: &str, denominator: &str) -> MathBox {
        let saved_x = self.cursor_x;
        let saved_base_y = self.base_y;

        let rule_thickness = Fp266::from_frac(self.font_size.to_int(), 16);
        let rule_gap = Fp266::from_frac(self.font_size.to_int(), 6);

        let num_size = Fp266::from_frac(self.font_size.to_int() * 8, 10);
        let den_size = Fp266::from_frac(self.font_size.to_int() * 8, 10);
        let saved_font_size = self.font_size;

        self.font_size = num_size;
        let (num_glyphs, num_box) = shape_text_run(numerator, num_size, self.font_data_ref());
        self.font_size = den_size;
        let (den_glyphs, den_box) = shape_text_run(denominator, den_size, self.font_data_ref());

        self.font_size = saved_font_size;

        let content_width = num_box.width.max(den_box.width);
        let total_width = content_width + Fp266::from_int(8);

        let num_x_offset = (total_width - num_box.width).div(Fp266::from_int(2));
        let den_x_offset = (total_width - den_box.width).div(Fp266::from_int(2));

        let rule_y = saved_base_y - rule_thickness;
        let num_y = rule_y + rule_gap + num_box.height;
        let den_y = rule_y - rule_gap - den_box.height + den_box.depth;

        for mut g in num_glyphs {
            g.x += saved_x + num_x_offset;
            g.y = num_y;
            self.glyphs.push(g);
        }

        for mut g in den_glyphs {
            g.x += saved_x + den_x_offset;
            g.y = den_y;
            self.glyphs.push(g);
        }

        let rule_x =
            saved_x + (total_width - num_box.width.max(den_box.width)).div(Fp266::from_int(2));
        let rule_width = num_box.width.max(den_box.width);
        self.glyphs.push(PositionedGlyph {
            glyph_id: -1,
            x: rule_x,
            y: rule_y,
            advance: rule_width,
        });

        self.cursor_x = saved_x + total_width;

        let height = num_box.height + rule_gap + rule_thickness;
        let depth = den_box.height - den_box.depth + rule_gap + rule_thickness;

        MathBox {
            width: total_width,
            height,
            depth,
        }
    }

    fn emit_radical(&mut self, inner: &MathToken) -> MathBox {
        let inner_box = self.layout_token(inner);
        let inner_width = inner_box.width;
        let rule_thickness = Fp266::from_frac(self.font_size.to_int(), 16);
        let padding = Fp266::from_int(4);

        let sqrt_symbol = "√";
        let (mut sqrt_glyphs, sqrt_box) =
            shape_text_run(sqrt_symbol, self.font_size, self.font_data_ref());

        let total_width = sqrt_box.width + padding + inner_width;
        let overbar_width = inner_width + Fp266::from_int(2);

        for g in &mut sqrt_glyphs {
            g.x += self.cursor_x;
            g.y = self.base_y;
        }
        self.glyphs.extend(sqrt_glyphs);

        let overbar_x = self.cursor_x + sqrt_box.width + padding - Fp266::from_int(1);
        let overbar_y = self.base_y - inner_box.height - rule_thickness;

        self.glyphs.push(PositionedGlyph {
            glyph_id: -1,
            x: overbar_x,
            y: overbar_y,
            advance: overbar_width,
        });

        self.cursor_x += total_width;

        let height = inner_box.height + Fp266::from_int(4);
        MathBox {
            width: total_width,
            height,
            depth: inner_box.depth,
        }
    }

    fn emit_big_op(&mut self, op: &str) -> MathBox {
        let display = match op {
            "sum" => "∑",
            "int" => "∫",
            "prod" => "∏",
            _ => op,
        };

        let big_size = Fp266::from_frac(self.font_size.to_int() * 3, 2);
        let saved_font_size = self.font_size;
        self.font_size = big_size;

        let (mut glyphs, box_info) = shape_text_run(display, big_size, self.font_data_ref());
        for g in &mut glyphs {
            g.x += self.cursor_x;
            g.y = self.base_y - Fp266::from_frac(saved_font_size.to_int(), 4);
        }

        self.font_size = saved_font_size;
        let width = box_info.width;
        self.cursor_x += width;
        self.glyphs.extend(glyphs);

        let height = Fp266::from_frac(saved_font_size.to_int(), 2);
        let depth = Fp266::from_frac(saved_font_size.to_int(), 2);

        MathBox {
            width,
            height,
            depth,
        }
    }

    fn emit_binop(&mut self, op: &str) -> MathBox {
        let spacing = Fp266::from_frac(self.font_size.to_int(), 6);
        self.cursor_x += spacing;

        let box_info = self.emit_text(op);

        self.cursor_x += spacing;

        MathBox {
            width: box_info.width + spacing.mul(Fp266::from_int(2)),
            height: box_info.height,
            depth: box_info.depth,
        }
    }

    fn emit_delimiter(&mut self, ch: &str) -> MathBox {
        self.emit_text(ch)
    }

    fn emit_scaled_delimiter(
        &mut self,
        ch: &str,
        total_height: Fp266,
        x: Fp266,
        y: Fp266,
    ) -> MathBox {
        let saved_font_size = self.font_size;
        let saved_cursor_x = self.cursor_x;
        let saved_base_y = self.base_y;

        self.font_size = total_height;
        self.cursor_x = x;
        self.base_y = y;

        let box_info = self.emit_text(ch);

        self.font_size = saved_font_size;
        self.cursor_x = saved_cursor_x;
        self.base_y = saved_base_y;

        box_info
    }

    fn layout_token(&mut self, token: &MathToken) -> MathBox {
        match token {
            MathToken::Text(text) => self.emit_text(text),
            MathToken::Superscript(sup) => {
                if !self.pending_base.is_empty() {
                    let base = self.pending_base.clone();
                    self.pending_base.clear();
                    self.emit_superscript(&base, sup)
                } else {
                    self.emit_text(sup)
                }
            }
            MathToken::Subscript(sub) => {
                if !self.pending_base.is_empty() {
                    let base = self.pending_base.clone();
                    self.pending_base.clear();
                    self.emit_subscript(&base, sub)
                } else {
                    self.emit_text(sub)
                }
            }
            MathToken::Fraction(num, den) => self.emit_fraction(num, den),
            MathToken::Radical(inner) => self.emit_radical(inner),
            MathToken::BigOp(op) => self.emit_big_op(op),
            MathToken::BinOp(op) => self.emit_binop(op),
            MathToken::LeftParen => self.emit_delimiter("("),
            MathToken::RightParen => self.emit_delimiter(")"),
            MathToken::LeftBracket => self.emit_delimiter("["),
            MathToken::RightBracket => self.emit_delimiter("]"),
            MathToken::BeginCases
            | MathToken::EndCases
            | MathToken::BeginMatrix { .. }
            | MathToken::EndMatrix
            | MathToken::LeftDelim(_)
            | MathToken::RightDelim(_)
            | MathToken::Ampersand
            | MathToken::RowSeparator => MathBox::zero(),
        }
    }

    fn layout_node(&mut self, node: &MathNode) -> MathBox {
        match node {
            MathNode::Text(text) => {
                self.pending_base.clear();
                self.emit_text(text)
            }
            MathNode::Superscript { base, sup } => {
                self.pending_base.clear();
                self.emit_superscript(base, sup)
            }
            MathNode::Subscript { base, sub } => {
                self.pending_base.clear();
                self.emit_subscript(base, sub)
            }
            MathNode::Fraction { num, den } => {
                self.pending_base.clear();
                self.emit_fraction(num, den)
            }
            MathNode::Radical { inner } => {
                self.pending_base.clear();
                self.emit_radical(inner)
            }
            MathNode::BigOp(op) => {
                self.pending_base.clear();
                self.emit_big_op(op)
            }
            MathNode::BinOp(op) => {
                self.pending_base.clear();
                self.emit_binop(op)
            }
            MathNode::Delimiter(ch) => {
                self.pending_base.clear();
                self.emit_delimiter(&ch.to_string())
            }
            MathNode::Cases { rows } => {
                self.pending_base.clear();
                self.layout_cases(rows)
            }
            MathNode::Matrix { rows, delimiters } => {
                self.pending_base.clear();
                self.layout_matrix(rows, *delimiters)
            }
            MathNode::StretchyGroup {
                left,
                content,
                right,
            } => {
                self.pending_base.clear();
                self.layout_stretchy_group(*left, content, *right)
            }
        }
    }

    fn layout_nodes(&mut self, nodes: &[MathNode]) -> MathBox {
        let mut max_height = Fp266::ZERO;
        let mut max_depth = Fp266::ZERO;

        for node in nodes {
            let box_info = self.layout_node(node);
            max_height = max_height.max(box_info.height);
            max_depth = max_depth.max(box_info.depth);
        }

        MathBox {
            width: self.cursor_x,
            height: max_height,
            depth: max_depth,
        }
    }

    fn layout_cases(&mut self, rows: &[Vec<Vec<MathNode>>]) -> MathBox {
        if rows.is_empty() {
            return MathBox::zero();
        }

        let saved_cursor_x = self.cursor_x;
        let saved_base_y = self.base_y;
        let line_height = Fp266::from_frac(self.font_size.to_int() * 14, 10);
        let col_gap = Fp266::from_frac(self.font_size.to_int(), 3);

        let mut cell_boxes: Vec<Vec<MathBox>> = Vec::new();
        let mut max_col_widths: Vec<Fp266> = Vec::new();
        let mut max_row_height = Fp266::ZERO;
        let mut max_row_depth = Fp266::ZERO;

        for row in rows {
            let mut row_measures = Vec::new();
            for cell_nodes in row {
                let mut ms = self.measure_state();
                let box_info = ms.layout_nodes(cell_nodes);
                row_measures.push(box_info);
            }
            while max_col_widths.len() < row_measures.len() {
                max_col_widths.push(Fp266::ZERO);
            }
            for (i, m) in row_measures.iter().enumerate() {
                max_col_widths[i] = max_col_widths[i].max(m.width);
            }
            let rh = row_measures
                .iter()
                .map(|m| m.height)
                .max()
                .unwrap_or(Fp266::ZERO);
            let rd = row_measures
                .iter()
                .map(|m| m.depth)
                .max()
                .unwrap_or(Fp266::ZERO);
            max_row_height = max_row_height.max(rh);
            max_row_depth = max_row_depth.max(rd);
            cell_boxes.push(row_measures);
        }

        let brace_width = Fp266::from_frac(self.font_size.to_int(), 2);
        let content_start_x = saved_cursor_x + brace_width + col_gap;

        let total_content_width: Fp266 = max_col_widths
            .iter()
            .enumerate()
            .map(|(i, w)| if i > 0 { *w + col_gap } else { *w })
            .fold(Fp266::ZERO, |acc, v| acc + v);

        for (row_idx, row) in rows.iter().enumerate() {
            self.base_y = saved_base_y + Fp266::from_int(row_idx as i32).mul(line_height);
            let mut cell_x = content_start_x;

            for (col_idx, cell_nodes) in row.iter().enumerate() {
                self.cursor_x = cell_x;
                self.layout_nodes(cell_nodes);
                if col_idx < max_col_widths.len() {
                    cell_x += max_col_widths[col_idx] + col_gap;
                }
            }
        }

        let total_height = if rows.len() > 1 {
            Fp266::from_int((rows.len() - 1) as i32).mul(line_height)
        } else {
            Fp266::ZERO
        };

        let brace_total = max_row_height + total_height + max_row_depth;
        let brace_y = saved_base_y + total_height.div(Fp266::from_int(2));
        self.emit_scaled_delimiter("{", brace_total, saved_cursor_x, brace_y);

        self.cursor_x = content_start_x + total_content_width;

        MathBox {
            width: brace_width + col_gap + total_content_width,
            height: max_row_height + total_height.div(Fp266::from_int(2)),
            depth: max_row_depth + total_height.div(Fp266::from_int(2)),
        }
    }

    fn layout_matrix(
        &mut self,
        rows: &[Vec<Vec<MathNode>>],
        delimiters: (Option<char>, Option<char>),
    ) -> MathBox {
        if rows.is_empty() {
            return MathBox::zero();
        }

        let saved_cursor_x = self.cursor_x;
        let saved_base_y = self.base_y;
        let line_height = Fp266::from_frac(self.font_size.to_int() * 14, 10);
        let col_gap = Fp266::from_frac(self.font_size.to_int(), 3);
        let delim_pad = Fp266::from_frac(self.font_size.to_int(), 2);

        let mut cell_boxes: Vec<Vec<MathBox>> = Vec::new();
        let mut max_col_widths: Vec<Fp266> = Vec::new();
        let mut max_row_height = Fp266::ZERO;
        let mut max_row_depth = Fp266::ZERO;

        for row in rows {
            let mut row_measures = Vec::new();
            for cell_nodes in row {
                let mut ms = self.measure_state();
                let box_info = ms.layout_nodes(cell_nodes);
                row_measures.push(box_info);
            }
            while max_col_widths.len() < row_measures.len() {
                max_col_widths.push(Fp266::ZERO);
            }
            for (i, m) in row_measures.iter().enumerate() {
                max_col_widths[i] = max_col_widths[i].max(m.width);
            }
            let rh = row_measures
                .iter()
                .map(|m| m.height)
                .max()
                .unwrap_or(Fp266::ZERO);
            let rd = row_measures
                .iter()
                .map(|m| m.depth)
                .max()
                .unwrap_or(Fp266::ZERO);
            max_row_height = max_row_height.max(rh);
            max_row_depth = max_row_depth.max(rd);
            cell_boxes.push(row_measures);
        }

        let content_start_x = saved_cursor_x + delimiters.0.map_or(Fp266::ZERO, |_| delim_pad);
        let total_content_width: Fp266 = max_col_widths
            .iter()
            .enumerate()
            .map(|(i, w)| if i > 0 { *w + col_gap } else { *w })
            .fold(Fp266::ZERO, |acc, v| acc + v);

        for (row_idx, row) in rows.iter().enumerate() {
            self.base_y = saved_base_y + Fp266::from_int(row_idx as i32).mul(line_height);
            let mut cell_x = content_start_x;

            for (col_idx, cell_nodes) in row.iter().enumerate() {
                let col_center_offset = if col_idx < max_col_widths.len()
                    && row_idx < cell_boxes.len()
                    && col_idx < cell_boxes[row_idx].len()
                {
                    (max_col_widths[col_idx] - cell_boxes[row_idx][col_idx].width)
                        .div(Fp266::from_int(2))
                } else {
                    Fp266::ZERO
                };
                self.cursor_x = cell_x + col_center_offset;
                self.layout_nodes(cell_nodes);
                if col_idx < max_col_widths.len() {
                    cell_x += max_col_widths[col_idx] + col_gap;
                }
            }
        }

        let total_height = if rows.len() > 1 {
            Fp266::from_int((rows.len() - 1) as i32).mul(line_height)
        } else {
            Fp266::ZERO
        };

        let delim_total = max_row_height + total_height + max_row_depth;
        let delim_y = saved_base_y + total_height.div(Fp266::from_int(2));

        if let Some(left_ch) = delimiters.0 {
            self.emit_scaled_delimiter(&left_ch.to_string(), delim_total, saved_cursor_x, delim_y);
        }

        if let Some(right_ch) = delimiters.1 {
            let right_x = content_start_x + total_content_width + delim_pad;
            self.emit_scaled_delimiter(&right_ch.to_string(), delim_total, right_x, delim_y);
        }

        let left_w = delimiters.0.map_or(Fp266::ZERO, |_| delim_pad);
        let right_w = delimiters.1.map_or(Fp266::ZERO, |_| delim_pad);
        self.cursor_x = content_start_x + total_content_width + right_w;

        MathBox {
            width: left_w + total_content_width + right_w,
            height: max_row_height + total_height.div(Fp266::from_int(2)),
            depth: max_row_depth + total_height.div(Fp266::from_int(2)),
        }
    }

    fn layout_stretchy_group(
        &mut self,
        left: Option<char>,
        content: &[MathNode],
        right: Option<char>,
    ) -> MathBox {
        let saved_cursor_x = self.cursor_x;
        let saved_base_y = self.base_y;
        let delim_pad = Fp266::from_frac(self.font_size.to_int(), 2);

        let content_start_x = saved_cursor_x + left.map_or(Fp266::ZERO, |_| delim_pad);
        self.cursor_x = content_start_x;
        let content_box = self.layout_nodes(content);
        let content_width = content_box.width;

        let delim_total = content_box.height + content_box.depth;
        let delim_y = saved_base_y;

        if let Some(left_ch) = left {
            self.emit_scaled_delimiter(&left_ch.to_string(), delim_total, saved_cursor_x, delim_y);
        }

        if let Some(right_ch) = right {
            let right_x = content_start_x + content_width + delim_pad;
            self.emit_scaled_delimiter(&right_ch.to_string(), delim_total, right_x, delim_y);
        }

        let left_w = left.map_or(Fp266::ZERO, |_| delim_pad);
        let right_w = right.map_or(Fp266::ZERO, |_| delim_pad);
        self.cursor_x = content_start_x + content_width + right_w;

        MathBox {
            width: left_w + content_width + right_w,
            height: content_box.height,
            depth: content_box.depth,
        }
    }
}

struct MathTokenParser {
    tokens: Vec<MathToken>,
    pos: usize,
}

impl MathTokenParser {
    fn new(tokens: Vec<MathToken>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn parse_expression(&mut self) -> Vec<MathNode> {
        let mut nodes = Vec::new();
        while self.pos < self.tokens.len() && !self.is_stop_token() {
            if let Some(node) = self.parse_node() {
                nodes.push(node);
            }
        }
        nodes
    }

    fn is_stop_token(&self) -> bool {
        if self.pos >= self.tokens.len() {
            return true;
        }
        matches!(
            self.tokens[self.pos],
            MathToken::EndCases
                | MathToken::EndMatrix
                | MathToken::RightDelim(_)
                | MathToken::Ampersand
                | MathToken::RowSeparator
        )
    }

    fn parse_node(&mut self) -> Option<MathNode> {
        if self.is_stop_token() {
            return None;
        }

        let token = self.tokens.get(self.pos)?.clone();
        self.pos += 1;

        match token {
            MathToken::Text(text) => {
                if self.pos < self.tokens.len() {
                    match &self.tokens[self.pos] {
                        MathToken::Subscript(sub) => {
                            let sub = sub.clone();
                            self.pos += 1;
                            return Some(MathNode::Subscript { base: text, sub });
                        }
                        MathToken::Superscript(sup) => {
                            let sup = sup.clone();
                            self.pos += 1;
                            return Some(MathNode::Superscript { base: text, sup });
                        }
                        _ => {}
                    }
                }
                Some(MathNode::Text(text))
            }
            MathToken::Subscript(sub) => Some(MathNode::Text(sub)),
            MathToken::Superscript(sup) => Some(MathNode::Text(sup)),
            MathToken::Fraction(num, den) => Some(MathNode::Fraction { num, den }),
            MathToken::Radical(inner) => Some(MathNode::Radical { inner }),
            MathToken::BigOp(op) => Some(MathNode::BigOp(op)),
            MathToken::BinOp(op) => Some(MathNode::BinOp(op)),
            MathToken::LeftParen => Some(MathNode::Delimiter('(')),
            MathToken::RightParen => Some(MathNode::Delimiter(')')),
            MathToken::LeftBracket => Some(MathNode::Delimiter('[')),
            MathToken::RightBracket => Some(MathNode::Delimiter(']')),
            MathToken::BeginCases => {
                let rows = self.parse_rows_until_cases();
                Some(MathNode::Cases { rows })
            }
            MathToken::BeginMatrix { left, right } => {
                let rows = self.parse_rows_until_matrix();
                Some(MathNode::Matrix {
                    rows,
                    delimiters: (left, right),
                })
            }
            MathToken::LeftDelim(d) => {
                let left_char = if d == '.' { None } else { Some(d) };
                let content = self.parse_expression();
                let right_char = if self.pos < self.tokens.len() {
                    if let MathToken::RightDelim(rd) = &self.tokens[self.pos] {
                        let r = if *rd == '.' { None } else { Some(*rd) };
                        self.pos += 1;
                        r
                    } else {
                        None
                    }
                } else {
                    None
                };
                Some(MathNode::StretchyGroup {
                    left: left_char,
                    content,
                    right: right_char,
                })
            }
            MathToken::EndCases
            | MathToken::EndMatrix
            | MathToken::RightDelim(_)
            | MathToken::Ampersand
            | MathToken::RowSeparator => None,
        }
    }

    fn parse_rows_until_cases(&mut self) -> Vec<Vec<Vec<MathNode>>> {
        let mut rows = Vec::new();
        rows.push(self.parse_row());
        while self.pos < self.tokens.len() {
            match &self.tokens[self.pos] {
                MathToken::RowSeparator => {
                    self.pos += 1;
                    rows.push(self.parse_row());
                }
                MathToken::EndCases => {
                    self.pos += 1;
                    break;
                }
                _ => break,
            }
        }
        rows
    }

    fn parse_rows_until_matrix(&mut self) -> Vec<Vec<Vec<MathNode>>> {
        let mut rows = Vec::new();
        rows.push(self.parse_row());
        while self.pos < self.tokens.len() {
            match &self.tokens[self.pos] {
                MathToken::RowSeparator => {
                    self.pos += 1;
                    rows.push(self.parse_row());
                }
                MathToken::EndMatrix => {
                    self.pos += 1;
                    break;
                }
                _ => break,
            }
        }
        rows
    }

    fn parse_row(&mut self) -> Vec<Vec<MathNode>> {
        let mut cells = Vec::new();
        cells.push(self.parse_expression());
        while self.pos < self.tokens.len() {
            if matches!(self.tokens[self.pos], MathToken::Ampersand) {
                self.pos += 1;
                cells.push(self.parse_expression());
            } else {
                break;
            }
        }
        cells
    }
}

/// Result of laying out a math expression.
pub struct MathLayoutResult {
    /// Positioned glyphs (glyph_id == -1 means a horizontal rule).
    pub glyphs: Vec<PositionedGlyph>,
    /// Total width of the math expression.
    pub width: Fp266,
    /// Height above the baseline.
    pub height: Fp266,
    /// Depth below the baseline.
    pub depth: Fp266,
}

/// Layout a math expression string into positioned glyph commands.
pub fn layout_math(
    content: &str,
    font_data: Option<&[u8]>,
    font_size: Fp266,
    base_y: Fp266,
) -> MathLayoutResult {
    let tokens = tokenize_math(content);
    if tokens.is_empty() {
        return MathLayoutResult {
            glyphs: Vec::new(),
            width: Fp266::ZERO,
            height: Fp266::ZERO,
            depth: Fp266::ZERO,
        };
    }

    let mut parser = MathTokenParser::new(tokens);
    let nodes = parser.parse_expression();
    if nodes.is_empty() {
        return MathLayoutResult {
            glyphs: Vec::new(),
            width: Fp266::ZERO,
            height: Fp266::ZERO,
            depth: Fp266::ZERO,
        };
    }

    let font_data_owned = font_data.map(|d| Arc::new(d.to_vec()));
    let mut state = LayoutState::new(font_size, base_y, font_data_owned);

    let box_info = state.layout_nodes(&nodes);

    MathLayoutResult {
        width: box_info.width,
        height: box_info.height,
        depth: box_info.depth,
        glyphs: state.glyphs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple_text() {
        let tokens = tokenize_math("x + y");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], MathToken::Text("x".to_string()));
        assert_eq!(tokens[1], MathToken::BinOp("+".to_string()));
        assert_eq!(tokens[2], MathToken::Text("y".to_string()));
    }

    #[test]
    fn test_tokenize_superscript_braced() {
        let tokens = tokenize_math("x^{2}");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], MathToken::Text("x".to_string()));
        assert_eq!(tokens[1], MathToken::Superscript("2".to_string()));
    }

    #[test]
    fn test_tokenize_superscript_single() {
        let tokens = tokenize_math("x^2");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], MathToken::Text("x".to_string()));
        assert_eq!(tokens[1], MathToken::Superscript("2".to_string()));
    }

    #[test]
    fn test_tokenize_subscript_braced() {
        let tokens = tokenize_math("x_{i}");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], MathToken::Text("x".to_string()));
        assert_eq!(tokens[1], MathToken::Subscript("i".to_string()));
    }

    #[test]
    fn test_tokenize_subscript_single() {
        let tokens = tokenize_math("x_i");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], MathToken::Text("x".to_string()));
        assert_eq!(tokens[1], MathToken::Subscript("i".to_string()));
    }

    #[test]
    fn test_tokenize_fraction() {
        let tokens = tokenize_math("\\frac{a}{b}");
        assert_eq!(tokens.len(), 1);
        if let MathToken::Fraction(num, den) = &tokens[0] {
            assert_eq!(num, "a");
            assert_eq!(den, "b");
        } else {
            panic!("expected Fraction");
        }
    }

    #[test]
    fn test_tokenize_sqrt() {
        let tokens = tokenize_math("\\sqrt{x}");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], MathToken::Radical(_)));
    }

    #[test]
    fn test_tokenize_big_ops() {
        let tokens = tokenize_math("\\sum \\int \\prod");
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], MathToken::BigOp(ref s) if s == "sum"));
        assert!(matches!(tokens[1], MathToken::BigOp(ref s) if s == "int"));
        assert!(matches!(tokens[2], MathToken::BigOp(ref s) if s == "prod"));
    }

    #[test]
    fn test_tokenize_greek() {
        let tokens = tokenize_math("α + β = γ");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], MathToken::Text("α".to_string()));
        assert_eq!(tokens[2], MathToken::Text("β".to_string()));
        assert_eq!(tokens[4], MathToken::Text("γ".to_string()));
    }

    #[test]
    fn test_tokenize_greek_command() {
        let tokens = tokenize_math("\\alpha + \\beta");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], MathToken::Text("α".to_string()));
        assert_eq!(tokens[2], MathToken::Text("β".to_string()));
    }

    #[test]
    fn test_tokenize_complex() {
        let tokens = tokenize_math("x^2 + \\frac{a}{b} = \\sqrt{c}");
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0], MathToken::Text("x".to_string()));
        assert_eq!(tokens[1], MathToken::Superscript("2".to_string()));
        assert!(matches!(tokens[2], MathToken::BinOp(_)));
        assert!(matches!(tokens[3], MathToken::Fraction(_, _)));
        assert!(matches!(tokens[4], MathToken::BinOp(_)));
        assert!(matches!(tokens[5], MathToken::Radical(_)));
    }

    #[test]
    fn test_tokenize_delimiters() {
        let tokens = tokenize_math("(x)");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], MathToken::LeftParen);
        assert_eq!(tokens[1], MathToken::Text("x".to_string()));
        assert_eq!(tokens[2], MathToken::RightParen);
    }

    #[test]
    fn test_tokenize_left_right() {
        let tokens = tokenize_math("\\left( x \\right)");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], MathToken::LeftDelim('('));
        assert_eq!(tokens[2], MathToken::RightDelim(')'));
    }

    #[test]
    fn test_tokenize_subsuperscript() {
        let tokens = tokenize_math("x_0^2");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], MathToken::Text("x".to_string()));
        assert_eq!(tokens[1], MathToken::Subscript("0".to_string()));
        assert_eq!(tokens[2], MathToken::Superscript("2".to_string()));
    }

    #[test]
    fn test_tokenize_cases() {
        let tokens = tokenize_math("\\begin{cases} x^2 & y \\\\ -x & z \\end{cases}");
        assert_eq!(tokens[0], MathToken::BeginCases);
        assert_eq!(tokens[1], MathToken::Text("x".to_string()));
        assert_eq!(tokens[2], MathToken::Superscript("2".to_string()));
        assert_eq!(tokens[3], MathToken::Ampersand);
        assert_eq!(tokens[4], MathToken::Text("y".to_string()));
        assert_eq!(tokens[5], MathToken::RowSeparator);
        assert_eq!(tokens[6], MathToken::BinOp("-".to_string()));
        assert_eq!(tokens[7], MathToken::Text("x".to_string()));
        assert_eq!(tokens[8], MathToken::Ampersand);
        assert_eq!(tokens[9], MathToken::Text("z".to_string()));
        assert_eq!(tokens[10], MathToken::EndCases);
    }

    #[test]
    fn test_tokenize_pmatrix() {
        let tokens = tokenize_math("\\begin{pmatrix} a & b \\\\ c & d \\end{pmatrix}");
        assert!(matches!(
            tokens[0],
            MathToken::BeginMatrix {
                left: Some('('),
                right: Some(')')
            }
        ));
        assert_eq!(tokens[1], MathToken::Text("a".to_string()));
        assert_eq!(tokens[2], MathToken::Ampersand);
        assert_eq!(tokens[3], MathToken::Text("b".to_string()));
        assert_eq!(tokens[4], MathToken::RowSeparator);
        assert_eq!(tokens[5], MathToken::Text("c".to_string()));
        assert_eq!(tokens[6], MathToken::Ampersand);
        assert_eq!(tokens[7], MathToken::Text("d".to_string()));
        assert_eq!(tokens[8], MathToken::EndMatrix);
    }

    #[test]
    fn test_tokenize_bmatrix() {
        let tokens = tokenize_math("\\begin{bmatrix} 1 \\end{bmatrix}");
        assert!(matches!(
            tokens[0],
            MathToken::BeginMatrix {
                left: Some('['),
                right: Some(']')
            }
        ));
    }

    #[test]
    fn test_tokenize_vmatrix() {
        let tokens = tokenize_math("\\begin{vmatrix} 1 \\end{vmatrix}");
        assert!(matches!(
            tokens[0],
            MathToken::BeginMatrix {
                left: Some('|'),
                right: Some('|')
            }
        ));
    }

    #[test]
    fn test_tokenize_vmatrix_big() {
        let tokens = tokenize_math("\\begin{Vmatrix} 1 \\end{Vmatrix}");
        assert!(matches!(
            tokens[0],
            MathToken::BeginMatrix {
                left: Some('\u{2016}'),
                right: Some('\u{2016}')
            }
        ));
    }

    #[test]
    fn test_tokenize_matrix_no_delim() {
        let tokens = tokenize_math("\\begin{matrix} 1 \\end{matrix}");
        assert!(matches!(
            tokens[0],
            MathToken::BeginMatrix {
                left: None,
                right: None
            }
        ));
    }

    #[test]
    fn test_tokenize_left_delim_variants() {
        let t = tokenize_math("\\left( \\right)");
        assert_eq!(t[0], MathToken::LeftDelim('('));
        assert_eq!(t[1], MathToken::RightDelim(')'));

        let t = tokenize_math("\\left[ \\right]");
        assert_eq!(t[0], MathToken::LeftDelim('['));
        assert_eq!(t[1], MathToken::RightDelim(']'));

        let t = tokenize_math("\\left{ \\right}");
        assert_eq!(t[0], MathToken::LeftDelim('{'));
        assert_eq!(t[1], MathToken::RightDelim('}'));

        let t = tokenize_math("\\left| \\right|");
        assert_eq!(t[0], MathToken::LeftDelim('|'));
        assert_eq!(t[1], MathToken::RightDelim('|'));

        let t = tokenize_math("\\left. \\right.");
        assert_eq!(t[0], MathToken::LeftDelim('.'));
        assert_eq!(t[1], MathToken::RightDelim('.'));
    }

    #[test]
    fn test_tokenize_ampersand() {
        let tokens = tokenize_math("a & b");
        assert_eq!(tokens[0], MathToken::Text("a".to_string()));
        assert_eq!(tokens[1], MathToken::Ampersand);
        assert_eq!(tokens[2], MathToken::Text("b".to_string()));
    }

    #[test]
    fn test_tokenize_row_separator() {
        let tokens = tokenize_math("a \\\\ b");
        assert_eq!(tokens[0], MathToken::Text("a".to_string()));
        assert_eq!(tokens[1], MathToken::RowSeparator);
        assert_eq!(tokens[2], MathToken::Text("b".to_string()));
    }

    #[test]
    fn test_tokenize_text_command() {
        let tokens = tokenize_math("\\text{hello}");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], MathToken::Text("hello".to_string()));
    }

    #[test]
    fn test_parse_cases_rows() {
        let tokens = tokenize_math("\\begin{cases} x^2 & y \\\\ -x & z \\end{cases}");
        let mut parser = MathTokenParser::new(tokens);
        let nodes = parser.parse_expression();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            MathNode::Cases { rows } => {
                assert_eq!(rows.len(), 2);
                assert_eq!(rows[0].len(), 2);
                assert_eq!(rows[1].len(), 2);
            }
            _ => panic!("expected Cases node"),
        }
    }

    #[test]
    fn test_parse_matrix_rows() {
        let tokens = tokenize_math("\\begin{pmatrix} a & b \\\\ c & d \\end{pmatrix}");
        let mut parser = MathTokenParser::new(tokens);
        let nodes = parser.parse_expression();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            MathNode::Matrix { rows, delimiters } => {
                assert_eq!(rows.len(), 2);
                assert_eq!(rows[0].len(), 2);
                assert_eq!(delimiters, &(Some('('), Some(')')));
            }
            _ => panic!("expected Matrix node"),
        }
    }

    #[test]
    fn test_parse_stretchy_group() {
        let tokens = tokenize_math("\\left( \\frac{a}{b} \\right)");
        let mut parser = MathTokenParser::new(tokens);
        let nodes = parser.parse_expression();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            MathNode::StretchyGroup { left, right, .. } => {
                assert_eq!(*left, Some('('));
                assert_eq!(*right, Some(')'));
            }
            _ => panic!("expected StretchyGroup node"),
        }
    }

    #[test]
    fn test_parse_stretchy_empty_delim() {
        let tokens = tokenize_math("\\left. x \\right)");
        let mut parser = MathTokenParser::new(tokens);
        let nodes = parser.parse_expression();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            MathNode::StretchyGroup { left, right, .. } => {
                assert_eq!(*left, None);
                assert_eq!(*right, Some(')'));
            }
            _ => panic!("expected StretchyGroup node"),
        }
    }

    #[test]
    fn test_parse_nested_expression() {
        let tokens = tokenize_math("x + y");
        let mut parser = MathTokenParser::new(tokens);
        let nodes = parser.parse_expression();
        assert_eq!(nodes.len(), 3);
        assert!(matches!(&nodes[0], MathNode::Text(t) if t == "x"));
        assert!(matches!(&nodes[1], MathNode::BinOp(op) if op == "+"));
        assert!(matches!(&nodes[2], MathNode::Text(t) if t == "y"));
    }

    #[test]
    fn test_layout_simple_text() {
        let result = layout_math("x + y", None, Fp266::from_int(12), Fp266::ZERO);
        assert!(result.width.to_f64() > 0.0);
        assert!(result.height.to_f64() > 0.0);
        assert!(!result.glyphs.is_empty());
    }

    #[test]
    fn test_layout_subscript() {
        let result = layout_math("x_i", None, Fp266::from_int(12), Fp266::ZERO);
        assert!(result.width.to_f64() > 0.0);
        assert!(!result.glyphs.is_empty());
    }

    #[test]
    fn test_layout_superscript() {
        let result = layout_math("x^2", None, Fp266::from_int(12), Fp266::ZERO);
        assert!(result.width.to_f64() > 0.0);
        assert!(!result.glyphs.is_empty());
    }

    #[test]
    fn test_layout_fraction() {
        let result = layout_math("\\frac{a}{b}", None, Fp266::from_int(12), Fp266::ZERO);
        assert!(result.width.to_f64() > 0.0);
        assert!(result.height.to_f64() > 0.0);
        let has_overbar = result.glyphs.iter().any(|g| g.glyph_id == -1);
        assert!(has_overbar, "fraction should have a rule (glyph_id == -1)");
    }

    #[test]
    fn test_layout_sqrt() {
        let result = layout_math("\\sqrt{x}", None, Fp266::from_int(12), Fp266::ZERO);
        assert!(result.width.to_f64() > 0.0);
        assert!(!result.glyphs.is_empty());
        let has_overbar = result.glyphs.iter().any(|g| g.glyph_id == -1);
        assert!(has_overbar, "sqrt should have an overbar (glyph_id == -1)");
    }

    #[test]
    fn test_layout_greek() {
        let result = layout_math("α + β = γ", None, Fp266::from_int(12), Fp266::ZERO);
        assert!(result.width.to_f64() > 0.0);
        assert!(!result.glyphs.is_empty());
    }

    #[test]
    fn test_layout_complex() {
        let result = layout_math("x^2 + \\frac{a}{b}", None, Fp266::from_int(12), Fp266::ZERO);
        assert!(result.width.to_f64() > 0.0);
        assert!(!result.glyphs.is_empty());
    }

    #[test]
    fn test_layout_empty() {
        let result = layout_math("", None, Fp266::from_int(12), Fp266::ZERO);
        assert!(result.glyphs.is_empty());
        assert!(result.width.is_zero());
    }

    #[test]
    fn test_layout_quadratic_formula() {
        let content = "x = \\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}";
        let result = layout_math(content, None, Fp266::from_int(12), Fp266::ZERO);
        assert!(result.width.to_f64() > 0.0);
        assert!(!result.glyphs.is_empty());
    }

    #[test]
    fn test_math_box_zero() {
        let box_info = MathBox::zero();
        assert!(box_info.width.is_zero());
        assert!(box_info.height.is_zero());
        assert!(box_info.depth.is_zero());
    }

    #[test]
    fn test_layout_cases() {
        let content = "\\begin{cases} x^2 & y \\\\ -x & z \\end{cases}";
        let result = layout_math(content, None, Fp266::from_int(12), Fp266::ZERO);
        assert!(result.width.to_f64() > 0.0);
        assert!(result.height.to_f64() > 0.0);
        assert!(!result.glyphs.is_empty());
    }

    #[test]
    fn test_layout_pmatrix() {
        let content = "\\begin{pmatrix} a & b \\\\ c & d \\end{pmatrix}";
        let result = layout_math(content, None, Fp266::from_int(12), Fp266::ZERO);
        assert!(result.width.to_f64() > 0.0);
        assert!(result.height.to_f64() > 0.0);
        assert!(!result.glyphs.is_empty());
    }

    #[test]
    fn test_layout_bmatrix() {
        let content = "\\begin{bmatrix} 1 & 0 \\\\ 0 & 1 \\end{bmatrix}";
        let result = layout_math(content, None, Fp266::from_int(12), Fp266::ZERO);
        assert!(result.width.to_f64() > 0.0);
        assert!(!result.glyphs.is_empty());
    }

    #[test]
    fn test_layout_matrix_no_delim() {
        let content = "\\begin{matrix} a \\\\ b \\end{matrix}";
        let result = layout_math(content, None, Fp266::from_int(12), Fp266::ZERO);
        assert!(result.width.to_f64() > 0.0);
        assert!(!result.glyphs.is_empty());
    }

    #[test]
    fn test_layout_stretchy_parens() {
        let content = "\\left( x + y \\right)";
        let result = layout_math(content, None, Fp266::from_int(12), Fp266::ZERO);
        assert!(result.width.to_f64() > 0.0);
        assert!(!result.glyphs.is_empty());
    }

    #[test]
    fn test_layout_stretchy_fraction() {
        let content = "\\left( \\frac{a}{b} \\right)";
        let result = layout_math(content, None, Fp266::from_int(12), Fp266::ZERO);
        assert!(result.width.to_f64() > 0.0);
        assert!(result.height.to_f64() > 0.0);
        assert!(!result.glyphs.is_empty());
    }

    #[test]
    fn test_layout_stretchy_brackets() {
        let content = "\\left[ x \\right]";
        let result = layout_math(content, None, Fp266::from_int(12), Fp266::ZERO);
        assert!(result.width.to_f64() > 0.0);
        assert!(!result.glyphs.is_empty());
    }

    #[test]
    fn test_layout_stretchy_braces() {
        let content = "\\left{ x \\right}";
        let result = layout_math(content, None, Fp266::from_int(12), Fp266::ZERO);
        assert!(result.width.to_f64() > 0.0);
        assert!(!result.glyphs.is_empty());
    }

    #[test]
    fn test_layout_stretchy_empty_left() {
        let content = "\\left. x \\right)";
        let result = layout_math(content, None, Fp266::from_int(12), Fp266::ZERO);
        assert!(result.width.to_f64() > 0.0);
        assert!(!result.glyphs.is_empty());
    }

    #[test]
    fn test_layout_cases_single_row() {
        let content = "\\begin{cases} x \\end{cases}";
        let result = layout_math(content, None, Fp266::from_int(12), Fp266::ZERO);
        assert!(result.width.to_f64() > 0.0);
        assert!(!result.glyphs.is_empty());
    }

    #[test]
    fn test_layout_3x3_matrix() {
        let content = "\\begin{pmatrix} 1 & 2 & 3 \\\\ 4 & 5 & 6 \\\\ 7 & 8 & 9 \\end{pmatrix}";
        let result = layout_math(content, None, Fp266::from_int(12), Fp266::ZERO);
        assert!(result.width.to_f64() > 0.0);
        assert!(!result.glyphs.is_empty());
    }
}
