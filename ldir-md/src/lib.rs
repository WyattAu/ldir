//! ldir-md — Markdown to S-IR parser.
//!
//! Converts CommonMark Markdown into an S-IR document suitable for
//! compilation by the LDIR compiler pipeline.
//!
//! # Supported Markdown
//!
//! | Element          | S-IR Mapping                                        |
//! |------------------|----------------------------------------------------|
//! | Document         | `PushBlock(Document)` root                         |
//! | Paragraph        | `PushBlock(Paragraph)` + `SetContent`             |
//! | Heading h1-h6     | `PushBlock(Heading)` + level                     |
//! | Code block       | `PushBlock(Code)` + `SetContent`                  |
//! | List (ul/ol)     | `PushBlock(List)`                                |
//! | Blockquote       | `PushBlock(BlockQuote)` + `SetContent`            |
//! | Thematic break   | `PushBlock(ThematicBreak)`                        |
//! | Bold (**text**)  | `ApplyStyle(BOLD)` + `SetContent`                  |
//! | Italic (*text*)   | `ApplyStyle(ITALIC)` + `SetContent`                |
//! | Inline code      | `ApplyStyle(MONO)` + `SetContent`                  |
//! | Links [t](url)   | `LinkData` + `SetContent`                         |
//! | Text             | `SetContent` (UTF-8 payload)                       |

use ldir_ir::sir::{
    BlockType, ROOT_SENTINEL, SIRDocument, SIRInstruction, SIROpcode, StyleModifier,
};

use std::collections::HashMap;

/// Check if a pulldown-cmark Tag is block-level (vs inline).
fn is_block_tag(tag: &pulldown_cmark::Tag) -> bool {
    matches!(
        tag,
        pulldown_cmark::Tag::Paragraph
            | pulldown_cmark::Tag::Heading { .. }
            | pulldown_cmark::Tag::CodeBlock(_)
            | pulldown_cmark::Tag::List(_)
            | pulldown_cmark::Tag::Item
            | pulldown_cmark::Tag::BlockQuote(_)
            | pulldown_cmark::Tag::HtmlBlock
            | pulldown_cmark::Tag::Table(_)
    )
}

