# LDIR Bidirectional Traceability Matrix

**Version:** 1.0.0
**Date:** 2026-05-15
**Source:** `.specs/01_research/yellow_paper_registry.toml`, `.specs/00_requirements/requirements.md`

---

## 1. Yellow Paper to Yellow Paper Dependencies

```
YP-NUMERICAL-FIXEDPOINT-001  (26.6 Fixed-Point Arithmetic)
  |
  +---> YP-LAYOUT-KNUTHPLASS-001      (Knuth-Plass Line Breaking)
  +---> YP-LAYOUT-PAGINATION-001       (Page Breaking and Float Placement)
  +---> YP-CONSTRAINT-CASSOWARY-001    (Cassowary Constraint Solver)

YP-MEMORY-ECS-001  (Entity Component System Memory Architecture)
  |
  +---> YP-CONCURRENCY-DETERM-001      (Deterministic Concurrency via Work-Stealing)
```

| Yellow Paper | Depends On | Dependency Type |
|---|---|---|
| YP-LAYOUT-KNUTHPLASS-001 | YP-NUMERICAL-FIXEDPOINT-001 | Axioms (fixed-point arithmetic) |
| YP-LAYOUT-PAGINATION-001 | YP-NUMERICAL-FIXEDPOINT-001 | Axioms (fixed-point arithmetic) |
| YP-CONSTRAINT-CASSOWARY-001 | YP-NUMERICAL-FIXEDPOINT-001 | Axioms (fixed-point arithmetic) |
| YP-CONCURRENCY-DETERM-001 | YP-MEMORY-ECS-001 | Axioms (ECS memory model) |

### Independent Yellow Papers (no YP dependencies)

- YP-IR-SEMANTICS-001 -- IR Type Semantics and Well-Formedness
- YP-NUMERICAL-FIXEDPOINT-001 -- 26.6 Fixed-Point Arithmetic
- YP-MEMORY-ECS-001 -- Entity Component System Memory Architecture

---

## 2. Yellow Paper to Blue Paper Mapping

| Blue Paper | Depends On | Rationale |
|---|---|---|
| BP-IR-COMPILER-001 | YP-IR-SEMANTICS-001 | Compiler correctness is defined by IR well-formedness theorems |

### Yellow Papers Consumed by Blue Papers

| Yellow Paper | Consumed By |
|---|---|
| YP-IR-SEMANTICS-001 | BP-IR-COMPILER-001 |

---

## 3. Requirement to Implementation Traceability

### 3.1 IR Semantics (YP-IR-SEMANTICS-001)

| Requirement ID | Yellow Paper | Crate | Implementation File | Test Coverage |
|---|---|---|---|---|
| REQ-3.1.1 | YP-IR-SEMANTICS-001 | ldir-ir | `src/sir/mod.rs` | `ldir-ir/tests/integration_roundtrip.rs` |
| REQ-3.1.2 | YP-IR-SEMANTICS-001 | ldir-ir | `src/sir/instruction.rs` | `ldir-ir/tests/integration_roundtrip.rs` |
| REQ-3.1.3 | YP-IR-SEMANTICS-001 | ldir-ir | `src/sir/opcode.rs` | Unit tests in `src/sir/opcode.rs` |
| REQ-3.1.4 | YP-IR-SEMANTICS-001 | ldir-ir | `src/sir/payload.rs` | Unit tests in `src/sir/payload.rs` |
| REQ-3.1.5 | YP-IR-SEMANTICS-001 | ldir-ir | `src/sir/serde.rs` | `ldir-ir/tests/integration_roundtrip.rs` |
| REQ-3.1.6 | YP-IR-SEMANTICS-001 | ldir-ir | `src/sir/mod.rs` | Unit tests in `src/sir/mod.rs` |
| REQ-3.2.1 | YP-IR-SEMANTICS-001 | ldir-ir | `src/gir/command.rs` | `ldir-ir/tests/integration_roundtrip.rs` |
| REQ-3.2.2 | YP-IR-SEMANTICS-001 | ldir-ir | `src/gir/command.rs` | Unit tests in `src/gir/command.rs` |
| REQ-3.2.3 | YP-IR-SEMANTICS-001 | ldir-ir | `src/gir/opcode.rs` | Unit tests in `src/gir/opcode.rs` |
| REQ-3.3.1 | BP-IR-COMPILER-001 | ldir-opt | `src/main.rs`, `src/pass_manager.rs` | `tests/tests/integration.rs` |
| REQ-3.3.2 | BP-IR-COMPILER-001 | ldir-opt | `src/passes.rs` | `tests/tests/integration.rs` |
| REQ-3.3.3 | BP-IR-COMPILER-001 | ldir-opt | `src/passes.rs` | `tests/tests/integration.rs` |
| REQ-3.3.4 | BP-IR-COMPILER-001 | ldir-opt | `src/passes.rs` | `tests/tests/integration.rs` |

