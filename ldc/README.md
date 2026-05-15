# ldc

CLI tool for compiling documents to PDF and other formats via the LDIR pipeline.

Supports 9 input formats: Markdown, LaTeX, Typst, HTML, Asciidoc, Org-mode, DOCX, SIR2, and LDIR text.
Parses input to S-IR, validates, compiles to G-IR, and emits the selected output format.

## Usage

```sh
ldc input.md -o output.pdf
ldc document.tex                    # outputs to document.pdf
ldc --font /path/to/font.ttf doc   # use a specific font
ldc --format html doc.md            # output HTML instead of PDF
ldc --format sir2 doc.md            # output S-IR v2 binary
```

## Options

| Option | Default | Description |
|---|---|---|
| `-o, --output` | `<input-stem>.<format>` | Output file path |
| `-f, --format` | `pdf` | Output format: `pdf`, `html`, `epub`, `txt`, `docx`, `gir`, `sir2`, `ldir` |
| `--font` | auto-detect | Path to a .ttf/.otf font file |

## Font Discovery

When no `--font` is specified, `ldc` searches common system paths for DejaVu Sans, Liberation Sans, or Noto Sans. If a primary font is found, it automatically locates Bold, Italic, BoldItalic, and Mono variants from the same font family.

## Pipeline

```
Input (md/tex/typ/html/adoc/org/docx)  --[frontend]-->  S-IR  --[ldir-core]-->  G-IR  --[backend]-->  PDF/HTML/EPUB/...
```

## License

MIT OR Apache-2.0
