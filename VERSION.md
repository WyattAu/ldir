# LDIR Version & State Tracker

## Current State
- **Phase:** Phase H — Formal Verification & Ecosystem Hardening
- **Version:** 3.6.0
- **Status:** ✅ All quality gates passing
- **Last Updated:** 2026-05-08

## Quality Metrics
| Metric | Value |
|--------|-------|
| **Total tests** | **1,652** |
| Test failures | 0 |
| Clippy errors | 0 (`-D warnings`) |
| `cargo fmt` | Clean |
| Lean4 proofs | 0 errors, 6 sorry (4 isAcyclic + 1 compile List.mem foldl + 1 cumWidth_mono; all with proof sketches) |
| Production unwrap/expect | 0 (all eliminated) |
| PDF determinism | Bit-identical verified |

## What Changed (v3.2.0 → v3.3.0)
### Accessibility & Compliance (E-4, E-5)
- **WCAG 2.1 structure types**: Expanded `StructureType` with H1-H6, ListLabel/Body, TableHeader/TableBody/TableHeaderCell/TableDataCell, FootnoteRef/FootnoteBody, Span, Artifact. 37 new tests.
- **PDF/A conformance**: `PdfConformance` enum (PdfA4, PdfA2b, PdfA3b) with `--pdfa-level` CLI flag, XMP metadata generation, output intent support. 12 new tests.

### ICC Color Management (E-3)
- **ICC profile handling**: `IccProfile` type with parsing, sRGB/CMYK/Gray built-in profiles, sRGB↔CMYK conversion, ICC alternate name mapping. Wired into PDF writer for OutputIntent streams.

### Formal Verification (E-1, E-2, E-8)
- **KP optimality**: `totalDemerits` definition, `kp_optimality` theorem stated, 3 supporting lemmas.
- **Compilation correctness**: `sirSemanticContent` and `girSemanticContent` defined; `compile_preserves_content` and `compile_nonempty_content_produces_glyphs` stated (2 sorry pending real compiler formalization).
- **5 sorry remaining**: 3 isAcyclic monotonicity, 2 compilation correctness.

### Ecosystem (E-6, E-7)
- **crates.io prep**: All 25 crates have complete metadata; `ldir-ir` passes `cargo publish --dry-run`. 18 READMEs created.
- **Continuous fuzzing**: 5 harnesses (parser, validator, compiler, lir_compile, pdf_emit), seed corpus, daily GitHub Actions workflow.

### Infrastructure
- Fixed 6 compilation errors from agent-generated code (conformance field, LUT8 array coercion, clippy is_multiple_of).
- `cargo fmt` applied to all new files.

## What Changed (v3.1.0 → v3.2.0)
### Proof Alignment (D-1)
- Lean4 BlockType expanded from 6 to 15 variants; acyclicity + payload integrity in `wellFormedSIR`; GIRCommand args `Fin 8 → Int`.

### Determinism (D-2)
- `HashMap`→`IndexMap` in all compiler hot paths; 6 SHA256/bit-identical determinism tests.

### Performance (D-3)
- `#[inline]` on 15 hot functions, `#[cold]` on 4 error paths, ASCII byte iteration — 6-7% speedup on 100-page docs.

### S-IR v2 Completeness (D-4)
- Table (caption, column_widths, header_row), Image (FloatPlacement), CodeBlock (content), PageStyle (dimensions, margins, header/footer).

### Backend Parity (D-5)
- HTML LaTeX→HTML math, HTML/TXT cross-refs, EPUB/DOCX image embedding, TXT heading underlines + nested lists + code blocks.

### Documentation (D-6)
- `docs/user-guide.md` (comprehensive), `docs/plugins.md` (with example).

### Advanced Features (D-7)
- Pattern-based English hyphenation (11 tests), optical margin alignment (9 tests), wired into Knuth-Plass.

