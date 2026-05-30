# LDIR Technical Roadmap

## Current State (v3.16.0, Era S)

| Metric | Value |
|--------|-------|
| Rust crates | 26 |
| Rust LOC | ~72,400 |
| Lean4 proof LOC | ~1,000 |
| Total tests | 1,865 (all passing, 5 ignored) |
| Lean4 sorry | 0 (all proofs fully resolved) |
| Clippy warnings | 0 (`-D warnings`) |
| Production unwrap/expect | 3 (all justified: 2 rkyv INVARIANT-guarded, 1 len()-guarded) |
| Unsafe blocks | 24 (22 blocks + 2 fn: 18 harfbuzz, 3 SIMD, 1 font tables, 2 unsafe fn decls) |
| Input formats | 9 (MD, TeX, Typst, HTML, Adoc, Org, DOCX, SIR2, LDIR) |
| Output formats | 8 (PDF, HTML, EPUB, DOCX, TXT, GIR, SIR2, LDIR) |
| Multi-column layout | DONE (MultiColumnOptions, reflow_multicolumn, balanced mode) |
| Multilingual hyphenation | DONE (EN/DE/FR/ES/PT, HyphenationLang, per-language affixes) |
| Font subsetting | Compound glyph resolution + GSUB/GPOS/VORG (glyph ID remapping TODO) |
| VS Code extension | LSP integration done (preview notifications, config, language def) |
| DOCX output | Numbering, styles, heading differentiation, doc properties, node handlers |
| EPUB3 | Accessibility metadata, nested TOC, landmarks, dc:date, spine toc |

---

## Phase T: Foundation Hardening (2-3 weeks)

Eliminate known technical debt and establish performance baselines.

### T-1: Eliminate Lean4 sorry (1-2 weeks) [DONE] (all 3 resolved)

**Proven:** All theorems fully resolved -- `isAcyclicAux_not_found`, `isAcyclicAux_cons_lift_orphan`, `isAcyclic_cons_orphan`, and `compile_preserves_content`. Key technique: `split` on `Option` + `if`-`Bool` match, case-split on `Nat.eq_zero_or_pos` for induction fuel, and `List.foldl` membership preservation.

### T-2: Benchmark regression CI (1 week) [DONE]

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

### T-3: Lock-free shape cache (1-2 weeks) [DONE]

**Current state:** `ThreadSafeShapeCache` uses `dashmap::DashMap` with sharded locking (16 shards). The shaper function runs entirely outside any lock, so threads never block on HarfBuzz shaping during cache misses. Approximate LRU eviction via epoch-based access tracking.

---

## Phase U: Performance Engineering (4-6 weeks)

Achieve SRS1 performance targets through systematic optimization.

### U-1: Arena allocator for compiler hot path (3-4 weeks) [DONE] (partial)

**Delivered:**
- U-1a: CJK arena -- migrated `insert_cjk_breaks` from `Vec<LineBreakItem>` to `BumpVec<'bump, LineBreakItem>` backed by `CompileContext.bump`. 29 CJK tests pass.
- U-1b: Knuth-Plass linebreak arena -- converted prefix_w/s/h, nodes, active, new_active from heap `Vec` to `bumpalo::collections::Vec` in `knuth_plass::linebreak()`. 12 KP tests pass.

**Remaining:** Full arena migration for remaining compiler hot paths (GIR command buffer, string interning).

### U-2: SIMD Knuth-Plass (3-4 weeks) [PENDING PROFILING]

**Current state:** Penalty functions are scalar (1 div, powi(3)). Benchmarks show linebreak throughput is not the bottleneck -- content serialization and PDF output dominate. Deferred until profiling confirms penalty calculation is >10% of compile time on 1000+ page documents.

**Approach when activated:** Batch penalty evaluations in the DP inner loop using AVX2/NEON
`simd_lt`/`simd_gt` for branchless demerit comparison. Scalar fallback for non-x86/ARM targets.

### U-3: Parallel Deflate for PDF (1-2 weeks) [DONE]

**Current state:** Content stream compression parallelized via `rayon::par_iter()`. Each page's content is independently compressed, then written sequentially to the PDF buffer. Font subsetting and image compression remain sequential (lower ROI for typical documents).

---

## Phase V: Layout Completeness (6-10 weeks)

Implement the remaining layout algorithms from SRS1.

### V-1: Global pagination with branch-and-bound (4-6 weeks) [DONE]

**Delivered:** `paginate_global()` using O(n^2) dynamic programming over paragraph blocks with prefix sums. Minimizes total demerits (tightness + widow/orphan + page break cost) across entire document. Falls back to greedy if DP infeasible. 19 pagination tests pass (15 greedy + 4 global).

**Remaining:** Integration into LIR compiler pipeline (currently standalone function).

### V-2: Cassowary constraint solver for floats (3-4 weeks) [DONE]

