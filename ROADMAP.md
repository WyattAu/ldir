# LDIR Roadmap

## Current State (v3.14.0, Era O)

LDIR is a low-level document intermediate representation language for deterministic typesetting. The codebase comprises 25 Rust crates, 1 Lean4 formal verification project, and supporting tooling.

### Metrics

| Metric | Value |
|--------|-------|
| Rust source files | 178 |
| Rust LOC | ~71,700 |
| Lean4 proof LOC | ~1,000 |
| Workspace crates | 25 |
| Test count | 1,863 (all passing) |
| Clippy warnings | 0 |
| Format issues | 0 |
| Production unwrap/expect | 0 |
| Lean4 sorry | 3 (down from 6) |
| Unsafe blocks | 25 (all justified FFI) |

### Architecture Summary

```
Input Formats (9)          IR Layers               Output Formats (8)
-----------------          ----------              -----------------
MD, TeX, Typst    -->      S-IR v1/v2      -->     PDF, HTML, EPUB
HTML, Adoc, Org            L-IR (Layout)           DOCX, TXT
DOCX, SIR2, LDIR           G-IR (Graphics)          GIR, SIR2, LDIR
                           SIR2 (Serialized)

Tooling: ldc (compiler), ldir-dis, ldir-as, ldir-diff, ldir-validate,
         ldir-opt (8 passes), ldir-link, ldir-lsp, ldir-vello (GPU), ldir-wasm
```

---

## Phase 1: Remaining Lean4 Proofs (Era P)

**Goal:** Close all 3 remaining sorry in `proof_ir_wellformedness.lean`.

### Remaining Theorems

| Theorem | Difficulty | Strategy |
|---------|-----------|----------|
| `isAcyclic_cons_root` | Medium | Depends on `isAcyclic_cons_orphan`; structural induction on cons |
| `isAcyclic_cons_orphan` | Medium | Nested match alignment with fuel parameter; requires `Nat.succ` reasoning |
| `compile_preserves_content` | Hard | List.mem foldl reasoning; requires auxiliary lemma about `foldl` accumulating content |

### Approach

1. Prove `isAcyclic_cons_orphan` first (unblocks `isAcyclic_cons_root`)
2. Key tactic: `simp` with custom `isAcyclicAux` unfolding lemma
3. For `compile_preserves_content`: extract `foldl_preserves_mem` as standalone lemma, then compose
4. Estimated effort: 2-3 focused sessions

### Success Criteria

- 0 sorry in both proof files
- `lake build` produces zero warnings (not just zero errors)
- Update VERSION.md proof status

---

## Phase 2: Performance Hardening (Era Q)

**Goal:** Establish performance baselines and optimize critical paths.

### 2.1 Benchmarking Infrastructure

- Integrate Criterion.rs benchmarks into CI (regression detection)
- Establish baselines for: parse (MD/TeX), compile S-IR, PDF generate, full pipeline
- Target: <100ms for 10-page document, <5s for 100-page document

### 2.2 Compiler Optimizations

- **S-IR compilation**: Profile hot paths; likely candidates are instruction dispatch and style resolution
- **G-IR emission**: Batch command serialization; reduce allocation in emitter
- **PDF generation**: Stream-based writing for large documents (avoid full buffer)
- **Font subsetting**: Cache subset results across compilations

### 2.3 Memory Optimization

- Arena allocation for S-IR instruction vectors (currently `Vec<SIRInstruction>`)
- String interning for repeated content payloads (BibTeX keys, font names)
- Profile with Valgrind/heaptrack; target <50MB for 100-page document

### 2.4 Incremental Compilation

- Dirty-tracking for L-IR re-layout (currently full re-layout on any change)
- Cache compiled G-IR pages; invalidate only affected pages
- LSP integration: re-compile only changed regions

### Success Criteria

- Criterion benchmarks in CI with regression detection
- 2x speedup on 100-page document compilation
- Memory usage <50MB for typical 100-page document

---

## Phase 3: Format Completeness (Era R)

**Goal:** Achieve production-quality input/output for all supported formats.

### 3.1 Input Parsers

| Parser | Status | Gap |
|--------|--------|-----|
| Markdown | Good | GFM task lists, footnotes, definition lists edge cases |
| TeX | Good | Macro expansion (\newcommand), conditional compilation |
| Typst | Good | Show rules, stateful context, bibliography |
| HTML | Adequate | CSS-based styling extraction, nested table colspan/rowspan |
| Adoc | Adequate | Callouts, include directives, attribute substitution |
| Org | Adequate | Property drawers, babel, export blocks |
| DOCX | Basic | Complex tables, tracked changes, embedded objects |

### 3.2 Output Backends

| Backend | Status | Gap |
|---------|--------|-----|
| PDF | Good | PDF/A-2b/3b conformance testing, embedded font subsetting optimization |
| HTML | Good | Responsive layout, CSS customization, MathJax/KaTeX integration |
| EPUB | Adequate | EPUB3 navigation, media overlays, accessibility audit |
| DOCX | Basic | Complex formatting, headers/footers, page numbering |
| TXT | Good | Table formatting, line width control |

### 3.3 Priority Actions

1. **TeX macro expansion**: Implement `\newcommand`, `\def`, `\let` for real-world TeX documents
2. **DOCX output**: Full OOXML compliance with styled paragraphs, tables, headers/footers
3. **HTML output**: Configurable CSS templates, responsive layout options
4. **EPUB3**: Navigation document, package document, accessibility metadata

### Success Criteria

- All 9 input parsers handle common real-world documents without errors
- All 8 output backends produce spec-compliant output
- Round-trip fidelity >95% for MD, TeX, Typst

