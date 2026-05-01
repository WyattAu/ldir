# LDIR Domain Analysis

## 1. Primary Domain

**Digital Typesetting & Document Layout Engines.** LDIR is a high-performance typesetting engine that transforms structured document intent into device-independent geometric output. The primary domain encompasses:

- Document structure modeling (paragraphs, headings, lists, floats, math)
- Text layout algorithms (line-breaking, pagination, hyphenation)
- Font technology (OpenType shaping, glyph substitution, kerning, subsetting)
- Output generation (PDF/A-4, GPU rendering, WASM/WebGL)
- Live/incremental re-layout for interactive editing

The system occupies the same problem space as TeX, but targets modern performance requirements (sub-5ms incremental re-layout) and modern output targets (GPU previewers, WASM browsers) that did not exist when TeX was designed in 1978.

## 2. Cross-Domain Intersections

### 2.1 Compiler Engineering

LDIR's architecture mirrors a compiler: S-IR is the source-level IR, G-IR is the machine-level IR, and the layout engine is the "middle-end" optimizer. The pipeline (S-IR → Layout → G-IR) follows standard compiler design: frontends parse input languages into S-IR, the optimizer transforms S-IR into optimal G-IR, and backends consume G-IR to produce output. Zero-copy serialization (rkyv) and mmap-based loading parallel compiler IR persistence.

### 2.2 Formal Verification

Lean4 is used to specify and verify correctness properties of the IR compilation pipeline, layout algorithms, and constraint solver. Key targets include IR well-formedness preservation (well-formed S-IR → well-formed G-IR), algorithm termination (Knuth-Plass, Cassowary), and determinism proofs for the parallel pipeline.

### 2.3 High-Performance Systems

The engine uses Data-Oriented Design (DoD), ECS architecture, SIMD (AVX2/NEON), lock-free data structures, arena allocators, and thread pinning to achieve sub-millisecond layout latency. Cache-line alignment, branchless inner loops, and Structure of Arrays (SoA) layout are pervasive.

### 2.4 Constraint Solving

Floating element positioning (images, sidebars) requires a linear constraint solver. LDIR uses a dual-simplex Cassowary solver with fixed-point arithmetic to maintain cross-platform determinism while solving spatial layout constraints.

## 3. Domain History & Evolution

| Era | System | Innovation | Limitation |
|---|---|---|---|
| 1978 | TeX | Knuth-Plass line-breaking, macro system | Batch-only, single-core, DVI output |
| 1982 | DVI | Device-independent output format | No color, no Unicode, limited font support |
| 1993 | PDF 1.0 | Portable document format with fonts & graphics | Binary, complex spec, not designed for live editing |
| 1993 | pdfTeX | Direct PDF generation from TeX | Still batch-oriented, monolithic |
| 2000s | LuaTeX | Scriptable TeX with embedded Lua | Performance limited by Lua VM, single-threaded |
| 2010s | SILE | Modern typesetting in Lua | Performance not competitive for large documents |
| 2019 | Typst | Rust-based, fast compilation | Proprietary IR, no formal verification, no WASM extensibility |
| 2026 | **LDIR** | IR pipeline, DoD/ECS, Lean4 verification, WASM ABI | New project, requires ecosystem build-out |

**Why LDIR is needed:**
- No existing system combines formal verification with high-performance layout
- No existing system provides a stable, versioned IR for cross-tool interoperability
- No existing system supports sandboxed extensibility (WASM) with deterministic layout
- Typst is fast but closed-ecosystem; TeX is open but architecturally frozen
- Modern use cases (collaborative editing, GPU previewers, browser rendering) demand architectures TeX cannot provide

## 4. Key Domain Concepts

| Concept | Definition | Relevance to LDIR |
|---|---|---|
| **S-IR** (Semantic IR) | Structural representation of document intent — what the document *means* structurally | Input format to the layout engine; 13-byte fixed wire format |
| **G-IR** (Geometric IR) | Device-independent command buffer describing exact glyph placement — what the document *looks like* | Output of the layout engine; consumed by all backends |
| **Knuth-Plass** | Optimal line-breaking algorithm minimizing a penalty function over all feasible breakpoints | Core line-breaking algorithm, vectorized via SIMD |
| **DoD** (Data-Oriented Design) | Memory layout optimization for cache efficiency: SoA over AoS, contiguous allocation | Foundational memory model for the entire engine |
| **ECS** (Entity-Component-System) | Architectural pattern where entities are IDs, components are data, systems are logic | Document node storage and iteration model |
| **SoA** (Structure of Arrays) | Storing homogeneous data fields in separate contiguous arrays | Cache-friendly storage for width, height, font_id, etc. |
| **Cassowary** | Linear constraint solver using the dual-simplex method | Positions floating elements, enforces spatial constraints |
| **26.6 fixed-point** | 32-bit signed integer with 26 whole bits and 6 fractional bits | Cross-platform deterministic coordinate representation |
| **Arena allocator** | Bump-pointer allocator that frees all memory at once | Zero-allocation hot path for layout passes |
| **rkyv** | Zero-copy deserialization library for Rust | S-IR serialization format enabling mmap-based loading |
| **Font shaping** | Process of converting Unicode codepoints to positioned glyphs via OpenType rules | Handled by HarfBuzz; fast-path bypass for ASCII/Latin-1 |
| **Font subsetting** | Extracting only used glyphs from a font file | Required for PDF/A-4 output to minimize file size |
| **DVI** (Device Independent) | TeX's original output format; a stream of positioning commands | G-IR is the spiritual successor ("DVI 2.0") |
| **WASM sandbox** | WebAssembly runtime for isolated, safe execution of user plugins | Hosts all user-defined macros and custom layout logic |
| **Epoch-based reclamation** | Lock-free memory reclamation scheme | Used for concurrent font shaping caches |

