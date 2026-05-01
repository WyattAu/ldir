# LDIR Unified System Requirements Specification

**Version:** 1.0.0
**Status:** APPROVED
**Date:** 2026-04-23
**Conflicts Resolved:** See ADR-001 through ADR-010

---

## 1. Project Vision & Scope

### 1.1 Vision

REQ-1.1.1 The system shall decouple content intent from physical layout through a tiered Intermediate Representation (IR) system for high-performance digital typesetting.

REQ-1.1.2 The system shall provide sub-5ms incremental re-layouts for documents spanning thousands of pages.

REQ-1.1.3 The system shall target all major output platforms: PDF/A-4, native GPU rendering, WASM/WebGL, and C-ABI embeddable library.

### 1.2 Scope

REQ-1.2.1 The system shall include a TeX frontend (`ldir-tex`) supporting `amsmath`, `geometry`, and basic `tabular` as MVP scope.

REQ-1.2.2 The system shall include a CommonMark-compliant Markdown frontend (`ldir-md`) as MVP scope.

REQ-1.2.3 The system shall not include mathematical formula rendering in the MVP. The `INSERT_MATH` opcode shall exist in the IR but math layout is deferred to post-MVP.

REQ-1.2.4 The system should render 1,000 classic TeX documents to produce visually identical results to `pdftex` as a long-term goal beyond MVP.

---

## 2. Architectural Principles

REQ-2.1 The system shall use an Entity-Component-System (ECS) architecture where document boxes are Entities and their properties are Components stored in Structure of Arrays (SoA) format.

REQ-2.2 The system shall maintain a zero-copy pipeline where data stays in its binary serialized form from the frontend through the engine until the final backend rendering.

REQ-2.3 The system shall use `rkyv` as the primary serialization format for S-IR to enable zero-copy, Rust-native, mmap-friendly access.

REQ-2.4 When interfacing with non-Rust languages at C API boundaries, the system should use FlatBuffers for cross-language interoperability.

REQ-2.5 The system shall assume a multi-core environment, using work-stealing schedulers for independent sections and SIMD for tight inner loops.

REQ-2.6 The system shall produce bit-identical G-IR output for identical S-IR input regardless of the host OS or CPU architecture.

REQ-2.7 The system shall produce bit-identical G-IR output regardless of thread count (1, 4, or 16 cores).

REQ-2.8 Determinism shall apply to G-IR (the command buffer before rasterization) and shall not apply to rasterized pixels, since GPU floating-point is non-deterministic across vendors.

---

## 3. IR Specification

### 3.1 Semantic-IR (S-IR) — "The Intent"

REQ-3.1.1 The system shall represent documents in S-IR as a structural description of content intent using a fixed instruction set.

REQ-3.1.2 Each S-IR instruction shall use a 13-byte fixed-cost wire-format header:

| Field | Size | Description |
|---|---|---|
| `OpCode` | 1 byte | Operation identifier (mapped to a 1-byte enum) |
| `EntityID` | 4 bytes | Unique entity identifier (32-bit generation index) |
| `ParentID` | 4 bytes | Parent entity reference (32-bit generation index) |
| `PayloadOffset` | 4 bytes | Offset into variable-length payload region |

REQ-3.1.3 The S-IR opcode enum shall include at minimum the following instructions:
- `PUSH_BLOCK(type: Paragraph | Heading | List)`
- `SET_CONTENT(blob_ref)`
- `APPLY_STYLE(style_id)`
- `INSERT_MATH(mathml_ref | tex_ref)`
- `LINK_DATA(ptr_address)`

REQ-3.1.4 Variable-length payloads referenced by `PayloadOffset` shall contain inline data (text blobs, style parameters, math expressions) stored contiguously after the instruction header region.

REQ-3.1.5 The system shall be able to `mmap` a 1GB S-IR file and begin layout in O(1) time via `rkyv` zero-copy deserialization.

REQ-3.1.6 For all S-IR entities, the system shall use 32-bit generation indices as identifiers with a maximum capacity of 2^32 nodes per document.

### 3.2 Geometric-IR (G-IR) — "The Result"

REQ-3.2.1 The system shall compile G-IR into a flat command buffer per page, optimized for direct GPU upload or PDF stream generation.

REQ-3.2.2 G-IR commands shall be aligned to 16-byte boundaries to allow vectorized iteration by backend rendering pipelines.

