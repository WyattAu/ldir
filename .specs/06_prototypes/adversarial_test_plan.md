# LDIR Adversarial Test Plan

**Version:** 1.0.0
**Status:** DRAFT
**Date:** 2026-04-23
**References:** SEC-STP-001 (Security Test Plan), BM-XXX (Benchmark Suite), REQ-9.1 (Continuous Fuzzing)
**Frameworks:** cargo-fuzz (libFuzzer), proptest, ThreadSanitizer, Valgrind, cargo-tarpaulin

---

## 1. Test Strategy Overview

### 1.1 Objectives

1. Break the compiler before users do — find panics, segfaults, and infinite loops in all untrusted input paths
2. Validate algebraic properties of fp26_6 arithmetic and compilation transforms
3. Confirm deterministic output under concurrent execution across thread configurations
4. Detect resource leaks in native and WASM targets
5. Establish baselines and coverage thresholds for Phase 5 completion criteria

### 1.2 Relationship to Existing Artifacts

| This Plan | References | Overlap |
|-----------|-----------|---------|
| FT-001 | STP-003.1 (rkyv fuzzing) | Shared corpus, this adds round-trip checks |
| FT-002 | STP-001.1 (font fuzzing) | Shared harness, this adds glyph extraction targets |
| FT-004 | STP-004.1 (fp26_6 boundary) | Property-based extends unit boundary analysis |
| CT-001 | BM-CONCURRENCY-001e (determinism) | CI gate vs benchmark measurement |
| RL-001 | STP-005.1 (memory budget) | Valgrind vs RSS measurement |

### 1.3 Toolchain Matrix

| Category | Tool | Version | Rust Channel | Sanitizer |
|----------|------|---------|--------------|-----------|
| Fuzzing | cargo-fuzz (libFuzzer) | 0.12+ | nightly | ASan + UBSan |
| Property-Based | proptest | 1.5+ | stable | — |
| Concurrency | ThreadSanitizer | LLVM 18+ | nightly | TSan |
| Memory | Valgrind (memcheck) | 3.22+ | stable | — |
| Coverage | cargo-tarpaulin | 0.31+ | stable | — |
| WASM | wasmtime | 28+ | stable | — |

---

## 2. Phase 5a: Fuzzing Targets

### FT-001: S-IR Parser Fuzzer

| Attribute | Value |
|-----------|-------|
| Target | `parse_sir(&[u8])` (COMP-IR-PARSER, IF-PARSE-001) |
| Tool | cargo-fuzz |
| Corpus | 50+ seeds: valid S-IR, truncated headers, invalid opcodes, zero-length payloads |
| Dictionary | S-IR opcode bytes (0x00-0x05), sentinel values (0xFFFFFFFF) |
| Duration | 8h continuous (nightly), 5min (CI gate) |
| Sanitizers | ASan, UBSan |
| Max RSS | 512 MB |
| Timeout per input | 10s |
| Pass criteria | No panic, no segfault, no OOM; invalid input returns `Err` |

**Traceability:** STP-003.1 (shared corpus), REQ-9.1 (continuous fuzzing)

### FT-002: Font Parser Fuzzer

| Attribute | Value |
|-----------|-------|
| Target | TTF/OTF table directory + glyf extraction |
| Tool | cargo-fuzz + AFL++ (cross-validate) |
| Corpus | 50+ seed fonts from STP-001.1 + minimal valid TTF |
| Dictionary | OpenType table tags (glyf, head, loca, CFF2, GSUB, GPOS) |
| Duration | 8h continuous (nightly), 5min (CI gate) |
| Sanitizers | ASan, MSan |
| Limits | `max_glyph_count: 65536`, `max_table_count: 100`, `max_memory_bytes: 512MB` |
| Pass criteria | No panic, no segfault, no OOM; malformed fonts return `Err` |

**Traceability:** STP-001.1 (shared harness), TV-FONT-001..007 (test vectors)

### FT-003: Constraint Solver Fuzzer

