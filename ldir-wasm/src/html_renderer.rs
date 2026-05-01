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

struct BlockSpan {
    block_type: BlockType,
    heading_level: u32,
    child_count: usize,
}

fn render_sir_to_html(doc: &SIRDocument) -> String {
    let mut html = String::from(r#"<div class="ldir-document">"#);

    let mut link_url_for_block: HashMap<u32, String> = HashMap::new();
    for instr in doc.iter() {
        if instr.opcode() == SIROpcode::LinkData {
            if let Some(url) = doc.payload_text(instr) {
                link_url_for_block.insert(instr.parent_id(), url.to_string());
            }
        }
    }

    let mut style_stack: Vec<StyleModifier> = Vec::new();
    let mut open_blocks: Vec<BlockSpan> = Vec::new();
    let mut entity_to_block: HashMap<u32, usize> = HashMap::new();

    for instr in doc.iter() {
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
                open_blocks.push(BlockSpan {
                    block_type: bt,
                    heading_level,
                    child_count: 0,
                });

                emit_block_open(&mut html, bt, heading_level, doc, instr.payload_offset());
            }
            SIROpcode::SetContent => {
                let parent_id = instr.parent_id();
                let parent_block_idx = entity_to_block.get(&parent_id);
                let bt = parent_block_idx
                    .map(|&idx| open_blocks[idx].block_type)
                    .unwrap_or(BlockType::Document);

                if bt == BlockType::Document {
                    continue;
                }

                if let Some(text) = doc.payload_text(instr) {
                    let escaped = escape_html(text);

                    if let Some(url) = link_url_for_block.get(&parent_id) {
                        html.push_str(&format!(
                            "<a href=\"{}\">{}</a>",
                            escape_attr(url),
                            escaped
                        ));
                    } else {
                        html.push_str(&escaped);
                    }
                }

                if let Some(&idx) = parent_block_idx {
                    open_blocks[idx].child_count += 1;
                }
            }
            SIROpcode::ApplyStyle => {
                let packed = instr.payload_offset();
                let (mods, is_push) = StyleModifier::from_packed(packed);
                if is_push {
                    style_stack.push(mods);
                } else {
                    style_stack.pop();
                }
            }
            SIROpcode::LinkData => {
                if let Some(url) = doc.payload_text(instr) {
                    let parent_id = instr.parent_id();
                    link_url_for_block.insert(parent_id, url.to_string());
                }
            }
            SIROpcode::InsertMath => {}
        }
    }

    while let Some(span) = open_blocks.pop() {
        emit_block_close(&mut html, span.block_type, span.heading_level);
    }

    html.push_str("</div>");
    html
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
    if let Some(bytes) = payload {
        if bytes.len() >= 5 {
            let level = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
            return level.clamp(1, 6);
        }
    }
    1
}

fn emit_block_open(html: &mut String, bt: BlockType, heading_level: u32, _doc: &SIRDocument, _payload_offset: u32) {
    match bt {
        BlockType::Document => {}
        BlockType::Paragraph => {
            html.push_str("<p>");
        }
        BlockType::Heading => {
            html.push_str(&format!("<h{}>", heading_level));
        }
        BlockType::Code => {
            html.push_str("<pre><code>");
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
    }
}

fn emit_block_close(html: &mut String, bt: BlockType, heading_level: u32) {
    match bt {
        BlockType::Document => {}
        BlockType::Paragraph => {
            html.push_str("</p>");
        }
        BlockType::Heading => {
            html.push_str(&format!("</h{}>", heading_level));
        }
        BlockType::Code => {
            html.push_str("</code></pre>");
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
            assert!(html.contains(&format!("<h{}>", level)), "missing <h{level}> in: {html}");
            assert!(html.contains(&format!("</h{}>", level)), "missing </h{level}> in: {html}");
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
        assert!(html.contains("<pre><code>"), "missing pre/code in: {html}");
        assert!(html.contains("</code></pre>"), "missing closing in: {html}");
        assert!(html.contains("fn main"), "missing code text in: {html}");
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
        assert!(html.contains("<blockquote>"), "missing <blockquote> in: {html}");
        assert!(html.contains("</blockquote>"), "missing </blockquote> in: {html}");
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
        assert!(html.contains("bold"), "missing bold text in: {html}");
    }

    #[test]
    fn test_italic_text() {
        let html = render_markdown("*italic*");
        assert!(html.contains("italic"), "missing italic text in: {html}");
    }

    #[test]
    fn test_link() {
        let html = render_markdown("[click](https://example.com)");
        assert!(html.contains("href=\"https://example.com\""), "missing link href in: {html}");
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
        assert!(html.contains("cargo build"), "missing inline code text in: {html}");
    }

    #[test]
    fn test_image() {
        let html = render_markdown("![alt text](image.png)");
        assert!(html.contains("image.png"), "missing image src in: {html}");
    }
}
