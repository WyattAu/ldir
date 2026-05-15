# LDIR Production Roadmap

## Current State (v3.16.0)

| Metric | Value |
|--------|-------|
| Rust crates | 26 (+ 1 Lean4 project) |
| Rust LOC | ~72,400 |
| Lean4 proof LOC | ~1,000 |
| Total tests | 1,863 (all passing locally) |
| Lean4 sorry | 0 (all proofs fully resolved) |
| Clippy warnings | 0 (`-D warnings`) |
| Production unwrap/expect | 3 (2 rkyv INVARIANT-guarded, 1 len()-guarded) |
| Unsafe blocks | 25 (all justified FFI: 19 harfbuzz, 4 font loader, 1 font tables, 1 ecs) |
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

## Phase 1: CI/CD Hardening (1-2 weeks)

### 1.1 Resolve Remaining CI Failures

- **Font tests on CI**: Install `fonts-dejavu-core` on Ubuntu runners, or mark font-db/font-loader tests as `#[cfg(feature = "system-fonts")]` and gate them behind a CI feature flag
- **veraPDF**: Use `apt install verapdf` on Ubuntu (available in universe since 22.04), or switch to the official Docker image
- **Lean4 caching**: Cache `~/.elan` and `ldir-lean/.lake` between runs to reduce 60min cold builds to ~10min
- **Feature Gates job**: Add `--exclude ldir-wasm` and system font install before wasm-plugins test

### 1.2 CI Quality Gates

- Make `Rust ubuntu-latest` a required status check (currently no branch protection)
- Add `cargo doc --workspace --no-deps` to CI to catch documentation warnings
- Add `cargo test --workspace --doc` to verify all doctests pass
- Add security audit: `cargo audit` for known vulnerability scanning

### 1.3 Release Pipeline

- The `release.yml` now creates GitHub Releases with `softprops/action-gh-release@v2`
- Add `aarch64-unknown-linux-gnu` target for ARM servers
- Add SHA256 checksum generation and upload
- Consider adding `cargo publish --dry-run` to release pipeline

### Success Criteria

- All CI jobs pass on `main` (except known font/Lean4 issues)
- Dependabot creates PRs for Cargo and Actions updates
- Tag push creates a GitHub Release with binaries for 5 targets

---

## Phase 2: Documentation Accuracy (1 week)

### 2.1 Fix Identified Documentation Issues

**High priority:**
- Update Rust version from 1.85 to 1.87 in `getting-started.md`, `user-guide.md`, `migration-guide.md`
- Fix API reference: replace `serialize_sir`/`deserialize_sir` with actual method-based API (`SIRDocument::to_bytes()`/`from_bytes()`)
- Fix `getting-started.md` code example to use `push_with_payload` instead of `push`
- Update workspace version references from `1.0` to `0.1.0` in `migration-guide.md`
- Remove `ldir-pdf` and `ldir-tex` from WASM-compatible crate lists (depend on native libs)

**Medium priority:**
- Add L-IR layer to architecture diagram in `getting-started.md`
- Move `ROOT_SENTINEL` out of fp266 constants table in API reference
- Document `DEFAULT_LINE_HEIGHT_FACTOR` interpretation (stored as 6, used as 0.6x)
- Update `ROADMAP.md` metrics (Lean4 sorry count, unwrap/expect count)

**Low priority:**
- Fix fp266 `MAX_VALUE` precision in API reference (`~524287.99` to `~524287.9921875`)
- Fix `fractional()` return range documentation for negative values

### 2.2 Specification Debt

From the specs audit, 5 HIGH issues need addressing:
- 7 of 8 test vector TOML files are missing (only `test_vectors_ir.toml` exists)
- `TRACEABILITY_MATRIX.md` does not exist at `.specs/` root
- `YP-LAYOUT-LIR-001.md` exists but is not registered in `yellow_paper_registry.toml`
- S-IR v2 `Dimension` uses `f64` but YP-NUMERICAL-FIXEDPOINT-001 mandates 26.6 fixed-point
- Lean4 proof file path in `blue_paper_registry.toml` is stale (migrated to `/ldir-lean/`)

### Success Criteria

- All documentation matches actual API and version
- No version mismatches across any `.md` file
- Test vector TOML files created for all Yellow Papers

---

## Phase 3: Code Quality Hardening (2-3 weeks)

### 3.1 Remaining Production unwrap/expect

Three justified exceptions remain. Document rationale in ADR:
- `SIRDocument::to_bytes()`: rkyv on plain enum-of-primitives; INVARIANT documented
- `SIRDocument::to_bytes_with_payload()`: Same invariant
- `link_modules()` single-module unwrap: Guarded by `len() == 1` check; SAFETY documented

### 3.2 Font Test Portability

Font-dependent tests (font::db, font::loader, shaping::harfbuzz) fail on CI runners without DejaVu. Options:
- **Option A**: Install fonts on CI (`sudo apt install fonts-dejavu-core`)
- **Option B**: Bundle a test font as a test fixture
- **Option C**: Gate behind `#[cfg(feature = "system-fonts")]`

Recommended: Option B (bundle test font) for deterministic CI.

### 3.3 Lean4 Proof Status

1 sorry remains: `compile_preserves_content` (deferred to Phase X-1 real compiler model).
All 3 blocked proofs from Era N have been resolved or deferred with complete proof strategies.

### 3.4 Unsafe Block Audit

25 unsafe blocks, all justified FFI:
- 19 harfbuzz (text shaping)
- 4 font loader (ttf_parser FFI)
- 1 font tables (lifetime transmute in test helper)
- 1 ECS (shipyard FFI)

All should have `// SAFETY:` comments. Verify with `cargo geiger` or manual audit.

### Success Criteria

- All 3 unwrap/expect exceptions have ADR documentation
- Font tests pass on all CI platforms
- All unsafe blocks have SAFETY comments

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

## Phase 5: crates.io Publication (2-3 weeks)

### 5.1 Prerequisites

- Stable public API for all 25 crates
- All crates pass `cargo publish --dry-run`
- API documentation on docs.rs (docs.yml CI job)
- README.md with badges, installation, quickstart

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
