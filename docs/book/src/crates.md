# Crate Overview

LDIR is organized as a workspace with 26 crates. Each crate has a single, well-defined responsibility.

## Core

| Crate | Version | Description |
|-------|---------|-------------|
| `ldir-core` | 0.1.0 | Core typesetting engine: shaping, layout, rendering, compilation pipeline |
| `ldir-ir` | 0.1.0 | Intermediate representations: GIR, S-IR v2, binary serialization (rkyv) |

## Parsers

| Crate | Version | Description |
|-------|---------|-------------|
| `ldir-md` | 0.1.0 | Markdown parser (CommonMark + extensions) |
| `ldir-tex` | 0.1.0 | LaTeX parser |
| `ldir-typst` | 0.1.0 | Typst parser |
| `ldir-adoc` | 0.1.0 | AsciiDoc parser |
| `ldir-org` | 0.1.0 | Org-mode parser |
| `ldir-html-reader` | 0.1.0 | HTML reader/parser |
| `ldir-docx-reader` | 0.1.0 | DOCX reader (OOXML) |

## Renderers

| Crate | Version | Description |
|-------|---------|-------------|
| `ldir-pdf` | 0.1.0 | PDF generator (pdf-writer, PDF/A support) |
| `ldir-html` | 0.1.0 | HTML generator |
| `ldir-epub` | 0.1.0 | EPUB generator |
| `ldir-txt` | 0.1.0 | Plain text generator |
| `ldir-docx` | 0.1.0 | DOCX generator |
| `ldir-vello` | 0.1.0 | GPU renderer (Vello + wgpu) |

## Utilities

| Crate | Version | Description |
|-------|---------|-------------|
| `ldir-as` | 0.1.0 | S-IR assembler (text to binary) |
| `ldir-dis` | 0.1.0 | S-IR disassembler (binary to text/JSON) |
| `ldir-diff` | 0.1.0 | S-IR structural diff |
| `ldir-validate` | 0.1.0 | S-IR well-formedness validator |
| `ldir-opt` | 0.1.0 | S-IR optimizer (transformation passes) |
| `ldir-link` | 0.1.0 | S-IR linker (merge multiple modules) |
| `ldir-lsp` | 0.1.0 | Language Server Protocol implementation |
| `ldir-wasm` | 0.1.0 | WebAssembly bindings (wasmtime) |

## CLI

| Crate | Version | Description |
|-------|---------|-------------|
| `ldc` | 0.1.0 | Main compiler CLI |

## Testing

| Crate | Version | Description |
|-------|---------|-------------|
| `ldir-test-helpers` | 0.1.0 | Shared test utilities |

## Formal Verification

| Directory | Description |
|-----------|-------------|
| `ldir-lean/` | Lean 4 formal proofs for core algorithms (0 `sorry`) |

## Quality Metrics

- **Tests**: 1,808 passed, 0 failed
- **Clippy**: 0 errors, 0 warnings (`-D warnings`)
- **Lean 4**: 0 `sorry` (all proofs complete)
- **MSRV**: 1.88 (Rust edition 2024)
- **Unsafe**: 25 blocks (19 HarfBuzz FFI, 4 font loader, 2 test-only)