## 5. Domain-Specific Risks

### 5.1 Typesetting Correctness

Knuth-Plass line-breaking is mathematically proven to find optimal breakpoints, but global pagination (cross-page widow/orphan elimination) is NP-hard in the general case. The Branch-and-Bound approach must balance optimality vs. compile time for 1,000+ page documents. Incorrect fixed-point arithmetic can accumulate rounding errors across pages, causing text to drift.

### 5.2 Cross-Platform Determinism

IEEE-754 floating-point is non-deterministic across architectures (x86 vs ARM) and compilers (different instruction ordering). LDIR mitigates this with 26.6 fixed-point arithmetic, but all third-party dependencies (HarfBuzz glyph widths, FreeType hinting) must also produce deterministic results. SIMD code paths must be verified to produce identical results on AVX2 and NEON.

### 5.3 Formal Verification Complexity

Proving IR well-formedness preservation in Lean4 requires formalizing the entire S-IR and G-IR instruction sets. Proving algorithm termination for the Cassowary constraint solver requires Bland's rule or similar anti-cycling arguments. The parallel determinism proof requires showing that all thread interleavings converge to the same G-IR, which is non-trivial for work-stealing schedulers.

### 5.4 Font Technology Complexity

OpenType shaping involves contextual substitution (GSUB), positioning (GPOS), and complex script rules for Arabic, Indic, and CJK scripts. Font subsetting must correctly handle glyph dependencies (composite glyphs, CFF charstrings) to produce valid subset fonts.

### 5.5 WASM ABI Stability

The zero-copy ABI between host and WASM guest must remain stable across LDIR versions while allowing the WASM interface to evolve independently. Memory safety within the WASM sandbox depends on correct pointer/length validation by the host.

## 6. Related Systems & Prior Art

| System | Language | Strengths | Weaknesses (relative to LDIR) |
|---|---|---|---|
| **TeX/pdfTeX** | Pascal/Web2C | Gold-standard typesetting quality, 45+ years of testing | Single-threaded, batch-only, no live preview, no GPU rendering |
| **LuaTeX** | C/Lua | Scriptable, Unicode support | Performance limited by Lua VM, complex C codebase |
| **XeTeX** | C | Native Unicode, system font access | No macro scripting, limited to TeX's layout model |
| **Typst** | Rust | Fast compilation, modern syntax | Closed IR, no formal verification, no WASM extensibility |
| **SILE** | Lua | Modern typesetting, extensible | Lua performance ceiling, not designed for massive documents |
| **Paged.js** | JavaScript | CSS-based pagination in browser | Performance limited by JS, browser dependency, no IR |
| **WeasyPrint** | Python | CSS to PDF, HTML/CSS input | Python performance, not designed for live editing |
| **HarfBuzz** | C/C++ | Industry-standard text shaping | Library only; must be integrated into a layout engine |
| **FreeType** | C | Industry-standard font rasterization | Library only; rasterization is LDIR's backend concern |
| **Cassowary** | Various | Mature linear constraint solver | Requires fixed-point adaptation for determinism |

## 7. Domain Standards

| Standard | Scope | Applicability |
|---|---|---|
| ISO 32000-2:2020 (PDF 2.0) | PDF file format specification | Primary output format for archival backend |
| ISO 19005-4:2020 (PDF/A-4) | Long-term archival PDF | Target conformance level for ldir-pdf |
| ISO 14496-22 (OpenType) | Font file format | Font parsing, subsetting, shaping |
| ISO/IEC 10646 (Unicode) | Universal character set | Text encoding, grapheme boundaries |
| ISO 15924 | Script codes | Script identification for shaping dispatch |
| CommonMark 0.31 | Markdown specification | Input format for ldir-md frontend |
| OpenType Feature File Spec | Font feature syntax | KERN, LIGA, CALT feature activation |
| WebAssembly Core Spec 2.0 | WASM binary format & semantics | WASM plugin sandboxing and ABI |
| W3C CSS Paged Media | CSS page layout rules | Reference for pagination behavior |
