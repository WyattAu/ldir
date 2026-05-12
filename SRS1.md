# LDIR: Rigorous System Requirements Specification (v1.0)

## 1. Core System & Memory Constraints
To achieve sub-millisecond latency per page, the engine must strictly adhere to Data-Oriented Design (DoD) and mechanical sympathy.

### 1.1 Memory Allocation & Layout
*   **REQ-1.1.1 (No Runtime Allocation):** The layout engine (`libldir`) shall perform zero dynamic heap allocations during the hot layout pass. All memory must be pre-allocated in typed **Arena Allocators** (Bump Allocators) during system initialization.
*   **REQ-1.1.2 (Structure of Arrays - SoA):** Document attributes shall strictly use SoA layout to guarantee L1/L2 cache saturation. E.g., `width`, `height`, `font_id`, and `glyph_id` must reside in contiguous, separate memory arrays, aligned to 64-byte (cache line) boundaries.
*   **REQ-1.1.3 (Pointer Avoidance):** The system shall not use raw pointers or Rust `Box`/`Rc`/`Arc` for document nodes. All relations must be mapped via **32-bit Generation Indices** (Entity IDs). Max capacity: $2^{32}$ nodes per document.

### 1.2 Concurrency & OS Interaction
*   **REQ-1.2.1 (Thread Pinning):** The engine shall utilize a custom thread pool pinned to physical CPU cores via CPU affinity masks to prevent OS scheduler thrashing and cache invalidation.
*   **REQ-1.2.2 (Lock-Free Caching):** Font shaping caches and calculated glyph widths must be stored in a highly concurrent, lock-free hash map (e.g., using hazard pointers or epoch-based reclamation) to prevent thread contention during parallel paragraph evaluation.

---

## 2. The LDIR Instruction Set Architecture (ISA)

### 2.1 Fixed-Point Geometry
*   **REQ-2.1.1 (Coordinate System):** To guarantee cross-platform determinism (avoiding IEEE-754 floating-point drift), all geometric calculations in G-IR shall use **32-bit signed fixed-point integers**.
*   **REQ-2.1.2 (Precision):** The system shall use $26.6$ fixed-point format (26 bits for whole numbers, 6 bits for fractions) matching the FreeType internal format, providing precision of $1/2^6 = 1/64$ of a unit per ULP. TeX scaled points (sp) provide finer granularity at $1/65536$ of a printer's point; the system shall document which unit is used at each pipeline stage.

### 2.2 Semantic-IR (S-IR) Protocol
*   **REQ-2.2.1 (Serialization):** S-IR must be serialized using a zero-copy protocol (`rkyv` or FlatBuffers). The engine must be able to `mmap` a 1GB S-IR file and begin layout in $O(1)$ time.
*   **REQ-2.2.2 (Data Schema):**
    *   `OpCode` (1 byte)
    *   `EntityID` (4 bytes)
    *   `ParentID` (4 bytes)
    *   `PayloadOffset` (4 bytes)

### 2.3 Geometric-IR (G-IR) Protocol
*   **REQ-2.3.1 (Page Buffer):** G-IR is compiled into a flat command buffer per page, optimized for direct GPU upload or PDF stream generation.
*   **REQ-2.3.2 (Command Alignment):** G-IR commands must be explicitly aligned to 16-byte boundaries to allow for vectorized iteration by the backend rendering pipelines.

---

## 3. Layout Optimization Algorithms (The "Middle-End")

### 3.1 Text Shaping Fast-Path
*   **REQ-3.1.1 (Vectorized ASCII):** The engine must implement an AVX2/NEON optimized fast-path for standard ASCII/Latin-1 text. If the S-IR block contains no complex grapheme clusters or kerning pairs, the engine bypasses the shaping engine (HarfBuzz) and calculates widths via a vectorized lookup table.

### 3.2 SIMD Line Breaking (Local Optimization)
*   **REQ-3.2.1 (Vectorized Penalty Calculation):** The Knuth-Plass "badness" calculation $b = 100 \times (w - t)^3 / s^3$ (where $w$=width, $t$=target, $s$=stretch) must be vectorized. The engine shall evaluate 8 potential line-break candidates simultaneously using 256-bit SIMD registers.
*   **REQ-3.2.2 (Branchless Evaluation):** The inner loop of the dynamic programming solver must be entirely branchless to prevent CPU pipeline stalls.

### 3.3 Global Pagination (Global Optimization)
*   **REQ-3.3.1 (DAG Construction):** The document's page-breaks must be modeled as a Directed Acyclic Graph (DAG).
*   **REQ-3.3.2 (Branch and Bound):** The engine must use a Branch-and-Bound algorithm to prune layout variations that exceed a predefined "maximum badness" threshold, preventing combinatorial explosion when paginating 1,000+ page documents.

