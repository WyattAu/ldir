use tower_lsp::lsp_types::{SemanticToken, SemanticTokensLegend};

use crate::folding::count_heading_level;

pub(crate) fn semantic_token_legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: vec![
            "heading".into(),
            "strong".into(),
            "emphasis".into(),
            "string".into(),
            "variable".into(),
            "comment".into(),
        ],
        token_modifiers: Vec::new(),
    }
}

/// Token type IDs matching the legend above
const TOKEN_HEADING: u32 = 0;
const TOKEN_BOLD: u32 = 1;
const TOKEN_ITALIC: u32 = 2;
const TOKEN_CODE: u32 = 3;
#[allow(dead_code)]
const TOKEN_LINK: u32 = 4;
const TOKEN_COMMENT: u32 = 5;

/// Compute semantic tokens for the given document content.
pub(crate) fn compute_semantic_tokens(content: &str) -> Vec<SemanticToken> {
    let mut tokens = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        if count_heading_level(line).is_some() {
            let start = line.find('#').unwrap_or(0);
            let trimmed = line.trim_start_matches('#').trim_start();
            let length = trimmed.len();
            if length > 0 {
                tokens.push(SemanticToken {
                    delta_line: line_idx as u32,
                    delta_start: start as u32,
                    length: length as u32,
                    token_type: TOKEN_HEADING,
                    token_modifiers_bitset: 0,
                });
            }
        }

        find_inline_token(line, "**", TOKEN_BOLD, &mut tokens, line_idx);
        find_inline_token(line, "*", TOKEN_ITALIC, &mut tokens, line_idx);
        find_inline_token(line, "`", TOKEN_CODE, &mut tokens, line_idx);

        if line.starts_with("> ") {
            tokens.push(SemanticToken {
                delta_line: line_idx as u32,
                delta_start: 0,
                length: line.len() as u32,
                token_type: TOKEN_COMMENT,
                token_modifiers_bitset: 0,
            });
        }
    }

    tokens
}

fn find_inline_token(
    line: &str,
    marker: &str,
    token_type: u32,
    tokens: &mut Vec<SemanticToken>,
    line_idx: usize,
) {
    let mut pos = 0;
    while let Some(idx) = line[pos..].find(marker) {
        let start = pos + idx;
        let remaining = &line[start + marker.len()..];
        if let Some(inner_end) = remaining.find(marker) {
            let content_len = inner_end;
            if content_len > 0 {
                tokens.push(SemanticToken {
                    delta_line: line_idx as u32,
                    delta_start: (start + marker.len()) as u32,
                    length: content_len as u32,
                    token_type,
                    token_modifiers_bitset: 0,
                });
            }
            pos = start + marker.len() + inner_end + marker.len();
        } else {
            pos = start + marker.len();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_headings() {
        let content = "# Title\n## Section";
        let tokens = compute_semantic_tokens(content);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].delta_line, 0);
        assert_eq!(tokens[0].length, 5);
        assert_eq!(tokens[1].delta_line, 1);
        assert_eq!(tokens[1].length, 7);
    }

    #[test]
    fn test_semantic_bold() {
        let content = "some **bold** text";
        let tokens = compute_semantic_tokens(content);
        let bold_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TOKEN_BOLD)
            .collect();
        assert_eq!(bold_tokens.len(), 1);
        // "**bold**" starts at pos 5; content "bold" starts at 7
        assert_eq!(bold_tokens[0].delta_start, 7);
        assert_eq!(bold_tokens[0].length, 4);
    }

    #[test]
    fn test_semantic_code() {
        let content = "use `println!` here";
        let tokens = compute_semantic_tokens(content);
        let code_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TOKEN_CODE)
            .collect();
        assert_eq!(code_tokens.len(), 1);
        assert_eq!(code_tokens[0].delta_start, 5);
        assert_eq!(code_tokens[0].length, 8);
    }

    #[test]
    fn test_semantic_blockquote() {
        let content = "> a quote\n> more";
        let tokens = compute_semantic_tokens(content);
        let comment_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TOKEN_COMMENT)
            .collect();
        assert_eq!(comment_tokens.len(), 2);
    }

    #[test]
    fn test_legend_has_six_types() {
        let legend = semantic_token_legend();
        assert_eq!(legend.token_types.len(), 6);
    }
}
