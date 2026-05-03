# ldir-md

Markdown to S-IR parser for the LDIR document pipeline. Converts
CommonMark Markdown with GFM extensions into an S-IR document tree
suitable for compilation by `ldir-core`.

## Features

- CommonMark parsing via `pulldown-cmark`
- GFM tables, footnotes, and task lists
- Inline styles: bold, italic, inline code, links
- Block elements: headings, paragraphs, code blocks, lists, blockquotes, images
- Thematic breaks and horizontal rules

## Supported Markdown

| Element | S-IR Mapping |
|---------|-------------|
| Headings h1-h6 | `PushBlock(Heading)` with level payload |
| Paragraphs | `PushBlock(Paragraph)` + `SetContent` |
| Bold | `ApplyStyle(BOLD)` |
| Italic | `ApplyStyle(ITALIC)` |
| Inline code | `ApplyStyle(MONO)` |
| Links | `LinkData` with URL payload |
| Code blocks | `PushBlock(Code)` |
| Blockquotes | `PushBlock(BlockQuote)` |
| Lists (ul/ol) | `PushBlock(List)` |
| GFM Tables | `PushBlock(Table)` + rows/cells |
| Task lists | `SetContent` with `[x]` / `[ ]` prefix |
| Footnotes | `\fnmark{N}` + `FootnoteBlock` |
| Images | `PushBlock(Image)` + path payload |

## API Overview

| Function | Description |
|----------|-------------|
| `parse_markdown` | Parse a Markdown string into `SIRDocument` |

## Usage

```rust
use ldir_md::parse_markdown;

let doc = parse_markdown("# Hello\n\nThis has **bold** text.");
println!("Parsed {} S-IR instructions", doc.len());
```

## Input / Output

- **Input**: CommonMark Markdown string (with GFM extensions)
- **Output**: `SIRDocument` (from `ldir-ir::sir`)

## License

MIT OR Apache-2.0

## Repository

[https://github.com/WyattAu/ldir](https://github.com/WyattAu/ldir)
