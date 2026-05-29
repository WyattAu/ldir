//! # ldir-md
//!
//! Markdown to S-IR parser for the LDIR document pipeline. Converts
//! CommonMark Markdown (with GFM extensions) into an S-IR document tree
//! suitable for compilation by `ldir-core`.

#![warn(clippy::unwrap_used, clippy::expect_used)]
#![deny(unsafe_code)]
//!
//! ## Key Types
//!
//! - [`parse_markdown`] — Main entry point: Markdown string to S-IR document
//!
//! ## Quick Start
//!
//! ```rust
//! use ldir_md::parse_markdown;
//!
//! let doc = parse_markdown("# Hello\n\nThis has **bold** text.");
//! println!("Parsed {} S-IR instructions", doc.len());
//! ```
//!
//! ## Supported Markdown
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
//! | Links \[t\](url)   | `LinkData` + `SetContent`                         |
//! | GFM Tables       | `PushBlock(Table)` + rows/cells                   |
//! | Task lists       | `SetContent` with `[x]` / `[ ]` prefix            |
//! | Footnotes        | `\fnmark{N}` + `FootnoteBlock`                   |
//! | Images           | `PushBlock(Image)` + path payload                 |
//!
//! ## References
//!
//! - [Repository](https://github.com/WyattAu/ldir)

use ldir_ir::sir::{
    BlockType, ROOT_SENTINEL, SIRDocument, SIRInstruction, SIROpcode, SourceSpan, StyleModifier,
};

use std::collections::HashMap;

struct LineIndex {
    line_offsets: Vec<usize>,
}

impl LineIndex {
    fn new(text: &str) -> Self {
        let mut offsets = vec![0usize];
        for (i, ch) in text.char_indices() {
            if ch == '\n' {
                offsets.push(i + 1);
            }
        }
        Self {
            line_offsets: offsets,
        }
    }

