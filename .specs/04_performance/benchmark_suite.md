# Benchmark Suite Specification

**Version:** 1.0.0
**Status:** DRAFT
**Date:** 2026-04-23

---

## 1. Overview

This document defines the complete benchmark suite for the LDIR typesetting engine. All benchmarks use Criterion.rs and are integrated into CI.

### 1.1 Benchmark File Layout

```
ldir-core/benches/
  bench_parse.rs       # BM-PARSE-001
  bench_compile.rs     # BM-COMPILE-001, BM-PAGINATE-001, BM-EMIT-001
  bench_fixed_point.rs # BM-FIXPT-001
  bench_layout.rs      # BM-LAYOUT-001
  bench_ecs.rs         # BM-ECS-001, BM-CONCURRENCY-001, BM-WASM-001
```

### 1.2 Fixture Data

| Fixture | Size | Description |
|---------|------|-------------|
| `small.sir` | ~10 KB | 1 page, 3 paragraphs, plain text |
| `medium.sir` | ~500 KB | 10 pages, mixed content, 1 float |
| `large.sir` | ~5 MB | 100 pages, mixed content, floats, tables |
| `war_and_peace.sir` | ~3 MB | Plain text novel, ~580K words |
| `cjk_document.sir` | ~1 MB | CJK text, no word spaces |
| `deep_nesting.sir` | ~50 KB | Deeply nested blocks (depth > 100) |

---

## 2. BM-PARSE-001: S-IR Parsing

**PRF:** PRF-001 (> 500 MB/s)

| ID | Name | Input | Measurement |
|----|------|-------|-------------|
| BM-PARSE-001a | `parse_small` | `small.sir` (10 KB) | Throughput (MB/s) |
| BM-PARSE-001b | `parse_medium` | `medium.sir` (500 KB) | Throughput (MB/s) |
| BM-PARSE-001c | `parse_large` | `large.sir` (5 MB) | Throughput (MB/s) |
| BM-PARSE-001d | `parse_mmap_zero_copy` | `large.sir` via mmap | Throughput + zero-alloc check |

```rust
fn bench_parse_sir(c: &mut Criterion) {
    let mut group = c.benchmark_group("BM-PARSE-001");
    let fixtures = [
        ("small", include_bytes!("fixtures/small.sir")),
        ("medium", include_bytes!("fixtures/medium.sir")),
        ("large", include_bytes!("fixtures/large.sir")),
    ];
    for (name, data) in &fixtures {
        group.throughput(Throughput::Bytes(data.len() as u64));
        group.bench_with_input(BenchmarkId::new("parse_sir", name), data, |b, data| {
            b.iter(|| { let doc = ldir_core::parse_sir(data).unwrap(); criterion::black_box(doc); });
        });
    }
    group.finish();
}
```

**Acceptance:** All sizes >= 500 MB/s, zero heap alloc with mmap, < 2% regression.

---

## 3. BM-COMPILE-001: S-IR to G-IR Compilation

**PRF:** PRF-002 (> 100 pages/s), PRF-011 (< 5ms incremental)

| ID | Name | Input | Measurement |
|----|------|-------|-------------|
| BM-COMPILE-001a | `compile_small` | `small.sir` | Latency (us) |
| BM-COMPILE-001b | `compile_100_pages` | `large.sir` | Throughput (pages/s) |
| BM-COMPILE-001c | `compile_war_and_peace` | `war_and_peace.sir` | Total latency (ms) |
| BM-COMPILE-001d | `compile_deep_nesting` | `deep_nesting.sir` | Latency (us) |
| BM-COMPILE-001e | `incremental_single_word` | `large.sir` + 1-word delta | Latency (ms) |

**Acceptance:** >= 100 pages/s, War & Peace < 100ms, incremental < 5ms, SHA256 identical across 10 runs.

---

## 4. BM-FIXPT-001: fp26_6 Arithmetic

**PRF:** PRF-003 (< 5 ns multiply)

| ID | Name | Operation | Target |
|----|------|-----------|--------|
| BM-FIXPT-001a | `fp26_6_add` | `a + b` (i32 wrapping) | < 1 ns |
| BM-FIXPT-001b | `fp26_6_mul` | `((a as i64 * b as i64) >> 6) as i32` | < 5 ns |
| BM-FIXPT-001c | `fp26_6_div` | `((a as i64) << 6) / b as i64` | < 10 ns |
| BM-FIXPT-001d | `fp26_6_sqrt` | Integer sqrt with 6-bit fractional | < 20 ns |
| BM-FIXPT-001e | `fp26_6_to_float` | `a as f32 / 64.0` | < 5 ns |
| BM-FIXPT-001f | `fp26_6_from_float` | `(v * 64.0) as i32` | < 5 ns |

```rust
fn bench_fp26_6_mul(c: &mut Criterion) {
    let vals: Vec<i32> = (0..1000).map(|i| ((i as f32 * 3.14159) * 64.0) as i32).collect();
    c.bench_function("fp26_6_mul", |b| {
        b.iter(|| {
            let mut acc: i32 = 0;
            for i in 0..vals.len() {
                acc = acc.wrapping_add(((vals[i] as i64 * vals[(i+1)%vals.len()] as i64) >> 6) as i32);
            }
            criterion::black_box(acc);
        });
    });
}
```

**Acceptance:** All targets met, no panic on edge values (NC-003, NC-004).

---

## 5. BM-LAYOUT-001: Line Breaking

**PRF:** PRF-004 (< 1 ms per paragraph)

