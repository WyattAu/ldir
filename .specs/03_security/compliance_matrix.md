# LDIR Security Compliance Matrix

**Document ID:** SEC-CM-001
**Version:** 1.0.0
**Status:** APPROVED
**Date:** 2026-04-23
**References:** applicable_standards.md, SEC-TM-001 (Threat Model), SEC-STP-001 (Security Test Plan)

---

## 1. NIST SP 800-53 Rev 5

| Standard | Control | Control Name | LDIR Requirement | Implementation | Test Verification | Status |
|----------|---------|-------------|-----------------|----------------|-------------------|--------|
| NIST 800-53 | AC-3 | Enforcement | REQ-11.2.1: All extensibility via WASM sandbox only | wasmtime sandbox with no native plugins; capability-based host imports | STP-002.3 (host import isolation), STP-002.4 (sandbox escape) | COMPLIANT |
| NIST 800-53 | AC-4 | Information Flow Enforcement | TM-011: WASM plugins cannot read host memory | wasmtime default sandboxing; only pointer+length passed to plugin memory; no host address exposure | STP-002.4 (sandbox escape), STP-002.5 (plugin isolation) | COMPLIANT |
| NIST 800-53 | SC-7 | Boundary Protection | Trust boundary between WASM plugins and host engine | wasmtime `StoreLimits`, `MemoryLimits`, fuel limits; minimal host imports | STP-002.1 (fuel), STP-002.2 (memory), STP-002.3 (imports) | COMPLIANT |
| NIST 800-53 | SC-39 | Process Isolation | REQ-11.2.2: WASM sandbox resource limits | Fuel: 100,000 instructions (REQ-7.3); Memory: 64MB max; Wall-clock: 5s timeout | STP-002.1, STP-002.2, STP-005.2, STP-005.3 | COMPLIANT |
| NIST 800-53 | SI-10 | Information Input Validation | REQ-3.3.4: Structured error diagnostics for malformed S-IR | COMP-IR-VALIDATOR (6 WF-SIR checks); font parser bounds checking; config schema validation | STP-001.2 (font tables), STP-003.2 (S-IR validation), STP-006.1-006.4 (path validation) | COMPLIANT |
| NIST 800-53 | SI-16 | Memory Protection | REQ-11.2.3: Continuous fuzzing; REQ-9.1: No OOB panic/segfault | cargo-fuzz on S-IR parser; AFL++ on font parser; ASan/UBSan in CI | STP-001.1 (font fuzzing), STP-003.1 (rkyv fuzzing) | COMPLIANT |
| NIST 800-53 | SA-11 | Developer Security Testing | Fuzzing, property-based testing, penetration testing | cargo-fuzz (libFuzzer), AFL++, proptest, manual WASM sandbox penetration test | All STP-001 through STP-006 | COMPLIANT |
| NIST 800-53 | SR-11 | Developer Configuration Management | Dependency pinning, Cargo.lock committed | Cargo workspace with precise version pins (REQ-10.2); Cargo.lock in VCS | CI verification of Cargo.lock integrity | PLANNED |
| NIST 800-53 | RA-5 | Vulnerability Scanning | Supply chain vulnerability detection | `cargo audit` in CI for dependency CVEs | CI job: `cargo audit` on every PR | PLANNED |
| NIST 800-53 | CM-6 | Configuration Settings | Security controls not overridable by config | Hard-coded maximums for resource limits; config cannot disable sandbox or validation | STP-005.3 (timeout), STP-006.4 (config path validation) | COMPLIANT |

---

## 2. OWASP Top 10 (2021)