REQ-3.2.3 The G-IR opcode set shall include at minimum the following instructions:
- `SET_FONT(id, size)`
- `MOVE_XY(h, v)`
- `PUT_GLYPH(unicode_id, advance_x)`
- `DRAW_RULE(width, height)`
- `PUSH_STACK` / `POP_STACK` (nested coordinate systems)
- `ATTACH_METADATA(key, val)` (tagging/accessibility)

REQ-3.2.4 All geometric calculations in G-IR shall use 32-bit signed fixed-point integers to guarantee cross-platform determinism and avoid IEEE-754 floating-point drift.

REQ-3.2.5 The system shall use 26.6 fixed-point format (26 bits for whole numbers, 6 bits for fractional parts) for all G-IR coordinates, matching FreeType's internal format.

REQ-3.2.6 The 26.6 fixed-point representation shall encode a value `v` as the integer `v * 64`, where the minimum representable value is -33554432.0 and the maximum is 33554431.984375.

REQ-3.2.7 Quantization from floating-point to 26.6 fixed-point shall be defined as the formal rounding operation `round(v * 64)` with a maximum error bound of ±1/128 (≈ 0.0078125) per coordinate.

### 3.3 IR Compilation Pipeline

REQ-3.3.1 The system shall compile documents through the pipeline: S-IR → Layout Engine → G-IR.

REQ-3.3.2 When S-IR is compiled, the system shall produce G-IR that is a faithful geometric realization of all semantic intent expressed in the S-IR.

REQ-3.3.3 The compilation contract shall guarantee that well-formed S-IR always compiles to well-formed G-IR.

REQ-3.3.4 When compilation encounters malformed S-IR, the system shall emit a structured error diagnostic with the offending EntityID and byte offset without panicking or segfaulting.

---

## 4. Core Engine (libldir)

### 4.1 Memory Model & Performance

REQ-4.1.1 The layout engine shall perform zero dynamic heap allocations during the hot layout pass. All memory shall be pre-allocated in typed arena (bump) allocators during system initialization.

REQ-4.1.2 For all document node attributes, the system shall use Structure of Arrays (SoA) layout to guarantee L1/L2 cache saturation.

REQ-4.1.3 Document attribute arrays (width, height, font_id, glyph_id) shall reside in contiguous, separate memory arrays aligned to 64-byte cache-line boundaries.

REQ-4.1.4 The system shall not use raw pointers or Rust `Box`/`Rc`/`Arc` for document nodes. All relations shall be mapped via 32-bit generation indices.

### 4.2 Concurrency Model

REQ-4.2.1 The engine shall utilize a custom thread pool pinned to physical CPU cores via CPU affinity masks to prevent OS scheduler thrashing and cache invalidation.

REQ-4.2.2 Font shaping caches and calculated glyph widths shall be stored in a highly concurrent, lock-free hash map using epoch-based reclamation to prevent thread contention during parallel paragraph evaluation.

REQ-4.2.3 When executing in parallel, the system shall use work-stealing schedulers for independent layout sections.

### 4.3 Layout Optimizer

#### 4.3.1 Text Shaping

REQ-4.3.1.1 The engine shall implement an AVX2/NEON optimized fast-path for standard ASCII/Latin-1 text.

REQ-4.3.1.2 If an S-IR block contains no complex grapheme clusters or kerning pairs, the engine shall bypass the shaping engine (HarfBuzz) and calculate widths via a vectorized lookup table.

#### 4.3.2 SIMD Line Breaking (Knuth-Plass)

REQ-4.3.2.1 The engine shall implement the Knuth-Plass line-breaking algorithm using SIMD to calculate multiple line-break penalties in parallel.

REQ-4.3.2.2 The Knuth-Plass "badness" calculation shall use the formula:

```
b = 100 × (w - t)³ / s³
```

where `w` = actual line width, `t` = target line width, `s` = stretchability.

REQ-4.3.2.3 The engine shall evaluate 8 potential line-break candidates simultaneously using 256-bit SIMD registers for the badness penalty calculation.

REQ-4.3.2.4 The inner loop of the dynamic programming solver shall be entirely branchless to prevent CPU pipeline stalls.

#### 4.3.3 Global Pagination

REQ-4.3.3.1 The engine shall model document page-breaks as a Directed Acyclic Graph (DAG).

REQ-4.3.3.2 The engine shall use a Branch-and-Bound algorithm to prune layout variations that exceed a predefined maximum badness threshold, preventing combinatorial explosion when paginating 1,000+ page documents.

REQ-4.3.3.3 The engine shall perform global pagination optimization across the entire document to eliminate widows and orphans across 100+ pages simultaneously.

#### 4.3.4 Constraint Solver

