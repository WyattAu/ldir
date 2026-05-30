//! HTML renderer for WASM — converts parsed Markdown to styled HTML.

use ldir_ir::sir::{BlockType, SIRDocument, SIROpcode, StyleModifier};

use std::collections::HashMap;

/// Render Markdown text to styled HTML.
pub fn render_markdown(markdown: &str) -> String {
    if markdown.trim().is_empty() {
        return String::new();
    }

    let doc = ldir_md::parse_markdown(markdown);
    render_sir_to_html(&doc)
}

/// Render an S-IR document to styled HTML.
pub fn render_sir_document(doc: &SIRDocument) -> String {
    render_sir_to_html(doc)
}

struct BlockSpan {
    block_type: BlockType,
    heading_level: u32,
    child_count: usize,
    is_header_row: bool,
}

fn render_sir_to_html(doc: &SIRDocument) -> String {
    let mut html = String::from(r#"<div class="ldir-document">"#);

    let mut link_url_for_block: HashMap<u32, String> = HashMap::new();
    for instr in doc.iter() {
        if instr.opcode() == SIROpcode::LinkData
            && let Some(url) = doc.payload_text(instr)
        {
            link_url_for_block.insert(instr.parent_id(), url.to_string());
        }
    }

    let mut open_blocks: Vec<BlockSpan> = Vec::new();
    let mut entity_to_block: HashMap<u32, usize> = HashMap::new();

    let instrs: Vec<_> = doc.iter().collect();
    let mut i = 0;

    while i < instrs.len() {
        let instr = instrs[i];
        match instr.opcode() {
            SIROpcode::PushBlock => {
                let bt = read_block_type(doc, instr.payload_offset());
                let heading_level = if bt == BlockType::Heading {
                    read_heading_level(doc, instr.payload_offset())
                } else {
                    0
                };

                let span_idx = open_blocks.len();
                entity_to_block.insert(instr.entity_id(), span_idx);
                let is_header_row = if bt == BlockType::TableRow {
                    let payload = doc.payload().get(instr.payload_offset(), 2);
                    payload.is_some_and(|b| b.len() >= 2 && b[1] == 1)
                } else {
                    false
                };

                open_blocks.push(BlockSpan {
                    block_type: bt,
                    heading_level,
                    child_count: 0,
                    is_header_row,
                });

                emit_block_open(&mut html, bt, heading_level, is_header_row);
            }
            SIROpcode::SetContent => {
                let parent_id = instr.parent_id();
                let parent_block_idx = entity_to_block.get(&parent_id);
                let bt = parent_block_idx
                    .map(|&idx| open_blocks[idx].block_type)
                    .unwrap_or(BlockType::Document);

                if bt == BlockType::Document {
                    i += 1;
                    continue;
                }

                if let Some(text) = doc.payload_text(instr) {
                    if bt == BlockType::Code {
                        let language = detect_language(text);
                        let highlighted = highlight_code(text, language);
                        let with_lines = add_line_numbers(&highlighted);
                        html.push_str(&with_lines);
                    } else if bt == BlockType::Image {
                        let src = escape_attr(text);
                        let alt = escape_html(text);
                        html.push_str(&format!("<img src=\"{src}\" alt=\"{alt}\">"));
                    } else {
                        let escaped = escape_html(text);

                        let mut style_stack: Vec<StyleModifier> = Vec::new();
                        let mut j = i + 1;
                        while j < instrs.len()
                            && instrs[j].opcode() == SIROpcode::ApplyStyle
                            && instrs[j].parent_id() == parent_id
                        {
                            let packed = instrs[j].payload_offset();
                            let (mods, is_push) = StyleModifier::from_packed(packed);
                            if is_push {
                                style_stack.push(mods);
                                j += 1;
                            } else {
                                break;
                            }
                        }

                        let styled = wrap_with_styles(&style_stack, &escaped);

                        if let Some(url) = link_url_for_block.get(&parent_id) {
                            html.push_str(&format!(
                                "<a href=\"{}\">{}</a>",
                                escape_attr(url),
                                styled
                            ));
                        } else {
                            html.push_str(&styled);
                        }
                    }
                }

                if let Some(&idx) = parent_block_idx {
                    open_blocks[idx].child_count += 1;
                }

                i += 1;
                continue;
            }
            SIROpcode::ApplyStyle | SIROpcode::LinkData | SIROpcode::InsertMath => {}
        }
        i += 1;
    }

    while let Some(span) = open_blocks.pop() {
        emit_block_close(
            &mut html,
            span.block_type,
            span.heading_level,
            span.is_header_row,
        );
    }

    html.push_str("</div>");
    html
}

fn wrap_with_styles(style_stack: &[StyleModifier], text: &str) -> String {
    let mut bold = false;
    let mut italic = false;
    let mut mono = false;
    let mut strike = false;
    for style in style_stack {
        if style.contains(StyleModifier::BOLD) {
            bold = true;
        }
        if style.contains(StyleModifier::ITALIC) {
            italic = true;
        }
        if style.contains(StyleModifier::MONO) {
            mono = true;
        }
        if style.contains(StyleModifier::STRIKE) {
            strike = true;
        }
    }

    if !bold && !italic && !mono && !strike {
        return text.to_string();
    }

    let mut result = String::with_capacity(text.len() + 40);
    if bold {
        result.push_str("<strong>");
    }
    if italic {
        result.push_str("<em>");
    }
    if mono {
        result.push_str("<code>");
    }
    if strike {
        result.push_str("<del>");
    }
    result.push_str(text);
    if strike {
        result.push_str("</del>");
    }
    if mono {
        result.push_str("</code>");
    }
    if italic {
        result.push_str("</em>");
    }
    if bold {
        result.push_str("</strong>");
    }
    result
}

fn detect_language(code: &str) -> &str {
    if code.contains("fn ")
        || code.contains("pub ")
        || code.contains("impl ")
        || code.contains("struct ")
        || code.contains("enum ")
        || code.contains("trait ")
        || code.contains("use ")
        || code.contains("mod ")
    {
        return "rust";
    }
    if code.contains("\\begin{")
        || code.contains("\\section")
        || code.contains("\\documentclass")
        || code.contains("\\usepackage")
    {
        return "latex";
    }
    if code.contains("#let ") || code.contains("#set ") || code.contains("#show ") {
        return "typst";
    }
    "text"
}

/// Apply syntax highlighting to a code block.
///
/// Tokenizes the input and wraps recognized tokens in `<span>` elements
/// with CSS classes: `.kw` (keyword), `.str` (string), `.cmt` (comment),
/// `.num` (number).
///
/// For unknown languages, returns the HTML-escaped input unchanged.
fn highlight_code(code: &str, language: &str) -> String {
    let keywords: &[&str] = match language {
        "rust" => &[
            "fn", "let", "mut", "pub", "struct", "impl", "use", "mod", "enum", "match", "if",
            "else", "for", "while", "loop", "return", "self", "super", "crate", "true", "false",
            "async", "await", "unsafe", "type", "trait", "where", "const", "static", "extern",
            "in",
        ],
        "latex" | "tex" => &[
            "\\section",
            "\\subsection",
            "\\subsubsection",
            "\\begin",
            "\\end",
            "\\textbf",
            "\\textit",
            "\\emph",
            "\\cite",
            "\\ref",
            "\\label",
            "\\equation",
            "\\frac",
            "\\sqrt",
            "\\int",
            "\\sum",
            "\\alpha",
            "\\beta",
            "\\gamma",
            "\\delta",
            "\\lambda",
            "\\pi",
            "\\infty",
            "\\partial",
            "\\documentclass",
            "\\usepackage",
            "\\hline",
        ],
        "typst" => &[
            "let", "set", "show", "import", "include", "context", "for", "if", "else", "while",
            "return", "break", "continue", "true", "false", "none", "auto",
        ],
        _ => return escape_html(code),
    };

    let mut result = String::with_capacity(code.len() * 2);
    let chars: Vec<char> = code.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Line comments (//)
        if i + 1 < len && chars[i] == '/' && chars[i + 1] == '/' {
            let start = i;
            while i < len && chars[i] != '\n' {
                i += 1;
            }
            let comment: String = chars[start..i].iter().collect();
            result.push_str("<span class=\"cmt\">");
            result.push_str(&escape_html(&comment));
            result.push_str("</span>");
            continue;
        }

        // Block comments (/* ... */)
        if i + 1 < len && chars[i] == '/' && chars[i + 1] == '*' {
            let start = i;
            i += 2;
            while i + 1 < len && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            if i + 1 < len {
                i += 2;
            } else {
                i = len;
            }
            let comment: String = chars[start..i].iter().collect();
            result.push_str("<span class=\"cmt\">");
            result.push_str(&escape_html(&comment));
            result.push_str("</span>");
            continue;
        }

        // Strings (double quote)
        if chars[i] == '"' {
            result.push_str("<span class=\"str\">");
            result.push('"');
            i += 1;
            while i < len && chars[i] != '"' {
                if chars[i] == '\\' && i + 1 < len {
                    let esc: String = chars[i..i + 2].iter().collect();
                    result.push_str(&escape_html(&esc));
                    i += 2;
                } else {
                    result.push_str(&escape_html(&chars[i].to_string()));
                    i += 1;
                }
            }
            if i < len {
                result.push('"');
                i += 1;
            }
            result.push_str("</span>");
            continue;
        }

        // Char literals (single quote) for Rust
        if chars[i] == '\'' && language == "rust" && i + 2 < len {
            let is_char = (chars[i + 1] != '\\' && chars[i + 2] == '\'')
                || (chars[i + 1] == '\\' && i + 3 < len && chars[i + 3] == '\'');
            if is_char {
                result.push_str("<span class=\"str\">");
                result.push('\'');
                i += 1;
                while i < len && chars[i] != '\'' {
                    if chars[i] == '\\' && i + 1 < len {
                        result.push(chars[i]);
                        result.push(chars[i + 1]);
                        i += 2;
                    } else {
                        result.push(chars[i]);
                        i += 1;
                    }
                }
                if i < len {
                    result.push('\'');
                    i += 1;
                }
                result.push_str("</span>");
                continue;
            }
        }

        // LaTeX commands (\word)
        if chars[i] == '\\' && (language == "latex" || language == "tex") {
            let start = i;
            i += 1;
            while i < len && chars[i].is_alphabetic() {
                i += 1;
            }
            let cmd: String = chars[start..i].iter().collect();
            if keywords.contains(&cmd.as_str()) {
                result.push_str("<span class=\"kw\">");
                result.push_str(&escape_html(&cmd));
                result.push_str("</span>");
            } else {
                result.push_str(&escape_html(&cmd));
            }
            continue;
        }

        // Typst hash-prefixed keywords (#let, #set, etc.)
        if chars[i] == '#'
            && language == "typst"
            && i + 1 < len
            && (chars[i + 1].is_alphabetic() || chars[i + 1] == '_')
        {
            let start = i;
            i += 1;
            while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let kw_part = &word[1..];
            if keywords.contains(&kw_part) {
                result.push_str("<span class=\"kw\">");
                result.push_str(&escape_html(&word));
                result.push_str("</span>");
            } else {
                result.push_str(&escape_html(&word));
            }
            continue;
        }

        // Numbers
        if chars[i].is_ascii_digit() {
            let start = i;
            while i < len && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == '_') {
                i += 1;
            }
            let num: String = chars[start..i].iter().collect();
            result.push_str("<span class=\"num\">");
            result.push_str(&escape_html(&num));
            result.push_str("</span>");
            continue;
        }

        // Words (potential keywords)
        if chars[i].is_alphabetic() || chars[i] == '_' {
            let start = i;
            while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            if keywords.contains(&word.as_str()) {
                result.push_str("<span class=\"kw\">");
                result.push_str(&escape_html(&word));
                result.push_str("</span>");
            } else {
                result.push_str(&escape_html(&word));
            }
            continue;
        }

        // Everything else: escape and output
        result.push_str(&escape_html(&chars[i].to_string()));
        i += 1;
    }

    result
}

