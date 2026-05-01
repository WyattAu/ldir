# LDIR Version & State Tracker

## Current State
- **Phase:** Phase B — Ecosystem & Hardening (A-1 ✅ … A-6 ✅, B-1 ✅, B-2 pending)
- **Version:** 3.0.0
- **Status:** 🔄 In Progress
- **Last Updated:** 2026-05-01

## Implementation Status
| Task | Description | Status | Tests |
|------|-------------|--------|-------|
| TASK-001 | Rust monorepo workspace | ✅ | — |
| TASK-002 | fp26_6 arithmetic | ✅ | 15 |
| TASK-003 | ECS core module | ✅ | 61 |
| TASK-004 | Error types & diagnostics | ✅ | — |
| TASK-005 | Tracing & profiling | ✅ | 9 |
| TASK-006 | S-IR types | ✅ | 67 |
| TASK-007 | rkyv serialization | ✅ | 8 |
| TASK-008 | S-IR parser | ✅ | 10 |
| TASK-009 | S-IR validator | ✅ | 31 |
| TASK-010 | Source mapping (LSP) | ✅ | 16 |
| TASK-011 | G-IR types | ✅ | (in TASK-006) |
| TASK-012 | S-IR→G-IR compiler | ✅ | 24 |
| TASK-013 | G-IR emitter | ✅ | 11 |
| TASK-014 | G-IR verifier | ✅ | 13 |
| TASK-015 | Determinism tests | ✅ | 20 |
| TASK-016 | Text shaping stub | ✅ | 23 |
| TASK-017 | Knuth-Plass line breaking | ✅ | 9 |
| TASK-018 | Pagination | ✅ | 12 |
| TASK-019 | Cassowary constraint solver | ✅ | 40 |
| TASK-020 | Incremental re-layout | ✅ | 15 |
| TASK-021 | Font loading stub | ✅ | 21 |
| TASK-022 | PDF/A-4 emission | ✅ | 11 |
| TASK-023 | Vello renderer | ✅ | 38 |
| TASK-024 | WASM bridge | ✅ | 35 |
| TASK-025 | Property-based tests | ✅ | 3 |
| TASK-026 | Property-based tests (extended) | ✅ | (in TASK-025) |
| TASK-027 | Performance benchmarks | ✅ | 10 benches |
| TASK-028 | CI/CD pipeline | ✅ | 4 jobs |
| TASK-029 | API docs | ✅ | 0 warnings |
| TASK-030 | User guide + examples | ✅ | 5 examples |

## Test Summary
| Category | Count |
|----------|-------|
| **Total tests** | **1,158** |
| All passing | ✅ |

## Artifact Summary
| Category | Lines |
|----------|-------|
| Rust source code | 42,008 |
| Lean4 proofs | 268 |
| Specs (Yellow/Blue papers, configs) | 10,732 |
| CI/CD configs | ~150 |
| **Total** | ~53,158 |

## Lean4 Proof Status
- **File:** `.specs/02_architecture/proofs/LDIRProofs/ProofIRWellformedness.lean`
- **State:** 0 errors, 0 sorry (List.Nodup approach)
- **Proven:** 10/10 theorems

## Workspace Structure
- 24 Rust crates + 1 Lean4 project
- Rust edition 2024, MSRV 1.85
- Zero unsafe code
- No external C dependencies in core pipeline

## Supported Formats
- **Input (9):** MD, TeX, Typst, HTML, Adoc, Org, DOCX, SIR2, LDIR
- **Output (8):** PDF, HTML, EPUB, DOCX, TXT, GIR, SIR2, LDIR

## CLI Tools
| Tool | Description |
|------|-------------|
| `ldc` | Main compiler driver |
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
- **HarfBuzz shaping:** UPEM-correct scale, font features API, offset consistency (structurally complete)
- **Performance:** Arena allocator (bumpalo) zero-alloc hot path, LRU shape cache with hit/miss stats, incremental compilation with dirty tracking
