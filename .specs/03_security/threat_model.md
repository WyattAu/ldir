# LDIR Threat Model

**Document ID:** SEC-TM-001
**Version:** 1.0.0
**Status:** APPROVED
**Date:** 2026-04-23
**Methodology:** STRIDE (Microsoft)
**Classification:** Non-safety-critical (per applicable_standards.md)

---

## 1. Overview

This threat model applies STRIDE analysis to all entry points and components of the LDIR typesetting engine. LDIR processes untrusted input (font files, configuration), semi-trusted input (S-IR via rkyv, WASM plugins), and produces output (PDF, G-IR command buffers).

### 1.1 Trust Boundaries

```
[Untrusted]                    [Semi-Trusted]                   [Trusted]
Font Files (TTF/OTF)  ──────►  ldir-core Engine  ◄────  S-IR (rkyv, self-produced)
Config (YAML/TOML)    ──────►  ldir-core Engine  ◄────  Frontends (ldir-tex, ldir-md)
WASM Plugins          ──────►  wasmtime Sandbox ◄────  G-IR (internal format)
                              ldir-pdf Backend
                              ldir-vello Backend
```

### 1.2 Data Flow Summary

| Flow | Source | Sink | Trust Level | Format |
|------|--------|------|-------------|--------|
| F-001 | Font file | Font parser | Untrusted | TTF/OTF binary |
| F-002 | S-IR bytes | rkyv deserializer | Trusted (assumption) | rkyv binary |
| F-003 | WASM module | wasmtime runtime | Untrusted | .wasm binary |
| F-004 | G-IR commands | PDF emitter | Trusted | Internal binary |
| F-005 | Config file | CLI parser | Untrusted | YAML/TOML text |
| F-006 | C ABI calls | Host application | Trusted | C function calls |

### 1.3 Entry Point Inventory

| ID | Entry Point | Component | Input Format | Trust Level |
|----|-------------|-----------|--------------|-------------|
| EP-001 | S-IR deserialization | COMP-IR-PARSER | rkyv binary | Trusted |
| EP-002 | Font file loading | ldir-pdf / ldir-core | TTF/OTF | Untrusted |
| EP-003 | WASM plugin execution | wasmtime sandbox | .wasm | Untrusted |
| EP-004 | Configuration parsing | ldc (CLI) | YAML/TOML | Untrusted |
| EP-005 | PDF output generation | ldir-pdf | G-IR commands | N/A (output) |

---

## 2. STRIDE Analysis

### 2.1 Spoofing Threats

| ID | Component | Threat Type | Threat Description | Impact | Mitigation | Priority |
|----|-----------|-------------|--------------------|--------|------------|----------|
| TM-007 | WASM Plugin Host | Spoofing | A malicious WASM plugin attempts to impersonate another plugin or spoof its identity in the host communication channel to gain elevated access to S-IR memory regions | Plugin could read/modify S-IR data beyond its granted scope, breaking document integrity | M-007a: Assign cryptographic plugin IDs verified at load time. M-007b: Enforce per-plugin memory region access via wasmtime `Memory::data` scoping. M-007c: Validate all host-import return values against expected types before passing to plugin. M-007d: Implement capability-based permissions per plugin (read-only S-IR view, write-only G-IR buffer). | P2 — Medium |

### 2.2 Tampering Threats