REQ-4.3.4.1 For floating elements (images, sidebars), the engine shall implement a dual-simplex Cassowary constraint solver.

REQ-4.3.4.2 The Cassowary solver shall use fixed-point arithmetic internally to maintain determinism across platforms and thread configurations.

REQ-4.3.4.3 Matrices for the constraint solver shall be stored in SoA format and solved using block-matrix SIMD operations.

### 4.4 Formal Verification (Lean4)

REQ-4.4.1 The system shall use Lean4 as the authoritative formal specification language, with Rust as the implementation.

REQ-4.4.2 Proofs shall live alongside code as independent correctness arguments in a specification-only approach.

REQ-4.4.3 The foundational Lean4 verification target shall be IR well-formedness: proving that well-formed S-IR always compiles to well-formed G-IR.

REQ-4.4.4 The verification effort should include algorithm termination proofs for the Knuth-Plass line-breaking solver and the Cassowary constraint solver.

REQ-4.4.5 The verification effort should include a determinism proof strategy showing that the parallel compilation pipeline preserves bit-identical G-IR output.

---

## 5. Frontends

### 5.1 TeX Frontend (ldir-tex)

REQ-5.1.1 The TeX frontend shall implement a pure-Rust TeX macro expander that handles `\def`, `\newcommand`, and `\expandafter`.

REQ-5.1.2 The TeX frontend shall lower expanded tokens directly into LDIR S-IR.

REQ-5.1.3 The lexer shall read `.tex` files into a lock-free ring buffer, processing tokens at > 500 MB/s.

REQ-5.1.4 The macro expander shall track state without recursion to prevent stack overflows on deeply nested macros (e.g., TikZ), using an explicit arena-allocated stack state machine.

REQ-5.1.5 The MVP scope shall support `amsmath`, `geometry`, and basic `tabular` environments.

### 5.2 Markdown Frontend (ldir-md)

REQ-5.2.1 The Markdown frontend shall adhere to the CommonMark specification.

REQ-5.2.2 The Markdown frontend shall map AST nodes (Header, List, Link, Code Block) directly to S-IR Block instructions.

### 5.3 LSP Source Mapping

REQ-5.3.1 For all tokens parsed by any frontend, the system shall generate a mapping from the token's byte-offset in the source file to its corresponding EntityID in the S-IR.

REQ-5.3.2 The G-IR shall retain EntityIDs from the S-IR to enable reverse mapping.

REQ-5.3.3 When a user hovers over a pixel coordinate in the rendered output, the system shall resolve back to the exact source text file and line number in < 2ms.

---

## 6. Backends

### 6.1 GPU/Native Backend (ldir-vello)

REQ-6.1.1 The GPU backend shall map G-IR directly into GPU compute shaders via WGPU/Vello.

REQ-6.1.2 The CPU shall only calculate bounding boxes. Path rasterization and anti-aliasing (MSAA) shall execute entirely on the GPU.

REQ-6.1.3 The viewer shall render pan/zoom updates at 144Hz, maintaining a frame budget of < 6.9ms.

### 6.2 PDF/A-4 Backend (ldir-pdf)

REQ-6.2.1 The PDF backend shall produce high-fidelity, tagged PDF/A-4 output with font subsetting.

REQ-6.2.2 The PDF writer shall construct PDF object dictionaries using pre-allocated byte buffers, performing zero dynamic heap allocations during generation.

REQ-6.2.3 PDF object streams (FlateDecode) shall be compressed in parallel using the work-stealing thread pool before being written to the output file sequentially.

REQ-6.2.4 The backend shall parse TrueType/OpenType font tables and subset fonts exactly to the glyphs used in the G-IR, running concurrently with PDF dictionary generation.

### 6.3 WASM/WebGL Backend

REQ-6.3.1 The WASM backend shall implement a thin-client renderer that interprets G-IR instructions onto an HTML Canvas using GPU shaders.

### 6.4 Native Embeddable (C ABI)

REQ-6.4.1 The engine shall compile to a C-compatible dynamic library (DLL/so) for host application embedding.

REQ-6.4.2 The C ABI shall allow host applications to push new S-IR data to the engine.

REQ-6.4.3 The C ABI shall allow host applications to query G-IR for bounding boxes (for mouse interaction).

REQ-6.4.4 The C ABI shall allow host applications to render a specific page to a provided memory buffer or texture.

---

## 7. WASM Extensibility ABI

REQ-7.1 All user-defined macros and custom layout logic shall execute within a `wasmtime` sandbox. No native C/Rust plugins shall be permitted for security and reproducibility.

