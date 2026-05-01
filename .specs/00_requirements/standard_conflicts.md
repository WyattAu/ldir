# LDIR Standard Conflicts — Requirements Document

## Overview
This document registers all identified conflicts between external standards, algorithms, and LDIR project requirements. Each conflict includes an impact analysis, confidence level, and stakeholder approval status.

---

## CONF-001: IEEE 754 Floating-Point vs. LDIR Determinism

| Field | Detail |
|-------|--------|
| **Standard 1** | IEEE 754-2019 (Binary Floating-Point Arithmetic) |
| **Standard 2** | LDIR Determinism Requirement (REQ-02.01.01) |
| **Nature of Conflict** | IEEE 754 permits different rounding modes, extended precision, and FMA contraction across platforms and compilers. This produces non-deterministic results for identical geometric calculations on x86 vs. ARM, or even between compiler optimization levels. |
| **Impact Analysis** | **Critical.** Without resolution, G-IR output would differ across platforms, violating the core determinism guarantee. Line-break decisions depend on sub-pixel measurements; even 1 ULP drift could alter page breaks in edge cases. |
| **Resolution** | Use 26.6 fixed-point integers for all geometric calculations in the layout engine. Cassowary constraint solver uses fixed-point internally with documented error bounds. |
| **Confidence Level** | High — fixed-point is proven in FreeType and TeX (scaled points). |
| **ADR Reference** | ADR-003 |
| **Stakeholder Approval** | Approved — Core team consensus (2026-04-23) |
| **Status** | Resolved |

---

## CONF-002: GPU Rendering vs. Cross-Platform Determinism

| Field | Detail |
|-------|--------|
| **Standard 1** | GPU Rendering (WGPU/Vello, GPU compute shaders) |
| **Standard 2** | LDIR Cross-Platform Determinism Requirement |
| **Nature of Conflict** | GPU floating-point operations are non-deterministic across vendors (NVIDIA vs. AMD vs. Intel). Driver implementations may reorder operations, use different internal precision, or apply different rounding. MSAA and blending are inherently non-deterministic. |
| **Impact Analysis** | **Moderate.** Affects visual output only at the rasterization stage. G-IR itself is deterministic; the conflict is confined to the display pipeline. |
| **Resolution** | Determinism guarantee is scoped to the G-IR level (pre-rasterization). Rasterization is treated as a display concern; pixel-perfect reproduction across GPUs is explicitly out of scope. PDF backend (deterministic) is the archival reference. |
| **Confidence Level** | High — standard practice in production renderers. |
| **ADR Reference** | ADR-004 |
| **Stakeholder Approval** | Approved — Core team consensus (2026-04-23) |
| **Status** | Resolved |

---

## CONF-003: WASM Specification vs. Zero-Copy ABI

| Field | Detail |
|-------|--------|
| **Standard 1** | WebAssembly Core Specification (2.0) |
| **Standard 2** | LDIR Zero-Copy ABI Requirement (REQ-04.01.02) |
| **Nature of Conflict** | WASM linear memory model is an isolated address space. The host cannot directly share pointers to its own memory without explicit shared memory setup (`--shared-memory` flag, `SharedArrayBuffer`). Even with shared memory, passing host pointers requires careful synchronization. |
| **Impact Analysis** | **Moderate.** Without resolution, every S-IR access from WASM would require a copy, adding latency proportional to document size. For large documents (1GB+), this is unacceptable. |
| **Resolution** | Host passes a 32-bit pointer and length corresponding to the host's memory-mapped S-IR via WASM shared memory. WASM guest reads directly from host memory without copying. Requires `--shared-memory` and structured concurrency protocol. |
| **Confidence Level** | Medium — shared memory WASM is stable but synchronization protocol needs careful implementation and testing. |
| **ADR Reference** | ADR-005 |
| **Stakeholder Approval** | Approved — Core team consensus (2026-04-23) |
| **Status** | Resolved |

---

## CONF-004: Cassowary Algorithm vs. Fixed-Point Arithmetic

| Field | Detail |
|-------|--------|
| **Standard 1** | Cassowary Constraint Solving Algorithm (Badros/Borning, 2001) |
| **Standard 2** | LDIR Fixed-Point Requirement (REQ-02.01.01) |
| **Nature of Conflict** | The original Cassowary algorithm and all reference implementations use IEEE 754 double-precision floating-point for the simplex tableau. The pivot operations in the dual-simplex method are numerically sensitive; converting to fixed-point may accumulate error beyond acceptable bounds. |
| **Impact Analysis** | **High.** The constraint solver is used for float positioning (images, sidebars). Accumulated error could cause elements to overlap or fail to satisfy constraints, producing visibly incorrect layouts. |
| **Resolution** | Adapt the Cassowary solver to use 26.6 fixed-point arithmetic with the following mitigations: (1) extended intermediate precision (48.16) during pivot operations, (2) documented error bounds per operation, (3) validation pass after solving to verify all constraints within tolerance. |
| **Confidence Level** | Medium — no prior art for fixed-point Cassowary; requires prototyping in Phase 3 to validate error bounds. |
| **ADR Reference** | ADR-006 |
| **Stakeholder Approval** | Approved — Core team consensus (2026-04-23), contingent on Phase 3 validation. |
| **Status** | Resolved (conditional on validation) |

---

## CONF-005: TeX Compatibility vs. Formal Verification

| Field | Detail |
|-------|--------|
| **Standard 1** | TeX (Knuth, 1984) / pdfTeX / XeTeX / LuaTeX |
| **Standard 2** | LDIR Formal Verification Requirement (Lean4) |
| **Nature of Conflict** | TeX has extensive undefined and implementation-defined behavior: `\halign` edge cases, `\expandafter` on empty tokens, `\lastbox` after discretionary breaks, etc. These behaviors cannot be formally verified because they are not formally specified. LuaTeX extensions are Turing-complete, making static analysis undecidable. |
| **Impact Analysis** | **Moderate.** Affects the TeX frontend (`ldir-tex`) compatibility claims. Full TeX compatibility is impossible to verify formally. |
| **Resolution** | LDIR defines its own formally specified behavior for all edge cases. TeX compatibility is aspirational (target: 99.9% of real-world documents render identically) but not formally verified. The Lean4 proof covers LDIR's own IR well-formedness, not TeX behavioral equivalence. A "Golden Master" test suite (1,000 classic TeX documents) provides empirical compatibility evidence. |
| **Confidence Level** | High — clean separation of concerns between formal spec and legacy compatibility. |
| **ADR Reference** | ADR-007 |
| **Stakeholder Approval** | Approved — Core team consensus (2026-04-23) |
| **Status** | Resolved |
