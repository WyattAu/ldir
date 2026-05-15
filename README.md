# ldir -- The LLVM of Documents

[![crates.io](https://img.shields.io/crates/v/ldir.svg)](https://crates.io/crates/ldir)
[![CI](https://img.shields.io/github/actions/workflow/status/WyattAu/ldir/ci.yml?branch=main)](https://github.com/WyattAu/ldir/actions)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

A universal intermediate representation (IR) for documents with multiple
frontends and backends. Compiles Markdown, LaTeX, Typst, HTML, and more to
PDF, HTML, EPUB, and other formats through a formally verified IR pipeline.

## Features

- **9 input formats**: Markdown, LaTeX, Typst, HTML, Asciidoc, Org, DOCX, SIR2, LDIR
- **8 output formats**: PDF, HTML, EPUB, TXT, DOCX, GIR, SIR2, LDIR
- **Three-layer IR**: S-IR (semantic) -> L-IR (layout) -> G-IR (rendering)
- **Formal verification**: Lean4 proofs for IR well-formedness (0 sorry)
- **Incremental re-layout**: Dirty-subtree tracking for fast recompilation
- **LSP support**: Language server with diagnostics, hover, go-to-definition
- **WASM**: Browser-based MD-to-HTML playground

## Architecture

```
Input -> [Parser] -> S-IR v2 -> [Compiler] -> L-IR -> [Renderer] -> PDF/HTML/EPUB/...
```

## Crates

| Crate | Description |
|-------|-------------|
| `ldir-ir` | IR type definitions (S-IR, L-IR, G-IR) |
| `ldir-core` | Compiler, layout, shaping, fonts, validator |
| `ldir-pdf` | PDF generation with TrueType embedding |
| `ldir-md` | Markdown parser (CommonMark + GFM) |
| `ldir-tex` | TeX/LaTeX parser |
| `ldir-typst` | Typst parser |
| `ldir-html` | HTML renderer |
| `ldir-html-reader` | HTML reader frontend |
| `ldir-epub` | EPUB generator |
| `ldir-txt` | Plain text renderer |
| `ldir-docx` | DOCX generator |
| `ldir-docx-reader` | DOCX reader frontend |
| `ldir-adoc` | Asciidoc parser |
| `ldir-org` | Org-mode parser |
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

## Quick Start

```sh
cargo install --path ldc
ldc input.md -o output.pdf
```

## WebAssembly

The core IR crates compile to `wasm32-unknown-unknown`:
- `ldir-ir` -- S-IR/G-IR types (pure Rust, no native deps)
- `ldir-md` -- Markdown parser (pure Rust)
- `ldir-tex` -- TeX parser (pure Rust)
- `ldir-wasm` -- WASM bridge and playground

Note: `ldir-pdf` and `ldir-core` depend on `harfbuzz-sys` (native FFI) and are
not suitable for WASM without a WASI-based HarfBuzz build.

## License

Licensed under either of [MIT](LICENSE) or [Apache-2.0](LICENSE) at your option.
