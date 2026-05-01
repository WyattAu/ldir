use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, Url};

use crate::detect_extension;

/// Compute diagnostics for the given document text based on its format.
pub fn compute_diagnostics(text: &str, uri: &Url) -> Vec<Diagnostic> {
    let ext = detect_extension(uri.path());
    match ext {
        "md" => compute_markdown_diagnostics(text),
        "tex" => compute_brace_diagnostics(text),
        "typ" => compute_brace_diagnostics(text),
        _ => Vec::new(),
    }
}

fn compute_markdown_diagnostics(text: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for (line_num, line) in text.lines().enumerate() {
        let mut bracket_depth = 0u32;
        let mut chars = line.char_indices().peekable();
        let mut in_link_url = false;
        while let Some((_, ch)) = chars.next() {
            match ch {
                '[' if !in_link_url => bracket_depth = bracket_depth.saturating_add(1),
                ']' if bracket_depth > 0 && !in_link_url => {
                    bracket_depth = bracket_depth.saturating_sub(1);
                    if chars.peek().map(|&(_, c)| c) == Some('(') {
                        chars.next();
                        in_link_url = true;
                    }
                }
                ')' if in_link_url => in_link_url = false,
                _ => {}
            }
        }
        if in_link_url {
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position {
                        line: line_num as u32,
                        character: 0,
                    },
                    end: Position {
                        line: line_num as u32,
                        character: line.len() as u32,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                code: None,
                code_description: None,
                source: Some("ldir-lsp".to_string()),
                message: "Unclosed link: missing closing parenthesis".to_string(),
                related_information: None,
                tags: None,
                data: None,
            });
        }
    }
    diagnostics
}

fn compute_brace_diagnostics(text: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut depth: i32 = 0;
    let lines: Vec<&str> = text.lines().collect();
    for (line_num, &line) in lines.iter().enumerate() {
        for ch in line.chars() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth < 0 {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position {
                                    line: line_num as u32,
                                    character: 0,
                                },
                                end: Position {
                                    line: line_num as u32,
                                    character: line.len() as u32,
                                },
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: None,
                            code_description: None,
                            source: Some("ldir-lsp".to_string()),
                            message: "Unmatched closing brace".to_string(),
                            related_information: None,
                            tags: None,
                            data: None,
                        });
                        depth = 0;
                    }
                }
                _ => {}
            }
        }
    }
    if depth > 0 {
        let last_line = lines.len().saturating_sub(1);
        let last_len = lines.last().map_or(0, |l| l.len());
        diagnostics.push(Diagnostic {
            range: Range {
                start: Position {
                    line: last_line as u32,
                    character: 0,
                },
                end: Position {
                    line: last_line as u32,
                    character: last_len as u32,
                },
            },
            severity: Some(DiagnosticSeverity::WARNING),
            code: None,
            code_description: None,
            source: Some("ldir-lsp".to_string()),
            message: format!("Unclosed brace(s): {depth} unmatched '{{'"),
            related_information: None,
            tags: None,
            data: None,
        });
    }
    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    fn make_uri(ext: &str) -> Url {
        Url::parse(&format!("file:///test/doc.{ext}")).unwrap()
    }

    #[test]
    fn test_empty_document_no_diagnostics() {
        let uri = make_uri("md");
        assert!(compute_diagnostics("", &uri).is_empty());
    }

    #[test]
    fn test_well_formed_markdown_links() {
        let uri = make_uri("md");
        let text = "[hello](https://example.com)\nNo link here";
        assert!(compute_diagnostics(text, &uri).is_empty());
    }

    #[test]
    fn test_unclosed_markdown_link() {
        let uri = make_uri("md");
        let text = "[click here](https://example.com";
        let diag = compute_diagnostics(text, &uri);
        assert_eq!(diag.len(), 1);
        assert_eq!(diag[0].severity, Some(DiagnosticSeverity::ERROR));
    }

    #[test]
    fn test_unclosed_brace_tex() {
        let uri = make_uri("tex");
        let text = r"\textbf{unclosed";
        let diag = compute_diagnostics(text, &uri);
        assert_eq!(diag.len(), 1);
        assert_eq!(diag[0].severity, Some(DiagnosticSeverity::WARNING));
    }

    #[test]
    fn test_extra_closing_brace_tex() {
        let uri = make_uri("tex");
        let text = r"text with extra } brace";
        let diag = compute_diagnostics(text, &uri);
        assert_eq!(diag.len(), 1);
        assert_eq!(diag[0].severity, Some(DiagnosticSeverity::ERROR));
    }

    #[test]
    fn test_balanced_braces_tex() {
        let uri = make_uri("tex");
        let text = r"\textbf{bold} and \textit{italic}";
        assert!(compute_diagnostics(text, &uri).is_empty());
    }

    #[test]
    fn test_balanced_braces_typst() {
        let uri = make_uri("typ");
        let text = "#let x = {1 + 2}";
        assert!(compute_diagnostics(text, &uri).is_empty());
    }

    #[test]
    fn test_unclosed_brace_typst() {
        let uri = make_uri("typ");
        let text = "#let x = {1 + 2";
        let diag = compute_diagnostics(text, &uri);
        assert_eq!(diag.len(), 1);
        assert_eq!(diag[0].severity, Some(DiagnosticSeverity::WARNING));
    }

    #[test]
    fn test_unknown_format_no_diagnostics() {
        let uri = Url::parse("file:///test.docx").unwrap();
        assert!(compute_diagnostics("anything", &uri).is_empty());
    }

    #[test]
    fn test_nested_braces() {
        let uri = make_uri("tex");
        let text = r"\textbf{outer \textit{inner}}";
        assert!(compute_diagnostics(text, &uri).is_empty());
    }
}