REQ-7.2 The WASM guest shall not copy strings. The host (LDIR) shall pass the guest a 32-bit pointer and length corresponding to the host's memory-mapped S-IR.

REQ-7.3 The engine shall inject fuel instructions into the WASM bytecode. If a custom macro exceeds 100,000 instructions, the engine shall trap execution and emit a compilation error, guaranteeing the compiler never hangs.

REQ-7.4 The engine shall support loading `.wasm` plugins that can intercept the S-IR to G-IR lowering process to implement custom macros or complex layout logic.

REQ-7.5 The WASM interface shall be versioned independently to allow runtime swapping of the WASM module without recompiling the host.

---

## 8. Telemetry & Observability

REQ-8.1 The system shall embed the `tracing` ecosystem, capturing entry/exit times of every major layout function using CPU Timestamp Counters (RDTSC on x86, CNTVCT_EL0 on ARM) for ultra-low overhead profiling.

REQ-8.2 Traces shall be exportable to Chrome Trace Format (`.json`) or Tracy for visual flame-graph analysis of cache misses and thread-stall times.

---

## 9. Quality & Verification

REQ-9.1 The engine shall be continuously fuzzed in CI using `cargo-fuzz`. No malformed S-IR graph shall cause an out-of-bounds panic, segfault, or infinite loop in the `libldir` layout engine.

REQ-9.2 Compiling the same S-IR multiple times using different thread counts (1 vs 4 vs 16 cores) shall yield a bitwise-identical G-IR hash.

REQ-9.3 The system shall include a golden master test suite of classic TeX documents that must render into G-IR producing visually identical results to `pdftex`.

REQ-9.4 Integrated benchmarking in CI shall fail if a commit increases layout latency by more than 2%.

REQ-9.5 The engine shall pass an idempotency test: repeated compilation of identical S-IR shall produce identical G-IR regardless of internal caching state.

---

## 10. Deployment & Monorepo Structure

REQ-10.1 The project shall be organized as a Cargo workspace with the following crate layout:

```text
/ldir
  /ldir-core       (ECS Engine, Layout Optimizer, IR compilation)
  /ldir-ir         (rkyv schemas, FlatBuffers schemas, opcode definitions)
  /ldir-tex        (LaTeX Macro Expander, token ring buffer)
  /ldir-md         (CommonMark Markdown Frontend)
  /ldir-pdf        (PDF/A-4 Backend)
  /ldir-vello      (Native GPU Previewer, WGPU/Vello)
  /ldir-wasm       (WASM/WebGL Browser Bundle)
  /ldc             (CLI Compiler)
  /ldir-lean       (Lean4 formal specifications and proofs)
```

REQ-10.2 Workspace dependencies shall use path dependencies for inter-crate references and precise version pins for external crates.

REQ-10.3 Each crate shall have a clearly defined public API and shall not depend on sibling crates except through the `ldir-ir` and `ldir-core` interfaces.

---

## 11. Non-Functional Requirements

### 11.1 Performance

REQ-11.1.1 The engine shall compile War and Peace (plain text to PDF) in < 100ms.

REQ-11.1.2 When a single paragraph changes in a 500-page document, paragraph-level re-layout shall complete in < 1ms.

REQ-11.1.3 When a single word changes in a 1,000-page document with viewer refresh, the incremental update shall complete in < 5ms.

REQ-11.1.4 Full document re-pagination for a 500-page document shall complete in < 50ms.

### 11.2 Security

REQ-11.2.1 All user-defined extensibility shall execute within a WASM sandbox. No native plugins shall be supported.

REQ-11.2.2 The WASM sandbox shall enforce fuel limits to prevent resource exhaustion attacks.

REQ-11.2.3 The engine shall be continuously fuzzed to ensure robustness against malformed input.

### 11.3 Portability

REQ-11.3.1 The engine shall produce deterministic G-IR output across x86-64 and AArch64 architectures.

REQ-11.3.2 The engine shall produce deterministic G-IR output across Linux, macOS, and Windows operating systems.

REQ-11.3.3 All geometric calculations shall use 26.6 fixed-point arithmetic to ensure cross-platform reproducibility.

### 11.4 Observability

REQ-11.4.1 All major layout functions shall be instrumented with nanosecond-resolution tracing.

REQ-11.4.2 Performance traces shall be exportable in Chrome Trace Format and Tracy format.

REQ-11.4.3 The engine shall expose frame profiling data including cache miss rates and thread stall times for visual flame-graph analysis.
