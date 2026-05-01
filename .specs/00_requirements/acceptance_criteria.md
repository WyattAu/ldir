# LDIR Acceptance Criteria

## Overview
Acceptance criteria derived from SRS1 and SRS2. Each criterion maps to one or more requirements and specifies a verification method.

## Criteria

| ID | Requirement Ref | Criterion | Verification Method | Priority |
|----|----------------|-----------|-------------------|----------|
| AC-001 | REQ-08.01.02 | Compiling same S-IR with 1/4/16 threads yields bitwise-identical G-IR hash | Automated test | Critical |
| AC-002 | REQ-08.01.01 | No malformed S-IR causes OOB panic, segfault, or infinite loop | Continuous fuzzing | Critical |
| AC-003 | REQ-08.01.03 | War and Peace (plain text → PDF) compiles in < 100ms | Benchmark suite | High |
| AC-004 | REQ-08.01.03 | Single word update in 1000-page document updates viewer in < 5ms | Benchmark suite | High |
| AC-005 | REQ-06.01.02 | GPU viewer renders pan/zoom at 144Hz (< 6.9ms frame budget) | Frame profiling | High |
| AC-006 | REQ-01.01.01 | Zero dynamic heap allocations during hot layout pass | Valgrind/ASAN | Critical |
| AC-007 | REQ-01.01.02 | Document attributes use SoA layout aligned to 64-byte boundaries | Static analysis | High |
| AC-008 | REQ-01.01.03 | No raw pointers/Rc/Arc for document nodes; all relations via 32-bit generation indices | Code review + Clippy | High |
| AC-009 | REQ-02.02.01 | S-IR can be mmap'd and layout begins in O(1) time | Benchmark suite | High |
| AC-010 | REQ-03.02.01 | Knuth-Plass evaluates 8 line-break candidates simultaneously (SIMD) | Benchmark + inspection | Medium |
| AC-011 | REQ-03.02.02 | Inner DP loop is branchless | Code review + static analysis | Medium |
| AC-012 | REQ-03.03.01 | Page-breaks modeled as DAG | Code review | High |
| AC-013 | REQ-04.01.01 | WASM plugins execute in wasmtime sandbox; no native plugins | Security audit | Critical |
| AC-014 | REQ-04.01.03 | WASM macro exceeding 100,000 instructions traps with error | Fuzzing test | High |
| AC-015 | REQ-05.02.01 | Every parsed token maps byte-offset → EntityID | Integration test | High |
| AC-016 | REQ-05.02.02 | Hover on pixel resolves to source file:line in < 2ms | Benchmark suite | Medium |
| AC-017 | REQ-06.02.03 | Font subsetting produces minimal glyph set from G-IR | Diff test against reference | Medium |
| AC-018 | REQ-07.01.01 | Nanosecond tracing via RDTSC/CNTVCT_EL0 | Integration test | Low |
| AC-019 | Lean4 | S-IR well-formedness proof compiles and passes in Lean4 | `lean` compilation | Critical |
| AC-020 | REQ-01.02.01 | Thread pool pinned to physical cores via CPU affinity | Runtime verification | Medium |
| AC-021 | REQ-01.02.02 | Font shaping caches use lock-free hash map under concurrent access | Stress test (16 threads) | High |
| AC-022 | REQ-02.01.02 | 26.6 fixed-point precision matches FreeType internal format | Unit test (known values) | High |
| AC-023 | REQ-03.01.01 | ASCII-only paragraphs bypass HarfBuzz via vectorized fast-path | Benchmark comparison | Medium |
| AC-024 | REQ-03.03.02 | Branch-and-bound pagination completes for 1000-page doc in < 50ms | Benchmark suite | High |
| AC-025 | REQ-03.04.01 | Cassowary solver with fixed-point arithmetic produces results within documented error bounds | Property-based test | High |
| AC-026 | REQ-04.01.02 | WASM zero-copy ABI: guest reads S-IR via pointer+length without data copy | ASAN + inspection | High |
| AC-027 | REQ-05.01.01 | Lexer processes tokens at > 500 MB/s | Benchmark suite | High |
| AC-028 | REQ-05.01.02 | Macro expansion handles deeply nested macros (100+ levels) without stack overflow | Fuzzing test | High |
| AC-029 | REQ-06.02.01 | PDF writer uses pre-allocated byte buffers (zero allocation during write) | Valgrind/ASAN | High |
| AC-030 | REQ-06.02.02 | PDF deflate streams compressed in parallel via Rayon | Benchmark + thread profiling | Medium |
