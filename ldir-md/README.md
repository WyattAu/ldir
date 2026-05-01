# ldir-md

Markdown to S-IR parser for the LDIR document pipeline.

Converts CommonMark Markdown into an S-IR document tree suitable for compilation by `ldir-core`.

## Supported Markdown

| Element | S-IR Mapping |
|---|---|
| Headings h1-h6 | `PushBlock(Heading)` with level payload |
| Paragraphs | `PushBlock(Paragraph)` + `SetContent` |
| Bold | `ApplyStyle(BOLD)` |
| Italic | `ApplyStyle(ITALIC)` |
| Inline code | `ApplyStyle(MONO)` |
| Links | `LinkData` with URL payload |
| Code blocks | `PushBlock(Code)` |
| Blockquotes | `PushBlock(BlockQuote)` |
| Lists (ul/ol) | `PushBlock(List)` |
| Thematic breaks | `PushBlock(ThematicBreak)` |

## Example

```rust
use ldir_md::parse_markdown;

let markdown = r#"# Hello

This has **bold** and *italic* text.

> A blockquote
"#;

let doc = parse_markdown(markdown);
println!("Parsed {} S-IR instructions", doc.len());

// Validate and compile
ldir_core::validator::validate_sir(&doc).expect("well-formed");
let gir = ldir_core::compiler::compile_sir(&doc).unwrap();
```

## License

MIT OR Apache-2.0
