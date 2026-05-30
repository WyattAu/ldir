# LDIR Production Roadmap

## Current State (v3.16.0)

| Metric | Value |
|--------|-------|
| Rust crates | 26 (+ 1 Lean4 project) |
| Rust LOC | ~72,400 |
| Lean4 proof LOC | ~1,000 |
| Total tests | 1,865 (all passing locally) |
| Lean4 sorry | 0 (all proofs fully resolved) |
| Clippy warnings | 0 (`-D warnings`) |
| Production unwrap/expect | 3 (2 rkyv INVARIANT-guarded, 1 len()-guarded) |
| Unsafe blocks | 24 (22 blocks + 2 fn: 18 harfbuzz, 3 SIMD, 1 font tables, 2 unsafe fn decls) |
| Input formats | 9 (MD, TeX, Typst, HTML, Adoc, Org, DOCX, SIR2, LDIR) |
| Output formats | 8 (PDF, HTML, EPUB, DOCX, TXT, GIR, SIR2, LDIR) |
| MSRV | 1.88 (edition 2024) |

---

## Known CI Issues (Pre-Existing, Not Introduced by Audit)

| Issue | Root Cause | Severity | Fix |
|-------|-----------|----------|-----|
| Font tests fail on CI | CI runners lack DejaVu Sans font | Medium | Install fonts or skip font-dependent tests on CI |
| veraPDF download fails | URL pattern changed or network issue | Low | Pin version, add retry logic, or use apt package |
| Lean4 build timeout (30-60min) | Cold elan + lake build + mathlib | Medium | Already increased to 60min; consider caching Lake build artifacts |
| Feature Gates wasm-plugins test fails | Font tests require system fonts | Medium | Same as font test issue above |

---

## Phase 1: CI/CD Hardening (1-2 weeks) -- DONE

### 1.1 Resolve Remaining CI Failures -- DONE

- [x] **aarch64 cross-compilation**: Install `libharfbuzz-dev:arm64` with `dpkg --add-architecture arm64`
- [x] **bench.yml --quick profile**: Replaced invalid `--quick` cargo flag with `CRITERION_MEASUREMENT_TIME` env var
- [x] **Pre-commit hook**: Removed redundant `--exclude ldir-core-fuzz` (already workspace-excluded)
- [x] **CI deterministic builds**: Added `--locked` to all cargo commands across ci.yml, docs.yml, release.yml
- [x] **CI job naming**: Renamed `feature-features` to `feature-gates`
- [x] **Font tests on CI**: `fonts-dejavu-core` installed; `ldir-test-helpers` bundles `DejaVuSans.ttf`; unguarded `face_count_after_load` assertion fixed
- [x] **veraPDF**: `continue-on-error: true` on pdfa-check job (tracked as ROADMAP_NEXT X-3)
- [x] **Lean4 caching**: `~/.elan` and `ldir-lean/.lake` cached between runs

### 1.2 CI Quality Gates -- DONE

- [x] Clippy with `-D warnings` enforced on all platforms
- [x] `cargo fmt --check` enforced on all platforms
- [x] `cargo doc --workspace --no-deps` in CI (docs.yml)
- [x] Dependabot configured for Cargo and Actions
- [x] Benchmark regression detection in CI
- [x] `cargo audit` job added (security-audit, continue-on-error)
- [x] `cargo test --workspace --doc` for doctest verification (added to rust-check matrix job)
- [ ] Make `Rust ubuntu-latest` a required status check (branch protection -- requires GitHub repo settings)

### 1.3 Release Pipeline -- DONE

- [x] `release.yml` creates GitHub Releases with `softprops/action-gh-release@v2`
- [x] `aarch64-unknown-linux-gnu` target for ARM servers
- [x] SHA256 checksum generation and upload
- [ ] Add `cargo publish --dry-run` to release pipeline (deferred -- manual dry-run completed for all crates)

### Success Criteria

- All CI jobs pass on `main` (except known font/Lean4 issues)
- Dependabot creates PRs for Cargo and Actions updates
- Tag push creates a GitHub Release with binaries for 5 targets

---

## Phase 2: Documentation Accuracy (1 week) -- DONE

### 2.1 Fix Identified Documentation Issues -- DONE

All high-priority documentation issues have been resolved:
- [x] Update MSRV to 1.88 in VERSION.md, ROADMAP.md, user-guide.md
- [x] Fix crate version references from 1.0 to 0.1 in getting-started.md, migration-guide.md
- [x] Correct input format count (9) and output format count (8) in README.md
- [x] Add all 26 crates to README.md table
- [x] Update Lean4 sorry count to 0 across VERSION.md, ROADMAP.md, ROADMAP_NEXT.md, ldir-lean/README.md
- [x] Fix WASM section in user-guide.md (remove ldir-pdf from WASM-compatible list)
- [x] Update ldc/README.md to reflect 9 input formats, 8 output formats
- [x] Update CAPABILITY_MATRIX.md (Wasmtime: NOT FOUND -> Available)
- [x] Add landing page (docs/index.html) for GitHub Pages
- [x] Update docs.yml to copy landing page to doc output