| ID | Name | Input | Measurement |
|----|------|-------|-------------|
| BM-LAYOUT-001a | `line_break_short` | 20-word paragraph | Latency (us) |
| BM-LAYOUT-001b | `line_break_typical` | 80-word paragraph | Latency (us) |
| BM-LAYOUT-001c | `line_break_long` | 200-word paragraph | Latency (us) |
| BM-LAYOUT-001d | `line_break_cjk` | 80-char CJK (no spaces) | Latency (us) |
| BM-LAYOUT-001e | `line_break_mixed_script` | Latin + CJK + Arabic | Latency (us) |
| BM-LAYOUT-001f | `line_break_1000_paragraphs` | 1000 x typical | Throughput (para/s) |

**Acceptance:** Typical < 1ms, CJK < 2ms, 1000 para > 1000/s, badness < threshold (NC-007).

---

## 6. BM-PAGINATE-001: Page Breaking

**PRF:** PRF-005 (< 0.5 ms per page)

| ID | Name | Input | Measurement |
|----|------|-------|-------------|
| BM-PAGINATE-001a | `page_break_text_only` | 100 pages, text only | Latency (ms) |
| BM-PAGINATE-001b | `page_break_with_floats` | 100 pages, text + floats | Latency (ms) |
| BM-PAGINATE-001c | `page_break_orphan_avoidance` | 100 pages, widow/orphan active | Latency (ms) |
| BM-PAGINATE-001d | `page_break_500_pages` | 500-page document | Total latency (ms) |

**Acceptance:** Per-page < 0.5ms, 500-page < 50ms (TC-004), no regression vs text-only.

---

## 7. BM-EMIT-001: PDF Emission

**PRF:** PRF-006 (< 2 ms per page)

| ID | Name | Input | Measurement |
|----|------|-------|-------------|
| BM-EMIT-001a | `emit_pdf_10_pages` | 10-page G-IR | Latency (ms) |
| BM-EMIT-001b | `emit_pdf_100_pages` | 100-page G-IR | Throughput (pages/s) |
| BM-EMIT-001c | `emit_pdf_parallel_compress` | 100-page, 4-core | Latency vs sequential |

**Acceptance:** Per-page < 2ms, > 2x parallel speedup, zero hot-path alloc (REQ-6.2.2).

---

## 8. BM-ECS-001: Entity-Component-System

**PRF:** PRF-007 (< 50 ms cold start)

| ID | Name | Input | Measurement |
|----|------|-------|-------------|
| BM-ECS-001a | `engine_cold_start` | Fresh process | Latency (ms) |
| BM-ECS-001b | `entity_create_10k` | 10,000 entities | Latency (us) |
| BM-ECS-001c | `entity_create_1m` | 1,000,000 entities | Latency (ms) |
| BM-ECS-001d | `component_query_soa` | Query 1M entities | Throughput (entities/us) |
| BM-ECS-001e | `archetype_iteration` | Iterate archetype | Throughput (entities/us) |

**Acceptance:** Cold start < 50ms, create < 10us/1K, SOA query > 100M/s, L1 miss < 5%.

---

## 9. BM-CONCURRENCY-001: Work-Stealing

**PRF:** PRF-012 (< 15% overhead @ 4-core)

| ID | Name | Input | Measurement |
|----|------|-------|-------------|
| BM-CONCURRENCY-001a | `compile_1core` | `large.sir`, 1 thread | Latency (ms) |
| BM-CONCURRENCY-001b | `compile_2core` | `large.sir`, 2 threads | Latency (ms) |
| BM-CONCURRENCY-001c | `compile_4core` | `large.sir`, 4 threads | Latency (ms) |
| BM-CONCURRENCY-001d | `compile_16core` | `large.sir`, 16 threads | Latency (ms) |
| BM-CONCURRENCY-001e | `determinism_multi_core` | 1/4/16 threads | G-IR SHA256 equality |

**Acceptance:** 4-core overhead < 15%, 16-core < 30%, bit-identical G-IR (REQ-2.7), > 1.5x at 4-core.

---

## 10. BM-WASM-001: Plugin Dispatch

**PRF:** PRF-013 (< 10% overhead)

| ID | Name | Input | Measurement |
|----|------|-------|-------------|
| BM-WASM-001a | `native_dispatch` | Rust closure, 10K calls | Latency (ns/call) |
| BM-WASM-001b | `wasm_dispatch` | wasmtime::Func, 10K calls | Latency (ns/call) |
| BM-WASM-001c | `wasm_dispatch_with_fuel` | wasmtime + fuel, 10K calls | Latency (ns/call) |
| BM-WASM-001d | `wasm_sir_intercept` | Full S-IR intercept pass | Latency (ms) |

**Acceptance:** < 10% vs native, fuel overhead < 20%, trap at 100K instructions (NC-010).

---

## 11. Summary

| Category | ID | Count | Critical PRFs |
|----------|----|-------|---------------|
| Parsing | BM-PARSE-001 | 4 | PRF-001 |
| Compilation | BM-COMPILE-001 | 5 | PRF-002, PRF-011 |
| Fixed-Point | BM-FIXPT-001 | 6 | PRF-003 |
| Line Breaking | BM-LAYOUT-001 | 6 | PRF-004 |
| Page Breaking | BM-PAGINATE-001 | 4 | PRF-005 |
| PDF Emission | BM-EMIT-001 | 3 | PRF-006 |
| ECS | BM-ECS-001 | 5 | PRF-007 |
| Concurrency | BM-CONCURRENCY-001 | 5 | PRF-012 |
| WASM | BM-WASM-001 | 4 | PRF-013 |
| **Total** | | **42** | |

---

*End of benchmark_suite.md v1.0.0*
