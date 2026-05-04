# LDIR User Guide

## Table of Contents

- [Getting Started](#getting-started)
  - [Installation](#installation)
  - [First Document: Hello World to PDF](#first-document-hello-world-to-pdf)
  - [CLI Tools](#cli-tools)
- [Input Formats](#input-formats)
  - [Markdown](#markdown)
  - [LaTeX](#latex)
  - [Typst](#typst)
  - [HTML](#html)
  - [AsciiDoc](#asciidoc)
  - [Org-mode](#org-mode)
  - [DOCX](#docx)
  - [S-IR v2 / LDIR (intermediate formats)](#s-ir-v2--ldir-intermediate-formats)
- [Output Formats](#output-formats)
  - [PDF](#pdf)
  - [HTML](#html)
  - [EPUB](#epub)
  - [DOCX](#docx)
  - [TXT](#txt)
  - [G-IR, S-IR v2 (intermediate formats)](#g-ir-s-ir-v2-intermediate-formats)
- [Advanced Features](#advanced-features)
  - [Cross-References and Bibliography](#cross-references-and-bibliography)
  - [Templates](#templates)
  - [Incremental Compilation via LSP](#incremental-compilation-via-lsp)
  - [WASM Playground](#wasm-playground)
  - [Font Management](#font-management)
  - [Page Layout](#page-layout)
- [Architecture Overview](#architecture-overview)
  - [S-IR to L-IR to G-IR Pipeline](#s-ir-to-l-ir-to-g-ir-pipeline)
  - [Module System and Linking](#module-system-and-linking)
  - [Plugin System](#plugin-system)

---

## Getting Started

### Installation

**From crates.io (when published):**

```sh
cargo install ldc
```

**From source:**

```sh
git clone https://github.com/WyattAu/ldir.git
cd ldir
cargo install --path ldc
```

This installs the `ldc` compiler driver. To build all tools:

```sh
cargo build --workspace
```

**Requirements:** Rust 1.85+ (edition 2024). No other dependencies are needed for basic text output. For PDF generation with embedded fonts, a TrueType font must be available (DejaVu Sans is auto-detected on most Linux systems).

### First Document: Hello World to PDF

Create `hello.md`:

```markdown
# Hello, LDIR

This is my first document compiled with the LDIR typesetting pipeline.

## Features

- **Bold**, *italic*, and `code` formatting
- Ordered lists
- [Links](https://github.com/WyattAu/ldir)
```

Compile to PDF:

```sh
ldc hello.md -o hello.pdf
```

Compile to HTML:

```sh
ldc hello.md -f html -o hello.html
```

Compile to plain text:

```sh
ldc hello.md -f txt -o hello.txt
```

The output format is auto-detected from the `-o` file extension, so the `-f` flag is optional when the extension is unambiguous.

### CLI Tools

LDIR ships 8 CLI tools:

| Tool | Description | Usage |
|------|-------------|-------|
| `ldc` | Main compiler driver | `ldc input.md -o output.pdf` |
| `ldir-dis` | IR disassembler | `ldir-dis document.gir` |
| `ldir-as` | IR assembler | `ldir-as instructions.txt -o document.gir` |
| `ldir-diff` | IR diff tool | `ldir-diff before.gir after.gir` |
| `ldir-validate` | IR validator | `ldir-validate document.sir2` |
| `ldir-opt` | IR optimizer (8 passes) | `ldir-opt input.sir2 -o optimized.sir2` |
| `ldir-link` | IR module linker | `ldir-link a.sir2 b.sir2 -o merged.sir2` |
| `ldir-lsp` | Language server | start via editor config |

**`ldc` flags:**

```
USAGE:
    ldc [INPUTS...] [OPTIONS]

OPTIONS:
    -o, --output <PATH>           Output file path
    -f, --format <FORMAT>         Output format: pdf, html, epub, txt, docx, sir2, ldir, gir, sir [default: pdf]
        --font <FAMILY>           Primary font family name
        --font-mono <FAMILY>      Monospace font family name
        --font-path <PATH>        Path to .ttf/.otf font file
        --list-fonts              List available system fonts
        --title <TITLE>           Document title (PDF metadata)
        --author <AUTHOR>         Document author (PDF metadata)
        --subject <SUBJECT>       Document subject (PDF metadata)
        --margin <INCHES>         Page margin in inches [default: 1.0]
        --page-size <SIZE>        Page size: a4, letter, legal
        --page-width <WIDTH_PT>   Custom page width in points
        --page-height <HEIGHT_PT> Custom page height in points
        --header-left <TEMPLATE>  Header left template (%page, %pages, %title, %author, %date)
        --header-center <TEMPLATE>
        --header-right <TEMPLATE>
        --footer-left <TEMPLATE>
        --footer-center <TEMPLATE>
        --footer-right <TEMPLATE> [default: %page]
        --no-header-rule          Disable header rule line
        --no-footer-rule          Disable footer rule line
        --drop-caps               Enable drop caps after headings
        --bibliography <PATH>     Path to .bib file for citations
        --lir                     Use L-IR layout pipeline (S-IR -> L-IR -> G-IR)
```

**Multiple input files** are merged with offset entity IDs, enabling multi-file documents:

```sh
ldc chapter1.md chapter2.md chapter3.md -o book.pdf
```

---

## Input Formats

LDIR auto-detects input format from file extension. No flags needed.

### Markdown

Extensions: `.md`, `.markdown`

The Markdown parser (`ldir-md`) supports:

- **CommonMark** base specification
- **GFM (GitHub Flavored Markdown) extensions:**
  - Tables (pipe-delimited)
  - Task lists (`- [x]` / `- [ ]`)
  - Strikethrough (`~~text~~`)
  - Autolinks
- **Footnotes** (`[^1]` with `[^1]: definition`)
- **Emphasis:** bold (`**`), italic (`*`), inline code (`` ` ``)
- **Block structure:** headings (ATX `#`), paragraphs, blockquotes, ordered/unordered lists
- **Links and images:** `[text](url)` and `![alt](src)`

```sh
ldc document.md -o document.pdf
```

### LaTeX

Extensions: `.tex`, `.latex`

The LaTeX parser (`ldir-tex`) supports:

- **Document structure:** `\documentclass`, `\begin{...}...\end{...}` environments
- **Sections:** `\section`, `\subsection`, `\subsubsection`, `\chapter`
- **Formatting:** `\textbf{}`, `\textit{}`, `\underline{}`, `\emph{}`
- **Math:** inline (`$...$`) and display (`\[...\]`) math modes
- **Environments:** `itemize`, `enumerate`, `equation`, `figure`, `table`, `abstract`
- **Bibliography:** `\cite{}`, `\bibliography{}`, `\bibliographystyle{}`
- **Cross-references:** `\label{}`, `\ref{}`, `\eqref{}`
- **Floats:** `\begin{figure}`, `\begin{table}` with captions

```sh
ldc paper.tex -o paper.pdf
ldc paper.tex --bibliography refs.bib -o paper.pdf
```

### Typst

Extension: `.typ`

The Typst parser (`ldir-typst`) converts Typst markup to S-IR v2:

```sh
ldc document.typ -o document.pdf
```

### HTML

Extensions: `.html`, `.htm`

The HTML reader (`ldir-html-reader`) parses HTML documents, extracting semantic structure (headings, paragraphs, lists, links) into S-IR v2:

```sh
ldc page.html -f txt -o page.txt
```

### AsciiDoc

Extensions: `.adoc`, `.asciidoc`

The AsciiDoc parser (`ldir-adoc`) handles:

- Section titles, paragraphs, lists, inline formatting
- Tables, code blocks, attributes

```sh
ldc document.adoc -o document.pdf
```

### Org-mode

Extension: `.org`

The Org-mode parser (`ldir-org`) supports:

- Headlines (`*`, `**`, `***`)
- Paragraphs, lists (ordered/unordered), checkboxes
- Code blocks, inline formatting
- Links (`[[target][description]]`)

```sh
ldc notes.org -o notes.pdf
```

### DOCX

Extension: `.docx`

The DOCX reader (`ldir-docx-reader`) extracts document structure from Office Open XML:

```sh
ldc document.docx -f pdf -o document.pdf
ldc document.docx -f html -o document.html
```

### S-IR v2 / LDIR (intermediate formats)

Extension: `.sir2`, `.ldir`

You can re-compile intermediate IR representations. This is useful for inspecting or debugging the pipeline:

```sh
ldc intermediate.sir2 -o final.pdf
```

---

## Output Formats

### PDF

Default output format. Uses the S-IR v2 to G-IR to PDF pipeline:

```sh
ldc input.md -o output.pdf
```

**Features:**
- TrueType font embedding with automatic system font discovery
- Font variant loading (bold, italic, bold-italic, monospace)
- Page size presets (A4, letter, legal) and custom dimensions
- Configurable margins, headers, and footers with template variables
- PDF metadata (title, author, subject)
- Bit-identical deterministic output
- Drop caps support
- BibTeX citation support

**L-IR pipeline** for advanced layout (Knuth-Plass line breaking, widow/orphan avoidance):

```sh
ldc input.md --lir -o output.pdf
```

### HTML

```sh
ldc input.md -f html -o output.html
ldc input.md -o output.html    # auto-detected from extension
```

### EPUB

```sh
ldc input.md -f epub -o output.epub
```

### DOCX

```sh
ldc input.md -f docx -o output.docx
```

### TXT

```sh
ldc input.md -f txt -o output.txt
```

### G-IR, S-IR v2 (intermediate formats)

Dump intermediate representations for inspection or further processing:

```sh
ldc input.md -f gir -o output.gir     # G-IR binary
ldc input.md -f sir -o output.sir2     # S-IR v2 binary
ldc input.md -f sir2 -o output.sir2    # S-IR v2 binary
ldc input.md -f ldir -o output.ldir    # S-IR v2 text format
```

---

## Advanced Features

### Cross-References and Bibliography

LDIR supports BibTeX bibliography files with `\cite{}` commands from LaTeX input:

```sh
ldc paper.tex --bibliography references.bib -o paper.pdf
```

The L-IR pipeline includes `LIRBibEntry`, `LIRBibliography`, and `LIRCitation` types with IEEE/APA formatting.

### Templates

Configure document appearance through `ldc` flags:

```sh
ldc report.md \
  --title "Quarterly Report" \
  --author "Jane Doe" \
  --page-size a4 \
  --margin 1.0 \
  --header-left "%title" \
  --header-right "%date" \
  --footer-center "%page / %pages" \
  --drop-caps \
  -o report.pdf
```

Available header/footer template variables: `%page`, `%pages`, `%title`, `%author`, `%date`.

### Incremental Compilation via LSP

The LSP server (`ldir-lsp`) provides:

- **Diagnostics:** Real-time error reporting as you type
- **Hover:** Type and documentation information on hover
- **Go-to-definition:** Navigate to referenced entities
- **Document symbols:** Outline view of document structure

Start via your editor's LSP configuration. The server tracks dirty subtrees for incremental recompilation.

### WASM Playground

Core crates compile to `wasm32-unknown-unknown` for browser-based rendering:

- `ldir-ir` — S-IR/G-IR types (pure Rust)
- `ldir-pdf` — PDF generation (pure Rust)
- `ldir-md` — Markdown parser (pure Rust)
- `ldir-core` — Compiler with ASCII fallback shaping on WASM

The WASM playground provides browser-based Markdown to HTML rendering.

### Font Management

**List system fonts:**

```sh
ldc --list-fonts
```

**Specify a font by family name:**

```sh
ldc input.md --font "Noto Serif" -o output.pdf
```

**Specify a font by file path:**

```sh
ldc input.md --font-path /path/to/MyFont.ttf -o output.pdf
```

**Specify monospace font:**

```sh
ldc input.md --font-mono "Fira Code" -o output.pdf
```

### Page Layout

**Preset sizes:**

```sh
ldc input.md --page-size a4 -o output.pdf
ldc input.md --page-size letter -o output.pdf
ldc input.md --page-size legal -o output.pdf
```

**Custom dimensions (in points, 1 inch = 72 pt):**

```sh
ldc input.md --page-width 595 --page-height 842 -o output.pdf
```

**Margins (in inches):**

```sh
ldc input.md --margin 0.75 -o output.pdf
```

---

## Architecture Overview

### S-IR to L-IR to G-IR Pipeline

LDIR uses a three-layer intermediate representation:

```
Input (MD/TeX/Typst/HTML/...)  →  S-IR v2  →  L-IR  →  G-IR  →  Output (PDF/HTML/...)
        [Parser]                    [Compiler]  [Layout]  [Render]
```

**S-IR (Source IR):** Tree-structured semantic representation of the document. Nodes represent sections, paragraphs, lists, math, links, etc. Serialized in binary (`.sir2`) or text (`.ldir`) format.

**L-IR (Layout IR):** Paginated layout with typeset positions. Includes Knuth-Plass line breaking, bibliography formatting, and page geometry. Used when the `--lir` flag is specified.

**G-IR (Graphical IR):** Flat sequence of rendering commands (PutGlyph, SetFont, PushStack, MoveXY, etc.) organized into pages. This is what backends consume to produce output.

### Module System and Linking

Documents can be split into modules and linked:

```sh
# Compile individual chapters to S-IR v2
ldc chapter1.md -f sir2 -o ch1.sir2
ldc chapter2.md -f sir2 -o ch2.sir2

# Link modules into a single document
ldir-link ch1.sir2 ch2.sir2 -o book.sir2

# Optimize the merged document
ldir-opt book.sir2 -o book-opt.sir2

# Validate
ldir-validate book-opt.sir2

# Compile to PDF
ldc book-opt.sir2 -o book.pdf
```

The optimizer includes 8 passes:
1. Dead node elimination
2. Dead style elimination
3. Dead resource elimination
4. Empty block collapse
5. Style inlining
6. Counter propagation
7. Label deduplication
8. Text node merging

The disassembler and assembler allow inspecting and editing G-IR:

```sh
ldir-dis document.gir > instructions.txt
ldir-as instructions.txt -o modified.gir
ldir-diff before.gir after.gir
```

### Plugin System

LDIR provides a trait-based plugin system for extending input and output formats:

- **FrontendPlugin** — register custom input parsers
- **BackendPlugin** — register custom output generators
- **PluginRegistry** — discover plugins by file extension or name

See [plugins.md](plugins.md) for implementation details.