/// Add line numbers to highlighted HTML.
///
/// Wraps each line in `<span class="ln">` for the number and tracks open
/// `<span>` tags across line boundaries (for multi-line block comments).
fn add_line_numbers(highlighted_html: &str) -> String {
    if highlighted_html.is_empty() {
        return String::new();
    }

    let mut result =
        String::with_capacity(highlighted_html.len() + highlighted_html.len() / 10 * 25);
    let mut open_tags: Vec<&str> = Vec::new();
    let mut line_num: usize = 1;

    result.push_str(&format!("<span class=\"ln\">{line_num:>4}</span>"));

    let bytes = highlighted_html.as_bytes();
    let mut pos = 0;

    while pos < highlighted_html.len() {
        // Check for <span open tags
        if bytes[pos] == b'<'
            && highlighted_html[pos..].starts_with("<span")
            && let Some(tag_end) = highlighted_html[pos..].find('>')
        {
            let tag = &highlighted_html[pos..=pos + tag_end];
            result.push_str(tag);
            open_tags.push(tag);
            pos += tag_end + 1;
            continue;
        }

        // Check for </span> close tags
        if bytes[pos] == b'<' && highlighted_html[pos..].starts_with("</span>") {
            if !open_tags.is_empty() {
                open_tags.pop();
            }
            result.push_str("</span>");
            pos += 7;
            continue;
        }

        // Newline: close open tags, emit line number, reopen tags
        if bytes[pos] == b'\n' {
            for _ in 0..open_tags.len() {
                result.push_str("</span>");
            }
            result.push('\n');
            line_num += 1;
            result.push_str(&format!("<span class=\"ln\">{line_num:>4}</span>"));
            for tag in &open_tags {
                result.push_str(tag);
            }
            pos += 1;
            continue;
        }

        result.push(bytes[pos] as char);
        pos += 1;
    }

    result
}

