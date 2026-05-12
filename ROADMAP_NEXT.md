# LDIR Technical Roadmap

## Current State (v3.16.0, Era S)

| Metric | Value |
|--------|-------|
| Rust crates | 25 |
| Rust LOC | ~72,400 |
| Lean4 proof LOC | ~1,000 |
| Total tests | 1,863 (all passing) |
| Lean4 sorry | 3 (all with complete proof sketches) |
| Clippy warnings | 0 (`-D warnings`) |
| Production unwrap/expect | 0 |
| Unsafe blocks | 25 (all justified FFI: 19 harfbuzz, 4 font loader, 1 font tables, 1 ecs) |
| Input formats | 9 (MD, TeX, Typst, HTML, Adoc, Org, DOCX, SIR2, LDIR) |
| Output formats | 8 (PDF, HTML, EPUB, DOCX, TXT, GIR, SIR2, LDIR) |

---

## Phase T: Foundation Hardening (2-3 weeks)

Eliminate known technical debt and establish performance baselines.

### T-1: Eliminate Lean4 sorry (1-2 weeks) ✅ DONE (2 of 3 eliminated)

**Proven:** `isAcyclicAux_not_found` (Line 491) and `isAcyclicAux_cons_lift_orphan` (Line 527) using Mathlib's `List.find?_eq_none` and `List.find?_cons` lemmas. Key technique: `split` on `Option` + `if`-`Bool` match, case-split on `Nat.eq_zero_or_pos` for induction fuel.

**Remaining:** 1 sorry (`compile_preserves_content`, line 761). Requires proving membership preservation through `List.foldl` — deferred to Phase X-1 (real compiler model).

### T-2: Benchmark regression CI (1 week) ✅ DONE

**Current state:** Criterion benchmarks exist (11 targets) but CI only smoke-tests them. No baseline comparison.

**Deliverables:**
- GitHub Actions job that runs `cargo bench -- --save-baseline main` on main branch
- PR job that runs `cargo bench -- --baseline main` and fails on >5% regression
- Store baseline artifacts in GitHub Actions cache
- Add missing critical benchmarks:
  - HarfBuzz shaping (cache hit vs miss)
  - Knuth-Plass at scale (1000+ paragraphs)
  - Incremental recompilation (single-word change in 1000-page document)
  - Memory allocation profiling (arena vs heap)

### T-3: Lock-free shape cache (1-2 weeks) ✅ DONE

**Current state:** `ThreadSafeShapeCache` uses `dashmap::DashMap` with sharded locking (16 shards). The shaper function runs entirely outside any lock, so threads never block on HarfBuzz shaping during cache misses. Approximate LRU eviction via epoch-based access tracking.

---

## Phase U: Performance Engineering (4-6 weeks)

Achieve SRS1 performance targets through systematic optimization.

### U-1: Arena allocator for compiler hot path (3-4 weeks)

**Current state:** Compiler uses `Vec`, `String`, `HashMap` throughout the compilation pipeline. Every paragraph/line-break allocates on the system heap.

**Approach:**
1. Introduce `bumpalo` arena in `CompileContext` (already a dependency via WASM sandbox).
2. Replace `Vec<GIRCommand>` page buffer with `bumpalo::collections::Vec`.
3. Replace `String` interning with arena-allocated `&str` (the `Interner` already exists).
4. Replace `HashMap` style lookups with `IndexMap` (already used) or arena-allocated flat maps.
5. Profile with `valgrind --tool=massif` and `dhat` before/after.

**Target:** Satisfy REQ-1.1.1 (zero heap allocation in hot path). Measure with `jemalloc` stats.

### U-2: SIMD Knuth-Plass (3-4 weeks) ⏸️ PENDING PROFILING

**Current state:** Penalty functions are scalar (1 div, powi(3)). Benchmarks show linebreak throughput is not the bottleneck — content serialization and PDF output dominate. Deferred until profiling confirms penalty calculation is >10% of compile time on 1000+ page documents.

**Approach when activated:** Batch penalty evaluations in the DP inner loop using AVX2/NEON
`simd_lt`/`simd_gt` for branchless demerit comparison. Scalar fallback for non-x86/ARM targets.

### U-3: Parallel Deflate for PDF (1-2 weeks) ✅ DONE

**Current state:** Content stream compression parallelized via `rayon::par_iter()`. Each page's content is independently compressed, then written sequentially to the PDF buffer. Font subsetting and image compression remain sequential (lower ROI for typical documents).

---

## Phase V: Layout Completeness (6-10 weeks)

Implement the remaining layout algorithms from SRS1.

### V-1: Global pagination with branch-and-bound (4-6 weeks)

**Current state:** Page breaks are greedy/local. No global optimization for widow/orphan avoidance across pages, float placement, or total page count minimization.

**Approach:**
1. Model document as a DAG where nodes are page-break candidates and edges represent feasibility constraints.
2. Implement dynamic programming with branch-and-bound pruning (per REQ-3.3.2).
3. Define "maximum badness" threshold to prune infeasible branches early.
4. Add float placement constraints (figures must appear on the same page as their first reference, or the next page).

**Target:** Satisfy REQ-3.3.1/3.3.2. Benchmark with 1000+ page documents.

### V-2: Cassowary constraint solver for floats (3-4 weeks)

