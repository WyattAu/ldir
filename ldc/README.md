# ldc

CLI tool for compiling Markdown documents to PDF via the LDIR pipeline.

Parses Markdown to S-IR, validates, compiles to G-IR, and emits PDF with embedded TrueType fonts.

## Usage

```sh
ldc input.md -o output.pdf
ldc document.md                    # outputs to document.pdf
ldc --font /path/to/font.ttf doc   # use a specific font
ldc --format sir doc.md            # output S-IR binary instead of PDF
```

## Options

| Option | Default | Description |
|---|---|---|
| `-o, --output` | `<input-stem>.<format>` | Output file path |
| `-f, --format` | `pdf` | Output format: `pdf`, `gir`, or `sir` |
| `--font` | auto-detect | Path to a .ttf/.otf font file |

## Font Discovery

When no `--font` is specified, `ldc` searches common system paths for DejaVu Sans, Liberation Sans, or Noto Sans. If a primary font is found, it automatically locates Bold, Italic, BoldItalic, and Mono variants from the same font family.

## Pipeline

```
Markdown  --[ldir-md]-->  S-IR  --[ldir-core]-->  G-IR  --[ldir-pdf]-->  PDF
```

## License

MIT OR Apache-2.0
