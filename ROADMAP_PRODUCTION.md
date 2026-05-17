# LDIR Production Roadmap -- From Current State to v1.0 and Beyond

## Current State (v3.16.0, 2026-05-17)

| Metric | Value |
|--------|-------|
| Rust crates | 26 (+ 1 Lean4 project) |
| Rust LOC | ~72,400 |
| Lean4 proof LOC | ~1,000 |
| Total tests | 1,810 (all passing, 0 failures) |
| Lean4 sorry | 0 (all proofs fully resolved) |
| Clippy warnings | 0 (`-D warnings`) |
| cargo fmt | Clean |
| Production unwrap/expect | 3 (2 rkyv INVARIANT-guarded, 1 len()-guarded in linker) |
| Unsafe blocks | 25 (all justified FFI: 19 harfbuzz, 4 font loader, 1 font tables, 1 ecs) |
| Input formats | 9 (MD, TeX, Typst, HTML, Adoc, Org, DOCX, SIR2, LDIR) |
| Output formats | 8 (PDF, HTML, EPUB, DOCX, TXT, GIR, SIR2, LDIR) |
| MSRV | 1.88 (edition 2024) |
| CI | Ubuntu:PASS, MSRV:PASS, Docs:PASS, Lean4:PASS, WASM:PASS |
| Pre-existing CI issues | macOS/Windows integration tests (font shaping), veraPDF apt |

---

## Phase A: Remaining Quality Debt (1-2 weeks)

Eliminate all known quality issues before crates.io publication.

### A-1: Fix macOS/Windows Integration Tests (3-5 days)

**Problem:** Integration tests in `tests/tests/integration.rs` invoke `ldc` which panics on macOS and Windows when shaping Unicode content without proper font fallback. The `test_unicode` test fails on macOS; 10+ tests fail on Windows.

**Root cause:** `shape_ascii` panics on non-ASCII input. The integration test uses `ldc` which calls the full shaping pipeline. On macOS/Windows CI runners, the system font loading path differs from Linux.

**Fix approach:**
1. Replace `shape_ascii` panic with graceful fallback to monospace stub for non-ASCII
2. Or: gate integration tests that invoke `ldc` behind `#[cfg(feature = "integration-cli")]` and exclude from macOS/Windows CI matrix
3. Or: install `fonts-dejavu-core` equivalent on macOS (`brew install font-dejavu`) and Windows (download from GitHub)

**Acceptance:** All 9 CI matrix jobs pass (3 OS x 3 checks).

### A-2: Structured Error Types for String-Error Functions (2-3 days)

10 production functions return `Result<_, String>` losing structured error information:
- `ldir-pdf/src/font/loader.rs:88` -- `FontFace::from_bytes()`
- `ldir-pdf/src/color.rs:19` -- `IccProfile::from_bytes()`
- `ldir-core/src/font/loader.rs:102,107` -- `load_font()` variants
- `ldir-core/src/verifier/mod.rs:18` -- `check_gir()` returns `Result<(), Vec<String>>`
- `ldir-core/src/compiler/bibtex.rs:13` -- `parse_bib()`
- `ldir-epub/src/builder.rs:37` -- `EpubBuilder::build()`
- `ldir-docx/src/builder.rs:17` -- `DocxBuilder::build()`
- `ldir-ir/src/sir/v2/serialize.rs:67` -- `deserialize_module()`
- `ldir-ir/src/sir/v2/text.rs:8` -- `text_to_module()`

**Fix:** Define proper error enums with `thiserror` for each crate. Maintain backward compatibility via `impl From<SpecificError> for String`.

### A-3: Reduce Dead Code Suppression (2-3 days)

25 files use blanket `#[allow(dead_code)]` at module level, masking potentially dead code. Approach:
1. Remove module-level `#[allow(dead_code)]` from each file
2. Compile and address each warning individually
3. For genuinely unused items: remove or gate behind feature flags
4. For items used only in tests: move into `#[cfg(test)]` modules
5. For items reserved for future use: add individual `#[allow(dead_code)]` with a comment explaining why

**Target:** Reduce blanket suppressions from 25 to fewer than 5.