### 3.4 Spatial Constraint Solver
*   **REQ-3.4.1 (Linear Programming):** For floating elements (images, sidebars), the engine shall implement a lightweight, dual-simplex Cassowary solver. Matrices for the solver must be stored in SoA format and solved using block-matrix SIMD operations.

---

## 4. WebAssembly (WASM) Extensibility ABI

*   **REQ-4.1.1 (Sandboxed Execution):** All user-defined macros and custom layout logic must execute within a `wasmtime` sandbox. No native C/Rust plugins are permitted for security and reproducibility.
*   **REQ-4.1.2 (Zero-Copy ABI):** The WASM guest must not copy strings. The host (LDIR) will pass the guest a 32-bit pointer and length corresponding to the host's memory-mapped S-IR.
*   **REQ-4.1.3 (Execution Limits):** The engine must inject "fuel" instructions into the WASM bytecode. If a custom macro exceeds 100,000 instructions (e.g., an infinite loop), the engine traps the execution and emits a compilation error, guaranteeing the compiler never hangs.

---

## 5. Frontends (AST Lowering & Parsing)

### 5.1 TeX Frontend (`ldir-tex`)
*   **REQ-5.1.1 (Token Ring Buffer):** The lexer must read `.tex` files into a lock-free ring buffer, processing tokens at $> 500 \text{ MB/s}$.
*   **REQ-5.1.2 (Expansion State Machine):** The macro expander must track state without recursion to prevent stack overflows on deeply nested macros (e.g., TikZ). It must use an explicit heap-allocated (or arena-allocated) stack state machine.

### 5.2 Language Server Protocol (LSP) Tracing
*   **REQ-5.2.1 (Source Mapping):** Every token parsed by *any* frontend must generate a mapping spanning its byte-offset in the source file to its corresponding `EntityID` in the S-IR.
*   **REQ-5.2.2 (Reverse Mapping):** The G-IR must retain these `EntityID`s. Hovering over a pixel coordinate in the G-IR output must resolve back to the exact source text file and line number in $< 2\text{ms}$.

---

## 6. Backends (Code Generators)

### 6.1 GPU / Native Backend (`ldir-vello`)
*   **REQ-6.1.1 (Compute Shader Mapping):** The backend must map G-IR directly into GPU compute shaders (via WGPU/Vello). CPU must only calculate bounding boxes; path rasterization and anti-aliasing (MSAA) must execute 100% on the GPU.
*   **REQ-6.1.2 (Frame Budget):** The viewer must render pan/zoom updates at 144Hz (i.e., $< 6.9\text{ms}$ frame budget).

### 6.2 Archival Backend (`ldir-pdf`)
*   **REQ-6.2.1 (Zero-Allocation Dictionary Writer):** The PDF writer must construct PDF object dictionaries using pre-allocated byte buffers.
*   **REQ-6.2.2 (Parallel Deflate):** PDF object streams (FlateDecode) must be compressed in parallel using the Rayon thread pool before being written to the output file sequentially.
*   **REQ-6.2.3 (Font Subsetting):** The backend must parse the TrueType/OpenType tables and subset fonts exactly to the glyphs used in the G-IR. The subsetting algorithm must run concurrently with the PDF dictionary generation.

---

## 7. Telemetry & Observability

*   **REQ-7.1.1 (Nanosecond Tracing):** The system must embed the `tracing` ecosystem, capturing entry/exit times of every major layout function using CPU Timestamp Counters (`RDTSC` on x86, `CNTVCT_EL0` on ARM) for ultra-low overhead profiling.
*   **REQ-7.1.2 (Frame Profiling):** Traces must be exportable to Chrome Trace Format (`.json`) or Tracy for visual flame-graph analysis of cache misses and thread-stall times.

---

## 8. Verification & Release Criteria

*   **REQ-8.1.1 (Differential Fuzzing):** The engine shall be continuously fuzzed in CI. The fuzzer will generate random S-IR graphs. The requirement is that no malformed S-IR graph can cause an out-of-bounds panic, segfault, or infinite loop in the `libldir` layout engine.
*   **REQ-8.1.2 (Idempotency Test):** Compiling the same S-IR multiple times using different thread counts (1 vs 4 vs 16 cores) must yield a bitwise-identical G-IR hash. Layout is not permitted to have race conditions that alter sub-pixel placement.
*   **REQ-8.1.3 (Benchmark Thresholds):** 
    *   Throughput target: Compile *War and Peace* (Plain Text -> PDF) in $< 100\text{ms}$.
    *   Incremental target: Update a single word in a 1,000-page document and update the GPU viewer in $< 5\text{ms}$.