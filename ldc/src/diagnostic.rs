use std::io::Write;
use std::path::{Path, PathBuf};

use crate::status::{styled, Color, COLOR_ENABLED};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DiagnosticKind {
    Error,
    Warning,
    Help,
    Note,
}

impl DiagnosticKind {
    fn label(self) -> &'static str {
        match self {
            DiagnosticKind::Error => "error",
            DiagnosticKind::Warning => "warning",
            DiagnosticKind::Help => "help",
            DiagnosticKind::Note => "note",
        }
    }

    fn color(self) -> Color {
        match self {
            DiagnosticKind::Error => Color::Red,
            DiagnosticKind::Warning => Color::Yellow,
            DiagnosticKind::Help => Color::Cyan,
            DiagnosticKind::Note => Color::Dim,
        }
    }
}

pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub code: Option<&'static str>,
    pub message: String,
    pub file: Option<PathBuf>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    #[allow(dead_code)]
    pub source_line: Option<String>,
    pub suggestion: Option<String>,
    pub help: Option<String>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            kind: DiagnosticKind::Error,
            code: None,
            message: message.into(),
            file: None,
            line: None,
            column: None,
            source_line: None,
            suggestion: None,
            help: None,
        }
    }

    #[allow(dead_code)]
    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            kind: DiagnosticKind::Warning,
            code: None,
            message: message.into(),
            file: None,
            line: None,
            column: None,
            source_line: None,
            suggestion: None,
            help: None,
        }
    }

    pub fn code(mut self, code: &'static str) -> Self {
        self.code = Some(code);
        self
    }

    pub fn file(mut self, path: impl Into<PathBuf>) -> Self {
        self.file = Some(path.into());
        self
    }

    #[allow(dead_code)]
    pub fn line(mut self, line: u32) -> Self {
        self.line = Some(line);
        self
    }

    #[allow(dead_code)]
    pub fn column(mut self, col: u32) -> Self {
        self.column = Some(col);
        self
    }

    #[allow(dead_code)]
    pub fn source_line(mut self, line: impl Into<String>) -> Self {
        self.source_line = Some(line.into());
        self
    }

    pub fn suggestion(mut self, s: impl Into<String>) -> Self {
        self.suggestion = Some(s.into());
        self
    }

    pub fn help(mut self, h: impl Into<String>) -> Self {
        self.help = Some(h.into());
        self
    }
}

pub fn emit(diag: &Diagnostic) {
    let _ = emit_to(&mut std::io::stderr(), diag);
}

pub fn emit_to(w: &mut dyn Write, diag: &Diagnostic) -> std::io::Result<()> {
    let use_color = COLOR_ENABLED.load(std::sync::atomic::Ordering::Relaxed);

    let label_color = diag.kind.color();
    let label = styled(diag.kind.label(), label_color);
    let code_part = match diag.code {
        Some(c) => format!("[{c}]"),
        None => String::new(),
    };
    writeln!(w, "{label}{code_part}: {}", diag.message)?;

    if let (Some(file), Some(line)) = (&diag.file, diag.line) {
        let col_part = diag.column.map(|c| format!(":{c}")).unwrap_or_default();
        writeln!(w, "  --> {}: {}{}", file.display(), line, col_part)?;
    }

    if let Some(ref src) = diag.source_line {
        let line_no = diag.line.unwrap_or(0);
        let width = line_no.to_string().len().max(2);
        writeln!(w, "{line_no:>width$} | {}", src)?;

        if diag.column.is_some() || diag.suggestion.is_some() {
            let col = diag.column.unwrap_or(1);
            let gutter = " ".repeat(width + 3);
            let underline = " ".repeat((col - 1) as usize)
                + if diag.suggestion.is_some() { "^" } else { "-" };
            let underline_color = styled(&underline, label_color);
            writeln!(w, "{gutter}{underline_color}")?;

            if let Some(ref sug) = diag.suggestion {
                let sug_prefix = " ".repeat(width + 3 + (col - 1) as usize);
                let sug_colored = styled(&format!("= {sug}"), Color::Cyan);
                writeln!(w, "{sug_prefix}{sug_colored}")?;
            }
        }

        writeln!(w, "{:>width$} |", "")?;
    }

    if let Some(ref help) = diag.help {
        let prefix = styled("= help:", Color::Cyan);
        let reset = if use_color { "\x1b[0m" } else { "" };
        writeln!(w, "  {prefix} {help}{reset}")?;
    }

    Ok(())
}

