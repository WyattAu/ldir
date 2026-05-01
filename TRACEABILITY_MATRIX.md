# LDIR Traceability Matrix

## Bidirectional Traceability

### Requirements → Architecture
| Requirement ID | Component ID | Interface ID | Test Case ID | Standard | Status |
|---------------|--------------|--------------|--------------|----------|--------|
| REQ-01.01.01 | COMP-ENGINE-001 | — | TC-MEM-001 | — | Planned |
| REQ-01.01.02 | COMP-ENGINE-001 | — | TC-MEM-002 | — | Planned |
| REQ-01.01.03 | COMP-ECS-001 | IF-ECS-001 | TC-ECS-001 | — | Planned |
| REQ-01.02.01 | COMP-THREADPOOL-001 | IF-POOL-001 | TC-THREAD-001 | — | Planned |
| REQ-01.02.02 | COMP-CACHE-001 | IF-CACHE-001 | TC-CACHE-001 | — | Planned |
| REQ-02.01.01 | COMP-GEOMETRY-001 | IF-FIXPT-001 | TC-FIXPT-001 | IEEE 754 (conflict resolved) | Planned |
| REQ-02.01.02 | COMP-GEOMETRY-001 | IF-FIXPT-001 | TC-FIXPT-002 | FreeType compat | Planned |
| REQ-02.02.01 | COMP-SIR-001 | IF-SIR-001 | TC-SIR-001 | — | Planned |
| REQ-02.02.02 | COMP-SIR-001 | IF-SIR-001 | TC-SIR-002 | — | Planned |
| REQ-02.03.01 | COMP-GIR-001 | IF-GIR-001 | TC-GIR-001 | — | Planned |
| REQ-02.03.02 | COMP-GIR-001 | IF-GIR-001 | TC-GIR-002 | — | Planned |
| REQ-03.01.01 | COMP-SHAPE-001 | IF-SHAPE-001 | TC-SHAPE-001 | — | Planned |
| REQ-03.02.01 | COMP-LINEBREAK-001 | IF-LB-001 | TC-LB-001 | — | Planned |
| REQ-03.02.02 | COMP-LINEBREAK-001 | IF-LB-001 | TC-LB-002 | — | Planned |
| REQ-03.03.01 | COMP-PAGINATION-001 | IF-PAGE-001 | TC-PAGE-001 | — | Planned |
| REQ-03.03.02 | COMP-PAGINATION-001 | IF-PAGE-001 | TC-PAGE-002 | — | Planned |
| REQ-03.04.01 | COMP-SOLVER-001 | IF-SOLVER-001 | TC-SOLVER-001 | Cassowary (adapted) | Planned |
| REQ-04.01.01 | COMP-WASM-001 | IF-WASM-001 | TC-WASM-001 | WASM spec | Planned |
| REQ-04.01.02 | COMP-WASM-001 | IF-WASM-002 | TC-WASM-002 | — | Planned |
| REQ-04.01.03 | COMP-WASM-001 | IF-WASM-003 | TC-WASM-003 | — | Planned |
| REQ-05.01.01 | COMP-TEX-001 | IF-LEX-001 | TC-LEX-001 | — | Planned |
| REQ-05.01.02 | COMP-TEX-001 | IF-MACRO-001 | TC-MACRO-001 | — | Planned |
| REQ-05.02.01 | COMP-LSP-001 | IF-SRCMAP-001 | TC-SRCMAP-001 | — | Planned |
| REQ-05.02.02 | COMP-LSP-001 | IF-SRCMAP-002 | TC-SRCMAP-002 | — | Planned |
| REQ-06.01.01 | COMP-VELLO-001 | IF-GPU-001 | TC-GPU-001 | WGPU/Vello | Planned |
| REQ-06.01.02 | COMP-VELLO-001 | IF-GPU-002 | TC-GPU-002 | — | Planned |
| REQ-06.02.01 | COMP-PDF-001 | IF-PDF-001 | TC-PDF-001 | PDF/A-4 | Planned |
| REQ-06.02.02 | COMP-PDF-001 | IF-PDF-002 | TC-PDF-002 | — | Planned |
| REQ-06.02.03 | COMP-PDF-001 | IF-PDF-003 | TC-PDF-003 | — | Planned |
| REQ-07.01.01 | COMP-TELEM-001 | IF-TRACE-001 | TC-TRACE-001 | — | Planned |
| REQ-07.01.02 | COMP-TELEM-001 | IF-TRACE-002 | TC-TRACE-002 | Chrome Trace Format | Planned |
| REQ-08.01.01 | COMP-FUZZ-001 | — | TC-FUZZ-001 | — | Planned |
| REQ-08.01.02 | COMP-FUZZ-001 | — | TC-IDEM-001 | — | Planned |
| REQ-08.01.03 | COMP-PERF-001 | — | TC-PERF-001 | — | Planned |