| Standard | Category | Risk Description | LDIR Relevance | Mitigation | Test Verification | Status |
|----------|----------|-----------------|---------------|------------|-------------------|--------|
| OWASP A01 | Broken Access Control | Unauthorized access to resources | WASM plugin accessing host memory/files | wasmtime sandbox; no file/network host imports; capability tokens | STP-002.3, STP-002.4, STP-002.5 | COMPLIANT |
| OWASP A02 | Cryptographic Failures | Data exposure via weak crypto | Plugin IDs and communication channel integrity | Cryptographic plugin ID verification (M-007a); rkyv trusted-only model | TM-007 mitigation | PLANNED |
| OWASP A03 | Injection | Injected content in output (PDF) | TM-006: PDF content injection via font metadata or S-IR content | PDF/A-4 whitelist; string sanitization; no executable content in PDF output | STP-001.4 (font metadata) | COMPLIANT |
| OWASP A04 | Insecure Design | Missing security-by-design controls | Path traversal in font loading (TM-005) | Path canonicalization; trusted directory enforcement; null byte rejection | STP-006.1, STP-006.2, STP-006.3 | COMPLIANT |
| OWASP A05 | Security Misconfiguration | Default insecure settings | Config overriding security limits (TM-009) | Hard-coded security maximums; strict config schema; unknown key rejection | STP-006.4 | COMPLIANT |
| OWASP A06 | Vulnerable Components | Using components with known CVEs | Rust crate dependencies (wasmtime, rkyv, etc.) | `cargo audit` in CI; pinned versions (REQ-10.2) | CI: `cargo audit` | PLANNED |
| OWASP A07 | Auth Failures | Authentication bypass | WASM plugin identity spoofing (TM-007) | Plugin ID verification at load time; capability-based permissions | STP-002.5 | PLANNED |
| OWASP A08 | Data Integrity Failures | Deserialization of untrusted data | rkyv deserialization of potentially modified bytes (TM-003) | Trusted-only assumption documented; COMP-IR-VALIDATOR defense-in-depth; bytecheck available | STP-003.1, STP-003.2 | COMPLIANT |
| OWASP A09 | Logging Failures | Insufficient security event logging | WASM plugin actions not auditable (TM-010) | Plugin load/unload logging; fuel consumption telemetry; structured error on termination | TM-010 mitigation | PLANNED |
| OWASP A10 | SSRF | Server-side request forgery | N/A — LDIR has no network surface | Not applicable | N/A | N/A |

---

## 3. CWE/SANS Top 25

| CWE ID | Weakness | LDIR Exposure | Mitigation | Status |
|--------|----------|--------------|------------|--------|
| CWE-119 | Buffer Overflow (Memory Corruption) | Font parsing (TM-001), rkyv deserialization (TM-003) | Rust ownership model; bounds checking in font parser; COMP-IR-VALIDATOR | MITIGATED |
| CWE-120 | Classic Buffer Overflow | Font binary parsing | Rust `&[u8]` slicing with bounds checks; no raw pointer arithmetic | MITIGATED |
| CWE-125 | Out-of-Bounds Read | Font table parsing, S-IR payload access | rkyv archive validation; table offset bounds checks (STP-001.2) | MITIGATED |
| CWE-190 | Integer Overflow | fp26_6 arithmetic (TM-004), font table sizes | Saturating/clamping arithmetic; explicit wrapping + validation (STP-004) | MITIGATED |
| CWE-20 | Improper Input Validation | All entry points | COMP-IR-VALIDATOR (S-IR); FontParseLimits (fonts); config schema (config) | MITIGATED |
| CWE-22 | Path Traversal | Font file loading (TM-005) | Path canonicalization; trusted directory enforcement (STP-006) | MITIGATED |
| CWE-787 | Out-of-Bounds Write | Font parser, WASM linear memory | Rust borrow checker; wasmtime memory limits | MITIGATED |
| CWE-502 | Deserialization of Untrusted Data | rkyv S-IR deserialization (TM-003) | Trusted-only assumption; bytecheck available; validator defense-in-depth | MITIGATED |
| CWE-400 | Uncontrolled Resource Consumption | Font OOM (TM-001), WASM exhaustion (TM-002), deep nesting (TM-008) | Memory caps, fuel limits, nesting depth limits, timeouts | MITIGATED |
| CWE-862 | Missing Authorization | WASM plugin privilege escalation (TM-015) | Capability-based host imports; minimal import surface | MITIGATED |

---

## 4. IEC 62443 (Industrial Security)

| Standard | Clause | Assessment | Justification |
|----------|--------|------------|---------------|
| IEC 62443-1-1 | General | **NOT APPLICABLE** | LDIR is a document typesetting library, not an industrial control system. No OT/IACS context. |
| IEC 62443-3-3 | System Security Requirements | **NOT APPLICABLE** | No industrial control functions, no PLC/SCADA integration. |
| IEC 62443-4-1 | Component Security Requirements | **REFERENCE** | Development lifecycle practices (secure coding, testing) align but formal IEC 62443 compliance is not required. |

---

## 5. ISO/IEC 27001:2022 (Subset)