### 2.2 Specification Debt

From the specs audit, remaining items:
- [x] Create test vector TOML files for remaining Yellow Papers
- [x] Update TRACEABILITY_MATRIX.md statuses (currently all "Planned")
- [x] Register YP-LAYOUT-LIR-001.md in yellow_paper_registry.toml
- [x] Resolve S-IR v2 Dimension f64 vs 26.6 fixed-point inconsistency

### Success Criteria

- All documentation matches actual API and version
- No version mismatches across any `.md` file
- Test vector TOML files created for all Yellow Papers

---

## Phase 3: Code Quality Hardening (2-3 weeks) -- DONE

### 3.1 Remaining Production unwrap/expect -- DONE

Three justified exceptions documented in ADR-0001:
- `SIRDocument::to_bytes()`: rkyv on plain enum-of-primitives; INVARIANT documented
- `SIRDocument::to_bytes_with_payload()`: Same invariant
- `link_modules()` single-module unwrap: Guarded by `len() == 1` check; SAFETY documented

### 3.2 Font Test Portability -- DONE

Audit complete. 41 font tests use bundled `DejaVuSans.ttf` via `ldir-test_helpers::test_font_data()` -- fully portable. 5 system-font tests in `font/db.rs` are guarded or platform-conditional. Unguarded `face_count_after_load` assertion fixed. MSRV job updated to install `fonts-dejavu-core`.

Recommended: Option B (bundle test font) for deterministic CI.

### 3.3 Lean4 Proof Status -- ALL RESOLVED

All proofs compile with 0 sorry. Verified properties:
- Entity uniqueness (entityUnique)
- Parent reference validity (parentExists)
- Acyclicity with fuel-based termination (isAcyclic)
- Single-root constraint (hasSingleRoot)
- Compiler termination (compile_terminates)
- Content preservation (compile_preserves_content)
- Knuth-Plass termination (kp_termination)
- Cumulative width monotonicity (cumWidth_mono)

### 3.4 Unsafe Block Audit