### Yellow Paper → Blue Paper Mapping
| Yellow Paper | Blue Paper | Elements Used | Verification |
|-------------|-----------|---------------|--------------|
| YP-IR-SEMANTICS-001 | BP-IR-COMPILER-001 | THM-WF-SIR-DECIDABLE, THM-WF-GIR-DECIDABLE, THM-COMPILE-TERMINATES, ALG-COMPILE-001, DEF-SIR, DEF-GIR, DEF-WF-SIR | Unit + Lean4 Proof |
| YP-IR-SEMANTICS-001 | BP-ENGINE-001 | THM-001, DEF-001 | Unit + Proof |
| YP-IR-GEOMETRY-001 | BP-GIR-001 | THM-001, ALG-001 | Unit + Proof |
| YP-LAYOUT-KNUTHPLASS-001 | BP-LINEBREAK-001 | ALG-001, THM-001 | Integration |
| YP-LAYOUT-PAGINATION-001 | BP-PAGINATION-001 | ALG-001, THM-002 | Integration |
| YP-CONSTRAINT-CASSOWARY-001 | BP-SOLVER-001 | ALG-001, THM-001 | Unit + Proof |
| YP-MEMORY-ECS-001 | BP-ECS-001 | DEF-001, THM-001 | Unit test |
| YP-CONCURRENCY-001 | BP-THREADPOOL-001 | THM-001, LEM-001 | Integration |

### Blue Paper → Interface Mapping
| Blue Paper | Interfaces | Components |
|-----------|-----------|------------|
| BP-IR-COMPILER-001 | IF-PARSE-001, IF-VALIDATE-001, IF-COMPILE-001, IF-EMIT-001 | COMP-IR-PARSER, COMP-IR-VALIDATOR, COMP-IR-COMPILER, COMP-IR-EMITTER |

### Requirements → Test Cases
| Requirement ID | Test Category | Test Case ID | Priority |
|---------------|--------------|--------------|----------|
| REQ-08.01.02 | Idempotency | TC-IDEM-001 | Critical |
| REQ-08.01.01 | Fuzzing | TC-FUZZ-001 | Critical |
| REQ-08.01.03 | Performance | TC-PERF-001 | Critical |
| REQ-01.01.01 | Memory | TC-MEM-001 | Critical |
| REQ-01.01.02 | Memory Layout | TC-MEM-002 | High |
| REQ-01.01.03 | ECS Safety | TC-ECS-001 | High |
| REQ-01.02.01 | Concurrency | TC-THREAD-001 | High |
| REQ-02.01.01 | Geometry | TC-FIXPT-001 | Critical |
| REQ-02.02.01 | S-IR | TC-SIR-001 | High |
| REQ-03.02.01 | SIMD | TC-LB-001 | High |
| REQ-03.03.01 | Pagination | TC-PAGE-001 | High |
| REQ-04.01.01 | Security | TC-WASM-001 | Critical |
| REQ-04.01.03 | WASM Limits | TC-WASM-003 | High |
| REQ-05.02.01 | Source Mapping | TC-SRCMAP-001 | High |
| REQ-06.01.02 | Frame Budget | TC-GPU-002 | High |
| REQ-06.02.03 | Font Subsetting | TC-PDF-003 | Medium |
| REQ-07.01.01 | Tracing | TC-TRACE-001 | Low |
| REQ-01.02.02 | Lock-Free | TC-CACHE-001 | High |
| REQ-03.02.02 | Branchless | TC-LB-002 | Medium |
| REQ-05.02.02 | Reverse Mapping | TC-SRCMAP-002 | Medium |

## Status: Phase 2 In Progress — BP-IR-COMPILER-001 created, remaining Blue Papers planned
