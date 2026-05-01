# Performance Requirements Specification

**Version:** 1.0.0
**Status:** DRAFT
**Date:** 2026-04-23
**Supersedes:** None

---

## 1. Overview

This document defines measurable performance targets for the LDIR typesetting engine. All targets are derived from system requirements (REQ-x.x.x) and domain constraints (TC/NC/MC/HC-xxx) defined in the LDIR requirements specification. Every target includes a measurement method, priority, and regression threshold.

### 1.1 Measurement Infrastructure

All benchmarks use Criterion.rs as the primary measurement framework. Cargo.toml dependencies:

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
criterion-perf-events = "0.3"
iai-callgrind = "0.12"

[profile.bench]
opt-level = 3
lto = true
codegen-units = 1

[[bench]]
name = "bench_parse"
harness = false

[[bench]]
name = "bench_compile"
harness = false

[[bench]]
name = "bench_fixed_point"
harness = false

[[bench]]
name = "bench_layout"
harness = false

[[bench]]
name = "bench_ecs"
harness = false
```

CI profiling tools:

| Tool | Purpose | Invocation |
|------|---------|------------|
| `perf` | Hardware counter profiling | `perf stat -e cache-misses,instructions,cycles` |
| `flamegraph` | Call graph visualization | `perf record -g -- cargo bench` |
| `Valgrind/massif` | Heap profiling | `valgrind --tool=massif ./target/release/ldc` |
| `Chrome tracing` | Interactive frame analysis | Traces exported via `tracing-chrome` |
| `iai-callgrind` | Instruction-level profiling | `cargo bench --bench bench_compile` |

### 1.2 Regression Policy

Per REQ-9.4, a commit that increases any Critical-priority benchmark by more than 2% shall fail CI. High-priority benchmarks trigger a warning at 5% regression. All regressions are tracked against the `main` branch baseline.

---

## 2. Throughput Requirements

### PRF-001: S-IR Parse Throughput

| Field | Value |
|-------|-------|
| **ID** | PRF-001 |
| **Metric** | S-IR deserialization throughput |
| **Target** | > 500 MB/s |
| **Measurement** | Criterion benchmark: deserialize 10MB rkyv S-IR blob, report MB/s |
| **Derived from** | REQ-5.1.3 (TC-007: TeX lexer 500 MB/s), REQ-3.1.5 (mmap O(1) access) |
| **Priority** | Critical |
| **Regression threshold** | 2% |
| **Validation** | `cargo bench --bench bench_parse -- parse_sir_10mb` |

### PRF-002: G-IR Compile Throughput

| Field | Value |
|-------|-------|
| **ID** | PRF-002 |
| **Metric** | S-IR to G-IR compilation throughput |
| **Target** | > 100 pages/s (single-threaded) |
| **Measurement** | Criterion benchmark: compile 100-page S-IR document, report pages/s |
| **Derived from** | REQ-11.1.1 (TC-003: War and Peace < 100ms), REQ-3.3.1 |
| **Priority** | Critical |
| **Regression threshold** | 2% |
| **Validation** | `cargo bench --bench bench_compile -- compile_100_pages` |

---

## 3. Latency Requirements

### PRF-003: fp26_6 Arithmetic Latency

| Field | Value |
|-------|-------|
| **ID** | PRF-003 |
| **Metric** | 26.6 fixed-point multiply latency |
| **Target** | < 5 ns per multiply |
| **Measurement** | Criterion microbenchmark: `i32::wrapping_mul` on fp26_6 values in tight loop |
| **Derived from** | REQ-3.2.5 (26.6 format), REQ-3.2.4 (32-bit signed fixed-point) |
| **Priority** | Critical |
| **Regression threshold** | 2% |
| **Notes** | Also benchmark add (< 1 ns), div (< 10 ns), sqrt (< 20 ns) |
| **Validation** | `cargo bench --bench bench_fixed_point -- fp26_6_mul` |

### PRF-004: Line Break Latency (per paragraph)

| Field | Value |
|-------|-------|
| **ID** | PRF-004 |
| **Metric** | Average Knuth-Plass line-breaking time per paragraph |
| **Target** | < 1 ms (average paragraph, ~80 words) |
| **Measurement** | Criterion benchmark: break 1000 typical paragraphs, report average |
| **Derived from** | REQ-11.1.2 (TC-001: < 1ms paragraph re-layout), REQ-4.3.2.1 (SIMD KP) |
| **Priority** | High |
| **Regression threshold** | 5% |
| **Variants** | Short (< 20 words), long (> 200 words), CJK (no spaces) |
| **Validation** | `cargo bench --bench bench_layout -- line_break_typical` |

### PRF-005: Page Break Latency (per page)

| Field | Value |
|-------|-------|
| **ID** | PRF-005 |
| **Metric** | Average page-breaking time per page |
| **Target** | < 0.5 ms (average page with text + floats) |
| **Measurement** | Criterion benchmark: paginate 1000 pages with mixed content |
| **Derived from** | REQ-11.1.4 (TC-004: 500 pages < 50ms), REQ-4.3.3.1 (DAG pagination) |
| **Priority** | High |
| **Regression threshold** | 5% |
| **Variants** | Text-only, with-floats, orphan/widow avoidance active |
| **Validation** | `cargo bench --bench bench_layout -- page_break_mixed` |

### PRF-006: PDF Emit Latency (per page)

| Field | Value |
|-------|-------|
| **ID** | PRF-006 |
| **Metric** | PDF/A-4 stream generation per page |
| **Target** | < 2 ms per page |
| **Measurement** | Criterion benchmark: emit PDF for 100-page G-IR document |
| **Derived from** | REQ-6.2.2 (zero-allocation PDF write), REQ-6.2.3 (parallel compression) |
| **Priority** | Medium |
| **Regression threshold** | 5% |
| **Validation** | `cargo bench --bench bench_compile -- emit_pdf_100_pages` |

### PRF-007: Cold Start (Library Init)

| Field | Value |
|-------|-------|
| **ID** | PRF-007 |
| **Metric** | Library initialization time (arena allocators, thread pool, font cache) |
| **Target** | < 50 ms |
| **Measurement** | Criterion benchmark: full `LdirEngine::new()` from cold start |
| **Derived from** | REQ-4.1.1 (arena pre-allocation), REQ-4.2.1 (thread pool pinning) |
| **Priority** | Low |
| **Regression threshold** | 10% |
| **Validation** | `cargo bench --bench bench_ecs -- engine_cold_start` |

---

## 4. Memory Requirements

### PRF-008: Peak Memory (100-page document)

| Field | Value |
|-------|-------|
| **ID** | PRF-008 |
| **Metric** | Peak RSS for compiling a 100-page document |
| **Target** | < 200 MB |
| **Measurement** | Valgrind massif: `valgrind --tool=massif ./target/release/ldc input.sir` |
| **Derived from** | REQ-4.1.1 (zero hot-path allocation), MC-001 |
| **Priority** | High |
| **Regression threshold** | 5% |
| **Breakdown targets** | S-IR buffer: < 20 MB, G-IR buffer: < 50 MB, Arena: < 80 MB, Font cache: < 30 MB, Overhead: < 20 MB |
| **Validation** | CI job: `massif_threshold = 200_000_000` |

### PRF-009: Hot Layout Pass Allocation

| Field | Value |
|-------|-------|
| **ID** | PRF-009 |
| **Metric** | Dynamic heap allocations during layout pass |
| **Target** | 0 bytes |
| **Measurement** | Custom allocator wrapper counting allocations during `compile_sir()` |
| **Derived from** | REQ-4.1.1 (MC-001), INV-COMP-002 |
| **Priority** | Critical |
| **Regression threshold** | Any non-zero allocation fails CI |
| **Validation** | `#[global_allocator]` counter in test harness |

