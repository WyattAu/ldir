# Optimization Roadmap

**Version:** 1.0.0
**Status:** DRAFT
**Date:** 2026-04-23

---

## 1. Strategy Overview

LDIR follows a correctness-first optimization strategy. Each phase builds on verified correctness from the previous phase. No SIMD, `unsafe`, or platform-specific code is introduced until the safe, portable baseline is proven correct and profiled.

### 1.1 Guiding Principles

1. **Profile before optimizing.** No optimization without `perf` or Criterion data proving the bottleneck.
2. **Correctness is non-negotiable.** Every optimization must pass the full test suite including determinism checks (REQ-9.2, REQ-9.5).
3. **Determinism is preserved.** All optimizations must maintain bit-identical G-IR across platforms (REQ-2.6, REQ-11.3.1).
4. **Zero-allocation invariant.** Hot-path allocation count must remain zero throughout all phases (REQ-4.1.1, PRF-009).

---

## 2. Phase A: Correctness-First Baseline

**Goal:** Ship a correct, portable, safe-Rust implementation with no SIMD or unsafe code.

### 2.1 Scope

| Area | Approach | Constraints |
|------|----------|-------------|
| fp26_6 arithmetic | Pure `i32` operations | No SIMD, no unsafe |
| Line breaking | Scalar Knuth-Plass DP | Correct but not vectorized |
| Page breaking | DAG-based branch-and-bound | Sequential |
| Memory | `Vec` with pre-allocation | Acceptable startup cost |
| Concurrency | Sequential only | Determinism verification first |

### 2.2 Expected Performance (Phase A Baseline)

| Metric | Expected | Target | Gap |
|--------|----------|--------|-----|
| Parse throughput | ~200 MB/s | > 500 MB/s | 2.5x |
| Compile throughput | ~30 pages/s | > 100 pages/s | 3.3x |
| fp26_6 multiply | ~3 ns | < 5 ns | Met |
| Line break (typical) | ~3 ms | < 1 ms | 3x |
| Page break (per page) | ~2 ms | < 0.5 ms | 4x |
| Cold start | ~100 ms | < 50 ms | 2x |

### 2.3 Deliverables

- [ ] All 42 benchmarks passing in CI
- [ ] Determinism tests passing: `SHA256(compile(1-thread)) == SHA256(compile(4-thread))`
- [ ] Fuzzing corpus with > 10M executions, zero crashes (REQ-9.1)
- [ ] Baseline flamegraph captured and archived
- [ ] `perf stat` baseline recorded for all benchmarks

### 2.4 Risk: Phase A May Not Meet Targets

**Mitigation:** Phase A is the correctness gate. If Phase A already meets some targets (e.g., fp26_6 multiply), those optimizations are unnecessary. Phase B/C/D are only entered for metrics that miss targets.

---

## 3. Phase B: Hot Path Optimization

**Goal:** Optimize identified hot paths using SIMD and branchless techniques while maintaining safety.

### 3.1 B1: SIMD fp26_6 Arithmetic

**Trigger:** fp26_6 operations account for > 20% of compile time (from Phase A profiling).

| Optimization | Technique | Expected Gain |
|-------------|-----------|---------------|
| Batch multiply | `std::arch::x86_64::_mm256_mul_epi32` (AVX2) | 4-8x throughput |
| Batch add | `std::arch::x86_64::_mm256_add_epi32` | 4-8x throughput |
| CJK text width calc | Vectorized glyph width lookup | 4x for CJK paragraphs |
| Badness calculation | SIMD Knuth-Plass `b = 100*(w-t)^3/s^3` | 8x (8 candidates in parallel, REQ-4.3.2.3) |

**Implementation approach:**
- Use `std::arch` with `#[cfg(target_feature)]` for portable SIMD
- Provide scalar fallback for non-AVX2/non-NEON targets
- Verify determinism: SIMD and scalar paths produce bit-identical results

### 3.2 B2: Branchless Line Breaking

**Trigger:** Line break inner loop has > 30% branch misprediction rate.

| Optimization | Technique | Expected Gain |
|-------------|-----------|---------------|
| Penalty comparison | `cmov` / SIMD select instead of branches | 2x on long paragraphs |
| Badness threshold | Branchless `min(badness, INFINITY)` | Reduce pipeline stalls (REQ-4.3.2.4) |
| Demoted penalty | Bitmask-based penalty classification | Eliminate cascading branches |

**Implementation approach:**
- Profile with `perf stat -e branch-misses` to confirm misprediction rate
- Replace conditional logic with `select`-style operations
- Verify output identical to Phase A (determinism gate)

### 3.3 B3: Font Cache Optimization

**Trigger:** Font glyph width lookup accounts for > 10% of line break time.