## What Changed (v3.0.0 → v3.1.0)
### Quality Hardening
- **Zero production unwrap/expect**: Eliminated all 42 instances. Removed all `#![allow(clippy::unwrap_used)]`.
- **Lean4 proof complete**: Resolved `kp_termination` sorry using constructive singleton witness.
- **Real payload integrity validator**: Bounds checking and UTF-8 validation (9 tests).
- **Bibliography in L-IR path**: `LIRBibEntry`, `LIRBibliography`, `LIRCitation` types (3 tests).
- **Full UBA N0.b bracket pairs**: BD16 pair identification per UAX#9 (14 tests).
- **Vello real glyph outlines**: ttf_parser + kurbo::BezPath rendering (5 tests).

## Test Summary
| Category | Count |
|----------|-------|
| ldir-core | 794 |
| ldir-ir | 187 |
| ldir-pdf | 163 |
| ldir-html | 42 |
| ldir-tex | 56 |
| ldir-md | 30 |
| ldir-typst | 26 |
| ldir-org | 27 |
| ldir-adoc | 24 |
| ldir-as | 36 |
| ldir-txt | 26 |
| ldir-docx | 14 |
| ldir-wasm | 70 |
| ldir-html-reader | 34 |
| ldir-docx-reader | 17 |
| ldir-diff | 15 |
| ldir-validate | 5 |
| ldir-epub | 11 |
| **Total** | **1,577** |

## Artifact Summary
| Category | Lines |
|----------|-------|
| Rust source code | ~67,000 |
| Lean4 proofs | ~900 |
| Specs (Yellow/Blue papers, configs) | ~11,000 |
| CI/CD configs | ~300 |
| **Total** | ~79,200 |

## Lean4 Proof Status
- **Active file:** `proof_ir_wellformedness.lean` — 5 sorry (3 isAcyclic monotonicity, 2 compilation correctness; proof sketches provided), ~20 theorems fully proven
- **Active file:** `ProofLayoutProperties.lean` — 0 sorry, kp_termination proven, kp_optimality stated

## Workspace Structure
- 25 Rust crates + 1 Lean4 project
- Rust edition 2024, MSRV 1.85
- 25 unsafe blocks (all justified FFI: 19 harfbuzz, 4 font loader, 1 font tables, 1 ecs)
- Zero production unwrap/expect calls

## Supported Formats
- **Input (9):** MD, TeX, Typst, HTML, Adoc, Org, DOCX, SIR2, LDIR
- **Output (8):** PDF, HTML, EPUB, DOCX, TXT, GIR, SIR2, LDIR

## CLI Tools
| Tool | Description |
|------|-------------|
| `ldc` | Main compiler driver (with `--pdfa-level` flag) |
| `ldir-dis` | IR disassembler |
| `ldir-as` | IR assembler |
| `ldir-diff` | IR diff tool |
| `ldir-validate` | IR validator |
| `ldir-opt` | IR optimizer (8 passes) |
| `ldir-link` | IR module linker |
| `ldir-lsp` | Language server |

## Ecosystem
- **VS Code extension:** Syntax highlighting (TeX, Typst), compile/preview commands, LSP integration
- **WASM playground:** Browser-based MD→HTML rendering
- **HarfBuzz shaping:** UPEM-correct scale, font features API, offset consistency
- **Performance:** Arena allocator (bumpalo), LRU shape cache, incremental compilation with dirty tracking
- **Vello:** Real glyph outlines via ttf_parser + kurbo::BezPath
- **Bibliography:** IEEE/APA formatting, full L-IR pipeline support
- **UBA:** Full UAX#9 L1-L4 + N0.b bracket pair resolution
- **Hyphenation:** Pattern-based English syllable boundaries, wired into Knuth-Plass
- **PDF/A:** Conformance levels (4, 2b, 3b), XMP metadata, ICC color profiles
- **WCAG:** H1-H6, TR/TH/TD, language spans, reading order, BBox
- **Fuzzing:** 5 harnesses, daily CI, seed corpus
- **crates.io:** All 25 crates metadata-complete