fn read_block_type(doc: &SIRDocument, payload_offset: u32) -> BlockType {
    let payload = doc.payload().get(payload_offset, 1);
    if let Some(&[bt_byte]) = payload {
        BlockType::from_u8(bt_byte).unwrap_or(BlockType::Document)
    } else {
        BlockType::Document
    }
}

fn read_heading_level(doc: &SIRDocument, payload_offset: u32) -> u32 {
    let payload = doc.payload().get(payload_offset, 5);
    if let Some(bytes) = payload
        && bytes.len() >= 5
    {
        let level = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
        return level.clamp(1, 6);
    }
    1
}

fn emit_block_open(html: &mut String, bt: BlockType, heading_level: u32, is_header_row: bool) {
    match bt {
        BlockType::Document => {}
        BlockType::Paragraph => {
            html.push_str("<p>");
        }
        BlockType::Heading => {
            html.push_str(&format!("<h{}>", heading_level));
        }
        BlockType::Code => {
            html.push_str(
                "<div class=\"code-container\">\
                 <button class=\"copy-btn\" onclick=\"copyCode(this)\">Copy</button>\
                 <pre class=\"code-block\"><code>",
            );
        }
        BlockType::List => {
            html.push_str("<ul>");
        }
        BlockType::BlockQuote => {
            html.push_str("<blockquote>");
        }
        BlockType::ThematicBreak => {
            html.push_str("<hr>");
        }
        BlockType::Image => {}
        BlockType::Table => {
            html.push_str("<table>");
        }
        BlockType::Math => {}
        BlockType::TableRow => {
            html.push_str("<tr>");
        }
        BlockType::TableCell => {
            if is_header_row {
                html.push_str("<th>");
            } else {
                html.push_str("<td>");
            }
        }
        BlockType::Footnote => {}
        BlockType::FootnoteBlock => {
            html.push_str("<div class=\"footnotes\">");
        }
        BlockType::Figure => {
            html.push_str("<figure>");
        }
    }
}