24 unsafe usages (22 blocks + 2 pub unsafe fn declarations):
- 18 harfbuzz (text shaping)
- 3 SIMD linebreak (vectorized penalty evaluation)
- 1 font tables (lifetime transmute in test helper)
- 2 pub unsafe fn (SIMD linebreak declarations, documented with # Safety)

All 22 unsafe blocks have `// SAFETY:` comments. Both unsafe fn declarations have `# Safety` doc sections. Audit complete.

### Success Criteria

- [x] All 3 unwrap/expect exceptions have ADR documentation (ADR-0001)
- [x] Font tests pass on all CI platforms (bundled fixture + guards)
- [x] All unsafe blocks have SAFETY comments (22/22 blocks, 2/2 fn decls)

---

## Phase 4: Performance to Production (4-6 weeks)

### 4.1 Performance Targets

| Metric | Current | Target | Priority |
|--------|---------|--------|----------|
| 100-page MD to PDF | ~5s (estimated) | <2s | High |
| Memory (100-page doc) | Unknown | <50MB | Medium |
| Incremental recompile (1-word change) | ~50ms (estimated) | <10ms | Medium |
| Shape cache hit rate | Unknown | >90% | Low |
| Startup time (ldc) | Unknown | <100ms | Low |

### 4.2 Optimization Priorities

1. **Profile first**: Use `perf` and `tracing-chrome` to identify actual bottlenecks (not assumed)
2. **PDF output**: Stream-based writing for large documents (currently builds entire PDF in memory)
3. **Font subsetting**: Optimize subset algorithm for large CJK fonts
4. **HarfBuzz shaping**: Cache glyph outlines beyond just advances
5. **Parallel compilation**: Rayon-based parallel page compilation

### 4.3 Memory Optimization

- Arena allocator for S-IR instruction vectors (partial: CJK and KP done)
- String interning (current HashMap-based, double-allocates)
- Profile with heaptrack; target <50MB for 100-page document

### Success Criteria

- Criterion benchmarks in CI with regression detection
- 2x speedup on 100-page document compilation (vs v3.16.0 baseline)
- Memory usage <50MB for typical 100-page document

---

## Phase 5: crates.io Publication (2-3 weeks) -- IN PROGRESS

### 5.1 Prerequisites

- [x] Stable public API for all 25 crates
- [ ] All crates pass `cargo publish --dry-run` (ldir-ir and ldir-test-helpers pass; remaining blocked on ldir-ir not yet published)
- [x] API documentation on docs.rs (docs.yml CI job)
- [x] All crates have proper metadata (repository, license, description, keywords, categories)
- [ ] README.md with badges, installation, quickstart (partial)

### 5.2 Publication Order

1. `ldir-ir` (foundation, no native deps)
2. `ldir-core` (depends on ldir-ir, harfbuzz-sys)
3. `ldir-md`, `ldir-tex`, `ldir-typst` (input parsers)
4. `ldir-pdf` (output backend)
5. `ldir-html`, `ldir-txt`, `ldir-epub`, `ldir-docx` (output backends)
6. `ldir-html-reader`, `ldir-docx-reader` (input readers)
7. `ldir-adoc`, `ldir-org` (input parsers)
8. `ldir-dis`, `ldir-as`, `ldir-diff`, `ldir-validate`, `ldir-opt`, `ldir-link` (tools)
9. `ldir-lsp` (language server)
10. `ldir-wasm` (WASM bridge)
11. `ldir-vello` (GPU renderer)
12. `ldc` (CLI compiler)

### 5.3 Version Strategy

- Start at `0.1.0` for all crates
- Follow SemVer: breaking changes bump major, new features bump minor
- Use workspace dependency management

### Success Criteria

- All 25 crates published to crates.io
- `cargo install ldc` works from crates.io
- docs.rs builds without warnings

---

## Phase 6: Ecosystem Growth (4-8 weeks)

### 6.1 CLI Polish

- Progress indicators for long compilations (`indicatif` crate)
- Error messages with source location and suggestions
- Configuration file support (`ldir.toml`)
- Shell completion (bash, zsh, fish) via `clap-complete`

### 6.2 Language Server

- Full LSP compliance: hover, goto definition, references, rename, completion
- Real-time preview via incremental compilation
- Multi-format support in single workspace
- VS Code extension with compile-on-save and PDF preview

### 6.3 WASM Playground

- In-browser MD/TeX/Typst to PDF rendering
- Interactive editor with syntax highlighting
- Shareable document URLs

### 6.4 Plugin System

- WASM-based plugin API for custom renderers (prototype exists)
- Plugin sandboxing with resource limits (fuel, memory, time)
- Example plugins: custom output format, bibliography style

### Success Criteria

- `ldc` provides a polished CLI experience
- VS Code extension published to marketplace
- WASM playground deployed to GitHub Pages

---

## Phase 7: Safety Certification Readiness (8-12 weeks, long-term)

### 7.1 Formal Verification Expansion

- Lean4 proofs for compiler correctness (S-IR to G-IR)
- Lean4 proofs for layout algorithm properties (termination, no overlap)
- Model checking for state machine properties (using TLA+)
- Proof coverage >80% of critical path code

### 7.2 Determinism Guarantees

- Bit-identical output across platforms (already verified for PDF)
- Reproducible builds with Nix flake
- Version-locked dependency tree

### 7.3 Compliance Artifacts

- ISO 26262 readiness assessment
- DO-178C documentation structure
- IEC 62304 safety classification
- Traceability matrix (requirements to tests to proofs)

### Success Criteria

- All critical path algorithms have Lean4 proofs
- Bit-identical output verified on 3+ platforms
- Compliance documentation ready for external audit

---

## Phase 8: Advanced Features (6-10 weeks, long-term)

### 8.1 Collaborative Editing

- CRDT-based concurrent editing (prototype exists in ldir-lsp)
- Conflict resolution with user-visible merge UI

### 8.2 Advanced Typesetting

- TeX macro expansion (`\newcommand`, `\def`)
- Multi-column layout with column spanning
- OpenType feature support (ligatures, old-style numerals, small caps)
- Extended hyphenation dictionaries (German, French, Spanish)

### 8.3 Format Completeness

- DOCX output: Full OOXML compliance
- EPUB3: Navigation document, accessibility metadata
- HTML output: Configurable CSS templates

### Success Criteria

- Collaborative editing works for 5+ concurrent users
- Plugin API supports custom output formats
- TeX macro expansion handles common real-world documents

---

## Decision Points

1. **After Phase 1**: If CI is green, begin Phase 5 (crates.io publication)
2. **After Phase 2**: If documentation is accurate, publish docs.rs site
3. **After Phase 4**: If performance targets met, benchmark against Typst and LaTeX
4. **After Phase 5**: If crates.io published, begin seeking external contributors
5. **After Phase 7**: If proofs complete, consider publishing a paper on the formal verification approach

## Non-Goals

- WYSIWYG editor (LDIR is a compiler, not an editor)
- Full TeX compatibility (subset sufficient for common documents)
- Binary format optimization (text-based SIR2 is the interchange format)
- Mobile platform support (focus on desktop/server/WASM)

## Effort Summary

| Phase | Duration | Priority | Dependencies |
|-------|----------|----------|-------------|
| 1: CI/CD Hardening | 1-2 weeks | Critical | None |
| 2: Documentation | 1 week | High | Phase 1 (CI must be green) |
| 3: Code Quality | 2-3 weeks | High | Phase 1 |
| 4: Performance | 4-6 weeks | High | Phase 3 |
| 5: crates.io | 2-3 weeks | Medium | Phase 2, 4 |
| 6: Ecosystem | 4-8 weeks | Medium | Phase 5 |
| 7: Certification | 8-12 weeks | Low | Phase 4, 6 |
| 8: Advanced | 6-10 weeks | Low | Phase 6 |

**Critical path:** Phase 1 -> Phase 3 -> Phase 4 -> Phase 5 -> Phase 6 (~13-22 weeks to crates.io publication)