pub fn suggest_alternatives<'a>(input: &str, candidates: &'a [&str]) -> Option<&'a str> {
    let input_lower = input.to_lowercase();

    for candidate in candidates {
        let cand_lower = candidate.to_lowercase();
        if levenshtein(&input_lower, &cand_lower) <= 2 {
            return Some(*candidate);
        }
    }

    if let Some(first_char) = input_lower.chars().next() {
        for candidate in candidates {
            if candidate.to_lowercase().starts_with(first_char) {
                return Some(*candidate);
            }
        }
    }

    None
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let m = a.len();
    let n = b.len();
    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    let mut row: Vec<usize> = (0..=n).collect();

    for i in 1..=m {
        let mut prev = row[0];
        row[0] = i;
        for j in 1..=n {
            let temp = row[j];
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            let new_val = (row[j] + 1).min(prev + cost).min(row[j - 1] + 1);
            row[j] = new_val;
            prev = temp;
        }
    }

    row[n]
}

pub fn suggest_similar_files(target: &str, directory: &Path) -> Vec<String> {
    let target_lower = target.to_lowercase();
    let target_stem = Path::new(target)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(target);

    let mut suggestions = Vec::new();

    if let Ok(entries) = std::fs::read_dir(directory) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.to_lowercase() == target_lower {
                    continue;
                }
                if levenshtein(&target_lower, &name.to_lowercase()) <= 3 {
                    suggestions.push(name.to_string());
                } else if let Some(stem) = Path::new(name).file_stem().and_then(|s| s.to_str())
                    && levenshtein(&target_stem.to_lowercase(), &stem.to_lowercase()) <= 2
                {
                    suggestions.push(name.to_string());
                }
            }
        }
    }

    suggestions.sort();
    suggestions.truncate(5);
    suggestions
}

pub fn diagnose_anyhow_error(err: &anyhow::Error) -> Diagnostic {
    let msg = err.to_string();

    let file_not_found = extract_path_from_context(err, |s| {
        s.contains("No such file or directory")
            || s.contains("os error 2")
            || s.contains("cannot find")
    });

    if let Some(path) = file_not_found {
        let mut diag = Diagnostic::error(format!("file not found: {}", path.display()));
        diag = diag.file(&path);

        if let Some(parent) = path.parent() {
            let similar = suggest_similar_files(
                path.file_name().and_then(|s| s.to_str()).unwrap_or(""),
                parent,
            );
            if !similar.is_empty() {
                diag = diag.suggestion(format!("Did you mean '{}'?", similar.join("', '")));
            }
        }

        diag = diag
            .help("Check that the file exists and you have permission to read it.");
        return diag;
    }

    if msg.contains("unsupported input format") {
        let supported = &[".md, .tex, .typ, .html, .htm, .adoc, .org, .docx"];
        return Diagnostic::error(msg)
            .code("E0001")
            .help(format!("Supported input formats: {}", supported.join(", ")));
    }

    if msg.contains("unsupported output format") {
        let candidates: &[&str] = &["pdf", "html", "epub", "txt", "docx", "sir2", "ldir", "gir"];
        let mut diag = Diagnostic::error(&msg).code("E0002");
        if let Some(fmt) = extract_format_from_message(&msg, "unsupported output format: ")
            && let Some(suggestion) = suggest_alternatives(fmt, candidates)
        {
            diag = diag.suggestion(format!("Did you mean '{}'?", suggestion));
        }
        diag = diag.help(format!("Supported output formats: {}", candidates.join(", ")));
        return diag;
    }

    if msg.contains("failed to read font")
        || msg.contains("invalid font file")
        || msg.contains("font family")
    {
        let mut diag = Diagnostic::error(msg).code("E0003");
        diag = diag.help(
            "Use --font <family> to specify a font family, \
             or --font-path <path> to specify a .ttf/.otf file directly.",
        );
        return diag;
    }

    if msg.contains("compilation failed") || msg.contains("L-IR compilation failed") {
        return Diagnostic::error(msg).code("E0004").help(
            "Review the source document for syntax errors. \
             Use --lir for the L-IR pipeline, or check nested block structure.",
        );
    }

    if msg.contains("failed to create")
        || msg.contains("failed to write")
        || msg.contains("Permission denied")
    {
        let mut diag = Diagnostic::error(msg).code("E0005");
        if let Some(path) = extract_path_from_context(err, |s| {
            s.contains("failed to create")
                || s.contains("failed to write")
                || s.contains("Permission denied")
        }) {
            diag = diag.file(path);
        }
        diag = diag.help("Check directory permissions and available disk space.");
        return diag;
    }

    Diagnostic::error(msg)
}