fn emit_block_close(html: &mut String, bt: BlockType, heading_level: u32, is_header_row: bool) {
    match bt {
        BlockType::Document => {}
        BlockType::Paragraph => {
            html.push_str("</p>");
        }
        BlockType::Heading => {
            html.push_str(&format!("</h{}>", heading_level));
        }
        BlockType::Code => {
            html.push_str("</code></pre></div>");
        }
        BlockType::List => {
            html.push_str("</ul>");
        }
        BlockType::BlockQuote => {
            html.push_str("</blockquote>");
        }
        BlockType::ThematicBreak => {}
        BlockType::Image => {}
        BlockType::Table => {
            html.push_str("</table>");
        }
        BlockType::Math => {}
        BlockType::TableRow => {
            html.push_str("</tr>");
        }
        BlockType::TableCell => {
            if is_header_row {
                html.push_str("</th>");
            } else {
                html.push_str("</td>");
            }
        }
        BlockType::Footnote => {}
        BlockType::FootnoteBlock => {
            html.push_str("</div>");
        }
        BlockType::Figure => {
            html.push_str("</figure>");
        }
    }
}

fn escape_html(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

fn escape_attr(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let html = render_markdown("");
        assert!(html.is_empty());
    }

    #[test]
    fn test_whitespace_only() {
        let html = render_markdown("   \n\n  ");
        assert!(html.is_empty());
    }

    #[test]
    fn test_heading() {
        let html = render_markdown("# Hello");
        assert!(html.contains("<h1>"), "missing <h1> in: {html}");
        assert!(html.contains("Hello"), "missing Hello in: {html}");
        assert!(html.contains("</h1>"), "missing </h1> in: {html}");
    }

    #[test]
    fn test_heading_levels() {
        for level in 1..=6 {
            let md = format!("{} Heading {}", "#".repeat(level), level);
            let html = render_markdown(&md);
            assert!(
                html.contains(&format!("<h{}>", level)),
                "missing <h{level}> in: {html}"
            );
            assert!(
                html.contains(&format!("</h{}>", level)),
                "missing </h{level}> in: {html}"
            );
        }
    }

    #[test]
    fn test_paragraph() {
        let html = render_markdown("Hello world");
        assert!(html.contains("<p>"), "missing <p> in: {html}");
        assert!(html.contains("</p>"), "missing </p> in: {html}");
        assert!(html.contains("Hello world"), "missing text in: {html}");
    }

    #[test]
    fn test_code_block() {
        let html = render_markdown("```rust\nfn main() {}\n```");
        assert!(html.contains("<pre"), "missing pre in: {html}");
        assert!(html.contains("<code>"), "missing code in: {html}");
        assert!(html.contains("</code>"), "missing /code in: {html}");
        assert!(html.contains("</pre>"), "missing /pre in: {html}");
        assert!(html.contains("fn"), "missing fn in: {html}");
        assert!(html.contains("main"), "missing main in: {html}");
    }

    #[test]
    fn test_code_block_wrapper() {
        let html = render_markdown("```rust\nfn main() {}\n```");
        assert!(
            html.contains("code-container"),
            "missing code-container in: {html}"
        );
        assert!(html.contains("copy-btn"), "missing copy-btn in: {html}");
        assert!(
            html.contains("code-block"),
            "missing code-block class in: {html}"
        );
    }

    #[test]
    fn test_unordered_list() {
        let html = render_markdown("- one\n- two");
        assert!(html.contains("<ul>"), "missing <ul> in: {html}");
        assert!(html.contains("</ul>"), "missing </ul> in: {html}");
    }

    #[test]
    fn test_blockquote() {
        let html = render_markdown("> quoted");
        assert!(
            html.contains("<blockquote>"),
            "missing <blockquote> in: {html}"
        );
        assert!(
            html.contains("</blockquote>"),
            "missing </blockquote> in: {html}"
        );
        assert!(html.contains("quoted"), "missing text in: {html}");
    }

    #[test]
    fn test_thematic_break() {
        let html = render_markdown("---");
        assert!(html.contains("<hr>"), "missing <hr> in: {html}");
    }

    #[test]
    fn test_bold_text() {
        let html = render_markdown("**bold**");
        assert!(
            html.contains("<strong>bold</strong>"),
            "missing <strong> tags in: {html}"
        );
    }

    #[test]
    fn test_italic_text() {
        let html = render_markdown("*italic*");
        assert!(
            html.contains("<em>italic</em>"),
            "missing <em> tags in: {html}"
        );
    }

    #[test]
    fn test_link() {
        let html = render_markdown("[click](https://example.com)");
        assert!(
            html.contains("href=\"https://example.com\""),
            "missing link href in: {html}"
        );
        assert!(html.contains("click"), "missing link text in: {html}");
    }

    #[test]
    fn test_html_escaping() {
        let html = render_markdown("foo & bar");
        assert!(html.contains("&amp;"), "missing &amp; in: {html}");
    }

    #[test]
    fn test_ldir_document_wrapper() {
        let html = render_markdown("hello");
        assert!(html.contains("ldir-document"), "missing wrapper in: {html}");
    }

    #[test]
    fn test_escape_html_special() {
        assert_eq!(escape_html("a&b<c>d"), "a&amp;b&lt;c&gt;d");
    }

    #[test]
    fn test_escape_attr_special() {
        let s = escape_attr("a&b'c\"d");
        assert!(s.contains("&amp;"));
        assert!(s.contains("&#39;"));
        assert!(s.contains("&quot;"));
    }

    #[test]
    fn test_multiple_paragraphs() {
        let html = render_markdown("First\n\nSecond");
        let count = html.matches("<p>").count();
        assert_eq!(count, 2, "expected 2 paragraphs in: {html}");
    }

    #[test]
    fn test_nested_heading_and_paragraph() {
        let html = render_markdown("# Title\n\nSome text");
        assert!(html.contains("<h1>"));
        assert!(html.contains("Title"));
        assert!(html.contains("<p>"));
        assert!(html.contains("Some text"));
    }

    #[test]
    fn test_inline_code() {
        let html = render_markdown("use `cargo build`");
        assert!(html.contains("<code>"), "missing <code> tag in: {html}");
        assert!(
            html.contains("cargo build"),
            "missing inline code text in: {html}"
        );
    }

    #[test]
    fn test_image() {
        let html = render_markdown("![alt text](image.png)");
        assert!(html.contains("<img"), "missing <img> tag in: {html}");
        assert!(html.contains("src=\"image.png\""), "missing src in: {html}");
    }

    #[test]
    fn highlight_rust_keywords() {
        let html = highlight_code("fn main() { let x = 1; }", "rust");
        assert!(
            html.contains("<span class=\"kw\">fn</span>"),
            "fn should be highlighted in: {html}"
        );
        assert!(
            html.contains("<span class=\"kw\">let</span>"),
            "let should be highlighted in: {html}"
        );
    }

    #[test]
    fn highlight_latex_commands() {
        let html = highlight_code(
            "\\section{Intro} \\begin{equation} \\frac{1}{2} \\end{equation}",
            "latex",
        );
        assert!(
            html.contains("<span class=\"kw\">\\section</span>"),
            "\\section should be highlighted in: {html}"
        );
        assert!(
            html.contains("<span class=\"kw\">\\begin</span>"),
            "\\begin should be highlighted in: {html}"
        );
        assert!(
            html.contains("<span class=\"kw\">\\end</span>"),
            "\\end should be highlighted in: {html}"
        );
        assert!(
            html.contains("<span class=\"kw\">\\frac</span>"),
            "\\frac should be highlighted in: {html}"
        );
    }

    #[test]
    fn highlight_unknown_language() {
        let code = "just plain text with <html> & stuff";
        let html = highlight_code(code, "unknown");
        assert_eq!(
            html,
            escape_html(code),
            "unknown language should passthrough escaped"
        );
        assert!(html.contains("&lt;html&gt;"));
        assert!(html.contains("&amp;"));
    }

    #[test]
    fn highlight_strings() {
        let html = highlight_code("let s = \"hello world\";", "rust");
        assert!(
            html.contains("<span class=\"str\">\"hello world\"</span>"),
            "string should be highlighted in: {html}"
        );
    }

    #[test]
    fn highlight_comments() {
        let html = highlight_code("// this is a comment\nlet x = 1;", "rust");
        assert!(
            html.contains("<span class=\"cmt\">// this is a comment</span>"),
            "line comment should be highlighted in: {html}"
        );
    }

    #[test]
    fn highlight_numbers() {
        let html = highlight_code("let x = 42;", "rust");
        assert!(
            html.contains("<span class=\"num\">42</span>"),
            "number should be highlighted in: {html}"
        );
    }

    #[test]
    fn line_numbers_added() {
        let html = render_markdown("```rust\nfn main() {}\nlet x = 1;\n```");
        assert!(
            html.contains("<span class=\"ln\">   1</span>"),
            "line 1 number missing in: {html}"
        );
        assert!(
            html.contains("<span class=\"ln\">   2</span>"),
            "line 2 number missing in: {html}"
        );
    }

    #[test]
    fn test_playground_html_exists() {
        let html = include_str!("../playground/index.html");
        assert!(
            html.contains("ldir Playground"),
            "playground HTML should contain title"
        );
        assert!(
            html.contains("renderFallback"),
            "playground HTML should have fallback rendering"
        );
    }

    #[test]
    fn detect_language_rust() {
        assert_eq!(detect_language("fn main() {}"), "rust");
        assert_eq!(detect_language("pub struct Foo;"), "rust");
        assert_eq!(detect_language("impl Foo {}"), "rust");
    }

    #[test]
    fn detect_language_latex() {
        assert_eq!(detect_language("\\begin{document}"), "latex");
        assert_eq!(detect_language("\\section{Intro}"), "latex");
    }

    #[test]
    fn detect_language_unknown() {
        assert_eq!(detect_language("just some text"), "text");
    }

    #[test]
    fn line_numbers_multiline_span() {
        let highlighted = "<span class=\"cmt\">/* line1\nline2 */</span>";
        let result = add_line_numbers(highlighted);
        assert!(
            result.contains("<span class=\"ln\">   1</span>"),
            "line 1 missing in: {result}"
        );
        assert!(
            result.contains("<span class=\"ln\">   2</span>"),
            "line 2 missing in: {result}"
        );
        assert!(
            result.contains("line1"),
            "content line1 missing in: {result}"
        );
        assert!(
            result.contains("line2"),
            "content line2 missing in: {result}"
        );
    }

    #[test]
    fn add_line_numbers_empty() {
        assert!(add_line_numbers("").is_empty());
    }

    #[test]
    fn highlight_typst_keywords() {
        let html = highlight_code("#let x = 5", "typst");
        assert!(
            html.contains("<span class=\"kw\">#let</span>"),
            "#let should be highlighted in: {html}"
        );
        assert!(
            html.contains("<span class=\"num\">5</span>"),
            "number should be highlighted in: {html}"
        );
    }
}