| Standard | Control | Control Name | LDIR Implementation | Status |
|----------|---------|-------------|-------------------|--------|
| ISO 27001 A.8.25 | Secure Development Lifecycle | Secure coding practices | Rust ownership model; no unsafe except FFI boundary; fuzzing in CI | COMPLIANT |
| ISO 27001 A.8.1 | Asset Inventory | Security-relevant asset tracking | WASM plugin registry with cryptographic IDs; font trust management | PLANNED |

---

## 6. WebAssembly Core Specification 2.0

| Standard | Clause | Requirement | LDIR Implementation | Status |
|----------|--------|-------------|-------------------|--------|
| WASM Core 2.0 | Section 2.6 | Memory Safety | wasmtime enforces linear memory bounds; no shared memory between plugins | COMPLIANT |
| WASM Core 2.0 | Section 2.7 | Control Flow Integrity | Structured control flow (no arbitrary jumps); wasmtime validation | COMPLIANT |
| WASM Core 2.0 | Section 2.10 | Host Functions | Minimal host imports (read-only S-IR, append-only G-IR); no file/network access | COMPLIANT |
| WASM Core 2.0 | Section 7 | Validation | All WASM modules validated by wasmtime before instantiation | COMPLIANT |

---

## 7. Compliance Summary

| Standard | Total Controls | Compliant | Planned | Not Applicable | Compliance % |
|----------|---------------|-----------|---------|----------------|-------------|
| NIST SP 800-53 | 10 | 7 | 3 | 0 | 70% |
| OWASP Top 10 | 10 | 6 | 3 | 1 | 60% (excl. N/A) |
| CWE/SANS Top 25 | 10 | 10 | 0 | 0 | 100% |
| IEC 62443 | 3 | 0 | 0 | 3 | N/A |
| ISO 27001 | 2 | 1 | 1 | 0 | 50% |
| WASM Core 2.0 | 4 | 4 | 0 | 0 | 100% |
| **Total** | **39** | **28** | **7** | **4** | **72%** |

---

## 8. Remediation Plan

| Priority | Gap | Standard | Control | Remediation Action | Target Phase |
|----------|-----|----------|---------|-------------------|-------------|
| P1 | Dependency vulnerability scanning | NIST RA-5 | Vulnerability Scanning | Add `cargo audit` to CI pipeline; fail on known CVEs | Phase 4 |
| P1 | Plugin identity verification | OWASP A07 | Auth Failures | Implement cryptographic plugin ID signing and verification | Phase 4 |
| P1 | Security event logging | OWASP A09 | Logging Failures | Add structured audit logging for WASM plugin lifecycle events | Phase 4 |
| P2 | Developer config management | NIST SR-11 | Config Management | Verify Cargo.lock integrity in CI; pin all transitive dependencies | Phase 4 |
| P2 | Asset inventory | ISO A.8.1 | Asset Inventory | Implement plugin registry with signed metadata | Phase 5 |
| P3 | Cryptographic plugin IDs | OWASP A02 | Crypto Failures | Define plugin signing key management; hash-based verification for MVP | Phase 5 |

---

## 9. Cross-Reference: Threat → Standard → Test

| Threat | NIST Control | OWASP Category | Security Test |
|--------|-------------|----------------|---------------|
| TM-001 (Font OOM) | SI-16, SI-10 | A05, A08 | STP-001.1, STP-001.2, STP-001.3, STP-005.1 |
| TM-002 (WASM exhaustion) | SC-39, SC-7 | A01 | STP-002.1, STP-002.2, STP-005.2 |
| TM-003 (rkyv untrusted) | SI-10, SI-16 | A08 | STP-003.1, STP-003.2 |
| TM-004 (fp26_6 overflow) | SI-10 | A08 | STP-004.1, STP-004.2, STP-004.3 |
| TM-005 (Path traversal) | SI-10 | A04 | STP-006.1, STP-006.2, STP-006.3, STP-006.4 |
| TM-006 (PDF injection) | SI-10 | A03 | STP-001.4 |
| TM-007 (Channel spoofing) | AC-4 | A07 | STP-002.5 |
| TM-008 (Deep nesting DoS) | SC-39, SI-10 | A05 | STP-003.3, STP-005.1, STP-005.3 |
| TM-011 (Info disclosure) | AC-4 | A01 | STP-002.4 |
| TM-015 (Privilege escalation) | AC-3, SC-7 | A01 | STP-002.3, STP-002.4 |

---

*End of SEC-CM-001 v1.0.0*