/// Parse a Markdown string into an S-IR document.
///
/// The resulting document has a single root `PushBlock(Document)` with
/// nested blocks for each Markdown element. Inline styles (bold, italic,
/// inline code) are emitted as `ApplyStyle` instructions.
pub fn parse_markdown(markdown: &str) -> SIRDocument {
    let mut doc = SIRDocument::new();
    let mut ctx = ParseContext::new(&mut doc);

    // Root block: PushBlock(Document)
    let root_id = ctx.push_root();

    // Parse markdown events into blocks
    let parser = pulldown_cmark::Parser::new_ext(
        markdown,
        pulldown_cmark::Options::ENABLE_TABLES | pulldown_cmark::Options::ENABLE_FOOTNOTES,
    );
    let mut blocks: Vec<Block> = Vec::new();
    let mut current: Option<InlineBuffer> = None;
    let mut current_image_url: Option<String> = None;
    let mut in_table: bool = false;
    let mut table_rows: Vec<Vec<String>> = Vec::new();
    let mut table_current_row: Vec<String> = Vec::new();
    let mut footnote_map: HashMap<String, u32> = HashMap::new();
    let mut footnote_counter: u32 = 0;
    let mut footnote_text_buf: Option<String> = None;
    let mut current_fn_label: Option<String> = None;
    let mut footnote_defs: Vec<(u32, String)> = Vec::new();

    for event in parser {
        match event {
            pulldown_cmark::Event::Start(tag) => {
                // Flush inline text for block-level tags
                if is_block_tag(&tag)
                    && let Some(buf) = current.take()
                {
                    blocks.push(buf.finish());
                }
                match tag {
                    pulldown_cmark::Tag::FootnoteDefinition(label) => {
                        footnote_text_buf = Some(String::new());
                        current_fn_label = Some(label.to_string());
                    }
                    pulldown_cmark::Tag::Image { dest_url, .. } => {
                        current_image_url = Some(dest_url.to_string());
                    }
                    pulldown_cmark::Tag::Table(_) => {
                        in_table = true;
                        table_rows.clear();
                        table_current_row.clear();
                    }
                    pulldown_cmark::Tag::TableHead => {
                        table_current_row.clear();
                    }
                    pulldown_cmark::Tag::TableRow => {
                        table_current_row.clear();
                    }
                    pulldown_cmark::Tag::TableCell => {
                        table_current_row.push(String::new());
                    }
                    pulldown_cmark::Tag::Heading { level, .. } => {
                        current = Some(InlineBuffer::new_heading(level as u32));
                    }
                    pulldown_cmark::Tag::Paragraph => {
                        current = Some(InlineBuffer::new_paragraph());
                    }
                    pulldown_cmark::Tag::CodeBlock(kind) => {
                        current = Some(InlineBuffer::new_code_block(&kind));
                    }
                    pulldown_cmark::Tag::List(..) => {
                        current = Some(InlineBuffer::new_list());
                    }
                    pulldown_cmark::Tag::Item => {
                        current = Some(InlineBuffer::new_list_item());
                    }
                    pulldown_cmark::Tag::BlockQuote(_) => {
                        current = Some(InlineBuffer::new_blockquote());
                    }
                    pulldown_cmark::Tag::Emphasis => {
                        if let Some(ref mut buf) = current {
                            buf.push_style_start(StyleModifier::ITALIC_STYLE);
                        }
                    }
                    pulldown_cmark::Tag::Strong => {
                        if let Some(ref mut buf) = current {
                            buf.push_style_start(StyleModifier::BOLD_STYLE);
                        }
                    }
                    pulldown_cmark::Tag::Link { dest_url, .. } => {
                        if let Some(ref mut buf) = current {
                            buf.push_link_start(&dest_url);
                        }
                    }
                    // Other inline tags: no special handling
                    _ => {}
                }
            }
            pulldown_cmark::Event::End(end_tag) => match end_tag {
                pulldown_cmark::TagEnd::Image => {
                    if let Some(url) = current_image_url.take() {
                        if let Some(buf) = current.take() {
                            blocks.push(buf.finish());
                        }
                        blocks.push(Block::Image { path: url });
                    }
                }
                pulldown_cmark::TagEnd::Table => {
                    in_table = false;
                    if !table_current_row.is_empty() {
                        table_rows.push(std::mem::take(&mut table_current_row));
                    }
                    let table_text = table_rows
                        .iter()
                        .map(|row| row.join(" | "))
                        .collect::<Vec<_>>()
                        .join("\n");
                    if !table_text.is_empty() {
                        blocks.push(Block::Table {
                            content: table_text,
                        });
                    }
                    table_rows.clear();
                }
                pulldown_cmark::TagEnd::TableHead | pulldown_cmark::TagEnd::TableRow => {
                    let _ = (!table_current_row.is_empty()).then(|| {
                        table_rows.push(std::mem::take(&mut table_current_row))
                    });
                }
                pulldown_cmark::TagEnd::TableCell => {
                    // Nothing to do at cell end
                }
                pulldown_cmark::TagEnd::FootnoteDefinition => {
                    if let (Some(text), Some(label)) = (footnote_text_buf.take(), current_fn_label.take()) {
                        let trimmed = text.trim().to_string();
                        if !trimmed.is_empty() {
                            let num = footnote_map
                                .entry(label.clone())
                                .or_insert_with(|| {
                                    footnote_counter += 1;
                                    footnote_counter
                                });
                            if *num > footnote_defs.len() as u32 {
                                footnote_defs.push((*num, trimmed));
                            } else {
                                for entry in &mut footnote_defs {
                                    if entry.0 == *num {
                                        entry.1 = trimmed;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                | pulldown_cmark::TagEnd::Paragraph
                | pulldown_cmark::TagEnd::CodeBlock
                | pulldown_cmark::TagEnd::List(_)
                | pulldown_cmark::TagEnd::Item
                | pulldown_cmark::TagEnd::BlockQuote(_) => {
                    if let Some(buf) = current.take() {
                        blocks.push(buf.finish());
                    }
                }
                pulldown_cmark::TagEnd::Emphasis => {
                    if let Some(ref mut buf) = current {
                        buf.push_style_end(StyleModifier::ITALIC_STYLE);
                    }
                }
                pulldown_cmark::TagEnd::Strong => {
                    if let Some(ref mut buf) = current {
                        buf.push_style_end(StyleModifier::BOLD_STYLE);
                    }
                }
                pulldown_cmark::TagEnd::Link => {
                    if let Some(ref mut buf) = current {
                        buf.push_link_end();
                    }
                }
                _ => {}
            },
            pulldown_cmark::Event::Text(text) => {
                if footnote_text_buf.is_some() {
                    if let Some(ref mut buf) = footnote_text_buf {
                        buf.push_str(&text);
                    }
                } else if in_table {
                    let last = table_current_row.len();
                    if last == 0 {
                        table_current_row.push(text.to_string());
                    } else {
                        table_current_row[last - 1].push_str(&text);
                    }
                } else if let Some(ref mut buf) = current {
                    buf.push_text(&text);
                }
            }
            pulldown_cmark::Event::Code(text) => {
                if let Some(ref mut buf) = current {
                    buf.push_inline_code(&text);
                }
            }
            pulldown_cmark::Event::SoftBreak => {
                if let Some(ref mut buf) = current {
                    buf.push_soft_break();
                }
            }
            pulldown_cmark::Event::HardBreak => {
                if let Some(ref mut buf) = current {
                    buf.push_hard_break();
                }
            }
            pulldown_cmark::Event::Rule => {
                if let Some(buf) = current.take() {
                    blocks.push(buf.finish());
                }
                blocks.push(Block::ThematicBreak);
            }
            pulldown_cmark::Event::FootnoteReference(label) => {
                let label_str = label.to_string();
                let num = *footnote_map.entry(label_str).or_insert_with(|| {
                    footnote_counter += 1;
                    footnote_counter
                });
                if let Some(ref mut buf) = current {
                    buf.push_text(&format!("\\fnmark{{{}}}", num));
                }
            }
            _ => {
                // Ignore: Html, TaskListMarker, FootnoteReference, etc.
            }
        }
    }

    // Flush remaining inline text
    if let Some(buf) = current.take() {
        blocks.push(buf.finish());
    }

    // Emit blocks as S-IR instructions
    ctx.emit_blocks(blocks, root_id);

    doc.footnotes = footnote_defs;

    doc
}

// --- Internal types ---

struct ParseContext<'a> {
    doc: &'a mut SIRDocument,
    next_id: u32,
}

impl<'a> ParseContext<'a> {
    fn new(doc: &'a mut SIRDocument) -> Self {
        Self { doc, next_id: 0 }
    }

    fn next_entity_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn push_root(&mut self) -> u32 {
        let id = self.next_entity_id();
        let payload_offset = self.doc.payload_mut().append(&[BlockType::Document as u8]);
        self.doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            id,
            ROOT_SENTINEL,
            payload_offset,
        ));
        id
    }

    fn emit_blocks(&mut self, blocks: Vec<Block>, root_id: u32) {
        for block in blocks {
            match block {
                Block::Heading {
                    level,
                    content,
                    inline_styles,
                    link_url,
                } => {
                    let bid =
                        self.emit_block(BlockType::Heading, root_id, Some(&[level]), &content);
                    self.emit_inline_styles(inline_styles, bid);
                    self.emit_link(link_url, bid);
                }
                Block::Paragraph {
                    content,
                    inline_styles,
                    link_url,
                } => {
                    let bid = self.emit_block(BlockType::Paragraph, root_id, None, &content);
                    self.emit_inline_styles(inline_styles, bid);
                    self.emit_link(link_url, bid);
                }
                Block::CodeBlock { content } => {
                    self.emit_block(BlockType::Code, root_id, None, &content);
                }
                Block::List { content } => {
                    self.emit_block(BlockType::List, root_id, None, &content);
                }
                Block::BlockQuote { content } => {
                    let bid = self.emit_block(BlockType::BlockQuote, root_id, None, &content);
                    // Indent blockquote content by 2em (48pt in 24pt/1em)
                    self.emit_indent(bid, 48 * 64);
                }
                Block::ThematicBreak => {
                    self.emit_block(BlockType::ThematicBreak, root_id, None, "");
                }
                Block::Image { path } => {
                    self.emit_block(BlockType::Image, root_id, None, &path);
                }
                Block::Table { content } => {
                    self.emit_block(BlockType::Table, root_id, None, &content);
                }
                Block::Text {
                    content,
                    inline_styles,
                    link_url,
                } => {
                    if !content.is_empty() {
                        let id = self.next_entity_id();
                        self.doc.push_with_payload(
                            SIRInstruction::new(SIROpcode::SetContent, id, root_id, 0),
                            content.as_bytes(),
                        );
                    }
                    self.emit_link(link_url, root_id);
                    // Inline styles for text-only blocks (list items) are not supported yet
                    let _ = inline_styles;
                }
            }
        }
    }

    fn emit_block(
        &mut self,
        block_type: BlockType,
        parent_id: u32,
        extra_payload: Option<&[u32]>,
        content: &str,
    ) -> u32 {
        let block_id = self.next_entity_id();

        // Build block payload: BlockType byte + optional extra data
        let mut payload = vec![block_type as u8];
        if let Some(extra) = extra_payload {
            for &val in extra {
                payload.extend_from_slice(&val.to_le_bytes());
            }
        }
        let payload_offset = self.doc.payload_mut().append(&payload);

        self.doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            block_id,
            parent_id,
            payload_offset,
        ));

        if !content.is_empty() {
            let content_id = self.next_entity_id();
            self.doc.push_with_payload(
                SIRInstruction::new(SIROpcode::SetContent, content_id, block_id, 0),
                content.as_bytes(),
            );
        }

        block_id
    }

    fn emit_inline_styles(&mut self, styles: Vec<InlineStyle>, parent_id: u32) {
        for style in styles {
            let id = self.next_entity_id();
            let packed = if style.is_enter {
                StyleModifier::push(style.modifier)
            } else {
                StyleModifier::pop()
            };
            self.doc.push(SIRInstruction::new(
                SIROpcode::ApplyStyle,
                id,
                parent_id,
                packed,
            ));
        }
    }

    fn emit_link(&mut self, link_url: Option<String>, parent_id: u32) {
        if let Some(url) = link_url
            && !url.is_empty()
        {
            let link_id = self.next_entity_id();
            let mut url_bytes = url.into_bytes();
            url_bytes.push(0);
            self.doc.push_with_payload(
                SIRInstruction::new(SIROpcode::LinkData, link_id, parent_id, 0),
                &url_bytes,
            );
        }
    }

    fn emit_indent(&mut self, parent_id: u32, indent_fp26_6: i32) {
        // Emit a DrawRule instruction to visually represent blockquote indent
        // (the compiler will handle this as a vertical rule)
        // For now, we use AttachMetadata to store the indent value
        let _ = (parent_id, indent_fp26_6);
        // TODO: proper blockquote rendering in compiler
    }
}

// --- Block representation ---

#[allow(clippy::enum_variant_names)]
enum Block {
    Heading {
        level: u32,
        content: String,
        inline_styles: Vec<InlineStyle>,
        link_url: Option<String>,
    },
    Paragraph {
        content: String,
        inline_styles: Vec<InlineStyle>,
        link_url: Option<String>,
    },
    CodeBlock {
        content: String,
    },
    List {
        content: String,
    },
    BlockQuote {
        content: String,
    },
    ThematicBreak,
    Text {
        content: String,
        inline_styles: Vec<InlineStyle>,
        link_url: Option<String>,
    },
    Image {
        path: String,
    },
    Table {
        content: String,
    },
}

/// An inline style boundary (enter or exit a styled span).
#[derive(Debug, Clone)]
struct InlineStyle {
    modifier: StyleModifier,
    is_enter: bool,
}

// --- Inline buffer ---

struct InlineBuffer {
    content: String,
    block_kind: BlockKind,
    /// Pending inline styles to emit after the block content.
    inline_styles: Vec<InlineStyle>,
    /// Active link URLs collected during this block.
    link_urls: Vec<String>,
}

#[derive(Clone, Copy)]
enum BlockKind {
    Heading { level: u32 },
    Paragraph,
    CodeBlock,
    List,
    ListItem,
    BlockQuote,
}

impl InlineBuffer {
    fn new_heading(level: u32) -> Self {
        Self {
            content: String::new(),
            block_kind: BlockKind::Heading { level },
            inline_styles: Vec::new(),
            link_urls: Vec::new(),
        }
    }

    fn new_paragraph() -> Self {
        Self {
            content: String::new(),
            block_kind: BlockKind::Paragraph,
            inline_styles: Vec::new(),
            link_urls: Vec::new(),
        }
    }

    fn new_code_block(_kind: &pulldown_cmark::CodeBlockKind) -> Self {
        Self {
            content: String::new(),
            block_kind: BlockKind::CodeBlock,
            inline_styles: Vec::new(),
            link_urls: Vec::new(),
        }
    }

    fn new_list() -> Self {
        Self {
            content: String::new(),
            block_kind: BlockKind::List,
            inline_styles: Vec::new(),
            link_urls: Vec::new(),
        }
    }

    fn new_list_item() -> Self {
        Self {
            content: String::new(),
            block_kind: BlockKind::ListItem,
            inline_styles: Vec::new(),
            link_urls: Vec::new(),
        }
    }

    fn new_blockquote() -> Self {
        Self {
            content: String::new(),
            block_kind: BlockKind::BlockQuote,
            inline_styles: Vec::new(),
            link_urls: Vec::new(),
        }
    }

    fn push_text(&mut self, text: &str) {
        self.content.push_str(text);
    }

    fn push_inline_code(&mut self, text: &str) {
        // Mark inline code boundaries with style events
        // We track position in the content and emit style instructions
        // that reference offsets within the text.
        // For now, emit mono style enter/exit around the code text.
        self.inline_styles.push(InlineStyle {
            modifier: StyleModifier::MONO_STYLE,
            is_enter: true,
        });
        self.content.push_str(text);
        self.inline_styles.push(InlineStyle {
            modifier: StyleModifier::MONO_STYLE,
            is_enter: false,
        });
    }

    fn push_soft_break(&mut self) {
        self.content.push(' ');
    }

    fn push_hard_break(&mut self) {
        self.content.push('\n');
    }

    fn push_style_start(&mut self, modifier: StyleModifier) {
        self.inline_styles.push(InlineStyle {
            modifier,
            is_enter: true,
        });
    }

    fn push_style_end(&mut self, modifier: StyleModifier) {
        self.inline_styles.push(InlineStyle {
            modifier,
            is_enter: false,
        });
    }

    fn push_link_start(&mut self, url: &str) {
        self.link_urls.push(url.to_string());
    }

    fn push_link_end(&mut self) {
        // Link URLs are collected; the last one is the active link for this segment.
    }

    fn finish(self) -> Block {
        let content = self.content.trim().to_string();
        // Use the last collected link URL for this block
        let link_url = self.link_urls.last().cloned().filter(|u| !u.is_empty());

        match self.block_kind {
            BlockKind::Heading { level } => Block::Heading {
                level,
                content,
                inline_styles: self.inline_styles,
                link_url,
            },
            BlockKind::Paragraph => Block::Paragraph {
                content,
                inline_styles: self.inline_styles,
                link_url,
            },
            BlockKind::CodeBlock => Block::CodeBlock { content },
            BlockKind::List => Block::List { content },
            BlockKind::ListItem => Block::Text {
                content,
                inline_styles: self.inline_styles,
                link_url,
            },
            BlockKind::BlockQuote => Block::BlockQuote { content },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let doc = parse_markdown("");
        assert!(doc.len() >= 1);
    }

    #[test]
    fn test_single_paragraph() {
        let doc = parse_markdown("Hello world");
        assert!(doc.len() >= 3);
    }

    #[test]
    fn test_heading() {
        let doc = parse_markdown("# Title");
        assert!(doc.len() >= 3);
    }

    #[test]
    fn test_heading_levels() {
        for level in 1..=6 {
            let md = format!("{} Heading {}", "#".repeat(level), level);
            let doc = parse_markdown(&md);
            assert!(
                doc.len() >= 3,
                "heading h{} should produce instructions",
                level
            );
        }
    }

    #[test]
    fn test_multiple_paragraphs() {
        let doc = parse_markdown("First\n\nSecond\n\nThird");
        assert!(doc.len() >= 7);
    }

    #[test]
    fn test_code_block() {
        let doc = parse_markdown("```rust\nfn main() {}\n```");
        assert!(doc.len() >= 3);
    }

    #[test]
    fn test_bold_italic_passthrough() {
        let doc = parse_markdown("This has **bold** and *italic* text.");
        let text = collect_all_text(&doc);
        assert!(
            text.contains("bold"),
            "text should contain 'bold', got: {}",
            text
        );
        assert!(
            text.contains("italic"),
            "text should contain 'italic', got: {}",
            text
        );
    }

    #[test]
    fn test_bold_emits_apply_style() {
        let doc = parse_markdown("**bold text**");
        let mut found_style = false;
        for instr in doc.iter() {
            if instr.opcode() == SIROpcode::ApplyStyle {
                found_style = true;
                let packed = instr.payload_offset();
                let (mods, is_push) = StyleModifier::from_packed(packed);
                assert!(is_push, "should be a push style");
                assert!(mods.contains(StyleModifier::BOLD));
                break;
            }
        }
        assert!(found_style, "should emit ApplyStyle for bold");
    }

    #[test]
    fn test_italic_emits_apply_style() {
        let doc = parse_markdown("*italic text*");
        let mut found_style = false;
        for instr in doc.iter() {
            if instr.opcode() == SIROpcode::ApplyStyle {
                found_style = true;
                let packed = instr.payload_offset();
                let (mods, is_push) = StyleModifier::from_packed(packed);
                assert!(is_push);
                assert!(mods.contains(StyleModifier::ITALIC));
                break;
            }
        }
        assert!(found_style, "should emit ApplyStyle for italic");
    }

    #[test]
    fn test_inline_code_emits_apply_style() {
        let doc = parse_markdown("use `cargo build`");
        let mut found_mono = false;
        for instr in doc.iter() {
            if instr.opcode() == SIROpcode::ApplyStyle {
                let packed = instr.payload_offset();
                let (mods, _is_push) = StyleModifier::from_packed(packed);
                if mods.contains(StyleModifier::MONO) {
                    found_mono = true;
                    break;
                }
            }
        }
        assert!(
            found_mono,
            "should emit ApplyStyle with MONO for inline code"
        );
    }

    #[test]
    fn test_link_emits_link_data() {
        let doc = parse_markdown("[click here](https://example.com)");
        let mut found_link = false;
        for instr in doc.iter() {
            if instr.opcode() == SIROpcode::LinkData {
                found_link = true;
                let url = doc.payload_text(instr);
                assert!(url.is_some(), "LinkData should have payload");
                assert!(url.unwrap().contains("example.com"));
                break;
            }
        }
        assert!(found_link, "should emit LinkData for links");
    }

    #[test]
    fn test_blockquote_emits_block() {
        let doc = parse_markdown("> quoted text");
        let mut found_bq = false;
        let mut found_text = false;
        for instr in doc.iter() {
            if instr.opcode() == SIROpcode::PushBlock {
                let payload = doc.payload().get(instr.payload_offset(), 1);
                if payload == Some(&[BlockType::BlockQuote as u8]) {
                    found_bq = true;
                }
            }
            if instr.opcode() == SIROpcode::SetContent {
                let text = doc.payload_text(instr);
                if let Some(t) = text {
                    if t.contains("quoted text") {
                        found_text = true;
                    }
                }
            }
        }
        assert!(
            found_bq,
            "should emit PushBlock(BlockQuote) for blockquotes"
        );
        assert!(found_text, "should emit SetContent for blockquote text");
    }

    #[test]
    fn test_thematic_break() {
        let doc = parse_markdown("---");
        let mut found_hr = false;
        for instr in doc.iter() {
            if instr.opcode() == SIROpcode::PushBlock {
                let payload = doc.payload().get(instr.payload_offset(), 1);
                if payload == Some(&[BlockType::ThematicBreak as u8]) {
                    found_hr = true;
                    break;
                }
            }
        }
        assert!(found_hr, "should emit PushBlock(ThematicBreak) for ---");
    }

    #[test]
    fn test_nested_heading_and_paragraph() {
        let doc = parse_markdown("# Title\n\nSome text");
        assert!(doc.len() >= 5);
    }

    #[test]
    fn test_realistic_document() {
        let markdown = r#"# Document Title

This is a paragraph with **bold** and *italic* text.

## Section

Another paragraph here.

```
code block
```

- list item 1
- list item 2

> A blockquote

---

[Link](https://example.com)
"#;
        let doc = parse_markdown(markdown);
        assert!(doc.len() >= 10);
    }

    #[test]
    fn test_image_emits_block() {
        let doc = parse_markdown("![alt text](image.png)");
        let mut found_image = false;
        for instr in doc.iter() {
            if instr.opcode() == SIROpcode::PushBlock {
                let payload = doc.payload().get(instr.payload_offset(), 1);
                if payload == Some(&[BlockType::Image as u8]) {
                    found_image = true;
                    break;
                }
            }
        }
        assert!(found_image, "should emit PushBlock(Image) for ![](path)");
    }

    #[test]
    fn test_gfm_table_emits_block() {
        let markdown = "| Header 1 | Header 2 |\n| --- | --- |\n| Cell 1 | Cell 2 |";
        let doc = parse_markdown(markdown);
        let mut found_table = false;
        for instr in doc.iter() {
            if instr.opcode() == SIROpcode::PushBlock {
                let payload = doc.payload().get(instr.payload_offset(), 1);
                if payload == Some(&[BlockType::Table as u8]) {
                    found_table = true;
                    break;
                }
            }
        }
        assert!(found_table, "should emit PushBlock(Table) for GFM tables");
    }

    #[test]
    fn test_footnote_reference_emits_fnmark() {
        let markdown = "Text with a footnote[^1].\n\n[^1]: The footnote text.";
        let doc = parse_markdown(markdown);
        let mut found_fnmark = false;
        for instr in doc.iter() {
            if instr.opcode() == SIROpcode::SetContent {
                if let Some(text) = doc.payload_text(instr) {
                    if text.contains("\\fnmark{") {
                        found_fnmark = true;
                    }
                }
            }
        }
        assert!(found_fnmark, "should emit \\fnmark in content for footnote reference");
    }

    #[test]
    fn test_footnote_stores_text() {
        let markdown = "Text[^1].\n\n[^1]: Footnote text here.";
        let doc = parse_markdown(markdown);
        assert_eq!(doc.footnotes.len(), 1, "should have one footnote entry");
        assert_eq!(doc.footnotes[0].1, "Footnote text here.");
    }

    #[test]
    fn test_multiple_footnotes_md() {
        let markdown = "Text[^a] and more[^b].\n\n[^a]: First.\n[^b]: Second.";
        let doc = parse_markdown(markdown);
        assert_eq!(doc.footnotes.len(), 2);
        let mut found_1 = false;
        let mut found_2 = false;
        for instr in doc.iter() {
            if instr.opcode() == SIROpcode::SetContent {
                if let Some(text) = doc.payload_text(instr) {
                    if text.contains("\\fnmark{1}") {
                        found_1 = true;
                    }
                    if text.contains("\\fnmark{2}") {
                        found_2 = true;
                    }
                }
            }
        }
        assert!(found_1, "should have fnmark 1");
        assert!(found_2, "should have fnmark 2");
    }

    #[test]
    fn test_payload_has_text() {
        let doc = parse_markdown("Hello");
        for instr in doc.iter() {
            if instr.opcode() == SIROpcode::SetContent {
                let text = doc.payload_text(instr);
                assert!(text.is_some(), "SetContent should have payload text");
                assert_eq!(text.unwrap(), "Hello");
                return;
            }
        }
        panic!("No SetContent instruction found");
    }

    #[test]
    fn test_unordered_list() {
        let doc = parse_markdown("- one\n- two\n- three");
        assert!(doc.len() >= 3);
    }

    #[test]
    fn test_ordered_list() {
        let doc = parse_markdown("1. first\n2. second");
        assert!(doc.len() >= 3);
    }

    fn collect_all_text(doc: &SIRDocument) -> String {
        let mut out = String::new();
        for instr in doc.iter() {
            if instr.opcode() == SIROpcode::SetContent {
                if let Some(text) = doc.payload_text(instr) {
                    out.push_str(text);
                    out.push(' ');
                }
            }
        }
        out
    }
}
