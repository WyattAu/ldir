# LDIR Project: System Requirements Specification (SRS)

## 1. Project Vision
LDIR aims to modernize digital typesetting by decoupling **content intent** from **physical layout** through a tiered Intermediate Representation (IR) system. It leverages Data-Oriented Design (DoD) and SIMD-accelerated algorithms to provide a "live" typesetting experience (sub-5ms re-layouts) for massive documents across all platforms (PDF, Web, Native).

## 2. Core Architectural Principles
*   **Zero-Copy Pipeline:** Data must stay in its binary serialized form (`rkyv`) from the frontend through the engine until the final backend rendering.
*   **Data-Oriented Design (DoD):** Use an Entity-Component-System (ECS) to store document nodes in contiguous memory for cache-local iteration.
*   **Parallel-First Optimization:** The layout engine must assume a multi-core environment, using work-stealing schedulers for independent sections and SIMD for tight inner loops.
*   **Deterministic Execution:** Identical IR input must produce bit-identical Geometric output regardless of the host OS or CPU architecture.

---

## 3. The LDIR Instruction Set (The "DVI 2.0")
The LDIR shall consist of two distinct IR layers:

### 3.1 Semantic-IR (S-IR) - "The Intent"
*   **Requirement:** A structural representation of the document.
*   **Instruction Examples:**
    *   `PUSH_BLOCK(type: Paragraph|Heading|List)`
    *   `SET_CONTENT(blob_ref)`
    *   `APPLY_STYLE(style_id)`
    *   `INSERT_MATH(mathml_ref | tex_ref)`
    *   `LINK_DATA(ptr_address)` (For live data-linking in DLLs)

### 3.2 Geometric-IR (G-IR) - "The Result"
*   **Requirement:** A low-level, device-independent instruction set similar to DVI but extended for modern typography.
*   **Instruction Examples (Opcodes):**
    *   `SET_FONT(id, size)`
    *   `MOVE_XY(h, v)`
    *   `PUT_GLYPH(unicode_id, advance_x)`
    *   `DRAW_RULE(width, height)`
    *   `PUSH_STACK` / `POP_STACK` (For nested coordinate systems)
    *   `ATTACH_METADATA(key, val)` (For tagging/accessibility)

---

## 4. The Core Engine (`libldir`)
### 4.1 The Layout Optimizer
*   **SIMD Line Breaking:** Implement the Knuth-Plass algorithm using SIMD to calculate multiple line-break penalties in parallel (Phase 1: Width measurement, Phase 2: Optimal path finding).
*   **Global Pagination:** Treat page-breaking as a global optimization problem across the entire G-IR graph to eliminate widows/orphans across 100+ pages simultaneously.
*   **Constraint Solver:** Implement a linear constraint solver (e.g., Cassowary) to handle complex float/image positioning ("Image must be within 1 inch of this paragraph").

### 4.2 Memory & Performance
*   **Memory Model:** Use an **ECS (Entity-Component-System)** where document boxes are "Entities" and their properties (Width, Height, Font) are "Components" stored in **SoA (Structure of Arrays)** format.
*   **Latency Target:** Re-layout of a changed paragraph in a 500-page document must complete in **< 1ms**. Full document re-pagination must complete in **< 50ms**.

### 4.3 Extensibility
*   **WASM Hook System:** The engine must support loading `.wasm` plugins that can intercept the S-IR to G-IR lowering process to implement custom macros or complex layout logic.

---

## 5. Frontend Requirements (Parsers)
### 5.1 LaTeX "Emulator" Frontend
*   **Macro Expansion:** Must implement a pure-Rust TeX macro expander that handles `\def`, `\newcommand`, and `\expandafter`.
*   **Output:** Lower expanded tokens directly into LDIR Semantic-IR.
*   **Scope:** Initial support for `amsmath`, `geometry`, and basic `tabular`.

### 5.2 Markdown Frontend
*   **Spec:** Adherence to CommonMark.
*   **Output:** Map AST nodes (Header, List, Link) directly to S-IR Block instructions.

---

## 6. Backend Requirements (Renderers)
*   **PDF/A-4 Backend:** High-fidelity, tagged PDF generation with font subsetting.
*   **WASM/WebGL Backend:** A "Thin-Client" renderer that interprets G-IR instructions onto a Canvas using GPU shaders.
*   **Native Embeddable (DLL/so):** The engine must compile to a C-compatible dynamic library that allows host applications to:
    1.  Push new S-IR data.
    2.  Query G-IR for bounding boxes (for mouse interaction).
    3.  Render a specific page to a provided memory buffer/texture.

---

## 7. Quality & Verification
*   **The "Golden Master" Test:** A suite of 1,000 classic TeX documents must render into G-IR and produce identical visual results to `pdftex`.
*   **Perf-Bench Suite:** Integrated benchmarking in CI that fails if a commit increases layout latency by more than 2%.
*   **Fuzzing:** Use `cargo-fuzz` on the S-IR binary parser to ensure the engine never crashes on malformed document data.

## 8. Deployment Monorepo Structure
```text
/ldir
  /ldir-core       (The ECS Engine & Optimizer)
  /ldir-ir         (Flatbuffers/rkyv Schemas)
  /ldir-tex        (LaTeX Macro Expander)
  /ldir-md         (Markdown Frontend)
  /ldir-pdf        (PDF Backend)
  /ldir-vello      (Native GPU Previewer)
  /ldir-wasm       (Browser Bundle)
  /ldc             (The CLI Compiler)
```