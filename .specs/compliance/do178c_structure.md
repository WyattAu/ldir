# DO-178C Compliance Structure

## Document Information

| Field | Value |
|-------|-------|
| Document ID | COMP-DO178C-001 |
| Version | 0.1.0 |
| Status | Draft |
| Date | 2026-05-31 |
| Author | ldir Team |

## 1. Scope

This document outlines the DO-178C compliance structure assessment for the
ldir document compiler.

Note: ldir is NOT airborne software and has no direct role in aircraft systems.
This assessment evaluates whether ldir could serve as a documentation tool
within an airborne software development toolchain (e.g., generating DO-178C
life-cycle documents, traceability matrices, or design descriptions).

## 2. Software Level Determination

| Aspect | Assessment |
|--------|-----------|
| Direct aircraft system impact | None (document compiler only) |
| Potential use in airborne chain | Indirect (generates documentation artifacts) |
| Recommended DAL | Level E (no safety impact) |

## 3. Compliance Gap Analysis

### 3.1 Software Planning (DO-178C Section 4)

| Requirement | Status | Gap | Mitigation |
|-------------|--------|-----|------------|
| Software development plan | Partial | No formal SDP | Not applicable for DAL E |
| Software verification plan | Partial | CI covers most objectives | Not applicable for DAL E |
| Software configuration management | Met | Git, Cargo.lock | -- |
| Software quality assurance | Met | CI, clippy, tests, code review | -- |

### 3.2 Software Development (DO-178C Section 5)

| Requirement | Status | Gap | Mitigation |
|-------------|--------|-----|------------|
| Software requirements | Met | requirements.md, ROADMAP.md | -- |
| Software design | Met | Blue Papers (IEEE 1016), IR spec | -- |
| Software coding | Met | Rust, clippy -D warnings, rustfmt | -- |
| Traceability | Met | TRACEABILITY_MATRIX.md | -- |

### 3.3 Software Verification (DO-178C Section 6)

| Requirement | Status | Gap | Mitigation |
|-------------|--------|-----|------------|
| Reviews | Met | PR-based code review | -- |
| Test coverage | Met | 2,055 tests across all crates | -- |
| Structural coverage | Partial | No MC/DC analysis | Not required for DAL E |
| Robustness testing | Partial | Adversarial test plan exists | Expand coverage |

### 3.4 Configuration Management (DO-178C Section 7)

| Requirement | Status | Gap | Mitigation |
|-------------|--------|-----|------------|
| Configuration identification | Met | Git tags, Cargo.toml | -- |
| Baselines | Met | Release branches | -- |
| Change tracking | Met | Commit history, PRs | -- |
| Archive and retrieval | Met | GitHub repository | -- |

### 3.5 Quality Assurance (DO-178C Section 8)

| Requirement | Status | Gap | Mitigation |
|-------------|--------|-----|------------|
| QA process | Met | CI pipeline, clippy, tests | -- |
| Compliance monitoring | Partial | No formal QA sign-off | N/A for DAL E |
| Problem reporting | Met | GitHub issues | -- |

## 4. Safety Arguments

### 4.1 Language Selection

Rust provides compile-time guarantees that address several DO-178C concerns:
- Memory safety without runtime overhead
- No undefined behavior (in safe Rust)
- Ownership model prevents data races
- Strong type system catches errors at compile time

### 4.2 Verification Evidence

- 2,055 unit and integration tests
- CI pipeline with 13 jobs across 3 platforms (Linux, macOS, Windows)
- Lean4 formal proofs for IR well-formedness
- Fuzz testing via cargo-fuzz

### 4.3 Documentation Evidence

- Requirements traced to implementation via TRACEABILITY_MATRIX.md
- Architecture documented in Blue Papers (IEEE 1016 format)
- Design decisions recorded in ADRs

## 5. Recommendations

1. Current practices are sufficient for DAL E (documentation tool usage)
2. If used in DAL C or higher toolchains, consider formal test case mapping
3. Document tool qualification if used as a verification tool per DO-178C Section 12
4. Maintain traceability matrix as requirements evolve

## 6. Conclusion

ldir meets all applicable DO-178C objectives for DAL E usage as a documentation
generation tool. It is not airborne software and has no direct flight safety
impact. For use in higher DAL environments, formal tool qualification and
additional verification evidence would be required per DO-178C Section 12.