fn extract_path_from_context(
    err: &anyhow::Error,
    predicate: impl Fn(&str) -> bool,
) -> Option<PathBuf> {
    let msg = err.to_string();

    if predicate(&msg)
        && let Some(path) = extract_first_path(&msg)
    {
        return Some(path);
    }

    for cause in err.chain().skip(1) {
        let cmsg = cause.to_string();
        if predicate(&cmsg)
            && let Some(path) = extract_first_path(&msg)
        {
            return Some(path);
        }
    }

    if let Some(path) = extract_first_path(&msg) {
        for cause in err.chain().skip(1) {
            if predicate(&cause.to_string()) {
                return Some(path);
            }
        }
    }

    None
}

fn extract_first_path(msg: &str) -> Option<PathBuf> {
    if let Some(idx) = msg.find('\'') {
        let rest = &msg[idx + 1..];
        if let Some(end) = rest.find('\'') {
            let path_str = &rest[..end];
            if Path::new(path_str).extension().is_some() {
                return Some(PathBuf::from(path_str));
            }
        }
    }
    for word in msg.split([' ', '(', ')']) {
        let trimmed = word.trim_matches(':').trim();
        if (trimmed.contains('/') || trimmed.contains('\\'))
            && Path::new(trimmed).extension().is_some()
        {
            return Some(PathBuf::from(trimmed));
        }
    }
    None
}

