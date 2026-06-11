use tower_lsp::lsp_types::{FoldingRange, FoldingRangeKind};

/// Compute folding ranges for the given document content.
pub fn compute_folding_ranges(content: &str) -> Vec<FoldingRange> {
    let mut ranges = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut stack: Vec<usize> = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        if let Some(level) = count_heading_level(line) {
            while let Some(&top) = stack.last() {
                let top_level = count_heading_level(lines[top]).unwrap_or(1);
                if top_level >= level {
                    let start_line = stack.pop().unwrap_or(i);
                    ranges.push(FoldingRange {
                        start_line: start_line as u32,
                        start_character: None,
                        end_line: i.saturating_sub(1) as u32,
                        end_character: None,
                        kind: Some(FoldingRangeKind::Region),
                        collapsed_text: None,
                    });
                } else {
                    break;
                }
            }
            stack.push(i);
            continue;
        }

        if line.trim_start().starts_with("```") {
            let top_is_code = stack
                .first()
                .is_some_and(|&l| lines[l].trim_start().starts_with("```"));
            if top_is_code {
                if let Some(start_line) = stack.pop() {
                    ranges.push(FoldingRange {
                        start_line: start_line as u32,
                        start_character: None,
                        end_line: i as u32,
                        end_character: None,
                        kind: Some(FoldingRangeKind::Region),
                        collapsed_text: None,
                    });
                }
            } else {
                stack.push(i);
            }
        }

        if line.starts_with("> ") {
            let in_blockquote = stack.last().is_some_and(|&l| lines[l].starts_with("> "));
            if !in_blockquote {
                stack.push(i);
            }
        } else if let Some(&start_line) = stack.last().filter(|&&l| lines[l].starts_with("> ")) {
            stack.pop();
            ranges.push(FoldingRange {
                start_line: start_line as u32,
                start_character: None,
                end_line: i.saturating_sub(1) as u32,
                end_character: None,
                kind: Some(FoldingRangeKind::Region),
                collapsed_text: None,
            });
        }

        let line_has_pipe = line.contains('|');
        let stack_top_has_pipe = stack.last().is_some_and(|&l| lines[l].contains('|'));
        if line_has_pipe && !stack_top_has_pipe {
            stack.push(i);
        } else if !line_has_pipe
            && stack_top_has_pipe
            && let Some(start_line) = stack.pop()
        {
            ranges.push(FoldingRange {
                start_line: start_line as u32,
                start_character: None,
                end_line: i.saturating_sub(1) as u32,
                end_character: None,
                kind: Some(FoldingRangeKind::Region),
                collapsed_text: None,
            });
        }
    }

    while let Some(start_line) = stack.pop() {
        ranges.push(FoldingRange {
            start_line: start_line as u32,
            start_character: None,
            end_line: (lines.len() - 1) as u32,
            end_character: None,
            kind: Some(FoldingRangeKind::Region),
            collapsed_text: None,
        });
    }

    ranges
}

pub(crate) fn count_heading_level(line: &str) -> Option<u32> {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') {
        let count = trimmed.chars().take_while(|&c| c == '#').count();
        if (1..=6).contains(&count) {
            return Some(count as u32);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_folding_headings() {
        let content = "# Title\n\nSome content\n\n## Section\n\nMore content";
        let ranges = compute_folding_ranges(content);
        assert_eq!(ranges.len(), 2);
        // ## Section folds from line 4 to end
        assert_eq!(ranges[0].start_line, 4);
        assert_eq!(ranges[0].end_line, 6);
        // # Title folds from line 0 to end (encompasses child sections)
        assert_eq!(ranges[1].start_line, 0);
        assert_eq!(ranges[1].end_line, 6);
    }

    #[test]
    fn test_folding_code_blocks() {
        let content = "```rust\nfn main() {}\n```";
        let ranges = compute_folding_ranges(content);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start_line, 0);
        assert_eq!(ranges[0].end_line, 2);
    }

    #[test]
    fn test_folding_blockquotes() {
        let content = "> line one\n> line two\n> line three\n\nother";
        let ranges = compute_folding_ranges(content);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start_line, 0);
        assert_eq!(ranges[0].end_line, 2);
    }

    #[test]
    fn test_folding_tables() {
        let content = "| A | B |\n|---|---|\n| 1 | 2 |\n\nparagraph";
        let ranges = compute_folding_ranges(content);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start_line, 0);
        assert_eq!(ranges[0].end_line, 2);
    }

    #[test]
    fn test_folding_nested() {
        let content = "# H1\n\n## H2\n\npara\n\n### H3\n\nsub\n\n## H2b\n\nend";
        let ranges = compute_folding_ranges(content);
        // H3 (level 3) closed by H2b (level 2): 3 >= 2
        // H2 (level 2) closed by H2b (level 2): 2 >= 2
        // H1 (level 1) NOT closed: 1 >= 2 is false
        // End: close remaining H2b and H1
        assert_eq!(ranges.len(), 4);
        // ### H3 closed by ## H2b
        assert_eq!(ranges[0].start_line, 6);
        assert_eq!(ranges[0].end_line, 9);
        // ## H2 closed by ## H2b (same level)
        assert_eq!(ranges[1].start_line, 2);
        assert_eq!(ranges[1].end_line, 9);
        // ## H2b closed at end
        assert_eq!(ranges[2].start_line, 10);
        assert_eq!(ranges[2].end_line, 12);
        // # H1 closed at end (encompasses all)
        assert_eq!(ranges[3].start_line, 0);
        assert_eq!(ranges[3].end_line, 12);
    }
}