---

## Phase 4: Typesetting Quality (Era S)

**Goal:** Achieve publication-quality typesetting output.

### 4.1 Knuth-Plass Improvements

- **Fitness classes**: Implement Knuth's demerit-based fitness classification
- **Loose/tight detection**: Prevent overly loose or tight paragraphs
- **Paragraph shaping**: Handle CJK line-breaking constraints (kinsoku)
- **Optical margin alignment**: Hang punctuation outside the margin

### 4.2 Advanced Layout

- **Float placement algorithm**: Top/bottom/page floats with priority and counter
- **Multi-column layout**: Balanced columns with column spanning
- **Footnote layout**: Bottom-of-page footnotes with continuation rules
- **Cross-references**: Resolved section/equation/figure/table numbers

### 4.3 Mathematical Typesetting

- **Display math**: Numbered equations with cross-references
- **Matrix environments**: pmatrix, bmatrix, cases, aligned
- **Symbol coverage**: AMS symbols, operator sizing, stretchy delimiters
- **Baseline alignment**: Inline math vertical alignment with text

### 4.4 Typography

- **OpenType features**: Ligatures, old-style numerals, small caps
- **Font fallback chains**: Multi-script font selection
- **Letter-spacing and tracking**: Optical spacing adjustments
- **Hyphenation**: Extended pattern dictionaries (German, French, Spanish)

### Success Criteria

- Knuth-Plass produces publication-quality paragraph breaks
- Float placement matches LaTeX behavior for common document classes
- Math rendering matches LaTeX output for standard equations

---

## Phase 5: Ecosystem and Distribution (Era T)

**Goal:** Make LDIR a usable, distributable tool.

### 5.1 CLI Polish

- Progress indicators for long compilations
- Error messages with source location and suggestions
- Configuration file support (`ldir.toml`)
- Shell completion (bash, zsh, fish)

### 5.2 Library API

- Stable public API for all 25 crates
- API documentation with examples for every public function
- SemVer guarantees with deprecation policy
- Feature flags for optional dependencies (WASM, Vello, PDF/A)

### 5.3 crates.io Publication

- Publish all 25 crates with complete metadata
- CI pipeline for version bumping and publishing
- Documentation hosting on docs.rs

### 5.4 Language Server

- Full LSP compliance (hover, goto definition, references, rename)
- Real-time preview via incremental compilation
- Multi-format support in single workspace

### 5.5 WASM Playground

- In-browser MD/TeX/Typst to PDF rendering
- Interactive editor with syntax highlighting
- Shareable document URLs

### Success Criteria

- `cargo install ldc` produces a working compiler
- All crates published to crates.io
- LSP provides real-time feedback in VS Code

---

## Phase 6: Safety Certification Readiness (Era U)

**Goal:** Position LDIR for safety-critical document generation (aerospace, medical, legal).

### 6.1 Formal Verification Expansion

- Lean4 proofs for compiler correctness (S-IR to G-IR)
- Lean4 proofs for layout algorithm properties (termination, no overlap)
- Model checking for state machine properties (using TLA+)
- Proof coverage >80% of critical path code

### 6.2 Determinism Guarantees

- Bit-identical output across platforms (already verified for PDF)
- Reproducible builds with Nix flake
- Version-locked dependency tree

### 6.3 Compliance Artifacts

- ISO 26262 readiness assessment
- DO-178C documentation structure
- IEC 62304 safety classification
- Traceability matrix (requirements to tests to proofs)

### Success Criteria

- All critical path algorithms have Lean4 proofs
- Bit-identical output verified on 3+ platforms
- Compliance documentation ready for external audit

---

## Phase 7: Advanced Features (Era V)

**Goal:** Differentiate LDIR with unique capabilities.

### 7.1 Collaborative Editing

- CRDT-based concurrent editing (prototype exists in ldir-lsp)
- Operational transformation for text operations
- Conflict resolution with user-visible merge UI

### 7.2 Plugin System

- WASM-based plugin API for custom renderers
- Plugin sandboxing with resource limits (fuel, memory)
- Plugin marketplace / registry

### 7.3 Template Engine

- Document templates with variables, conditionals, loops
- Template inheritance and composition
- CLI-based template rendering

### 7.4 Bibliography Integration

- BibTeX/BibLaTeX full parsing
- CSL (Citation Style Language) formatting
- Citation graph with forward/backward references

### Success Criteria

- Collaborative editing works for 5+ concurrent users
- Plugin API supports custom output formats
- Template engine handles real-world document templates

---

## Effort Estimation

| Phase | Duration | Priority |
|-------|----------|----------|
| P: Lean4 Proofs | 1-2 weeks | High |
| Q: Performance | 2-4 weeks | High |
| R: Format Completeness | 4-8 weeks | High |
| S: Typesetting Quality | 4-8 weeks | Medium |
| T: Ecosystem | 4-6 weeks | Medium |
| U: Certification | 8-12 weeks | Low (long-term) |
| V: Advanced Features | 6-10 weeks | Low (long-term) |

## Decision Points

1. **After Phase P**: If proofs are complete, consider publishing a paper on the formal verification approach
2. **After Phase Q**: If performance targets are met, benchmark against Typst and LaTeX
3. **After Phase R**: If format coverage is sufficient, begin crates.io publication
4. **After Phase T**: If ecosystem is stable, begin seeking external contributors

## Non-Goals

- WYSIWYG editor (LDIR is a compiler, not an editor)
- Full TeX compatibility (subset sufficient for common documents)
- Binary format optimization (text-based SIR2 is the interchange format)
- Mobile platform support (focus on desktop/server/WASM)