### 3.2 Numerical / Fixed-Point (YP-NUMERICAL-FIXEDPOINT-001)

| Requirement ID | Yellow Paper | Crate | Implementation File | Test Coverage |
|---|---|---|---|---|
| REQ-3.2.4 | YP-NUMERICAL-FIXEDPOINT-001 | ldir-ir | `src/fp266.rs` | Unit tests in `src/fp266.rs` |
| REQ-3.2.5 | YP-NUMERICAL-FIXEDPOINT-001 | ldir-ir | `src/fp266.rs` | Unit tests in `src/fp266.rs` |
| REQ-3.2.6 | YP-NUMERICAL-FIXEDPOINT-001 | ldir-ir | `src/fp266.rs` | Unit tests in `src/fp266.rs` |
| REQ-3.2.7 | YP-NUMERICAL-FIXEDPOINT-001 | ldir-ir | `src/fp266.rs` | Unit tests in `src/fp266.rs` |
| REQ-11.3.1 | YP-NUMERICAL-FIXEDPOINT-001 | ldir-ir | `src/fp266.rs` | Cross-arch CI tests |
| REQ-11.3.2 | YP-NUMERICAL-FIXEDPOINT-001 | ldir-ir | `src/fp266.rs` | Cross-platform CI tests |
| REQ-11.3.3 | YP-NUMERICAL-FIXEDPOINT-001 | ldir-ir | `src/fp266.rs` | Unit tests in `src/fp266.rs` |

### 3.3 Layout -- Knuth-Plass (YP-LAYOUT-KNUTHPLASS-001)

| Requirement ID | Yellow Paper | Crate | Implementation File | Test Coverage |
|---|---|---|---|---|
| REQ-4.3.2.1 | YP-LAYOUT-KNUTHPLASS-001 | ldir-core | TBD (line break module) | Test vectors: `test_vectors_linebreak.toml` |
| REQ-4.3.2.2 | YP-LAYOUT-KNUTHPLASS-001 | ldir-core | TBD (badness calc) | Test vectors: `test_vectors_linebreak.toml` |
| REQ-4.3.2.3 | YP-LAYOUT-KNUTHPLASS-001 | ldir-core | TBD (SIMD badness) | Benchmark tests |
| REQ-4.3.2.4 | YP-LAYOUT-KNUTHPLASS-001 | ldir-core | TBD (branchless DP) | Benchmark tests |

### 3.4 Layout -- Pagination (YP-LAYOUT-PAGINATION-001)

| Requirement ID | Yellow Paper | Crate | Implementation File | Test Coverage |
|---|---|---|---|---|
| REQ-4.3.3.1 | YP-LAYOUT-PAGINATION-001 | ldir-core | TBD (page-break DAG) | Test vectors: `test_vectors_pagination.toml` |
| REQ-4.3.3.2 | YP-LAYOUT-PAGINATION-001 | ldir-core | TBD (branch-and-bound) | Test vectors: `test_vectors_pagination.toml` |
| REQ-4.3.3.3 | YP-LAYOUT-PAGINATION-001 | ldir-core | TBD (global pagination) | Benchmark tests |

### 3.5 Constraint Solver (YP-CONSTRAINT-CASSOWARY-001)

| Requirement ID | Yellow Paper | Crate | Implementation File | Test Coverage |
|---|---|---|---|---|
| REQ-4.3.4.1 | YP-CONSTRAINT-CASSOWARY-001 | ldir-core | TBD (cassowary solver) | Test vectors: `test_vectors_constraint.toml` |
| REQ-4.3.4.2 | YP-CONSTRAINT-CASSOWARY-001 | ldir-core | TBD (cassowary solver) | Test vectors: `test_vectors_constraint.toml` |
| REQ-4.3.4.3 | YP-CONSTRAINT-CASSOWARY-001 | ldir-core | TBD (SoA matrix) | Benchmark tests |

### 3.6 Memory / ECS (YP-MEMORY-ECS-001)

| Requirement ID | Yellow Paper | Crate | Implementation File | Test Coverage |
|---|---|---|---|---|
| REQ-2.1 | YP-MEMORY-ECS-001 | ldir-core | TBD (ECS core) | Test vectors: `test_vectors_ecs.toml` |
| REQ-4.1.1 | YP-MEMORY-ECS-001 | ldir-core | TBD (arena allocators) | Test vectors: `test_vectors_ecs.toml` |
| REQ-4.1.2 | YP-MEMORY-ECS-001 | ldir-core | TBD (SoA component storage) | Test vectors: `test_vectors_ecs.toml` |
| REQ-4.1.3 | YP-MEMORY-ECS-001 | ldir-core | TBD (cache-line alignment) | Benchmark tests |
| REQ-4.1.4 | YP-MEMORY-ECS-001 | ldir-core | TBD (index-based refs) | Test vectors: `test_vectors_ecs.toml` |

