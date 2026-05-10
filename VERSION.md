# LDIR Version & State Tracker

## Current State
- **Phase:** Phase N -- Quality Hardening, Zero-Warning Enforcement, Pre-commit Gates
- **Version:** 3.14.0
- **Status:** All quality gates passing
- **Last Updated:** 2026-05-10

## Quality Metrics
| Metric | Value |
|--------|-------|
| **Total tests** | **1,863** |
| Test failures | 0 |
| Clippy errors | 0 (`-D warnings`) |
| Clippy warnings | 0 (test + lib) |
| `cargo fmt` | Clean |
| Lean4 proofs | 0 errors, 3 sorry (isAcyclic_cons_root, isAcyclic_cons_orphan, compile_preserves_content; all with proof sketches) |
| Production unwrap/expect | 0 (all eliminated) |
| PDF determinism | Bit-identical verified |

## What Changed (v3.13.0 -> v3.14.0)
### Quality Hardening (N-1)
- **Zero warnings enforcement**: Eliminated all 6 remaining compiler warnings (unused imports, unused mut, unused variables, unused assignments) across `ldir-core`, `ldir-ir`, `ldir-pdf`.
- **`ldir-pdf` test compilation fix**: Added `#[allow(unsafe_code)]` on font table test module to resolve `#![deny(unsafe_code)]` conflict with test helper using lifetime transmute.
- **Formatting normalization**: Applied `cargo fmt` across entire workspace; all 30+ formatting diffs resolved.
- **`.cargo/config.toml` cleanup**: Removed invalid `build.targets` key that produced config warning.
- **Documentation audit**: Removed all emoji characters from VERSION.md and CAPABILITY_MATRIX.md; replaced with plain-text status indicators.
- **Test count update**: Verified 1,863 tests passing (up from 1,610 documented).
- **Lean4 proof status reconciliation**: Updated sorry count from 5 to 3 to reflect Era N proofs (`cumWidth_mono`, `isAcyclicAux_mono`).
- **Pre-commit hook**: Added `.git/hooks/pre-commit` enforcing `cargo fmt --check`, `cargo clippy -D warnings`, and `cargo test`.

## What Changed (v3.9.0 -> v3.10.0)
### Lean4 Proof Progress (K-1)
- **`isAcyclic_single` FULLY PROVEN**: Closed 1 of 6 sorry. Key insight: `by_cases` on structural equality (`instr.parent_id = rootSentinel`) bridges the Prop/Bool `¬` gap. In the `parent ≠ root` case, `simp [h_eq]` rewrites BEq to `decide`, `unfold isAcyclicAux` reduces fuel=0 to `false`, and `simp` closes. Down from 6 sorry to 5.
- **Deep Lean4 sorry analysis**: Identified exact resolution paths for all remaining sorry: `isAcyclicAux_mono` step (nested match alignment), `isAcyclic_cons_root`/`cons_orphan` (depend on mono), `compile_preserves_content` (List.mem foldl), `cumWidth_mono` (prefix monotonicity via foldl).

### Cross-Reference Resolution (K-2)
- **`ldir-core/src/cross_ref.rs`**: New module with `LabelRegistry` (register/lookup/unresolved_refs), `ResolvedRef` (label, page, section, RefType), `resolve_ref` parser. 11 RefType variants (Internal, External, Bibliography, Equation, Figure, Table). 33 new tests (31 unit + 2 integration).

## What Changed (v3.8.0 → v3.9.0)
### Deep Lean4 Sorry Analysis (J-1)
- Identified exact root cause of all 6 sorry: Lean4 `¬` for Bool is `(b → False) : Prop`, not `Bool.not b : Bool`. 7 tactic strategies attempted and documented (rfl, unfold+rfl, show, decide, native_decide, absurd, congrArg Bool.not).
- Clippy fix: `image.rs` line 181 `needlessly_taken_reference` → `data.get(0..4)`.

## What Changed (v3.7.0 → v3.8.0)
### PDF Table Rendering (J-2)
- `draw_table` method: grid via horizontal/vertical `drawRule`, configurable columns/rows/line_width. 6 tests.

### Image Dimension Detection (J-3)
- PNG IHDR at offset 16, JPEG SOF0/SOF2 scan. `ImageDimensions` struct. 8 tests.

## What Changed (v3.6.0 → v3.7.0)
### CI/CD (I-1)
- `ci.yml`: Rust (clippy+fmt+test), Lean4 (lake build), WASM32 (cargo caching), parallel jobs.
- `release.yml`: Tag-triggered ldc binary artifact.

### Benchmarks (I-2)
- Criterion: 7 functions (md/tex parse small/medium, compile SIR, PDF generate, validator 10/50/100).

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
| ldir-core | 827 |
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
| **Total** | **1,610** |

## Artifact Summary
| Category | Lines |
|----------|-------|
| Rust source code | ~67,500 |
| Lean4 proofs | ~1,000 |
| Specs (Yellow/Blue papers, configs) | ~11,000 |
| CI/CD configs | ~300 |
| **Total** | ~79,200 |

## Lean4 Proof Status
- **`proof_ir_wellformedness.lean`** -- 3 sorry (isAcyclic_cons_root, isAcyclic_cons_orphan, compile_preserves_content), ~20 theorems fully proven including `isAcyclic_single`, `isAcyclicAux_mono` [PROVEN]
- **`ProofLayoutProperties.lean`** -- 0 sorry, `cumWidth_mono` proven, `kp_termination` proven, 3 KP theorems proven

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