| Optimization | Technique | Expected Gain |
|-------------|-----------|---------------|
| Glyph width cache | Pre-computed `HashMap<GlyphId, i32>` | Eliminate repeated HarfBuzz calls |
| ASCII fast-path | Lookup table for Latin-1 glyphs (REQ-4.3.1.2) | Skip HarfBuzz entirely for ASCII |

### 3.4 Deliverables

- [ ] SIMD kernels pass all determinism tests
- [ ] Branchless line break produces identical output to Phase A
- [ ] `perf stat` shows branch-miss reduction > 50%
- [ ] Flamegraph comparison archived (Phase A vs Phase B)

---

## 4. Phase C: Memory Optimization

**Goal:** Eliminate unnecessary allocations, improve cache locality, and reduce peak memory.

### 4.1 C1: Arena Allocation

**Trigger:** Allocation profiler shows > 1M small allocations per compile.

| Optimization | Technique | Expected Gain |
|-------------|-----------|---------------|
| Document arena | Bump allocator for all S-IR/G-IR nodes | Eliminate fragmentation |
| Paragraph scratch | Thread-local arena for line break DP | Zero-alloc hot path (REQ-4.1.1) |
| String interning | Intern all style names, font identifiers | Reduce string duplication |

**Implementation approach:**
- Use `typed-arena` or `bumpalo` crate
- Verify zero-allocation in hot path via custom allocator counter (PRF-009)
- Arena reset between compilations, not per-paragraph

### 4.2 C2: Zero-Copy Pipeline

**Trigger:** Profiling shows > 20% time in memcpy/serialization.

| Optimization | Technique | Expected Gain |
|-------------|-----------|---------------|
| rkyv zero-copy deserialization | Already specified (REQ-2.3, REQ-3.1.5) | Verify in practice |
| G-IR pre-allocated buffers | Pre-size output buffers from S-IR statistics | Eliminate Vec reallocation |
| Font table sharing | Immutable font tables shared via `Arc` | Reduce memory per compilation |

### 4.3 C3: SoA Layout Optimization

**Trigger:** Cache miss rate > 10% on entity iteration.

| Optimization | Technique | Expected Gain |
|-------------|-----------|---------------|
| Separate attribute arrays | Width[], Height[], FontID[] as distinct `Vec<i32>` | L1 cache saturation (REQ-4.1.2) |
| Cache-line alignment | 64-byte alignment on all SoA arrays (MC-003) | Eliminate false sharing |
| Entity ordering | Sort entities by access pattern | Prefetcher friendliness |

### 4.4 Deliverables

- [ ] Peak memory < 200 MB for 100-page document (PRF-008)
- [ ] Hot-path allocation count = 0 (PRF-009)
- [ ] `Valgrind massif` peak snapshot archived
- [ ] Cache miss rate reduced > 30% vs Phase B

---

## 5. Phase D: Parallel Scaling

**Goal:** Achieve near-linear speedup on multi-core systems while preserving determinism.

### 5.1 D1: Work-Stealing Scheduler

**Trigger:** Sequential compilation exceeds latency targets and profile shows parallelizable sections.

| Optimization | Technique | Expected Gain |
|-------------|-----------|---------------|
| Section-level parallelism | Independent sections compiled concurrently | 2-4x at 4 cores |
| Work-stealing deque | `crossbeam-deque` for load balancing | Better utilization |
| CPU affinity | Pin threads to physical cores (REQ-4.2.1) | Reduce cache thrashing |

**Determinism strategy (REQ-2.7):**
- Sections are assigned deterministic IDs
- Compilation order within a section is DFS order (same as single-threaded)
- G-IR pages are assembled in deterministic section-ID order
- No floating-point accumulation across threads

### 5.2 D2: Lock-Free Font Cache

**Trigger:** Font cache contention detected with > 4 threads.

| Optimization | Technique | Expected Gain |
|-------------|-----------|---------------|
| Lock-free hash map | `crossbeam-skiplist` or epoch-based reclamation | Eliminate mutex contention (REQ-4.2.2) |
| Read-copy-update | Font tables are immutable after initial load | Zero-contention reads |

### 5.3 D3: Parallel PDF Compression

**Trigger:** PDF emission bottleneck in FlateDecode compression.

| Optimization | Technique | Expected Gain |
|-------------|-----------|---------------|
| Parallel stream compression | Compress page streams in work-stealing pool | 2-3x at 4 cores (REQ-6.2.3) |
| Sequential assembly | Compressed streams assembled deterministically | Preserve output determinism |

### 5.4 Deliverables

- [ ] 4-core speedup >= 1.5x vs single-threaded (PRF-012)
- [ ] Work-stealing overhead < 15% (PRF-012)
- [ ] Determinism verified: 1/4/16-thread G-IR bit-identical (REQ-2.7)
- [ ] Scaling benchmark graph archived (1, 2, 4, 8, 16 cores)

