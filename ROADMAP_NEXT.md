# LDIR Technical Roadmap

## Current State (v3.16.0, Era S)

| Metric | Value |
|--------|-------|
| Rust crates | 25 |
| Rust LOC | ~72,400 |
| Lean4 proof LOC | ~1,000 |
| Total tests | 1,863 (all passing) |
| Lean4 sorry | 1 (with complete proof sketch, deferred to X-1) |
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

### U-1: Arena allocator for compiler hot path (3-4 weeks) ✅ DONE (partial)

**Delivered:**
- U-1a: CJK arena — migrated `insert_cjk_breaks` from `Vec<LineBreakItem>` to `BumpVec<'bump, LineBreakItem>` backed by `CompileContext.bump`. 29 CJK tests pass.
- U-1b: Knuth-Plass linebreak arena — converted prefix_w/s/h, nodes, active, new_active from heap `Vec` to `bumpalo::collections::Vec` in `knuth_plass::linebreak()`. 12 KP tests pass.

**Remaining:** Full arena migration for remaining compiler hot paths (GIR command buffer, string interning).

### U-2: SIMD Knuth-Plass (3-4 weeks) ⏸️ PENDING PROFILING

**Current state:** Penalty functions are scalar (1 div, powi(3)). Benchmarks show linebreak throughput is not the bottleneck — content serialization and PDF output dominate. Deferred until profiling confirms penalty calculation is >10% of compile time on 1000+ page documents.

**Approach when activated:** Batch penalty evaluations in the DP inner loop using AVX2/NEON
`simd_lt`/`simd_gt` for branchless demerit comparison. Scalar fallback for non-x86/ARM targets.

### U-3: Parallel Deflate for PDF (1-2 weeks) ✅ DONE

**Current state:** Content stream compression parallelized via `rayon::par_iter()`. Each page's content is independently compressed, then written sequentially to the PDF buffer. Font subsetting and image compression remain sequential (lower ROI for typical documents).

---

## Phase V: Layout Completeness (6-10 weeks)

Implement the remaining layout algorithms from SRS1.

### V-1: Global pagination with branch-and-bound (4-6 weeks) ✅ DONE

**Delivered:** `paginate_global()` using O(n^2) dynamic programming over paragraph blocks with prefix sums. Minimizes total demerits (tightness + widow/orphan + page break cost) across entire document. Falls back to greedy if DP infeasible. 19 pagination tests pass (15 greedy + 4 global).

**Remaining:** Integration into LIR compiler pipeline (currently standalone function).

### V-2: Cassowary constraint solver for floats (3-4 weeks) ✅ DONE

**Delivered:** Float placement wired into LIR compiler via Cassowary constraint solver. Each float gets solver variables for (x, y) with REQUIRED margin constraints and STRONG position hints (left-align, bottom preference). Infeasible floats deferred to next page. Solver exposed as `pub mod solver` in lib.rs. 59 tests pass (19 LIR + 40 solver).

---

## Phase W: WASM & Extensibility (4-6 weeks)

Enable browser-based compilation and user-defined plugins.

### W-1: WASM HarfBuzz integration (2-3 weeks) ⏸️ ANALYSIS COMPLETE

**Current state:** WASM builds use `fast_path::shape_unicode_basic` (ttf_parser cmap+hmtx only). No kerning, ligatures, or complex shaping. `harfbuzz-wasm` not yet evaluated.

**Key findings:**
- `harfbuzz-sys` gated behind `cfg(not(wasm32))` — correctly excluded
- `ttf_parser` 0.25 does NOT parse GPOS/GSUB tables
- WASM shaping tests: none exist
- `harfbuzz-wasm` crate: not on crates.io (typically built via Emscripten)

**Plan when activated:**
1. Evaluate `harfbuzz-wasm` via Emscripten or alternative WASM-compatible HarfBuzz builds
2. Fallback: manually parse GPOS kern table + GSUB liga tables via raw font data
3. Add WASM shaping test suite + CI via `wasm-pack test`

### W-2: Wasmtime plugin ABI (4-6 weeks) ✅ DONE

**Delivered:** `wasm_host` module behind `wasm-plugins` feature flag. Host-guest ABI: `plugin_name/version/alloc/execute/output_ptr/free`. Fuel injection (configurable instruction limit, default 100k). WASI preview1 integration via `wasmtime-wasi`. `from_file()` and `from_bytes()` loaders with ABI version validation. 6 tests pass. Default build unaffected (wasmtime is optional dep).
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

| Phase | Duration | Status |
|-------|----------|--------|
| T: Foundation | 2-3 weeks | ✅ DONE |
| U: Performance | 4-6 weeks | ✅ DONE (U-2 deferred) |
| V: Layout | 6-10 weeks | 🟡 V-2 partial, V-1 pending |
| W: WASM | 4-6 weeks | 🔴 Not started (W-1 analyzed) |
| X: Quality | ongoing | 🔴 Not started |
| Y: GPU | 8-12 weeks | 🔴 Not started |

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