| ID | Component | Threat Type | Threat Description | Impact | Mitigation | Priority |
|----|-----------|-------------|--------------------|--------|------------|----------|
| TM-006 | PDF Emitter (ldir-pdf) | Tampering | Attacker crafts S-IR content or font metadata that injects malicious content into the generated PDF stream (e.g., JavaScript actions, embedded URLs, malformed cross-reference table) | PDF consumer may execute injected content, leaking information or spreading malware | M-006a: Emit only PDF/A-4 compliant objects; strip all executable content (JavaScript, Launch actions, SubmitForm). M-006b: Validate all string content written to PDF streams; escape or reject control characters. M-006c: Use a whitelist of permitted PDF dictionary keys and object types. M-006d: Validate font metadata (names, copyright strings) for embedded content injection. | P1 — High |
| TM-003 | rkyv Deserializer | Tampering | Although S-IR is currently assumed trusted (self-produced by frontends), if rkyv bytes are modified post-serialization or sourced from an untrusted party, the zero-copy deserializer could produce a corrupted SIRDocument with invalid invariants | Memory corruption, panics, segfaults, or incorrect G-IR output | M-003a: Document the trust assumption: rkyv input MUST be produced by a trusted frontend. M-003b: COMP-IR-VALIDATOR serves as a defense-in-depth layer catching invalid S-IR regardless of source. M-003c: Enable rkyv's `bytecheck` validation on deserialization if untrusted input is ever accepted. M-003d: Never accept raw rkyv bytes from network or user upload without validation. | P2 — Medium |
| TM-009 | Configuration Parser | Tampering | Malicious configuration file specifies paths to unauthorized font files, excessive resource limits, or disables security controls | Engine loads untrusted resources, bypasses sandbox limits | M-009a: Whitelist permitted configuration keys. M-009b: Validate all file paths in configuration against a trusted font directory. M-009c: Enforce hard-coded maximum values for resource limits (cannot be overridden by config). M-009d: Parse configuration with a strict schema validator (reject unknown keys). | P2 — Medium |

### 2.3 Repudiation Threats

| ID | Component | Threat Type | Threat Description | Impact | Mitigation | Priority |
|----|-----------|-------------|--------------------|--------|------------|----------|
| TM-010 | WASM Plugin System | Repudiation | A WASM plugin performs a denial-of-service action or corrupts layout data, but there is no audit trail to identify which plugin caused the failure | Inability to diagnose or attribute security incidents | M-010a: Log all WASM plugin load/unload events with plugin ID and hash. M-010b: Record fuel consumption per plugin invocation in telemetry. M-010c: Emit structured error diagnostics when a plugin is terminated (REQ-7.3). | P3 — Low |

### 2.4 Information Disclosure Threats

| ID | Component | Threat Type | Threat Description | Impact | Mitigation | Priority |
|----|-----------|-------------|--------------------|--------|------------|----------|
| TM-011 | WASM Sandbox | Information Disclosure | WASM plugin attempts to read host memory beyond its allocated linear memory region, leaking S-IR document content or other sensitive data | Document content confidentiality breach | M-011a: Use wasmtime's default sandboxing (no host imports that expose memory). M-011b: Only pass pointers/lengths to plugin-owned memory; never expose host addresses. M-011c: Enable wasmtime's `MemoryCreation` and `TableCreation` limits. M-011d: Disable all WASI capabilities unless explicitly needed. | P1 — High |
| TM-012 | Telemetry Subsystem | Information Disclosure | Tracing data exported via Chrome Trace Format may contain document content, file paths, or font names in span metadata | Sensitive document metadata leaked via trace files | M-012a: Sanitize trace spans to exclude document content. M-012b: Configure `tracing` filters to omit payload data from exported traces. M-012c: Document trace content policy in user-facing documentation. | P3 — Low |

### 2.5 Denial of Service Threats

