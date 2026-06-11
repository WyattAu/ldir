# ldir -- The LLVM of Documents

[![crates.io](https://img.shields.io/crates/v/ldir.svg)](https://crates.io/crates/ldir)
[![CI](https://img.shields.io/github/actions/workflow/status/WyattAu/ldir/ci.yml?branch=main)](https://github.com/WyattAu/ldir/actions)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-orange.svg)](https://github.com/rust-lang/rust/releases)
[![Lean4](https://img.shields.io/badge/formal%20verification-Lean4-purple.svg)](ldir-proofs/)
[![Tests](https://img.shields.io/badge/tests-2127%20passing-brightgreen.svg)]()

A universal intermediate representation (IR) for documents with multiple
frontends and backends. Compiles Markdown, LaTeX, Typst, HTML, and more to
PDF, HTML, EPUB, and other formats through a formally verified IR pipeline.

## Features

- **9 input formats**: Markdown, LaTeX, Typst, HTML, Asciidoc, Org, DOCX, SIR2, LDIR
- **11 output formats**: PDF, HTML, EPUB, TXT, DOCX, ODT, Pandoc AST, Jupyter, GIR, SIR2, LDIR
- **Three-layer IR**: S-IR (semantic) -> L-IR (layout) -> G-IR (rendering)
- **Formal verification**: Lean4 proofs for IR well-formedness (0 sorry)
- **PDF/A compliance**: PDF/A-1b, 2b, 3b output modes with veraPDF validation
- **Incremental re-layout**: Dirty-subtree tracking for fast recompilation
- **LSP support**: Completion, references, rename, code actions, incremental preview
- **WASM**: Browser-based MD-to-HTML playground
- **Multi-column layout**: Balanced column reflow with full-width spanning
- **CJK support**: CmapIterator covering 7 CJK Unicode ranges, compact glyph ID remapping
- **Multilingual hyphenation**: 5 languages (EN, DE, FR, ES, PT) + Liou pattern engine
- **Bibliography**: IEEE, APA, Chicago, MLA citation styles with year disambiguation
- **Font subsetting**: Compact TrueType embedding with glyph ID remapping and CIDToGIDMap
- **Tracked changes**: Insert/delete with author, date, revision metadata (DOCX, HTML)
- **Rich CLI**: Progress indicators, gcc/rustc-style diagnostics, ldir.toml config, shell completions

## Architecture

```
Input -> [Parser] -> S-IR v2 -> [Compiler] -> L-IR -> [Renderer] -> PDF/HTML/EPUB/...
                                  |                    |
                                  v                    v
                            [Optimizer]           [LSP Server]
                           (8 IR passes)      (incremental preview)
```

### IR Pipeline

1. **S-IR (Semantic IR)**: Document structure, styles, cross-references, citations
2. **L-IR (Layout IR)**: Page geometry, line/column breaks, positioned elements
3. **G-IR (Graphics IR)**: Rendering commands, font references, path operations

### Key Design Decisions

- S-IR uses f64 for semantic precision; G-IR uses Fp26.6 (26.6 fixed-point) for pixel-exact rendering
- Knuth-Plass line breaking with configurable penalty model
- Streaming PDF writer for constant-memory output regardless of document size

## Quick Start

```sh
# Install from source
cargo install --path ldc

# Compile markdown to PDF
ldc input.md -o output.pdf

# Compile with PDF/A compliance
ldc input.md --pdfa-level 2b -o output.pdf

# Compile to multiple formats
ldc input.md -o output.pdf
ldc input.md -o output.html
ldc input.md -o output.epub
ldc input.md -o output.docx

# Use configuration file
echo '[output]
format = "pdf"
pdfa_level = "2b"

[layout]
columns = 2' > ldir.toml
ldc input.md

# Dump effective configuration
ldc --dump-config
```

## Configuration

`ldc` reads configuration from (in priority order):
1. CLI flags (highest priority)
2. `./ldir.toml` or `./.ldir.toml` in the current or parent directories
3. `$XDG_CONFIG_HOME/ldir/config.toml` (usually `~/.config/ldir/config.toml`)

See `ldc --dump-config` for the effective merged configuration.

## CLI Flags

```
ldc [OPTIONS] <INPUT>

Options:
  -o, --output <PATH>         Output file path
  -f, --format <FORMAT>       Output format: pdf, html, epub, docx, txt, sir2, gir
      --pdfa-level <LEVEL>    PDF/A level: off, 1b, 2b, 3b
      --pdf-version <VER>     PDF version: 1.4, 1.5, 1.6, 1.7
      --ot-features <LIST>     OpenType features (default: kern,liga)
      --font-path <PATH>       Custom font file path
      --page-width <DIM>       Page width (e.g. 210mm, 8.5in)
      --page-height <DIM>      Page height (e.g. 297mm, 11in)
      --margin <DIM>           Page margins
      --font-size <SIZE>       Base font size (e.g. 11pt)
      --line-height <RATIO>    Line height ratio
      --columns <N>            Number of columns
      --color <WHEN>           Color output: always, never, auto
      --config <PATH>          Configuration file path
      --no-config              Skip configuration file loading
      --dump-config            Print effective configuration and exit
  -h, --help                   Show help
  -V, --version                Show version
```

Shell completions for bash, zsh, fish, and PowerShell are available via `clap-complete`.

## Crates

| Crate | Description |
|-------|-------------|
| `ldir-ir` | IR type definitions (S-IR, L-IR, G-IR) |
| `ldir-core` | Compiler, layout, shaping, fonts, hyphenation, bibliography, validator |
| `ldir-pdf` | PDF generation with TrueType embedding and PDF/A support |
| `ldir-md` | Markdown parser (CommonMark + GFM) |
| `ldir-tex` | TeX/LaTeX parser |
| `ldir-typst` | Typst parser |
| `ldir-html` | HTML renderer |
| `ldir-html-reader` | HTML reader frontend |
| `ldir-epub` | EPUB3 generator (accessibility metadata, nested TOC, landmarks) |
| `ldir-txt` | Plain text renderer |
| `ldir-docx` | DOCX generator (numbering, styles, image embedding) |
| `ldir-docx-reader` | DOCX reader frontend |
| `ldir-adoc` | Asciidoc parser |
| `ldir-org` | Org-mode parser |
| `ldir-odt` | ODT generator (ISO 26300) |
| `ldir-pandoc` | Pandoc JSON AST writer |
| `ldir-jupyter` | Jupyter notebook exporter |
| `ldir-bench` | Criterion.rs benchmarks with tracing-chrome |
| `ldir-wasm` | WASM playground |
| `ldir-vello` | GPU renderer (Vello/WGPU) |
| `ldir-lsp` | Language Server Protocol server |
| `ldir-opt` | IR optimizer (8 passes) |
| `ldir-link` | IR module linker |
| `ldir-dis` | IR disassembler |
| `ldir-as` | IR assembler |
| `ldir-diff` | IR structural diff |
| `ldir-validate` | IR well-formedness validator |
| `ldir-test-helpers` | Shared test utilities |
| `ldc` | CLI compiler |

## VS Code Extension

A VS Code extension is available in `editors/vscode/` with:
- Compile-on-save to PDF
- PDF preview panel with live reload
- LSP integration (diagnostics, completions, go-to-definition)
- Configuration via VS Code settings

## WebAssembly

The core IR crates compile to `wasm32-unknown-unknown`:
- `ldir-ir` -- S-IR/G-IR types (pure Rust, no native deps)
- `ldir-md` -- Markdown parser (pure Rust)
- `ldir-tex` -- TeX parser (pure Rust)
- `ldir-wasm` -- WASM bridge and playground

Note: `ldir-pdf` and `ldir-core` depend on `harfbuzz-sys` (native FFI) and are
not suitable for WASM without a WASI-based HarfBuzz build.

## Development

```sh
# Build all crates
cargo build --workspace

# Run tests (2,127 tests)
cargo test --workspace

# Lint (0 warnings)
cargo clippy --workspace -- -D warnings

# Format check
cargo fmt --check
```

### MSRV

Minimum Supported Rust Version: **1.88** (edition 2024, resolver 3)

### CI

GitHub Actions runs on every push and PR with 13+ jobs:
- `rust-check` (ubuntu, macos, windows matrix)
- `msrv-check`, `feature-gates`, `bench-check`
- `lean4-check`, `completions-check`, `pdfa-check`
- `cross-platform-determinism`, `security-audit`, `pdfa-conformance`

## Formal Verification

Lean4 proofs in `ldir-proofs/` verify IR well-formedness properties
with **0 `sorry`** (all proofs fully resolved).

## License

Licensed under either of [MIT](LICENSE) or [Apache-2.0](LICENSE) at your option.
