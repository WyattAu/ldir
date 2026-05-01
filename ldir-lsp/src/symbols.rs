use tower_lsp::lsp_types::{DocumentSymbol, Position, Range, SymbolKind, Url};

use crate::detect_extension;

#[allow(deprecated)]
fn make_symbol(
    name: String,
    detail: Option<String>,
    kind: SymbolKind,
    range: Range,
    selection_range: Range,
) -> DocumentSymbol {
    DocumentSymbol {
        name,
        detail,
        kind,
        tags: None,
        deprecated: None,
        range,
        selection_range,
        children: None,
    }
}

/// Extract document symbols (headings, environments) from the given text.
pub fn extract_symbols(text: &str, uri: &Url) -> Vec<DocumentSymbol> {
    let ext = detect_extension(uri.path());
    match ext {
        "md" => extract_markdown_symbols(text),
        "tex" => extract_tex_symbols(text),
        "typ" => extract_typst_symbols(text),
        _ => Vec::new(),
    }
}

fn heading_symbol_kind(level: usize) -> SymbolKind {
    match level {
        1 => SymbolKind::MODULE,
        2 => SymbolKind::NAMESPACE,
        3 | 4 => SymbolKind::FUNCTION,
        5 | 6 => SymbolKind::CONSTANT,
        _ => SymbolKind::STRING,
    }
}

fn make_heading_range(line_num: usize, line: &str, indent: usize, content_start: usize) -> (Range, Range) {
    let range = Range {
        start: Position {
            line: line_num as u32,
            character: indent as u32,
        },
        end: Position {
            line: line_num as u32,
            character: line.len() as u32,
        },
    };
    let selection_range = Range {
        start: Position {
            line: line_num as u32,
            character: content_start as u32,
        },
        end: Position {
            line: line_num as u32,
            character: line.len() as u32,
        },
    };
    (range, selection_range)
}

fn extract_markdown_symbols(text: &str) -> Vec<DocumentSymbol> {
    let mut symbols = Vec::new();
    for (line_num, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix('#') {
            let (extra, name) = count_hashes(rest);
            let level = 1 + extra;
            if (1..=6).contains(&level) && !name.is_empty() {
                let indent = line.len() - trimmed.len();
                let (range, selection_range) =
                    make_heading_range(line_num, line, indent, indent + level + 1);
                symbols.push(make_symbol(
                    name.trim().to_string(),
                    Some(format!("Heading {level}")),
                    heading_symbol_kind(level),
                    range,
                    selection_range,
                ));
            }
        }
    }
    symbols
}

fn count_hashes(s: &str) -> (usize, &str) {
    let count = s.chars().take_while(|&c| c == '#').count();
    (count, &s[count..])
}

fn extract_tex_symbols(text: &str) -> Vec<DocumentSymbol> {
    let mut symbols = Vec::new();
    for (line_num, line) in text.lines().enumerate() {
        if let Some((name, kind, prefix_len)) = extract_tex_heading(line) {
            let (range, selection_range) =
                make_heading_range(line_num, line, 0, prefix_len);
            symbols.push(make_symbol(
                name.to_string(),
                Some(kind.to_string()),
                tex_heading_symbol_kind(kind),
                range,
                selection_range,
            ));
        } else if let Some((env_name, prefix_len)) = extract_tex_environment(line) {
            let (range, selection_range) =
                make_heading_range(line_num, line, 0, prefix_len);
            symbols.push(make_symbol(
                env_name.to_string(),
                Some(format!("\\begin{{{env_name}}}")),
                SymbolKind::CLASS,
                range,
                selection_range,
            ));
        }
    }
    symbols
}

fn extract_tex_heading(line: &str) -> Option<(&str, &str, usize)> {
    const COMMANDS: &[(&str, &str)] = &[
        ("\\chapter{", "chapter"),
        ("\\section{", "section"),
        ("\\subsection{", "subsection"),
        ("\\subsubsection{", "subsubsection"),
        ("\\paragraph{", "paragraph"),
        ("\\subparagraph{", "subparagraph"),
    ];
    for &(cmd, kind) in COMMANDS {
        if let Some(rest) = line.strip_prefix(cmd) {
            if let Some(end) = rest.find('}') {
                return Some((&rest[..end], kind, cmd.len()));
            }
        }
    }
    None
}