| Attribute | Value |
|-----------|-------|
| Target | Cassowary dual-simplex solver (REQ-4.3.4.1) |
| Tool | cargo-fuzz |
| Corpus | 20+ seed constraint sets: feasible, infeasible, degenerate, over-constrained |
| Mutation strategy | Struct-aware: mutate coefficients, edit variable bounds, add/remove constraints |
| Duration | 4h continuous (nightly), 5min (CI gate) |
| Sanitizers | ASan |
| Pass criteria | Solver terminates (no infinite loop); no panic; result within fp26_6 range |

### FT-004: fp26_6 Arithmetic Fuzzer

| Attribute | Value |
|-----------|-------|
| Target | fp26_6 add, sub, mul, div, sqrt operations |
| Tool | cargo-fuzz |
| Corpus | Boundary values from STP-004.1 (TV-FP-001..008) + random i32 values |
| Duration | 2h continuous (nightly), 3min (CI gate) |
| Sanitizers | UBSan (signed overflow detection) |
| Pass criteria | No panic, no UBSan violation; results saturated in range; round-trip within ±1/128 |

**Traceability:** STP-004.1 (boundary values), REQ-3.2.7 (±1/128 quantization bound)

### FT-005: Line Breaking Fuzzer

| Attribute | Value |
|-----------|-------|
| Target | Knuth-Plass line-breaking algorithm (REQ-4.3.2.1) |
| Tool | cargo-fuzz |
| Corpus | Paragraphs of varying lengths, mixed scripts, single-char words, empty input |
| Mutation strategy | Mutate glyph widths (including zero-width), target line widths (1..10000fp), stretch/shrink |
| Duration | 4h continuous (nightly), 5min (CI gate) |
| Sanitizers | ASan |
| Pass criteria | Terminates within 1s per input; no panic on degenerate inputs |

---

## 3. Phase 5b: Property-Based Testing

### PBT-001: Compilation Idempotency

| Attribute | Value |
|-----------|-------|
| Property | `SHA256(compile(doc))` is identical across 10 sequential compilations |
| Tool | proptest |
| Generator | Random well-formed S-IR (depth 1..10, 1..100 instructions) |
| Traceability | REQ-9.5, INV-COMP-001, TV-IR-P02 |

### PBT-002: Round-Trip Fidelity

| Attribute | Value |
|-----------|-------|
| Property | `parse_sir(emit_sir(doc)) == doc` and `parse_gir(emit_gir(doc)) == doc` |
| Tool | proptest |
| Traceability | POST-EMIT-001, IF-EMIT-001 |

### PBT-003: Well-Formedness Preservation

| Attribute | Value |
|-----------|-------|
| Property | `wellFormed(doc) ⟹ wellFormed(compile(doc))` — WF-SIR always compiles to WF-GIR |
| Tool | proptest |
| Traceability | REQ-3.3.3, THM-COMPILE-WF-001, DEF-004 → DEF-005, TV-IR-P01 |

### PBT-004: fp26_6 Monotonicity

| Attribute | Value |
|-----------|-------|
| Property | `a ≤ b ⟹ fp_mul(a, c) ≤ fp_mul(b, c)` for all `c ≥ 0` |
| Tool | proptest |
| Traceability | STP-004.1, REQ-3.2.5 |

### PBT-005: Entity Uniqueness Preservation

| Attribute | Value |
|-----------|-------|
| Property | All S-IR entity IDs appear exactly once in compiled G-IR metadata |
| Tool | proptest |
| Traceability | THM-COMPILE-COVERAGE-001, POST-COMP-002 |

---

## 4. Phase 5c: Concurrency Testing

### CT-001: Deterministic Work-Stealing

| Attribute | Value |
|-----------|-------|
| Target | Parallel compilation (REQ-2.7) |
| Tool | Custom harness + SHA256 comparison |
| Thread configs | [1, 2, 4, 8, 16], 10 iterations each |
| Test vectors | TV-IR-001..005, `large.sir`, `war_and_peace.sir` |
| Pass criteria | SHA256 identical across all configs and iterations |
| Traceability | REQ-2.7, REQ-9.2, BM-CONCURRENCY-001e |

