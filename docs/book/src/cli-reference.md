# CLI Reference

## ldc

The main LDIR compiler binary.

```
ldc [OPTIONS] [INPUTS]...
```

### Positional Arguments

| Argument | Description |
|----------|-------------|
| `INPUTS` | Input file(s). Multiple files are merged with offset IDs. Supported: `.md`, `.tex`, `.typ`, `.html`, `.htm`, `.adoc`, `.org`, `.docx` |

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `-o`, `--output` | Output file path | Auto-derived from input |
| `-f`, `--format` | Output format (`pdf`, `html`, `epub`, `txt`, `docx`, `sir2`, `ldir`, `gir`, `sir`) | `pdf` |
| `--font` | Primary font family name | Auto-detected |
| `--font-mono` | Monospace font family name | Auto-detected |
| `--font-path` | Path to primary font file (`.ttf`/`.otf`) | None |
| `--list-fonts` | List available system fonts and exit | false |
| `--title` | Document title for PDF metadata | None |
| `--author` | Document author for PDF metadata | None |
| `--subject` | Document subject for PDF metadata | None |
| `--margin` | Page margin in inches | `1.0` |
| `--page-size` | Page size preset (`a4`, `letter`, `legal`) | None |
| `--page-width` | Custom page width in points | None |
| `--page-height` | Custom page height in points | None |
| `--header-left` | Header left template | None |
| `--header-center` | Header center template | None |
| `--header-right` | Header right template | None |
| `--footer-left` | Footer left template | None |
| `--footer-center` | Footer center template | None |
| `--footer-right` | Footer right template | None |
| `--no-header-rule` | Disable header rule line | false |
| `--no-footer-rule` | Disable footer rule line | false |
| `--drop-caps` | Enable drop caps | false |
| `--bibliography` | Path to BibTeX (`.bib`) file | None |
| `--lir` | Use L-IR layout pipeline | false |
| `--pdfa-level` | PDF/A conformance level | `4` |

### Header/Footer Templates

Templates support the following placeholders:

| Placeholder | Description |
|-------------|-------------|
| `%page` | Current page number |
| `%pages` | Total page count |
| `%title` | Document title |
| `%author` | Document author |
| `%date` | Current date |

### Examples

```sh
# Markdown to PDF with custom font
ldc input.md -o output.pdf --font "Noto Serif"

# Typst to HTML with A4 size
ldc input.typ -f html --page-size a4

# LaTeX to PDF with headers/footers
ldc input.tex --header-left "%title" --footer-center "%page / %pages"

# Multiple inputs merged
ldc chapter1.md chapter2.md -o book.pdf
```

## Utility Binaries

### ldir-as

S-IR assembler: `.ldir` text format to binary S-IR.

```sh
ldir-as input.ldir -o output.sir2
```

### ldir-dis

S-IR disassembler: binary S-IR to `.ldir` text or JSON.

```sh
ldir-dis input.sir2          # text output
ldir-dis input.sir2 -f json  # JSON output
```

### ldir-diff

Structural diff between two S-IR modules.

```sh
ldir-diff before.sir2 after.sir2
```

### ldir-validate

Validate S-IR module well-formedness.

```sh
ldir-validate input.sir2
```

### ldir-opt

S-IR optimizer with transformation passes.

```sh
ldir-opt input.sir2 -o optimized.sir2
```

### ldir-link

Link multiple S-IR modules into one.

```sh
ldir-link part1.sir2 part2.sir2 -o combined.sir2
```
