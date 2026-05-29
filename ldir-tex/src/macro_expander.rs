use std::collections::HashMap;

use ldir_ir::sir::v2::SourceSpan;

use crate::lexer::{SpannedToken, Token};

#[derive(Clone)]
pub struct MacroDef {
    pub name: String,
    pub num_params: u8,
    pub body_tokens: Vec<Token>,
}

pub type MacroTable = HashMap<String, MacroDef>;

const MAX_EXPANSION_DEPTH: u32 = 100;

pub fn expand_macros(tokens: Vec<SpannedToken>) -> Vec<SpannedToken> {
    let mut macros = MacroTable::new();
    expand_with_macros(tokens, &mut macros, 0)
}

fn expand_with_macros(
    tokens: Vec<SpannedToken>,
    macros: &mut MacroTable,
    depth: u32,
) -> Vec<SpannedToken> {
    if depth >= MAX_EXPANSION_DEPTH {
        return tokens;
    }

    let mut output = Vec::new();
    let mut i = 0;
    let n = tokens.len();

    while i < n {
        if let Token::ControlSequence(cmd) = &tokens[i].token {
            if (cmd == "newcommand" || cmd == "renewcommand")
                && let Some((new_i, def)) = parse_newcommand_form(&tokens, i)
            {
                macros.insert(def.name.clone(), def);
                i = new_i;
                continue;
            }
            if cmd == "def"
                && let Some((new_i, def)) = parse_def_form(&tokens, i)
            {
                macros.insert(def.name.clone(), def);
                i = new_i;
                continue;
            }
            if let Some(def) = macros.get(cmd) {
                let call_span = tokens[i].span;
                let def_clone = def.clone();
                let (new_i, args) = collect_macro_args(&tokens, i + 1, def_clone.num_params);
                let substituted = substitute_params(&def_clone.body_tokens, &args, call_span);
                let expanded = expand_with_macros(substituted, macros, depth + 1);
                output.extend(expanded);
                i = new_i;
                continue;
            }
        }
        output.push(tokens[i].clone());
        i += 1;
    }

    output
}

fn parse_newcommand_form(tokens: &[SpannedToken], i: usize) -> Option<(usize, MacroDef)> {
    let n = tokens.len();

    if i + 1 >= n {
        return None;
    }
    match &tokens[i + 1].token {
        Token::BraceOpen => {}
        _ => return None,
    }

    if i + 3 >= n {
        return None;
    }
    let name = match &tokens[i + 2].token {
        Token::ControlSequence(cs) => cs.clone(),
        _ => return None,
    };
    match &tokens[i + 3].token {
        Token::BraceClose => {}
        _ => return None,
    }

    let mut pos = i + 4;
    let num_params = if pos < n && matches!(tokens[pos].token, Token::BracketOpen) {
        pos += 1;
        let num_str = if pos < n {
            match &tokens[pos].token {
                Token::Text(s) => s.clone(),
                _ => return None,
            }
        } else {
            return None;
        };
        let num: u8 = num_str.parse().unwrap_or(0);
        pos += 1;
        if pos < n && matches!(tokens[pos].token, Token::BracketClose) {
            pos += 1;
        } else {
            return None;
        }
        num
    } else {
        0
    };

    let body_tokens = collect_brace_group_tokens(tokens, pos)?;
    let body_len = body_tokens.len();

    Some((
        pos + body_len + 2,
        MacroDef {
            name,
            num_params,
            body_tokens,
        },
    ))
}

fn parse_def_form(tokens: &[SpannedToken], i: usize) -> Option<(usize, MacroDef)> {
    let n = tokens.len();

    if i + 1 >= n {
        return None;
    }
    let name = match &tokens[i + 1].token {
        Token::ControlSequence(cs) => cs.clone(),
        _ => return None,
    };

    let mut pos = i + 2;
    let mut num_params: u8 = 0;

    while pos + 1 < n {
        if matches!(tokens[pos].token, Token::Hash)
            && let Token::Text(s) = &tokens[pos + 1].token
            && let Ok(d) = s.parse::<u8>()
        {
            if d > num_params {
                num_params = d;
            }
            pos += 2;
            continue;
        }
        break;
    }

    let body_tokens = collect_brace_group_tokens(tokens, pos)?;
    let body_len = body_tokens.len();

    Some((
        pos + body_len + 2,
        MacroDef {
            name,
            num_params,
            body_tokens,
        },
    ))
}

fn collect_brace_group_tokens(tokens: &[SpannedToken], pos: usize) -> Option<Vec<Token>> {
    if pos >= tokens.len() {
        return None;
    }
    if !matches!(tokens[pos].token, Token::BraceOpen) {
        return None;
    }

    let mut body = Vec::new();
    let mut i = pos + 1;
    let mut depth = 1u32;

    while i < tokens.len() && depth > 0 {
        match &tokens[i].token {
            Token::BraceOpen => {
                depth += 1;
                body.push(Token::BraceOpen);
            }
            Token::BraceClose => {
                depth -= 1;
                if depth > 0 {
                    body.push(Token::BraceClose);
                }
            }
            t => body.push(t.clone()),
        }
        i += 1;
    }

    Some(body)
}