fn extract_format_from_message<'a>(msg: &'a str, prefix: &str) -> Option<&'a str> {
    let idx = msg.find(prefix)?;
    let rest = &msg[idx + prefix.len()..];
    let end = rest.find('.').unwrap_or(rest.len());
    Some(rest[..end].trim())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    fn regex_agnostic(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == 0x1B && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
                i += 2;
                while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b';') {
                    i += 1;
                }
                if i < bytes.len() && bytes[i].is_ascii_alphabetic() {
                    i += 1;
                }
            } else {
                result.push(bytes[i] as char);
                i += 1;
            }
        }
        result
    }

    #[test]
    fn test_diagnostic_formatting() {
        COLOR_ENABLED.store(false, Ordering::Relaxed);
        let mut buf = Vec::new();

        let diag = Diagnostic::error("unknown input format 'doc'")
            .code("E0001")
            .file("/path/to/input.md")
            .line(12)
            .column(5)
            .source_line("ldc --format doc input.md")
            .suggestion("Did you mean 'pdf'?")
            .help("Supported formats: pdf, html, epub, docx");

        emit_to(&mut buf, &diag).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("error[E0001]: unknown input format 'doc'"));
        assert!(output.contains("--> /path/to/input.md: 12:5"));
        assert!(output.contains("ldc --format doc input.md"));
        assert!(output.contains("Did you mean 'pdf'?"));
        assert!(output.contains("= help: Supported formats: pdf, html, epub, docx"));
    }

    #[test]
    fn test_diagnostic_formatting_with_colors() {
        COLOR_ENABLED.store(true, Ordering::Relaxed);
        let mut buf = Vec::new();

        let diag = Diagnostic::warning("deprecated option used").help("Use --new-opt instead.");
        emit_to(&mut buf, &diag).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let stripped = regex_agnostic(&output);
        assert!(stripped.contains("warning: deprecated option used"));
        assert!(output.contains("\x1b[33m"));
        assert!(stripped.contains("= help: Use --new-opt instead."));
    }

    #[test]
    fn test_suggest_alternatives() {
        let candidates: &[&str] = &["pdf", "html", "epub", "txt", "docx", "sir2", "ldir"];
        assert_eq!(suggest_alternatives("pd", candidates), Some("pdf"));
        assert_eq!(suggest_alternatives("epup", candidates), Some("epub"));
        assert_eq!(suggest_alternatives("htlm", candidates), Some("html"));
        assert_eq!(suggest_alternatives("xyz", candidates), None);
        assert_eq!(suggest_alternatives("", candidates), None);
        assert_eq!(suggest_alternatives("p", candidates), Some("pdf"));
    }

    #[test]
    fn test_file_not_found_diagnostic() {
        let tmp = tempfile::tempdir().unwrap();
        let missing = tmp.path().join("my_doc.md");

        std::fs::write(tmp.path().join("my_doc_copy.md"), "test").unwrap();

        let mut suggestions = suggest_similar_files(
            missing.file_name().and_then(|s| s.to_str()).unwrap_or(""),
            tmp.path(),
        );

        if suggestions.is_empty() {
            std::fs::write(tmp.path().join("my_doc_v2.md"), "test").unwrap();
            suggestions = suggest_similar_files(
                missing.file_name().and_then(|s| s.to_str()).unwrap_or(""),
                tmp.path(),
            );
        }

        let _ = suggestions;

        let err = std::fs::read(missing.as_os_str()).unwrap_err();
        let err = anyhow::Error::from(err)
            .context(format!("failed to read {}", missing.display()));

        let diag = diagnose_anyhow_error(&err);

        assert_eq!(diag.kind, DiagnosticKind::Error);
        assert!(diag.file.is_some());
    }

    #[test]
    fn test_unknown_format_diagnostic() {
        let candidates: &[&str] = &["pdf", "html", "epub", "txt", "docx", "sir2", "ldir", "gir"];
        let err = anyhow::anyhow!("unsupported output format: pdff");
        let diag = diagnose_anyhow_error(&err);

        assert_eq!(diag.kind, DiagnosticKind::Error);
        assert!(diag.code == Some("E0002"));
        assert!(diag.suggestion.is_some());
        assert!(diag.help.is_some());

        let suggestion = diag.suggestion.unwrap();
        let suggestion_has_candidate = candidates.iter().any(|c| suggestion.contains(*c));
        assert!(
            suggestion_has_candidate,
            "suggestion '{suggestion}' should reference a known format"
        );
    }

    #[test]
    fn test_levenshtein() {
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("abc", "abc"), 0);
        assert_eq!(levenshtein("abc", "abd"), 1);
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("pdf", "pdff"), 1);
    }

    #[test]
    fn test_diagnostic_builder_pattern() {
        let diag = Diagnostic::error("test error")
            .code("E0099")
            .file("/tmp/test.rs")
            .line(42);

        assert_eq!(diag.kind, DiagnosticKind::Error);
        assert_eq!(diag.code, Some("E0099"));
        assert_eq!(diag.file.as_deref(), Some(Path::new("/tmp/test.rs")));
        assert_eq!(diag.line, Some(42));
    }
}