fn extract_tex_environment(line: &str) -> Option<(&str, usize)> {
    let prefix = "\\begin{";
    if let Some(rest) = line.strip_prefix(prefix) {
        if let Some(end) = rest.find('}') {
            return Some((&rest[..end], prefix.len()));
        }
    }
    None
}

fn tex_heading_symbol_kind(kind: &str) -> SymbolKind {
    match kind {
        "chapter" => SymbolKind::MODULE,
        "section" => SymbolKind::NAMESPACE,
        "subsection" => SymbolKind::FUNCTION,
        "subsubsection" | "paragraph" | "subparagraph" => SymbolKind::CONSTANT,
        _ => SymbolKind::STRING,
    }
}

fn extract_typst_symbols(text: &str) -> Vec<DocumentSymbol> {
    let mut symbols = Vec::new();
    for (line_num, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('=') {
            let (level, name) = count_equals(trimmed);
            if (1..=6).contains(&level) && !name.is_empty() {
                let indent = line.len() - trimmed.len();
                let (range, selection_range) =
                    make_heading_range(line_num, line, indent, indent + level + 1);
                symbols.push(make_symbol(
                    name.trim().to_string(),
                    Some(format!("Heading {level}")),
                    heading_symbol_kind(level),
                    range,
                    selection_range,
                ));
            }
        }
    }
    symbols
}