### A-4: Resolve WASM compile_and_render Stub (1 day)

`ldir-wasm/src/lib.rs:47` has a public function that silently returns empty:
```rust
pub fn compile_and_render(_sir_bytes: &[u8]) -> Vec<u8> { Vec::new() }
```

**Fix:** Either implement the function or mark it with `#[cfg(feature = "unstable")]` and document the limitation.

### A-5: Self-Referential Struct Safety (2-3 days)

`ldir-pdf/src/font/loader.rs:108` uses `std::mem::transmute` to extend a lifetime to `'static` for a self-referential struct (ttf_parser Face + font data). This is a well-known Rust footgun.

**Fix:** Replace with `self_cell` or `ouroboros` crate for a safe self-referential struct pattern.

---

## Phase B: crates.io Publication (2-3 weeks)

Publish all 25 public crates to crates.io.

### B-1: API Stabilization (1 week)

1. Audit all `pub` items for stability -- mark unstable APIs with `#[doc(hidden)]` or `#[cfg(feature = "unstable")]`
2. Ensure all public types implement `Debug`, `Clone` where appropriate
3. Verify all crate-level `//!` documentation is complete (ldir-link and ldir-opt now done)
4. Ensure `cargo doc --workspace --no-deps` produces zero warnings
5. Add `#[doc(alias = "...")]` for common alternative names

### B-2: Publication Dry-Run (2-3 days)

1. `cargo publish --dry-run` for each crate in dependency order
2. Fix any missing metadata (repository, license, description, keywords, categories)
3. Ensure `Cargo.lock` is committed and `Cargo.toml` versions are consistent

### B-3: Publication Order

1. `ldir-ir` (foundation, no native deps)
2. `ldir-test-helpers` (test utility)
3. `ldir-core` (depends on ldir-ir, harfbuzz-sys)
4. `ldir-md`, `ldir-tex`, `ldir-typst` (input parsers, no native deps)
5. `ldir-html-reader`, `ldir-docx-reader` (input readers)
6. `ldir-adoc`, `ldir-org` (input parsers)
7. `ldir-pdf` (output backend, native deps)
8. `ldir-html`, `ldir-txt`, `ldir-epub`, `ldir-docx` (output backends)
9. `ldir-dis`, `ldir-as`, `ldir-diff`, `ldir-validate`, `ldir-opt`, `ldir-link` (tools)
10. `ldir-lsp` (language server, tower dependencies)
11. `ldir-vello` (GPU renderer, wgpu deps)
12. `ldir-wasm` (WASM bridge)
13. `ldc` (CLI compiler, meta-crate)

### B-4: Post-Publication Verification

1. `cargo install ldc` works from crates.io
2. docs.rs builds without warnings for all crates
3. Verify reverse dependency resolution

---

## Phase C: Performance to Production (4-6 weeks)

### C-1: Profile-Guided Optimization (1-2 weeks)

1. Run `perf` and `tracing-chrome` on 100-page and 1000-page documents
2. Identify actual bottlenecks (not assumed) -- measure before optimizing
3. Establish hard performance baselines with Criterion

**Targets:**

| Metric | Baseline (v3.16) | Target |
|--------|-------------------|--------|
| 100-page MD to PDF | ~5s (estimated) | <2s |
| Memory (100-page doc) | Unknown | <50MB |
| Incremental recompile (1-word change) | ~50ms (estimated) | <10ms |
| Shape cache hit rate | Unknown | >90% |
| Startup time (ldc) | Unknown | <100ms |

### C-2: PDF Output Streaming (1-2 weeks)

Currently the entire PDF is built in memory. For large documents:
1. Stream page content to a buffered writer
2. Write cross-reference table and trailer only after all pages
3. Target: constant memory usage regardless of document size

### C-3: Font Subsetting Optimization (1 week)

1. Optimize subset algorithm for large CJK fonts (can be 10-25MB)
2. Lazy glyph loading -- only load glyphs actually used
3. Target: subset a 15MB CJK font to <500KB for typical documents

### C-4: Parallel Page Compilation (2-3 weeks)