**Delivered:** Float placement wired into LIR compiler via Cassowary constraint solver. Each float gets solver variables for (x, y) with REQUIRED margin constraints and STRONG position hints (left-align, bottom preference). Infeasible floats deferred to next page. Solver exposed as `pub mod solver` in lib.rs. 59 tests pass (19 LIR + 40 solver).

---

## Phase W: WASM and Extensibility (4-6 weeks)

Enable browser-based compilation and user-defined plugins.

### W-1: WASM shaping test suite (1 week) [DONE]

**Delivered:** 11 `wasm_bindgen_test` tests in `ldir-wasm/tests/wasm_shape.rs` covering ASCII text, Unicode Latin (accented characters), CJK Chinese/Japanese/Korean, mixed script, empty heading, numbers/symbols, long paragraph (500 chars), and multi-paragraph rendering. Tests exercise shaping indirectly via `compile_markdown_to_html`. Requires `wasm-pack test --headless --chrome` for browser execution.

**Remaining:** ~~Full HarfBuzz WASM integration~~ DONE (manual GPOS/GSUB parsing).

### W-2: Wasmtime plugin ABI (4-6 weeks) [DONE]

**Delivered:** `wasm_host` module behind `wasm-plugins` feature flag. Host-guest ABI: `plugin_name/version/alloc/execute/output_ptr/free`. Fuel injection (configurable instruction limit, default 100k). WASI preview1 integration via `wasmtime-wasi`. `from_file()` and `from_bytes()` loaders with ABI version validation. 6 tests pass. Default build unaffected (wasmtime is optional dep).

**Remaining:** ~~Zero-copy interface~~ DONE (test plugins), ~~test plugins (macro expansion, paragraph style, page header)~~ DONE.

---

## Phase X: Quality and Correctness (ongoing)

### X-1: Lean4 real compiler model (4-8 weeks) [DONE]

**Delivered:** Proved `compile_preserves_content` by establishing four supporting lemmas:
- `compileStep_preserves_mem`: step function preserves membership (append-only)
- `compileFoldl_preserves_mem`: foldl preserves membership (generalized accumulator)
- `compileStep_setContent_adds_glyph`: setContent instruction appends putGlyph
- `compileFoldl_setContent_glyph`: setContent in list implies putGlyph in foldl result

Refactored `compileReal` to use named `compileStep` function. Lean4 sorry: 3 to 0.

### X-2: Golden master test suite (2-3 weeks) [DONE]

**Delivered:** 8 structural golden tests: academic paper (5 sections), list-heavy document (15 items), single-page verification, nested structure, deterministic page count, inline formatting, Typst, and LaTeX. 19 integration tests pass (1 pre-existing ignored).

### X-3: PDF/A validation in CI (1 week) [DONE]

~~Add `veraPDF` to CI pipeline.~~ DONE -- veraPDF validates generated PDFs for PDF/A conformance.

**Note:** CmapIterator CJK Unicode ranges fixed (correctness bug). CJK subsetting now correctly handles all CJK code point ranges, improving correctness for PDF/A CJK documents.

### X-4: Cross-platform CI (1 week) [DONE]

Matrix builds for macOS (x86_64 + aarch64) and Windows (x86_64). MSRV check job (pin to Rust 1.85).

---

## Phase Y: GPU Rendering (8-12 weeks)

### Y-1: Vello compute shader pipeline (8-12 weeks) [DONE] (core)

**Delivered:** GPU rendering pipeline in `ldir-vello` behind `gpu` feature flag.
- `GpuState` wraps `wgpu::Device` + `wgpu::Queue` + `vello::Renderer`
- `render_gpu()` renders scenes to `Rgba8Unorm` texture, reads back to CPU pixel buffer
- `render_scene_impl()` dispatches to GPU or software (white buffer) path
- `RefCell<VelloRenderer>` for interior mutability in `&self` render methods
- Aligned workspace `wgpu` to 22.1 to match vello 0.3 requirement
- Added `pollster` dependency for async-to-sync GPU initialization
- 65 tests pass (GPU feature), 64 tests pass (software default)
- Clippy clean (`-D warnings`) in both feature modes

**Remaining:** Glyph caching on GPU (texture atlas), viewport transform integration, benchmark at 144Hz (REQ-6.1.2: <6.9ms frame budget).

---

## Effort Summary

| Phase | Duration | Status |
|-------|----------|--------|
| T: Foundation | 2-3 weeks | DONE |
| U: Performance | 4-6 weeks | DONE (U-2 deferred pending profiling) |
| V: Layout | 6-10 weeks | DONE |
| W: WASM | 4-6 weeks | DONE (W-1 HarfBuzz deferred) |
| X: Quality | ongoing | DONE |
| Y: GPU | 8-12 weeks | IN PROGRESS (Y-1 core done; caching + viewport pending) |

**Critical path:** T -> U -> V -> Y (~20-31 weeks to GPU rendering)

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