fn collect_macro_args(
    tokens: &[SpannedToken],
    start: usize,
    num_params: u8,
) -> (usize, Vec<Vec<Token>>) {
    if num_params == 0 {
        return (start, Vec::new());
    }

    let mut args = Vec::new();
    let mut pos = start;
    let n = tokens.len();

    for _ in 0..num_params {
        if pos >= n {
            break;
        }
        match &tokens[pos].token {
            Token::BraceOpen => {
                let group = collect_brace_group_tokens(tokens, pos).unwrap_or_default();
                let group_len = group.len();
                args.push(group);
                pos += group_len + 2;
            }
            t => {
                args.push(vec![t.clone()]);
                pos += 1;
            }
        }
    }

    (pos, args)
}

fn substitute_params(
    body: &[Token],
    args: &[Vec<Token>],
    call_span: SourceSpan,
) -> Vec<SpannedToken> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < body.len() {
        if matches!(body[i], Token::Hash)
            && i + 1 < body.len()
            && let Token::Text(s) = &body[i + 1]
            && let Ok(idx) = s.parse::<usize>()
            && idx >= 1
            && idx <= args.len()
        {
            for t in &args[idx - 1] {
                result.push(SpannedToken {
                    token: t.clone(),
                    span: call_span,
                });
            }
            i += 2;
            continue;
        }
        result.push(SpannedToken {
            token: body[i].clone(),
            span: call_span,
        });
        i += 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::TeXLexer;

    fn expand_str(input: &str) -> Vec<SpannedToken> {
        let tokens = TeXLexer::new(input).tokenize_with_spans();
        expand_macros(tokens)
    }

    fn tokens_to_string(tokens: &[SpannedToken]) -> String {
        tokens.iter().map(|t| t.token.to_string()).collect()
    }

    #[test]
    fn test_simple_newcommand() {
        let tokens = expand_str(r"\newcommand{\foo}{bar}\foo");
        let s = tokens_to_string(&tokens);
        assert!(s.contains("bar"), "should contain 'bar', got: {s}");
    }

    #[test]
    fn test_newcommand_with_one_param() {
        let tokens = expand_str(r"\newcommand{\bold}[1]{\textbf{#1}}\bold{hello}");
        let s = tokens_to_string(&tokens);
        assert!(
            s.contains("\\textbf{hello}"),
            "should contain \\textbf{{hello}}, got: {s}"
        );
    }

    #[test]
    fn test_newcommand_with_two_params() {
        let tokens = expand_str(r"\newcommand{\greet}[2]{#1 and #2}\greet{hello}{world}");
        let s = tokens_to_string(&tokens);
        assert!(
            s.contains("hello and world"),
            "should contain 'hello and world', got: {s}"
        );
    }

    #[test]
    fn test_def_simple() {
        let tokens = expand_str(r"\def\x{y}\x");
        let s = tokens_to_string(&tokens);
        assert!(s.contains("y"), "should contain 'y', got: {s}");
    }

    #[test]
    fn test_recursive_expansion() {
        let tokens = expand_str(r"\newcommand{\foo}{\bar}\newcommand{\bar}{baz}\foo");
        let s = tokens_to_string(&tokens);
        assert!(s.contains("baz"), "should contain 'baz', got: {s}");
    }

    #[test]
    fn test_infinite_loop_protection() {
        let tokens = expand_str(r"\newcommand{\loop}{\loop}\loop");
        let count = tokens.len();
        assert!(count < 1000, "should stop expansion, got {count} tokens");
    }

    #[test]
    fn test_no_macros_passthrough() {
        let tokens = expand_str(r"\section{Hello} world");
        let s = tokens_to_string(&tokens);
        assert!(s.contains("\\section"), "should keep \\section, got: {s}");
        assert!(s.contains("Hello"), "should keep Hello, got: {s}");
    }

    #[test]
    fn test_renewcommand() {
        let tokens = expand_str(r"\newcommand{\foo}{old}\renewcommand{\foo}{new}\foo");
        let s = tokens_to_string(&tokens);
        assert!(s.contains("new"), "should be 'new', got: {s}");
        assert!(!s.contains("old"), "should not contain 'old', got: {s}");
    }

    #[test]
    fn test_renewcommand_before_newcommand() {
        let tokens = expand_str(r"\newcommand{\foo}{old}\foo\renewcommand{\foo}{new}\foo");
        let s = tokens_to_string(&tokens);
        let first_foo = s.find("old").is_some();
        let second_foo = s.find("new").is_some();
        assert!(first_foo, "first \\foo should expand to 'old', got: {s}");
        assert!(second_foo, "second \\foo should expand to 'new', got: {s}");
    }
}