    fn lookup(&self, offset: usize) -> (u32, u32) {
        let line_idx = self.line_offsets.partition_point(|&o| o <= offset) - 1;
        let line = (line_idx + 1) as u32;
        let col = (offset - self.line_offsets[line_idx] + 1) as u32;
        (line, col)
    }
}

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
///
/// GFM extensions enabled: tables, footnotes, task lists.
pub fn parse_markdown(markdown: &str) -> SIRDocument {
    let mut doc = SIRDocument::new();
    let mut ctx = ParseContext::new(&mut doc);

    // Root block: PushBlock(Document)
    let root_id = ctx.push_root();

    // Parse markdown events into blocks
    let parser = pulldown_cmark::Parser::new_ext(
        markdown,
        pulldown_cmark::Options::ENABLE_TABLES
            | pulldown_cmark::Options::ENABLE_FOOTNOTES
            | pulldown_cmark::Options::ENABLE_TASKLISTS
            | pulldown_cmark::Options::ENABLE_STRIKETHROUGH,
    );
    let mut current_span: Option<SourceSpan> = None;
    let mut blocks: Vec<(Block, Option<SourceSpan>)> = Vec::new();
    let mut current: Option<InlineBuffer> = None;
    let mut current_image_url: Option<String> = None;
    let mut in_table: bool = false;
    let mut table_rows: Vec<TableRowData> = Vec::new();
    let mut table_current_row: Option<TableRowData> = None;
    let mut table_current_cell: Option<TableCellData> = None;
    let mut table_col_alignments: Vec<pulldown_cmark::Alignment> = Vec::new();
    let mut footnote_map: HashMap<String, u32> = HashMap::new();
    let mut footnote_counter: u32 = 0;
    let mut footnote_text_buf: Option<String> = None;
    let mut current_fn_label: Option<String> = None;
    let mut footnote_defs: Vec<(u32, String)> = Vec::new();
    let mut in_list_item: bool = false;
    let mut task_list_marker: Option<bool> = None;
    let line_index = LineIndex::new(markdown);

    for (event, range) in parser.into_offset_iter() {
        let (line, col) = line_index.lookup(range.start);
        current_span = Some(SourceSpan::new(
            line,
            col,
            range.start as u32,
            (range.end - range.start) as u32,
        ));
        match event {
            pulldown_cmark::Event::Start(tag) => {
                if is_block_tag(&tag)
                    && let Some(buf) = current.take()
                {
                    blocks.push((buf.finish(), current_span));
                }
                match tag {
                    pulldown_cmark::Tag::FootnoteDefinition(label) => {
                        footnote_text_buf = Some(String::new());
                        current_fn_label = Some(label.to_string());
                    }
                    pulldown_cmark::Tag::Image { dest_url, .. } => {
                        current_image_url = Some(dest_url.to_string());
                    }
                    pulldown_cmark::Tag::Table(alignment) => {
                        in_table = true;
                        table_col_alignments = alignment;
                        table_rows.clear();
                        table_current_row = None;
                        table_current_cell = None;
                    }
                    pulldown_cmark::Tag::TableHead => {
                        table_current_row = Some(TableRowData {
                            is_header: true,
                            cells: Vec::new(),
                        });
                    }
                    pulldown_cmark::Tag::TableRow => {
                        table_current_row = Some(TableRowData {
                            is_header: false,
                            cells: Vec::new(),
                        });
                    }
                    pulldown_cmark::Tag::TableCell => {
                        table_current_cell = Some(TableCellData {
                            content: String::new(),
                        });
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
                        in_list_item = true;
                        task_list_marker = None;
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
                    pulldown_cmark::Tag::Strikethrough => {
                        if let Some(ref mut buf) = current {
                            buf.push_style_start(StyleModifier::STRIKE_STYLE);
                        }
                    }
                    pulldown_cmark::Tag::Link { dest_url, .. } => {
                        if let Some(ref mut buf) = current {
                            buf.push_link_start(&dest_url);
                        }
                    }
                    _ => {}
                }
            }
            pulldown_cmark::Event::End(end_tag) => match end_tag {
                pulldown_cmark::TagEnd::Image => {
                    if let Some(url) = current_image_url.take() {
                        if let Some(buf) = current.take() {
                            blocks.push((buf.finish(), current_span));
                        }
                        blocks.push((Block::Image { path: url }, current_span));
                    }
                }
                pulldown_cmark::TagEnd::Table => {
                    in_table = false;
                    if let Some(row) = table_current_row.take() {
                        table_rows.push(row);
                    }
                    let num_cols = table_col_alignments.len();
                    let mut rows: Vec<StructuredTableRow> = Vec::new();
                    for tr in &table_rows {
                        let mut cells: Vec<String> = Vec::new();
                        for cell in &tr.cells {
                            cells.push(cell.content.trim().to_string());
                        }
                        rows.push(StructuredTableRow {
                            is_header: tr.is_header,
                            cells,
                        });
                    }
                    if !rows.is_empty() {
                        blocks.push((
                            Block::StructuredTable {
                                num_cols,
                                rows,
                                alignments: std::mem::take(&mut table_col_alignments),
                            },
                            current_span,
                        ));
                    }
                    table_rows.clear();
                    table_col_alignments.clear();
                }
                pulldown_cmark::TagEnd::TableHead | pulldown_cmark::TagEnd::TableRow => {
                    if let Some(cell) = table_current_cell.take()
                        && let Some(ref mut row) = table_current_row
                    {
                        row.cells.push(cell);
                    }
                    if let Some(row) = table_current_row.take() {
                        table_rows.push(row);
                    }
                }
                pulldown_cmark::TagEnd::TableCell => {
                    if let Some(cell) = table_current_cell.take()
                        && let Some(ref mut row) = table_current_row
                    {
                        row.cells.push(cell);
                    }
                }
                pulldown_cmark::TagEnd::FootnoteDefinition => {
                    if let (Some(text), Some(label)) =
                        (footnote_text_buf.take(), current_fn_label.take())
                    {
                        let trimmed = text.trim().to_string();
                        if !trimmed.is_empty() {
                            let num = footnote_map.entry(label.clone()).or_insert_with(|| {
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
                pulldown_cmark::TagEnd::Paragraph
                | pulldown_cmark::TagEnd::CodeBlock
                | pulldown_cmark::TagEnd::List(_) => {
                    if let Some(buf) = current.take() {
                        blocks.push((buf.finish(), current_span));
                    }
                }
                pulldown_cmark::TagEnd::Item => {
                    in_list_item = false;
                    if let Some(buf) = current.take() {
                        blocks.push((
                            buf.finish_with_task_marker(task_list_marker.take()),
                            current_span,
                        ));
                    }
                }
                pulldown_cmark::TagEnd::BlockQuote(_) => {
                    if let Some(buf) = current.take() {
                        blocks.push((buf.finish(), current_span));
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
                pulldown_cmark::TagEnd::Strikethrough => {
                    if let Some(ref mut buf) = current {
                        buf.push_style_end(StyleModifier::STRIKE_STYLE);
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
                    if let Some(ref mut cell) = table_current_cell {
                        cell.content.push_str(&text);
                    }
                } else if let Some(ref mut buf) = current {
                    buf.push_text(&text);
                }
            }
            pulldown_cmark::Event::Code(text) => {
                if in_table {
                    if let Some(ref mut cell) = table_current_cell {
                        cell.content.push_str(&text);
                    }
                } else if let Some(ref mut buf) = current {
                    buf.push_inline_code(&text);
                }
            }
            pulldown_cmark::Event::SoftBreak => {
                if in_table {
                    if let Some(ref mut cell) = table_current_cell {
                        cell.content.push(' ');
                    }
                } else if let Some(ref mut buf) = current {
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
                    blocks.push((buf.finish(), current_span));
                }
                blocks.push((Block::ThematicBreak, current_span));
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
            pulldown_cmark::Event::TaskListMarker(checked) if in_list_item => {
                task_list_marker = Some(checked);
            }
            _ => {}
        }
    }

    // Flush remaining inline text
    if let Some(buf) = current.take() {
        blocks.push((buf.finish(), current_span));
    }

    // Emit footnote definitions as a FootnoteBlock at the end
    if !footnote_defs.is_empty() {
        footnote_defs.sort_by_key(|(num, _)| *num);
        blocks.push((
            Block::FootnoteBlock {
                entries: footnote_defs,
            },
            current_span,
        ));
    }

    // Emit blocks as S-IR instructions
    ctx.emit_blocks(blocks, root_id);

    doc
}

// --- Internal types ---

struct ParseContext<'a> {
    doc: &'a mut SIRDocument,
    next_id: u32,
    current_span: Option<SourceSpan>,
}

impl<'a> ParseContext<'a> {
    fn new(doc: &'a mut SIRDocument) -> Self {
        Self {
            doc,
            next_id: 0,
            current_span: None,
        }
    }

    fn next_entity_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn push_root(&mut self) -> u32 {
        let id = self.next_entity_id();
        let payload_offset = self.doc.payload_mut().append(&[BlockType::Document as u8]);
        self.push_instr(SIRInstruction::new(
            SIROpcode::PushBlock,
            id,
            ROOT_SENTINEL,
            payload_offset,
        ));
        id
    }

    fn push_instr(&mut self, instr: SIRInstruction) {
        self.doc.push(instr);
        self.doc.source_spans.push(self.current_span);
    }

    fn push_instr_with_payload(&mut self, instr: SIRInstruction, payload: &[u8]) {
        self.doc.push_with_payload(instr, payload);
        self.doc.source_spans.push(self.current_span);
    }

    fn emit_blocks(&mut self, blocks: Vec<(Block, Option<SourceSpan>)>, root_id: u32) {
        for (block, span) in blocks {
            self.current_span = span;
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
                Block::CodeBlock { content, language } => {
                    let extra = language.as_deref().map(|lang| {
                        let mut payload = vec![BlockType::Code as u8];
                        let lang_bytes = lang.as_bytes();
                        payload.extend_from_slice(&(lang_bytes.len() as u32).to_le_bytes());
                        payload.extend_from_slice(lang_bytes);
                        payload
                    });
                    let block_id = self.next_entity_id();
                    let payload_offset = if let Some(ref p) = extra {
                        self.doc.payload_mut().append(p)
                    } else {
                        self.doc.payload_mut().append(&[BlockType::Code as u8])
                    };
                    self.push_instr(SIRInstruction::new(
                        SIROpcode::PushBlock,
                        block_id,
                        root_id,
                        payload_offset,
                    ));
                    if !content.is_empty() {
                        let content_id = self.next_entity_id();
                        self.push_instr_with_payload(
                            SIRInstruction::new(SIROpcode::SetContent, content_id, block_id, 0),
                            content.as_bytes(),
                        );
                    }
                }
                Block::List { content } => {
                    self.emit_block(BlockType::List, root_id, None, &content);
                }
                Block::BlockQuote { content } => {
                    let bid = self.emit_block(BlockType::BlockQuote, root_id, None, &content);
                    self.emit_indent(bid, 48 * 64);
                }
                Block::ThematicBreak => {
                    self.emit_block(BlockType::ThematicBreak, root_id, None, "");
                }
                Block::Image { path } => {
                    self.emit_block(BlockType::Image, root_id, None, &path);
                }
                Block::Text {
                    content,
                    inline_styles,
                    link_url,
                    task_marker,
                } => {
                    let display_content = match task_marker {
                        Some(true) => format!("[x] {}", content),
                        Some(false) => format!("[ ] {}", content),
                        None => content,
                    };
                    if !display_content.is_empty() {
                        let id = self.next_entity_id();
                        self.push_instr_with_payload(
                            SIRInstruction::new(SIROpcode::SetContent, id, root_id, 0),
                            display_content.as_bytes(),
                        );
                    }
                    self.emit_link(link_url, root_id);
                    self.emit_inline_styles(inline_styles, root_id);
                }
                Block::StructuredTable {
                    num_cols,
                    rows,
                    alignments,
                } => {
                    self.emit_structured_table(root_id, num_cols, &rows, &alignments);
                }
                Block::FootnoteBlock { entries } => {
                    self.emit_footnote_block(root_id, &entries);
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

        self.push_instr(SIRInstruction::new(
            SIROpcode::PushBlock,
            block_id,
            parent_id,
            payload_offset,
        ));

        if !content.is_empty() {
            let content_id = self.next_entity_id();
            self.push_instr_with_payload(
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
            self.push_instr(SIRInstruction::new(
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
            self.push_instr_with_payload(
                SIRInstruction::new(SIROpcode::LinkData, link_id, parent_id, 0),
                &url_bytes,
            );
        }
    }

    fn emit_indent(&mut self, parent_id: u32, indent_fp26_6: i32) {
        let _ = (parent_id, indent_fp26_6);
    }

    fn emit_structured_table(
        &mut self,
        root_id: u32,
        num_cols: usize,
        rows: &[StructuredTableRow],
        alignments: &[pulldown_cmark::Alignment],
    ) {
        let table_id = self.next_entity_id();
        let mut payload = vec![BlockType::Table as u8];
        payload.extend_from_slice(&(num_cols as u32).to_le_bytes());
        for align in alignments {
            let align_byte = match align {
                pulldown_cmark::Alignment::None => 0u8,
                pulldown_cmark::Alignment::Left => 1,
                pulldown_cmark::Alignment::Center => 2,
                pulldown_cmark::Alignment::Right => 3,
            };
            payload.push(align_byte);
        }
        let payload_offset = self.doc.payload_mut().append(&payload);
        self.push_instr(SIRInstruction::new(
            SIROpcode::PushBlock,
            table_id,
            root_id,
            payload_offset,
        ));

        for row in rows {
            let row_id = self.next_entity_id();
            let row_payload = vec![BlockType::TableRow as u8, if row.is_header { 1 } else { 0 }];
            let row_offset = self.doc.payload_mut().append(&row_payload);
            self.push_instr(SIRInstruction::new(
                SIROpcode::PushBlock,
                row_id,
                table_id,
                row_offset,
            ));

            for cell_content in &row.cells {
                let cell_id = self.next_entity_id();
                let cell_payload = self.doc.payload_mut().append(&[BlockType::TableCell as u8]);
                self.push_instr(SIRInstruction::new(
                    SIROpcode::PushBlock,
                    cell_id,
                    row_id,
                    cell_payload,
                ));

                if !cell_content.is_empty() {
                    let text_id = self.next_entity_id();
                    self.push_instr_with_payload(
                        SIRInstruction::new(SIROpcode::SetContent, text_id, cell_id, 0),
                        cell_content.as_bytes(),
                    );
                }
            }
        }
    }

    fn emit_footnote_block(&mut self, root_id: u32, entries: &[(u32, String)]) {
        let block_id = self.next_entity_id();
        let payload_offset = self
            .doc
            .payload_mut()
            .append(&[BlockType::FootnoteBlock as u8]);
        self.push_instr(SIRInstruction::new(
            SIROpcode::PushBlock,
            block_id,
            root_id,
            payload_offset,
        ));

        for (_num, text) in entries {
            let fn_id = self.next_entity_id();
            let payload_offset = self.doc.payload_mut().append(&[BlockType::Footnote as u8]);
            self.push_instr(SIRInstruction::new(
                SIROpcode::PushBlock,
                fn_id,
                block_id,
                payload_offset,
            ));

            if !text.is_empty() {
                let content_id = self.next_entity_id();
                self.push_instr_with_payload(
                    SIRInstruction::new(SIROpcode::SetContent, content_id, fn_id, 0),
                    text.as_bytes(),
                );
            }
        }
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
        language: Option<String>,
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
        task_marker: Option<bool>,
    },
    Image {
        path: String,
    },
    StructuredTable {
        num_cols: usize,
        rows: Vec<StructuredTableRow>,
        alignments: Vec<pulldown_cmark::Alignment>,
    },
    FootnoteBlock {
        entries: Vec<(u32, String)>,
    },
}

struct StructuredTableRow {
    is_header: bool,
    cells: Vec<String>,
}

struct TableRowData {
    is_header: bool,
    cells: Vec<TableCellData>,
}

struct TableCellData {
    content: String,
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
    inline_styles: Vec<InlineStyle>,
    link_urls: Vec<String>,
}

#[derive(Clone)]
enum BlockKind {
    Heading { level: u32 },
    Paragraph,
    CodeBlock { language: Option<String> },
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

    fn new_code_block(kind: &pulldown_cmark::CodeBlockKind) -> Self {
        let language = match kind {
            pulldown_cmark::CodeBlockKind::Fenced(lang) => {
                let s = lang.to_string();
                if s.is_empty() { None } else { Some(s) }
            }
            pulldown_cmark::CodeBlockKind::Indented => None,
        };
        Self {
            content: String::new(),
            block_kind: BlockKind::CodeBlock { language },
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

    fn push_link_end(&mut self) {}

    fn finish(self) -> Block {
        self.finish_impl(None)
    }

    fn finish_with_task_marker(self, marker: Option<bool>) -> Block {
        self.finish_impl(marker)
    }

    fn finish_impl(self, task_marker: Option<bool>) -> Block {
        let content = self.content.trim().to_string();
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
            BlockKind::CodeBlock { language } => Block::CodeBlock { content, language },
            BlockKind::List => Block::List { content },
            BlockKind::ListItem => Block::Text {
                content,
                inline_styles: self.inline_styles,
                link_url,
                task_marker,
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
        let mut found_row = false;
        let mut found_cell = false;
        for instr in doc.iter() {
            if instr.opcode() == SIROpcode::PushBlock {
                let payload = doc.payload().get(instr.payload_offset(), 1);
                if payload == Some(&[BlockType::Table as u8]) {
                    found_table = true;
                }
                if payload == Some(&[BlockType::TableRow as u8]) {
                    found_row = true;
                }
                if payload == Some(&[BlockType::TableCell as u8]) {
                    found_cell = true;
                }
            }
        }
        assert!(found_table, "should emit PushBlock(Table) for GFM tables");
        assert!(found_row, "should emit PushBlock(TableRow) for table rows");
        assert!(
            found_cell,
            "should emit PushBlock(TableCell) for table cells"
        );
    }

    #[test]
    fn test_gfm_table_has_header_row() {
        let markdown = "| H1 | H2 |\n| --- | --- |\n| C1 | C2 |";
        let doc = parse_markdown(markdown);
        let mut found_header = false;
        for instr in doc.iter() {
            if instr.opcode() == SIROpcode::PushBlock {
                let payload = doc.payload().get(instr.payload_offset(), 2);
                if payload == Some(&[BlockType::TableRow as u8, 1]) {
                    found_header = true;
                }
            }
        }
        assert!(found_header, "first row should be header (is_header=1)");
    }

    #[test]
    fn test_gfm_table_cell_content() {
        let markdown = "| Header |\n| --- |\n| Cell content |";
        let doc = parse_markdown(markdown);
        let mut found_cell_text = false;
        for instr in doc.iter() {
            if instr.opcode() == SIROpcode::SetContent {
                if let Some(text) = doc.payload_text(instr) {
                    if text == "Cell content" {
                        found_cell_text = true;
                    }
                }
            }
        }
        assert!(found_cell_text, "should have SetContent with cell text");
    }

    #[test]
    fn test_task_list_checked() {
        let markdown = "- [x] Done item";
        let doc = parse_markdown(markdown);
        let mut found = false;
        for instr in doc.iter() {
            if instr.opcode() == SIROpcode::SetContent {
                if let Some(text) = doc.payload_text(instr) {
                    if text.starts_with("[x]") && text.contains("Done item") {
                        found = true;
                    }
                }
            }
        }
        assert!(found, "should have [x] prefix for checked task list item");
    }

    #[test]
    fn test_task_list_unchecked() {
        let markdown = "- [ ] Todo item";
        let doc = parse_markdown(markdown);
        let mut found = false;
        for instr in doc.iter() {
            if instr.opcode() == SIROpcode::SetContent {
                if let Some(text) = doc.payload_text(instr) {
                    if text.starts_with("[ ]") && text.contains("Todo item") {
                        found = true;
                    }
                }
            }
        }
        assert!(found, "should have [ ] prefix for unchecked task list item");
    }

    #[test]
    fn test_mixed_task_list() {
        let markdown = "- [x] Done\n- [ ] Todo\n- Regular item";
        let doc = parse_markdown(markdown);
        let mut checked_found = false;
        let mut unchecked_found = false;
        let mut regular_found = false;
        for instr in doc.iter() {
            if instr.opcode() == SIROpcode::SetContent {
                if let Some(text) = doc.payload_text(instr) {
                    if text.starts_with("[x]") {
                        checked_found = true;
                    }
                    if text.starts_with("[ ]") {
                        unchecked_found = true;
                    }
                    if text == "Regular item" {
                        regular_found = true;
                    }
                }
            }
        }
        assert!(checked_found, "should have checked item");
        assert!(unchecked_found, "should have unchecked item");
        assert!(
            regular_found,
            "should have regular list item without marker"
        );
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
        assert!(
            found_fnmark,
            "should emit \\fnmark in content for footnote reference"
        );
    }

    #[test]
    fn test_footnote_stores_text() {
        let markdown = "Text[^1].\n\n[^1]: Footnote text here.";
        let doc = parse_markdown(markdown);
        let mut found = false;
        for instr in doc.iter() {
            if instr.opcode() == SIROpcode::SetContent {
                if let Some(text) = doc.payload_text(instr) {
                    if text == "Footnote text here." {
                        found = true;
                        break;
                    }
                }
            }
        }
        assert!(found, "should store footnote text in SetContent");
    }

    #[test]
    fn test_multiple_footnotes_md() {
        let markdown = "Text[^a] and more[^b].\n\n[^a]: First.\n[^b]: Second.";
        let doc = parse_markdown(markdown);
        let mut found_fnmark1 = false;
        let mut found_fnmark2 = false;
        let mut found_fn_block = false;
        for instr in doc.iter() {
            if instr.opcode() == SIROpcode::SetContent {
                if let Some(text) = doc.payload_text(instr) {
                    if text.contains("\\fnmark{1}") {
                        found_fnmark1 = true;
                    }
                    if text.contains("\\fnmark{2}") {
                        found_fnmark2 = true;
                    }
                    if text == "First." || text == "Second." {
                        found_fn_block = true;
                    }
                }
            }
            if instr.opcode() == SIROpcode::PushBlock {
                let payload = doc.payload().get(instr.payload_offset(), 1);
                if payload == Some(&[BlockType::FootnoteBlock as u8]) {
                    found_fn_block = true;
                }
                if payload == Some(&[BlockType::Footnote as u8]) {
                    found_fn_block = true;
                }
            }
        }
        assert!(found_fnmark1, "should have fnmark 1");
        assert!(found_fnmark2, "should have fnmark 2");
        assert!(found_fn_block, "should have FootnoteBlock");
    }

    #[test]
    fn test_footnote_block_structure() {
        let markdown = "Text[^1].\n\n[^1]: Footnote text.";
        let doc = parse_markdown(markdown);
        let mut found_block = false;
        let mut found_fn = false;
        let mut found_content = false;
        for instr in doc.iter() {
            if instr.opcode() == SIROpcode::PushBlock {
                let payload = doc.payload().get(instr.payload_offset(), 1);
                if payload == Some(&[BlockType::FootnoteBlock as u8]) {
                    found_block = true;
                }
                if payload == Some(&[BlockType::Footnote as u8]) {
                    found_fn = true;
                }
            }
            if instr.opcode() == SIROpcode::SetContent {
                if let Some(text) = doc.payload_text(instr) {
                    if text == "Footnote text." {
                        found_content = true;
                    }
                }
            }
        }
        assert!(found_block, "should emit FootnoteBlock");
        assert!(found_fn, "should emit Footnote inside block");
        assert!(found_content, "should have footnote content");
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