1. Use `rayon` for parallel compilation of independent pages
2. Pages are independent after layout -- ideal for parallelism
3. Benchmark scaling from 1 to 8 threads
4. Target: >4x speedup on 8-core machines

### C-5: String Interning (1 week)

Current HashMap-based interning double-allocates. Replace with:
1. `string_interner` crate or custom arena-based interner
2. Use `Arc<str>` or `InternedString` wrapper throughout
3. Target: 30% reduction in string memory

---

## Phase D: CLI Polish and Ecosystem (4-6 weeks)

### D-1: CLI User Experience (2-3 weeks)

1. Progress indicators for long compilations (`indicatif` crate)
2. Error messages with source location, context, and suggestions
3. Configuration file support (`ldir.toml` or `.ldirrc`)
4. Shell completion generation (bash, zsh, fish, powershell) via `clap-complete`
5. Man page generation via `clap_mangen`
6. Color output with `--color` flag

### D-2: Language Server Enhancement (2-3 weeks)

1. Full LSP compliance: hover with type info, go-to-definition, references, rename
2. Real-time preview via incremental compilation
3. Multi-format support in single workspace
4. Diagnostics with quick-fix suggestions

### D-3: VS Code Extension (1-2 weeks)

1. Publish to VS Code Marketplace
2. Compile-on-save with PDF preview panel
3. Syntax highlighting for all input formats
4. LSP integration for diagnostics and completion
5. Theme support

### D-4: Documentation Site (1-2 weeks)