---

## 5. Interactive / Real-Time Requirements

### PRF-010: Frame Budget (Interactive Preview)

| Field | Value |
|-------|-------|
| **ID** | PRF-010 |
| **Metric** | End-to-end frame time for interactive preview at 60 FPS |
| **Target** | < 16 ms per frame |
| **Measurement** | Chrome trace: measure from input event to pixel output |
| **Derived from** | REQ-6.1.3 (TC-005: 6.9ms at 144Hz GPU), REQ-1.1.2 (< 5ms incremental) |
| **Priority** | Critical |
| **Regression threshold** | 2% |
| **Budget allocation** | Parse: 2ms, Compile: 4ms, Layout: 6ms, Render: 4ms |
| **Validation** | `tracing-chrome` export + Chrome DevTools Performance tab |

### PRF-011: Incremental Update (single word change)

| Field | Value |
|-------|-------|
| **ID** | PRF-011 |
| **Metric** | Time from single-word edit to updated pixel output in 1000-page doc |
| **Target** | < 5 ms |
| **Measurement** | Criterion benchmark: modify one word in 1000-page S-IR, recompile affected paragraph, re-paginate |
| **Derived from** | REQ-11.1.3 (TC-002), REQ-1.1.2 |
| **Priority** | Critical |
| **Regression threshold** | 2% |
| **Validation** | `cargo bench --bench bench_compile -- incremental_single_word_1000pp` |

