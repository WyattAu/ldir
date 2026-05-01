# LDIR Applicable Standards

**Project:** LDIR — Low-level Document Intermediate Representation
**Classification:** Non-safety-critical software (no injury/death risk)
**Date:** 2026-04-23

> **Note:** DO-178C, ISO 26262, IEC 61508, and IEC 62304 are NOT applicable. LDIR is a typesetting engine with no safety-critical function.

---

## 1. Software Engineering Process Standards

| Standard | Clause | Applicability | Priority | Notes |
|---|---|---|---|---|
| **ISO/IEC 12207:2017** | Full standard | Software Life Cycle Processes | **Mandatory** | Governs all development activities: requirements, design, implementation, testing, maintenance. Applied via the LDIR development workflow and CI/CD pipeline. |
| **ISO/IEC 15288:2023** | Full standard | System Life Cycle Processes | **Mandatory** | Governs system-level activities including architectural design, integration, and technical management. Applied to LDIR as a system composed of multiple crates and frontends/backends. |
| **IEEE 1016-2009** | Full standard | Software Design Descriptions | **Mandatory** | Template for Blue Papers (architectural design documents). All `.specs/02_architecture/` documents shall follow this standard's structure. |
| **IEEE 829-2008** | Full standard | Software Test Documentation | **Mandatory** | Governs test plans, test cases, test logs, and test summary reports. Applied to the golden master suite, fuzzing corpus, and benchmark regression tests. |

## 2. Security & Privacy Standards

| Standard | Clause | Applicability | Priority | Notes |
|---|---|---|---|---|
| **NIST SP 800-53 Rev 5** | AC-3, AC-4, SC-7, SC-39, SI-10, SI-16 | Security Controls | **Mandatory** | WASM sandboxing (AC-3: enforcement, SC-7: boundary protection), fuel limits (SC-39: resource availability), input validation (SI-10: input validation), fuzzing (SI-16: memory protection). |
| **ISO/IEC 27001:2022** | A.8.1, A.8.25 | Information Security Management | **Recommended** | Applied to the WASM ABI boundary and C ABI surface. Secure development practices (A.8.25) for the sandboxing architecture. |
| **ISO/IEC 27034-1:2011** | Full standard | Application Security | **Recommended** | Guides secure design of the WASM plugin system and C ABI embedding interface. |
| **CWE/SANS Top 25** | Full list | Common Weakness Enumeration | **Mandatory** | All Rust code shall be free of Top 25 weaknesses. Rust's ownership model mitigates many; focus on logic errors, integer overflow in fixed-point, and WASM ABI boundary issues. |

## 3. Document & Font Format Standards

| Standard | Clause | Applicability | Priority | Notes |
|---|---|---|---|---|
| **ISO 32000-2:2020** | Full standard | PDF 2.0 | **Mandatory** | Primary output format for `ldir-pdf`. All generated PDFs must conform to this standard. |
| **ISO 19005-4:2020** | Full standard | PDF/A-4 (Archival) | **Mandatory** | `ldir-pdf` shall produce PDF/A-4 compliant output for long-term archival. |
| **ISO 15930-8:2020** | Full standard | PDF/X-8 (Print) | **Reference** | Future consideration for print-optimized output. |
| **ISO 14496-22** | Full standard | OpenType Font Format | **Mandatory** | Font parsing and subsetting in `ldir-pdf`. Covers TrueType outlines (glyf), CFF outlines, GSUB/GPOS tables. |
| **ISO/IEC 10646:2020** | Full standard | Universal Coded Character Set (Unicode) | **Mandatory** | Text encoding throughout the pipeline. Unicode normalization for input, grapheme cluster boundaries for line-breaking. |
| **ISO 15924** | Full standard | Script Codes | **Reference** | Script identification for shaping dispatch (Latin, CJK, Arabic, Indic, etc.). |

## 4. Markup & Input Standards

| Standard | Clause | Applicability | Priority | Notes |
|---|---|---|---|---|
| **CommonMark 0.31** | Full specification | Markdown | **Mandatory** | `ldir-md` frontend must pass the CommonMark conformance test suite. |
| **ISO 8879:1986** | SGML concepts | TeX macro concepts | **Reference** | Historical context for TeX's markup model. Not directly applicable. |
| **LaTeX2e** | Unversioned | LaTeX input | **Mandatory** | `ldir-tex` frontend target. MVP scope: `amsmath`, `geometry`, `tabular`. |

## 5. Execution Environment Standards

| Standard | Clause | Applicability | Priority | Notes |
|---|---|---|---|---|
| **WebAssembly Core Spec 2.0** | Full specification | WASM Runtime | **Mandatory** | WASM plugin sandboxing via `wasmtime`. Covers module format, validation, instantiation, and execution semantics. |
| **WASI Preview 2** | Full specification | WASM System Interface | **Recommended** | WASI for file system and I/O access by WASM plugins, if needed beyond the zero-copy ABI. |
| **ECMA-262** | N/A | JavaScript (browser host) | **Reference** | Relevant only for the WASM/WebGL backend's browser integration. |

## 6. Programming & Quality Standards

| Standard | Clause | Applicability | Priority | Notes |
|---|---|---|---|---|
| **ISO/IEC 14882:2024** | N/A | C++ (LLVM dependency) | **Reference** | LLVM toolchain used for JIT compilation if needed. Not directly relevant to Rust codebase. |
| **Rust API Guidelines** | Full guidelines | Rust crate design | **Mandatory** | All public crate APIs shall follow the Rust API Guidelines for naming, documentation, and ergonomics. |
| **RFC 2119** | Full document | Requirements language | **Mandatory** | All specification documents use RFC 2119 keywords (SHALL, SHOULD, MAY) consistently. |
| **SemVer 2.0.0** | Full specification | Versioning | **Mandatory** | All crates and the WASM ABI version follow Semantic Versioning. |

## 7. Compliance Summary

| Category | Mandatory | Recommended | Reference | Total |
|---|---|---|---|---|
| Process (12207, 15288, 1016, 829) | 4 | 0 | 0 | 4 |
| Security (800-53, 27001, CWE) | 2 | 2 | 0 | 4 |
| Document Formats (PDF, OpenType, Unicode) | 5 | 0 | 1 | 6 |
| Input Standards (CommonMark, LaTeX) | 2 | 0 | 1 | 3 |
| Execution (WASM, WASI) | 1 | 1 | 1 | 3 |
| Quality (Rust, RFC 2119, SemVer) | 3 | 0 | 1 | 4 |
| **Total** | **17** | **3** | **4** | **24** |