fn count_equals(s: &str) -> (usize, &str) {
    let count = s.chars().take_while(|&c| c == '=').count();
    (count, &s[count..])
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    fn make_uri(ext: &str) -> Url {
        Url::parse(&format!("file:///test/doc.{ext}")).unwrap()
    }

    fn make_unknown_uri() -> Url {
        Url::parse("file:///test/doc.xyz").unwrap()
    }

    #[test]
    fn test_empty_markdown() {
        let uri = make_uri("md");
        let symbols = extract_symbols("", &uri);
        assert!(symbols.is_empty());
    }

    #[test]
    fn test_empty_tex() {
        let uri = make_uri("tex");
        let symbols = extract_symbols("", &uri);
        assert!(symbols.is_empty());
    }

    #[test]
    fn test_empty_typst() {
        let uri = make_uri("typ");
        let symbols = extract_symbols("", &uri);
        assert!(symbols.is_empty());
    }

    #[test]
    fn test_unknown_format() {
        let uri = make_unknown_uri();
        let symbols = extract_symbols("some content", &uri);
        assert!(symbols.is_empty());
    }

    #[test]
    fn test_markdown_headings() {
        let uri = make_uri("md");
        let text = "# Title\n\n## Section\n\n### Subsection\n\n#### Level 4";
        let symbols = extract_symbols(text, &uri);
        assert_eq!(symbols.len(), 4);
        assert_eq!(symbols[0].name, "Title");
        assert_eq!(symbols[0].kind, SymbolKind::MODULE);
        assert_eq!(symbols[1].name, "Section");
        assert_eq!(symbols[1].kind, SymbolKind::NAMESPACE);
        assert_eq!(symbols[2].name, "Subsection");
        assert_eq!(symbols[2].kind, SymbolKind::FUNCTION);
        assert_eq!(symbols[3].name, "Level 4");
        assert_eq!(symbols[3].kind, SymbolKind::FUNCTION);
    }

    #[test]
    fn test_markdown_heading_levels_5_6() {
        let uri = make_uri("md");
        let text = "##### H5\n\n###### H6";
        let symbols = extract_symbols(text, &uri);
        assert_eq!(symbols.len(), 2);
        assert_eq!(symbols[0].kind, SymbolKind::CONSTANT);
        assert_eq!(symbols[1].kind, SymbolKind::CONSTANT);
    }

    #[test]
    fn test_markdown_heading_with_indent() {
        let uri = make_uri("md");
        let text = "  # Indented Heading";
        let symbols = extract_symbols(text, &uri);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "Indented Heading");
        assert_eq!(symbols[0].range.start.character, 2);
    }

    #[test]
    fn test_markdown_non_heading_hashes() {
        let uri = make_uri("md");
        let text = "This is not a heading #hashtag";
        let symbols = extract_symbols(text, &uri);
        assert!(symbols.is_empty());
    }

    #[test]
    fn test_tex_sections() {
        let uri = make_uri("tex");
        let text = r"\section{Intro}
\subsection{Details}
\subsubsection{Deep}";
        let symbols = extract_symbols(text, &uri);
        assert_eq!(symbols.len(), 3);
        assert_eq!(symbols[0].name, "Intro");
        assert_eq!(symbols[0].kind, SymbolKind::NAMESPACE);
        assert_eq!(symbols[1].name, "Details");
        assert_eq!(symbols[1].kind, SymbolKind::FUNCTION);
        assert_eq!(symbols[2].name, "Deep");
        assert_eq!(symbols[2].kind, SymbolKind::CONSTANT);
    }

    #[test]
    fn test_tex_chapter() {
        let uri = make_uri("tex");
        let text = r"\chapter{First Chapter}";
        let symbols = extract_symbols(text, &uri);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "First Chapter");
        assert_eq!(symbols[0].kind, SymbolKind::MODULE);
    }

    #[test]
    fn test_tex_environments() {
        let uri = make_uri("tex");
        let text = r"\begin{equation}\end{equation}
\begin{figure}\end{figure}
\begin{table}\end{table}";
        let symbols = extract_symbols(text, &uri);
        assert_eq!(symbols.len(), 3);
        for sym in &symbols {
            assert_eq!(sym.kind, SymbolKind::CLASS);
        }
        assert_eq!(symbols[0].name, "equation");
        assert_eq!(symbols[1].name, "figure");
        assert_eq!(symbols[2].name, "table");
    }

    #[test]
    fn test_tex_mixed() {
        let uri = make_uri("tex");
        let text = r"\chapter{Chap}
\section{Sec}
\begin{equation}x=1\end{equation}";
        let symbols = extract_symbols(text, &uri);
        assert_eq!(symbols.len(), 3);
        assert_eq!(symbols[0].kind, SymbolKind::MODULE);
        assert_eq!(symbols[1].kind, SymbolKind::NAMESPACE);
        assert_eq!(symbols[2].kind, SymbolKind::CLASS);
    }

    #[test]
    fn test_typst_headings() {
        let uri = make_uri("typ");
        let text = "= Title\n\n== Section\n\n=== Subsection";
        let symbols = extract_symbols(text, &uri);
        assert_eq!(symbols.len(), 3);
        assert_eq!(symbols[0].name, "Title");
        assert_eq!(symbols[0].kind, SymbolKind::MODULE);
        assert_eq!(symbols[1].name, "Section");
        assert_eq!(symbols[1].kind, SymbolKind::NAMESPACE);
        assert_eq!(symbols[2].name, "Subsection");
        assert_eq!(symbols[2].kind, SymbolKind::FUNCTION);
    }

    #[test]
    fn test_typst_heading_levels() {
        let uri = make_uri("typ");
        let text = "==== L4\n\n===== L5\n\n====== L6";
        let symbols = extract_symbols(text, &uri);
        assert_eq!(symbols.len(), 3);
        assert_eq!(symbols[0].kind, SymbolKind::FUNCTION);
        assert_eq!(symbols[1].kind, SymbolKind::CONSTANT);
        assert_eq!(symbols[2].kind, SymbolKind::CONSTANT);
    }

    #[test]
    fn test_typst_heading_with_indent() {
        let uri = make_uri("typ");
        let text = "  == Indented";
        let symbols = extract_symbols(text, &uri);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "Indented");
        assert_eq!(symbols[0].range.start.character, 2);
    }

    #[test]
    fn test_symbol_ranges() {
        let uri = make_uri("md");
        let text = "# Hello\nWorld\n## Sub";
        let symbols = extract_symbols(text, &uri);
        assert_eq!(symbols.len(), 2);
        assert_eq!(symbols[0].range.start.line, 0);
        assert_eq!(symbols[0].range.end.line, 0);
        assert_eq!(symbols[1].range.start.line, 2);
        assert_eq!(symbols[1].range.start.character, 0);
    }

    #[test]
    fn test_symbol_details() {
        let uri = make_uri("md");
        let text = "# My Heading";
        let symbols = extract_symbols(text, &uri);
        assert_eq!(symbols[0].detail.as_deref(), Some("Heading 1"));
    }
}
