# LDIR Standard Conflicts Register

| ID | Standard 1 | Standard 2 | Nature of Conflict | Resolution | ADR Reference | Status |
|----|-----------|-----------|-------------------|------------|---------------|--------|
| CONF-001 | IEEE 754 (FP) | LDIR Determinism | IEEE 754 allows different rounding across platforms | Use 26.6 fixed-point for all geometry; Cassowary uses fixed-point internally | ADR-003 | Resolved |
| CONF-002 | GPU Rendering | Cross-platform Determinism | GPU FP is non-deterministic across vendors | Determinism guaranteed at G-IR level only; rasterization is display concern | ADR-004 | Resolved |
| CONF-003 | WASM Spec | Zero-Copy ABI | WASM linear memory model requires copy for some data | Host passes pointer+length to mmap'd S-IR; WASM guest reads directly from host memory | ADR-005 | Resolved |
| CONF-004 | Cassowary Algorithm | Fixed-Point Arithmetic | Original Cassowary uses floating-point | Adapt solver to fixed-point with documented error bounds | ADR-006 | Resolved |
| CONF-005 | TeX Compatibility | Formal Verification | TeX has undefined behavior in edge cases | Formal spec defines LDIR behavior; TeX compatibility is aspirational, not formally verified | ADR-007 | Resolved |