### 3.7 Concurrency (YP-CONCURRENCY-DETERM-001)

| Requirement ID | Yellow Paper | Crate | Implementation File | Test Coverage |
|---|---|---|---|---|
| REQ-2.5 | YP-CONCURRENCY-DETERM-001 | ldir-core | TBD (thread pool) | Test vectors: `test_vectors_concurrency.toml` |
| REQ-2.6 | YP-CONCURRENCY-DETERM-001 | ldir-core | TBD (determinism guard) | Test vectors: `test_vectors_concurrency.toml` |
| REQ-2.7 | YP-CONCURRENCY-DETERM-001 | ldir-core | TBD (determinism guard) | Test vectors: `test_vectors_concurrency.toml` |
| REQ-4.2.1 | YP-CONCURRENCY-DETERM-001 | ldir-core | TBD (pinned thread pool) | Test vectors: `test_vectors_concurrency.toml` |
| REQ-4.2.2 | YP-CONCURRENCY-DETERM-001 | ldir-core | TBD (lock-free cache) | Concurrency stress tests |
| REQ-4.2.3 | YP-CONCURRENCY-DETERM-001 | ldir-core | TBD (work-stealing) | Test vectors: `test_vectors_concurrency.toml` |

### 3.8 Frontends

| Requirement ID | Yellow Paper | Crate | Implementation File | Test Coverage |
|---|---|---|---|---|
| REQ-5.1.1 | YP-IR-SEMANTICS-001 | ldir-tex | `src/lexer.rs`, `src/parser.rs` | Unit tests |
| REQ-5.1.2 | YP-IR-SEMANTICS-001 | ldir-tex | `src/parser.rs` | Unit tests |
| REQ-5.1.3 | YP-IR-SEMANTICS-001 | ldir-tex | `src/lexer.rs` | Benchmark tests |
| REQ-5.1.4 | YP-IR-SEMANTICS-001 | ldir-tex | `src/parser.rs` | Unit tests (deep nesting) |
| REQ-5.2.1 | YP-IR-SEMANTICS-001 | ldir-md | `src/lib.rs` | Unit tests |
| REQ-5.2.2 | YP-IR-SEMANTICS-001 | ldir-md | `src/lib.rs` | Unit tests |

### 3.9 Backends

| Requirement ID | Yellow Paper | Crate | Implementation File | Test Coverage |
|---|---|---|---|---|
| REQ-6.1.1 | YP-IR-SEMANTICS-001 | ldir-vello | `src/gir_to_scene.rs` | `ldir-vello/src/lib.rs` |
| REQ-6.1.2 | YP-IR-SEMANTICS-001 | ldir-vello | `src/renderer.rs` | `ldir-vello/src/lib.rs` |
| REQ-6.1.3 | YP-IR-SEMANTICS-001 | ldir-vello | `src/viewport.rs` | Benchmark tests |
| REQ-6.2.1 | YP-IR-SEMANTICS-001 | ldir-pdf | `src/conformance.rs` | `src/pdf_test.rs` |
| REQ-6.2.2 | YP-IR-SEMANTICS-001 | ldir-pdf | `src/writer.rs` | `src/pdf_test.rs` |
| REQ-6.2.3 | YP-IR-SEMANTICS-001 | ldir-pdf | `src/writer.rs` | Benchmark tests |
| REQ-6.2.4 | YP-IR-SEMANTICS-001 | ldir-pdf | `src/font/subset.rs`, `src/font/loader.rs` | Unit tests |
| REQ-6.3.1 | YP-IR-SEMANTICS-001 | ldir-wasm | `src/html_renderer.rs` | `ldir-wasm/tests/wasm_build.rs` |

### 3.10 WASM Extensibility

| Requirement ID | Yellow Paper | Crate | Implementation File | Test Coverage |
|---|---|---|---|---|
| REQ-7.1 | YP-CONCURRENCY-DETERM-001 | ldir-wasm | `src/sandbox.rs` | `ldir-wasm/tests/wasm_shape.rs` |
| REQ-7.2 | YP-MEMORY-ECS-001 | ldir-wasm | `src/bridge.rs` | `ldir-wasm/tests/wasm_build.rs` |
| REQ-7.3 | YP-CONCURRENCY-DETERM-001 | ldir-wasm | `src/sandbox.rs` | `ldir-wasm/tests/wasm_shape.rs` |
| REQ-7.4 | YP-IR-SEMANTICS-001 | ldir-wasm | `src/bridge.rs` | `ldir-wasm/tests/wasm_build.rs` |
| REQ-7.5 | YP-IR-SEMANTICS-001 | ldir-wasm | `src/versioning.rs` | Unit tests |