### CT-002: Race Condition Detection

| Attribute | Value |
|-----------|-------|
| Target | Lock-free hash map (font cache, REQ-4.2.2), work-stealing scheduler |
| Tool | ThreadSanitizer (`RUSTFLAGS="-Z sanitizer=thread"`) |
| Scenarios | 8 threads × 10000 ops: concurrent cache insert/lookup, concurrent paragraph eval |
| Pass criteria | Zero TSan warnings (no data races, no lock-order inversions) |

### CT-003: Deadlock Detection

| Attribute | Value |
|-----------|-------|
| Target | Arena allocator deallocation, cross-component locks |
| Tool | Timeout-based + `parking_lot` deadlock detector (debug builds) |
| Scenarios | 8 threads concurrent allocation/deallocation with nested resource acquisition |
| Timeout | 30s per test |
| Pass criteria | All tests complete within timeout; no deadlock report |

---

## 5. Phase 5d: Resource Leak Testing

### RL-001: Memory Leak Detection (Valgrind)

| Attribute | Value |
|-----------|-------|
| Target | Full pipeline: parse → validate → compile → emit |
| Tool | `valgrind --leak-check=full --error-exitcode=1 --suppressions=rust.supp` |
| Vectors | TV-IR-001..005, `large.sir`, `war_and_peace.sir` |
| Pass criteria | Zero definitely lost, zero possibly lost |

### RL-002: File Handle Leak Detection

| Attribute | Value |
|-----------|-------|
| Target | Font loading, S-IR mmap, PDF emission |
| Tool | `/proc/self/fd` count comparison pre/post |
| Scenario | Compile 100 documents sequentially, each loading 3 fonts |
| Pass criteria | FD count unchanged between start and end |

### RL-003: WASM Instance Cleanup Verification

| Attribute | Value |
|-----------|-------|
| Target | wasmtime engine instance lifecycle |
| Tool | Custom harness + RSS monitoring |
| Scenario | Load/execute/unload 1000 WASM plugin instances |
| Pass criteria | RSS growth < 10MB over 1000 iterations |

---

## 6. Completion Criteria

| ID | Criterion | Measurement | Threshold | Status |
|----|-----------|-------------|-----------|--------|
| CC-001 | Test vectors pass within tolerance | TV-IR-001..005, TV-IR-A01..A09 | 100% pass | PENDING |
| CC-002 | No critical security vulnerabilities | STRIDE P1 from SEC-STP-001 | 0 open P1 | PENDING |
| CC-003 | Branch coverage — overall | cargo-tarpaulin report | >= 80% | PENDING |
| CC-004 | Branch coverage — critical paths | Parser, Validator, Compiler, fp26_6 | >= 95% | PENDING |
| CC-005 | No race conditions | ThreadSanitizer CI run | 0 warnings | PENDING |
| CC-006 | No deadlocks | Timeout + deadlock detector | 0 deadlocks | PENDING |
| CC-007 | No memory leaks | Valgrind CI run | 0 leaks | PENDING |
| CC-008 | No file handle leaks | FD count comparison | 0 leaked FDs | PENDING |
| CC-009 | No WASM instance leaks | RSS growth test | < 10MB growth | PENDING |
| CC-010 | Performance baselines established | BM-XXX benchmarks recorded | All baselines in CI | PENDING |

---

## 7. Test Summary

| Category | Count | IDs |
|----------|-------|-----|
| Fuzzing Targets (5a) | 5 | FT-001..005 |
| Property-Based Tests (5b) | 5 | PBT-001..005 |
| Concurrency Tests (5c) | 3 | CT-001..003 |
| Resource Leak Tests (5d) | 3 | RL-001..003 |
| Completion Criteria | 10 | CC-001..010 |
| **Total** | **26** | |

---

*End of adversarial_test_plan.md v1.0.0*