| ID | Component | Threat Type | Threat Description | Impact | Mitigation | Priority |
|----|-----------|-------------|--------------------|--------|------------|----------|
| TM-001 | Font Parser | Denial of Service | Maliciously crafted TTF/OTF font file with inflated glyph tables (e.g., glyf table claiming millions of glyphs) or recursive table references causes OOM or excessive CPU during font parsing and subsetting | Engine crash (OOM), system resource exhaustion, build pipeline stall | M-001a: Set hard memory limit for font parsing (e.g., 512MB per font). M-001b: Validate table directory sizes against file size before allocation. M-001c: Cap glyph count per font (e.g., 65536 max). M-001d: Reject fonts where table sizes exceed remaining file data. M-001e: Implement timeout for font parsing operations. | P1 — High |
| TM-002 | WASM Sandbox | Denial of Service | Malicious WASM plugin consumes excessive CPU (infinite loop) or memory (unbounded allocation) within the sandbox, degrading overall engine performance | Engine hangs, other plugins starved, user experience degraded | M-002a: Enforce fuel limits (REQ-7.3: 100,000 instruction limit). M-002b: Configure wasmtime memory limits (e.g., 64MB max linear memory). M-002c: Configure wall-clock timeout per plugin invocation (e.g., 5 seconds). M-002d: Use wasmtime's `StoreLimits` to constrain memory growth. M-002e: Periodically yield to the host's async executor to detect hangs. | P1 — High |
| TM-004 | fp26_6 Arithmetic | Denial of Service | Integer overflow in 26.6 fixed-point arithmetic causes incorrect coordinate calculations, potentially leading to infinite loops in the Knuth-Plass line breaker (penalty calculations never converge) or incorrect page breaks | Incorrect layout output, potential infinite compilation, assertion failures | M-004a: Use Rust's `i32` wrapping semantics explicitly with `.wrapping_add()/.wrapping_mul()` and validate results. M-004b: Clamp results to fp26_6 representable range [-33554432.0, 33554431.984375] after each arithmetic operation. M-004c: Add saturating arithmetic helpers for all fp26_6 operations. M-004d: Emit `ERR-COMP-001` warning on overflow (per IF-COMPILE-001). M-004e: Formal verification of fixed-point closure property (AX-005 in Lean4). | P1 — High |
| TM-008 | S-IR Compiler | Denial of Service | Deeply nested document structure (thousands of nested blocks via PUSH_BLOCK) causes stack overflow in DFS traversal or excessive memory in the coordinate stack, or triggers combinatorial explosion in pagination DAG | Stack overflow (panic/segfault), OOM, compilation timeout | M-008a: Enforce maximum nesting depth (e.g., 256 levels) in COMP-IR-VALIDATOR. M-008b: Use explicit arena-allocated stack (not recursion) for DFS traversal. M-008c: Apply Branch-and-Bound pruning in pagination to prevent combinatorial explosion (REQ-4.3.3.2). M-008d: Cap maximum entity count per document (e.g., 10M nodes). M-008e: Implement fuel/step counter in compiler to detect pathological cases. | P1 — High |
| TM-013 | Layout Engine | Denial of Service | Extremely wide content (e.g., single word wider than page) or pathological constraint configurations cause the Cassowary constraint solver to oscillate without converging | Compilation hangs, excessive CPU usage | M-013a: Set iteration limit for Cassowary solver (e.g., 1000 iterations). M-013b: Detect oscillation via delta tracking and bail with error diagnostic. M-013c: Use fixed-point arithmetic in solver to avoid floating-point instability. | P2 — Medium |

### 2.6 Elevation of Privilege Threats

| ID | Component | Threat Type | Threat Description | Impact | Mitigation | Priority |
|----|-----------|-------------|--------------------|--------|------------|----------|
| TM-014 | C ABI Interface | Elevation of Privilege | Host application passes malicious S-IR data or malformed font file paths through the C ABI, exploiting the boundary between managed Rust and unmanaged C code | Memory corruption in host application, arbitrary code execution in host process | M-014a: Validate all inputs at C ABI boundary (re-run COMP-IR-VALIDATOR). M-014b: Use `#[no_mangle]` + `unsafe` boundary with explicit safety documentation. M-014c: Sanitize all file paths received via C ABI. M-014d: Return error codes for invalid input; never panic across FFI boundary. | P2 — Medium |
| TM-015 | WASM Host Imports | Elevation of Privilege | Malicious WASM plugin invokes host-imported functions with crafted arguments to perform operations beyond its intended capability (e.g., requesting arbitrary file I/O via host imports) | Unauthorized file access, privilege escalation within the host process | M-015a: Minimize host imports: provide only `ldir_sir_read` (read-only S-IR view) and `ldir_gir_write` (append-only G-IR buffer). M-015b: Validate all arguments passed from WASM to host imports. M-015c: Never expose file system, network, or environment host imports to plugins. M-015d: Use capability tokens: plugins must present a valid token to invoke host imports. | P1 — High |
| TM-005 | Font File Loader | Elevation of Privilege | Font file path references use path traversal (e.g., `../../etc/passwd`, `..\\Windows\\System32\\config`) to read arbitrary files from the host filesystem | Arbitrary file read, sensitive data exposure, potential code execution if binary file is interpreted as font | M-005a: Canonicalize all font file paths before opening. M-005b: Verify resolved path is within the trusted font directory (or user-specified allowed directories). M-005c: Reject paths containing `..` segments before canonicalization. M-005d: Use platform-specific path validation (no symbolic link following outside trusted dirs). M-005e: Log all font file access attempts. | P1 — High |