**Current state:** `ldir-core/src/solver/cassowary.rs` exists (40 tests) but is not wired into the compiler.

**Approach:**
1. Define constraint variables for float position (x, y, width, height) and page margins.
2. Add constraints: float must fit within page margins, text must wrap around float, float placement preference (top/bottom/page).
3. Integrate solver into pagination pass: after initial page breaks, solve constraints to position floats.
4. Fallback: if solver is infeasible, place float at top of next page.

**Target:** Satisfy REQ-3.4.1.

---

## Phase W: WASM & Extensibility (4-6 weeks)

Enable browser-based compilation and user-defined plugins.

### W-1: WASM HarfBuzz integration (2-3 weeks)

**Current state:** WASM build compiles but shaping falls back to ASCII stub. No real text shaping in browser.

**Approach:**
1. Evaluate `harfbuzz-wasm` (Emscripten-compiled HarfBuzz for WASM).
2. If viable, add as optional dependency behind `#[cfg(target_arch = "wasm32")]`.
3. Fallback: implement a more complete Unicode shaping path using `ttf_parser` directly (GPOS/GSUB table parsing).
4. Add WASM test suite using `wasm-pack test` or `wasm-bindgen-test` runner.

### W-2: Wasmtime plugin ABI (4-6 weeks)

**Current state:** Plugin system skeleton exists (`ldir-core/src/plugin/`) but Wasmtime is not installed and no sandbox runs.

**Approach:**
1. Install Wasmtime in CI and development environment.
2. Define the host-guest ABI: S-IR pointer + length passed to WASM guest, compiled S-IR returned.
3. Implement fuel injection (REQ-4.1.3): trap after 100,000 instructions.
4. Add zero-copy interface (REQ-4.1.2): pass raw pointers, no string copying.
5. Write 3 test plugins: custom macro expansion, custom paragraph style, custom page header.

**Target:** Satisfy REQ-4.1.1/4.1.2/4.1.3.

---

## Phase X: Quality & Correctness (ongoing)

### X-1: Lean4 real compiler model (4-8 weeks)

The current Lean4 proofs model a trivial `compileReal` stub. The real Rust compiler has ~3700 lines of opcode handling. To prove correctness of the real compiler:

1. Model each opcode handler in Lean4 as a function on `SIRDocument -> GIRDocument`.
2. Prove well-formedness preservation per opcode.
3. Prove semantic content preservation (THM-COMPILE-CORRECTNESS-001) by induction on the instruction list.
4. This is the largest single proof effort and should be broken into weekly milestones.

### X-2: Golden master test suite (2-3 weeks)

Create 100+ reference documents (TeX, MD, Typst) with expected PDF output. Use pixel-diff or structural comparison to detect regressions. Priority:

- Academic paper (multi-column equations, bibliography, figures)
- Book chapter (TOC, headings, footnotes, cross-references)
- Table-heavy document (complex grids, merged cells)
- CJK document (Chinese, Japanese, Korean mixed-script)
- RTL document (Arabic, Hebrew bidirectional text)
- Large document (1000+ pages for performance regression)

### X-3: PDF/A validation in CI (1 week)

Add `veraPDF` to CI pipeline. Validate that all generated PDFs pass PDF/A-4 conformance. Fail the build on conformance violations.

### X-4: Cross-platform CI (1 week)

Add matrix builds for macOS (x86_64 + aarch64) and Windows (x86_64). Add MSRV check job (pin to Rust 1.85).

---

## Phase Y: GPU Rendering (8-12 weeks)

### Y-1: Vello compute shader pipeline

**Current state:** `ldir-vello/` maps G-IR to Vello scenes but does not use GPU compute shaders.

**Approach:**
1. Map G-IR commands to Vello's `Scene` builder API directly.
2. Implement glyph caching on GPU (texture atlas).
3. Implement viewport transform (pan/zoom) entirely on GPU.
4. Target 144Hz pan/zoom (REQ-6.1.2: <6.9ms frame budget).

---

## Effort Summary

| Phase | Duration | Dependencies |
|-------|----------|-------------|
| T: Foundation | 2-3 weeks | None |
| U: Performance | 4-6 weeks | T-2 (benchmarks) |
| V: Layout | 6-10 weeks | U-1 (arena) |
| W: WASM | 4-6 weeks | T-1 (proofs) |
| X: Quality | ongoing | None |
| Y: GPU | 8-12 weeks | V-1 (pagination) |

**Critical path:** T -> U -> V -> Y (~20-31 weeks to GPU rendering)

**Quick wins (first 2 weeks):**
1. T-2: Benchmark regression CI
2. T-1: Lean4 sorry elimination
3. U-3: Parallel Deflate

---

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Lean4 sorry resist all tactics | Medium | Low | Document as known limitation; refactor predicates |
| Arena allocation breaks compiler correctness | Medium | High | Property-based tests before/after; SHA256 determinism checks |
| SIMD intrinsics diverge across platforms | Medium | Medium | Scalar fallback; CI on all platforms |
| Global pagination exponential blowup | Medium | High | B&B pruning with tight badness threshold; fallback to greedy |
| Wasmtime version conflicts | Low | Medium | Pin Wasmtime version; test in CI |
| Vello API instability | High | Low | Abstract behind trait; Vello is already a dependency |