### 3.11 LIR / PDF Rendering

| Requirement ID | Yellow Paper | Crate | Implementation File | Test Coverage |
|---|---|---|---|---|
| REQ-6.2.1 | YP-IR-SEMANTICS-001 | ldir-pdf | `src/lir_render.rs` | `src/pdf_test.rs` |
| REQ-6.2.1 | YP-IR-SEMANTICS-001 | ldir-pdf | `src/structure.rs` | Unit tests |
| REQ-6.2.1 | YP-IR-SEMANTICS-001 | ldir-pdf | `src/color.rs` | Unit tests |
| REQ-6.2.1 | YP-IR-SEMANTICS-001 | ldir-pdf | `src/xmp.rs` | Unit tests |
| REQ-6.2.1 | YP-IR-SEMANTICS-001 | ldir-pdf | `src/image.rs` | Unit tests |

### 3.12 CLI / Linking

| Requirement ID | Yellow Paper | Crate | Implementation File | Test Coverage |
|---|---|---|---|---|
| REQ-3.3.1 | BP-IR-COMPILER-001 | ldc | `src/main.rs` | `tests/tests/integration.rs` |
| REQ-10.1 | YP-MEMORY-ECS-001 | ldir-link | `src/main.rs`, `src/linker.rs` | `ldir-link/src/lib.rs` |

---

## 4. Test Vector Coverage Summary

| Yellow Paper | Test Vector File | Count | Categories |
|---|---|---|---|
| YP-IR-SEMANTICS-001 | `test_vectors/test_vectors_ir.toml` | 23 | nominal (5), boundary (4), adversarial (9), random (3), regression (2) |
| YP-NUMERICAL-FIXEDPOINT-001 | `test_vectors/test_vectors_numerical.toml` | 7 | nominal (3), boundary (2), adversarial (2) |
| YP-LAYOUT-KNUTHPLASS-001 | `test_vectors/test_vectors_linebreak.toml` | 6 | nominal (2), boundary (2), adversarial (2) |
| YP-LAYOUT-PAGINATION-001 | `test_vectors/test_vectors_pagination.toml` | 6 | nominal (2), boundary (2), adversarial (2) |
| YP-CONSTRAINT-CASSOWARY-001 | `test_vectors/test_vectors_constraint.toml` | 6 | nominal (5), adversarial (1) |
| YP-MEMORY-ECS-001 | `test_vectors/test_vectors_ecs.toml` | 5 | nominal (2), boundary (1), adversarial (2) |
| YP-CONCURRENCY-DETERM-001 | `test_vectors/test_vectors_concurrency.toml` | 5 | nominal (4), boundary (1) |

**Total test vectors:** 58 (across all 7 yellow papers)
**Missing test vector files:** 0

---

## 5. Reverse Traceability Index

### By Crate

| Crate | Yellow Papers | Requirement Sections |
|---|---|---|
| ldir-ir | YP-IR-SEMANTICS-001, YP-NUMERICAL-FIXEDPOINT-001 | 3.1, 3.2, 11.3 |
| ldir-core | YP-LAYOUT-KNUTHPLASS-001, YP-LAYOUT-PAGINATION-001, YP-CONSTRAINT-CASSOWARY-001, YP-MEMORY-ECS-001, YP-CONCURRENCY-DETERM-001 | 2.1, 2.5, 2.6, 2.7, 4.1, 4.2, 4.3 |
| ldir-opt | BP-IR-COMPILER-001 (-> YP-IR-SEMANTICS-001) | 3.3 |
| ldir-tex | YP-IR-SEMANTICS-001 | 5.1 |
| ldir-md | YP-IR-SEMANTICS-001 | 5.2 |
| ldir-pdf | YP-IR-SEMANTICS-001 | 6.2 |
| ldir-vello | YP-IR-SEMANTICS-001 | 6.1 |
| ldir-wasm | YP-IR-SEMANTICS-001, YP-CONCURRENCY-DETERM-001, YP-MEMORY-ECS-001 | 6.3, 7 |
| ldc | BP-IR-COMPILER-001 (-> YP-IR-SEMANTICS-001) | 10.1 |
| ldir-link | YP-MEMORY-ECS-001 | 10.1 |