1. Enable GitHub Pages in repo settings
2. Convert docs/*.md to HTML (use mdBook or simple static site)
3. Deploy user guide, getting started, plugins reference alongside cargo doc
4. Add search functionality
5. Add version selector for docs

### D-5: WASM Playground Enhancement (2-3 weeks)

1. In-browser MD/TeX/Typst to HTML rendering
2. Interactive editor with syntax highlighting (CodeMirror 6)
3. Live preview panel
4. Shareable document URLs
5. Embed in documentation site

---

## Phase E: Advanced Typesetting (6-10 weeks)

### E-1: TeX Macro Expansion (3-4 weeks)

1. `\newcommand`, `\def`, `\let` macro definitions
2. Macro argument parsing with proper brace matching
3. Conditional compilation (`\ifx`, `\ifnum`, `\ifdim`)
4. Common real-world macro packages (amsmath subset, graphicx subset)

### E-2: Multi-Column Layout (2-3 weeks)

1. CSS-style column count and gap specification
2. Column spanning (full-width elements)
3. Balanced columns (equal height)
4. Column break control

### E-3: OpenType Feature Support (2-3 weeks)

1. GPOS kerning (already partially implemented via HarfBuzz)
2. GSUB ligatures (standard + discretionary)
3. Old-style numerals, small caps, stylistic sets
4. Feature specification in S-IR style system

### E-4: Extended Hyphenation (1-2 weeks)

1. Additional hyphenation patterns (German, French, Spanish, Portuguese)
2. Hyphenation exception dictionary
3. Configurable hyphenation penalties per language

### E-5: Cross-Reference Completeness (1-2 weeks)

1. Equation numbering and referencing
2. Figure/table numbering
3. Bibliography citation (already partially implemented)
4. Index generation
5. Table of contents with page numbers

---

## Phase F: Safety Certification Readiness (8-12 weeks, long-term)

### F-1: Formal Verification Expansion (4-6 weeks)

1. Lean4 proofs for S-IR to G-IR compiler correctness
2. Lean4 proofs for layout algorithm properties (termination, no overlap, no content loss)
3. Model checking for state machine properties (TLA+ for concurrent systems)
4. Proof coverage >80% of critical path code

### F-2: Determinism Guarantees (2-3 weeks)

1. Bit-identical output verified on 3+ platforms (already verified for PDF on Linux)
2. Reproducible builds with Nix flake
3. Version-locked dependency tree
4. Timestamp and UUID injection for reproducibility

### F-3: Compliance Artifacts (4-6 weeks)

1. ISO 26262 readiness assessment (if applicable to document processing domain)
2. DO-178C documentation structure (for safety-critical document generation)
3. IEC 62304 safety classification
4. Complete traceability matrix (requirements to tests to proofs)

---

## Phase G: Advanced Features (6-10 weeks, future)

### G-1: Collaborative Editing (4-6 weeks)

1. CRDT-based concurrent editing (prototype exists in ldir-lsp CRDT module)
2. Conflict resolution with user-visible merge UI
3. Operational transform fallback for simple cases
4. Presence indicators

### G-2: Format Completeness (2-3 weeks)

1. DOCX output: Full OOXML compliance (currently subset)
2. EPUB3: Navigation document, accessibility metadata, media overlays
3. HTML output: Configurable CSS templates and themes
4. ODT output (OpenDocument Text)

### G-3: Bibliography Engine (2-3 weeks)

1. Full BibTeX/BibLaTeX parser
2. IEEE, APA, Chicago, MLA citation styles
3. Citation key resolution and disambiguation
4. Bibliography database management

### G-4: Plugin System Production (3-4 weeks)

1. WASM-based plugin API (prototype exists via wasmtime)
2. Plugin sandboxing with resource limits (fuel, memory, time)
3. Plugin marketplace/registry
4. Example plugins: custom output format, bibliography style, macro language

---

## Phase H: Ecosystem and Community (ongoing)

### H-1: Community Building

1. CONTRIBUTING.md with clear contribution guidelines
2. Issue templates and PR templates (already has PR template)
3. RFC process for major changes
4. Monthly development updates

### H-2: Education and Adoption

1. Tutorial series (basic to advanced)
2. Example gallery (real-world document types)
3. Comparison page vs LaTeX, Typst, Pandoc
4. Blog posts on formal verification approach

### H-3: Integration Partnerships

1. Pandoc integration (as a Pandoc writer)
2. Jupyter notebook export
3. Hugo/Hexo static site generator integration
4. CI/CD integration (generate PDFs in pipelines)

---

## Effort Summary

| Phase | Duration | Priority | Dependencies |
|-------|----------|----------|-------------|
| A: Quality Debt | 1-2 weeks | Critical | None |
| B: crates.io | 2-3 weeks | High | Phase A |
| C: Performance | 4-6 weeks | High | Phase A |
| D: Ecosystem | 4-6 weeks | Medium | Phase B |
| E: Typesetting | 6-10 weeks | Medium | Phase C |
| F: Certification | 8-12 weeks | Low | Phase C, E |
| G: Advanced | 6-10 weeks | Low | Phase D, E |
| H: Community | Ongoing | Medium | Phase B |

**Critical path to v1.0 (crates.io publication):** Phase A -> Phase B (~3-5 weeks)

**Critical path to production-ready:** Phase A -> Phase B -> Phase C -> Phase D (~11-17 weeks)

**Full roadmap to advanced features:** ~31-49 weeks

---

## Decision Points

1. **After Phase A:** If CI is fully green, begin Phase B (crates.io)
2. **After Phase B:** If published, begin Phase D (ecosystem) and Phase C (performance) in parallel
3. **After Phase C:** If performance targets met, benchmark against Typst and LaTeX publicly
4. **After Phase D:** If CLI is polished, seek external contributors and publish VS Code extension
5. **After Phase F:** If proofs are comprehensive, publish a paper on the formal verification approach

## Non-Goals

- WYSIWYG editor (LDIR is a compiler, not an editor)
- Full TeX compatibility (targeting 80% coverage of common documents)
- Binary format optimization (text-based SIR2 is the interchange format)
- Mobile platform support (focus on desktop/server/WASM)
- Python/JS bindings (Rust FFI only; WASM for browser)

## Known Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| macOS/Windows font shaping | High | Medium | Fix in Phase A-1; gate tests if needed |
| crates.io name conflicts | Medium | Low | Use `ldir-` prefix consistently |
| HarfBuzz API breaking changes | Low | High | Pin version; CI catches breakage |
| Vello/wgpu API instability | Medium | Low | Abstract behind trait; feature-gated |
| Performance targets unachievable | Medium | Medium | Profile first; adjust targets based on data |
| Lean4 proof complexity | Low | Medium | Existing proofs compile; new proofs incremental |
