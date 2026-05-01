# ldir

[![crates.io](https://img.shields.io/crates/v/ldir.svg)](https://crates.io/crates/ldir)
[![CI](https://img.shields.io/github/actions/workflow/status/WyattAu/ldir/ci.yml?branch=main)](https://github.com/WyattAu/ldir/actions)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

A low-level document intermediate representation language for deterministic typesetting.

Compiles Markdown and TeX documents to PDF with a formally verified IR layer, real font rendering via HarfBuzz, and Knuth-Plass line breaking -- all in pure Rust with no C dependencies.

## Features

- CommonMark: bold, italic, mono, links, blockquotes, code blocks
- TrueType font embedding (Type0 + CIDFontType2)
- FlateDecode compression
- Multi-page layout
- Knuth-Plass line breaking
- S-IR / G-IR intermediate representation
- Lean 4 formal verification of IR well-formedness

## Quick Start

```sh
cargo install ldc
ldc document.md -o document.pdf
```

## Crates

| Crate | Description |
|---|---|
| [ldir-ir](ldir-ir/) | S-IR and G-IR data structures with rkyv serialization |
| [ldir-core](ldir-core/) | Compiler, validator, emitter, text shaping, line breaking |
| [ldir-md](ldir-md/) | Markdown to S-IR parser |
| [ldir-pdf](ldir-pdf/) | G-IR to PDF converter |
| [ldc](ldc/) | CLI compiler |

## WebAssembly

The core library crates compile to `wasm32-unknown-unknown`:
- `ldir-ir` — S-IR/G-IR types (pure Rust)
- `ldir-pdf` — PDF generation (pure Rust)
- `ldir-md` — Markdown parser (pure Rust)
- `ldir-tex` — TeX parser (pure Rust)
- `ldir-core` — Compiler (shapes via HarfBuzz on native, ASCII fallback on WASM)

## License

Licensed under either of [MIT](LICENSE) or [Apache-2.0](LICENSE) at your option.
