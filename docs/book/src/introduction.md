# LDIR Documentation

LDIR (Low-level Document Intermediate Representation) is a deterministic typesetting engine for compiling structured documents to high-quality PDF, HTML, EPUB, and other formats.

## Quick Start

```sh
# Install
cargo install ldc

# Compile Markdown to PDF
ldc input.md -o output.pdf

# Compile Typst to HTML
ldc input.typ -f html -o output.html

# Compile LaTeX to EPUB
ldc input.tex -f epub -o output.epub
```

## Input Formats

| Format | Extension | Parser |
|--------|-----------|--------|
| Markdown | `.md` | `ldir-md` |
| LaTeX | `.tex` | `ldir-tex` |
| Typst | `.typ` | `ldir-typst` |
| AsciiDoc | `.adoc` | `ldir-adoc` |
| Org-mode | `.org` | `ldir-org` |
| HTML | `.html`, `.htm` | `ldir-html-reader` |
| DOCX | `.docx` | `ldir-docx-reader` |

## Output Formats

| Format | Extension | Backend |
|--------|-----------|---------|
| PDF | `.pdf` | `ldir-pdf` |
| HTML | `.html` | `ldir-html` |
| EPUB | `.epub` | `ldir-epub` |
| TXT | `.txt` | `ldir-txt` |
| DOCX | `.docx` | `ldir-docx` |
| S-IR | `.sir2` | `ldir-ir` |
| L-IR | `.ldir` | `ldir-core` |

## Architecture

LDIR uses a three-stage compilation pipeline:

```
Input -> [Parser] -> S-IR -> [Compiler] -> L-IR -> [Renderer] -> Output
```

- **S-IR** (Semantic IR): Format-agnostic document tree with typed nodes
- **L-IR** (Layout IR): Page-level layout with positioned glyphs and geometry
- **G-IR** (Graphic IR): Low-level rendering commands (move, draw, set-font)

## CLI Reference

See `ldc --help` for the full CLI reference. Shell completions are available in the `completions/` directory for Bash, Zsh, Fish, and PowerShell.