---

## 3. Threat Summary

### 3.1 Priority Distribution

| Priority | Count | Threat IDs |
|----------|-------|------------|
| P1 — High | 7 | TM-001, TM-002, TM-004, TM-005, TM-006, TM-008, TM-011, TM-015 |
| P2 — Medium | 4 | TM-003, TM-007, TM-009, TM-013, TM-014 |
| P3 — Low | 2 | TM-010, TM-012 |
| **Total** | **15** | |

### 3.2 Component Threat Density

| Component | Threat Count | Highest Priority |
|-----------|-------------|-----------------|
| WASM Plugin System | 4 | P1 |
| Font Parser/Loader | 2 | P1 |
| S-IR Compiler | 2 | P1 |
| PDF Emitter | 1 | P1 |
| rkyv Deserializer | 1 | P2 |
| C ABI Interface | 1 | P2 |
| Configuration Parser | 1 | P2 |
| Telemetry | 1 | P3 |

### 3.3 Attack Surface Ranking

1. **WASM Plugin System** — Untrusted code execution, highest attack surface
2. **Font File Parsing** — Untrusted binary input, complex format
3. **S-IR / Compiler** — Trusted-only assumption must be enforced; deeply nested input risks
4. **PDF Output** — Output injection via crafted document content
5. **C ABI** — FFI boundary between Rust and unmanaged languages

---

## 4. Assumptions & Limitations

### 4.1 Trust Assumptions

| ID | Assumption | Risk if Violated | Mitigating Control |
|----|-----------|-----------------|-------------------|
| TA-001 | S-IR input is produced by trusted frontends (ldir-tex, ldir-md) | TM-003: corrupted S-IR causes undefined behavior | COMP-IR-VALIDATOR defense-in-depth |
| TA-002 | Font files are obtained from legitimate sources | TM-001: malicious font causes OOM | Font parsing limits (M-001a-e) |
| TA-003 | Host application using C ABI is trusted | TM-014: malicious host corrupts engine | FFI boundary validation |
| TA-004 | wasmtime runtime is correctly configured | TM-002, TM-011, TM-015: sandbox bypass | Regular wasmtime security updates, minimal host imports |

### 4.2 Out of Scope

- Network-based attacks (LDIR is a local processing engine, no network surface)
- Side-channel attacks (timing attacks on deterministic layout — mitigated by constant-time algorithms where possible)
- Supply chain attacks (dependency vulnerability management is a separate concern)

---

## 5. Cross-Reference to Requirements

| Threat ID | Related REQ | Related Security Test |
|-----------|-------------|----------------------|
| TM-001 | REQ-6.2.4 (font parsing), REQ-11.2.3 (fuzzing) | STP-001 |
| TM-002 | REQ-7.3 (fuel limits), REQ-11.2.2 (resource limits) | STP-002, STP-005 |
| TM-003 | REQ-3.1.5 (rkyv), REQ-3.3.4 (error diagnostics) | STP-003 |
| TM-004 | REQ-3.2.5 (26.6 format), REQ-3.2.6 (range) | STP-004 |
| TM-005 | REQ-6.2.4 (font loading) | STP-006 |
| TM-006 | REQ-6.2.1 (PDF/A-4) | STP-001 (font metadata) |
| TM-007 | REQ-7.1 (WASM sandbox) | STP-002 |
| TM-008 | REQ-4.3.3.2 (Branch-and-Bound), REQ-9.1 (fuzzing) | STP-003 |
| TM-015 | REQ-7.1, REQ-11.2.1 (sandbox only) | STP-002 |

---

*End of SEC-TM-001 v1.0.0*