---

## 6. Profiling Strategy

### 6.1 Toolchain

| Phase | Tool | Command | Output |
|-------|------|---------|--------|
| All | Criterion | `cargo bench` | JSON + HTML reports |
| A | `perf stat` | `perf stat -e cycles,instructions,cache-misses,branch-misses cargo bench` | Counter summary |
| A/B | `perf record` | `perf record -g cargo bench -- compile_100_pages` | Flamegraph SVG |
| C | Valgrind massif | `valgrind --tool=massif --massif-out-file=massif.out ./target/release/ldc large.sir` | Heap profile |
| C | iai-callgrind | `cargo bench --bench bench_compile` | Instruction counts |
| D | `perf trace` | `perf trace -e sched:sched_switch cargo bench` | Thread scheduling |

### 6.2 Hot Path Identification Methodology

1. **Run Criterion baseline** on `main` branch, archive results
2. **Run `perf record -g`** on the slowest benchmark, generate flamegraph
3. **Identify top-3 functions** by inclusive time
4. **Run `perf stat`** on those functions to get cache/branch metrics
5. **Check if bottleneck is**:
   - Cache misses -> Phase C (SoA, alignment)
   - Branch misprediction -> Phase B (branchless)
   - Instruction count -> Phase B (SIMD)
   - Contention -> Phase D (lock-free)
6. **Implement targeted optimization**, re-benchmark
7. **Verify determinism** after each optimization

### 6.3 Regression Detection

Per REQ-9.4, CI enforces:

| Priority | Threshold | Action |
|----------|-----------|--------|
| Critical (PRF-001,002,003,009,010,011) | > 2% regression | CI FAIL |
| High (PRF-004,005,008,012) | > 5% regression | CI WARNING |
| Medium (PRF-006,013) | > 10% regression | CI WARNING |
| Low (PRF-007) | Logged only | No gate |

CI workflow:

```yaml
bench_regression:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
      with:
        fetch-depth: 0
    - run: cargo bench -- --save-baseline main
    - run: cargo bench -- --baseline main 2>&1 | tee bench-results.txt
    - name: Check regressions
      run: |
        if grep -q "regressed" bench-results.txt; then
          echo "::error::Performance regression detected"
          exit 1
        fi
```

### 6.4 Archival Policy

- Flamegraphs archived per phase in `.specs/04_performance/flamegraphs/`
- Criterion HTML reports archived in CI artifacts (30-day retention)
- `perf stat` baselines stored in `.specs/04_performance/baselines/main.perf`

---

## 7. Phase Timeline

| Phase | Duration | Entry Criterion | Exit Criterion |
|-------|----------|----------------|----------------|
| A | 4 weeks | IR types defined | All benchmarks pass, determinism verified |
| B | 3 weeks | Phase A baseline archived | SIMD/branchless paths meet targets, determinism verified |
| C | 2 weeks | Phase B profiling shows memory bottleneck | Peak memory < 200 MB, hot-path alloc = 0 |
| D | 3 weeks | Phase C memory targets met | 4-core speedup >= 1.5x, determinism verified |

**Total:** 12 weeks from Phase A entry to Phase D completion.

---

## 8. Key Performance Risks

| ID | Risk | Likelihood | Impact | Mitigation |
|----|------|-----------|--------|------------|
| RISK-PERF-001 | Knuth-Plass SIMD path not deterministic across x86/AArch64 | Medium | High | Fallback to scalar on arch mismatch; verify with cross-compile CI |
| RISK-PERF-002 | Arena allocation introduces subtle memory bugs | Low | Critical | Comprehensive ASAN/Valgrind testing in CI |
| RISK-PERF-003 | Work-stealing scheduler breaks determinism (REQ-2.7) | Medium | Critical | Determinism gate between every phase; binary G-IR hash comparison |
| RISK-PERF-004 | WASM plugin overhead exceeds 10% target (PRF-013) | Medium | Medium | Batch WASM calls; cache compiled modules; limit plugin count per document |
| RISK-PERF-005 | CJK line breaking significantly slower than Latin | High | Medium | CJK-specific benchmark (BM-LAYOUT-001d); dedicated CJK optimization pass |
| RISK-PERF-006 | Deep nesting (> 100 levels) causes stack overflow | Low | High | Explicit stack in DFS traversal (REQ-5.1.4); benchmark BM-COMPILE-001d |
| RISK-PERF-007 | HarfBuzz shaping becomes bottleneck for complex scripts | Medium | Medium | ASCII fast-path (REQ-4.3.1.2); shaping result cache |

---

*End of optimization_roadmap.md v1.0.0*