---

## 6. Concurrency Requirements

### PRF-012: Work-Stealing Overhead

| Field | Value |
|-------|-------|
| **ID** | PRF-012 |
| **Metric** | Overhead of work-stealing scheduler vs single-threaded baseline |
| **Target** | < 15% overhead at 4 cores, < 25% at 2 cores |
| **Measurement** | Criterion: compare 1-core vs 4-core compilation of same document |
| **Derived from** | REQ-2.5 (work-stealing schedulers), REQ-4.2.1 (pinned thread pool) |
| **Priority** | High |
| **Regression threshold** | 5% |
| **Validation** | `cargo bench --bench bench_ecs -- parallel_speedup_4core` |

---

## 7. Extensibility Requirements

### PRF-013: WASM Plugin Dispatch Overhead

| Field | Value |
|-------|-------|
| **ID** | PRF-013 |
| **Metric** | Latency overhead of dispatching through wasmtime sandbox vs native call |
| **Target** | < 10% overhead per plugin invocation |
| **Measurement** | Comparative benchmark: native Rust closure vs wasmtime::Func call |
| **Derived from** | REQ-7.1 (wasmtime sandbox), REQ-7.3 (fuel limits) |
| **Priority** | Medium |
| **Regression threshold** | 10% |
| **Validation** | `cargo bench --bench bench_ecs -- wasm_dispatch_vs_native` |

---

## 8. Summary Table

| ID | Metric | Target | Priority | REQ Ref |
|----|--------|--------|----------|---------|
| PRF-001 | S-IR parse throughput | > 500 MB/s | Critical | REQ-5.1.3 |
| PRF-002 | G-IR compile throughput | > 100 pages/s | Critical | REQ-11.1.1 |
| PRF-003 | fp26_6 multiply latency | < 5 ns | Critical | REQ-3.2.5 |
| PRF-004 | Line break (per paragraph) | < 1 ms avg | High | REQ-11.1.2 |
| PRF-005 | Page break (per page) | < 0.5 ms avg | High | REQ-11.1.4 |
| PRF-006 | PDF emit (per page) | < 2 ms | Medium | REQ-6.2.2 |
| PRF-007 | Cold start (library init) | < 50 ms | Low | REQ-4.1.1 |
| PRF-008 | Peak memory (100-page) | < 200 MB | High | REQ-4.1.1 |
| PRF-009 | Hot path allocation | 0 bytes | Critical | REQ-4.1.1 |
| PRF-010 | Frame budget (interactive) | 16 ms | Critical | REQ-6.1.3 |
| PRF-011 | Incremental update | < 5 ms | Critical | REQ-11.1.3 |
| PRF-012 | Work-stealing overhead | < 15% @ 4-core | High | REQ-2.5 |
| PRF-013 | WASM plugin overhead | < 10% | Medium | REQ-7.1 |

---

*End of performance_requirements.md v1.0.0*
