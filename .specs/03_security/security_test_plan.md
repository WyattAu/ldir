# LDIR Security Test Plan

**Document ID:** SEC-STP-001
**Version:** 1.0.0
**Status:** APPROVED
**Date:** 2026-04-23
**References:** SEC-TM-001 (Threat Model), REQ-9.1 (Continuous Fuzzing), REQ-11.2 (Security NFRs)
**Test Framework:** Rust (cargo test, cargo-fuzz, proptest), AFL++ (font fuzzing), wasmtime test utilities

---

## 1. Test Strategy

### 1.1 Objectives

1. Verify all P1 and P2 threat mitigations from SEC-TM-001 are effective
2. Establish continuous security testing in CI (REQ-9.1, REQ-11.2.3)
3. Validate resource limit enforcement across all untrusted input paths
4. Confirm WASM sandbox integrity and no privilege escalation

### 1.2 Test Categories

| Category | Tool | Frequency | Target |
|----------|------|-----------|--------|
| Unit Tests | `cargo test` | Every commit | Individual functions, validators |
| Property-Based Tests | `proptest` | Every commit | Arithmetic, serialization |
| Fuzzing | `cargo-fuzz` (libFuzzer) | Nightly CI | S-IR parser, rkyv deserialization |
| Font Fuzzing | AFL++ with custom harness | Nightly CI | TTF/OTF font parser |
| Integration Tests | `cargo test -- --ignored` | Every PR | WASM sandbox, resource limits |
| Manual Penetration | Manual | Per release | WASM sandbox escape, C ABI abuse |

### 1.3 Environment Requirements

| Requirement | Specification |
|-------------|--------------|
| Rust toolchain | Stable + nightly (for cargo-fuzz) |
| AFL++ | v4.x with persistent mode and deferred forkserver |
| wasmtime | Latest stable with all security patches |
| Memory limits (CI) | 2GB RSS cap per test process |
| Time limits (CI) | 5 minutes per fuzz corpus, 30 seconds per unit test |
| Sanitizers | AddressSanitizer (ASan), MemorySanitizer (MSan), UndefinedBehaviorSanitizer (UBSan) |

---

## 2. Test Cases

### 2.1 STP-001: Font Fuzzing

**Threat:** TM-001 (Malicious font OOM), TM-006 (PDF content injection via font metadata)
**Priority:** P1
**Type:** Fuzzing (AFL++ / libFuzzer)

#### STP-001.1: Font Parser Fuzzing Harness

| Attribute | Value |
|-----------|-------|
| Test ID | STP-001.1 |
| Target | TTF/OTF font parser (table directory parsing, glyf table, CFF outlines, GSUB/GPOS) |
| Tool | AFL++ with persistent mode, or libFuzzer via cargo-fuzz |
| Input | Mutated TTF/OTF binary files |
| Corpus | 50+ seed fonts (OpenType reference fonts, minimal valid fonts, edge cases) |
| Duration | 8 hours continuous |
| Pass Criteria | No OOM, no panic, no segfault, no timeout after 10 seconds per input |
| Coverage Target | > 90% line coverage of font parsing module |

**Fuzz harness pseudo-code:**
```rust
fn fuzz_font(data: &[u8]) {
    let result = std::panic::catch_unwind(|| {
        let _ = FontParser::parse(data, FontParseLimits {
            max_glyph_count: 65536,
            max_memory_bytes: 512 * 1024 * 1024,
            max_table_count: 100,
        });
    });
    assert!(result.is_ok(), "Font parser panicked on input");
}
```

#### STP-001.2: Font Table Size Validation

| Attribute | Value |
|-----------|-------|
| Test ID | STP-001.2 |
| Target | Table directory size validation |
| Type | Unit test |
| Description | Verify that fonts with table offset + length > file size are rejected |
| Test Vectors | TV-FONT-001: table offset beyond EOF, TV-FONT-002: table length overflows u32, TV-FONT-003: overlapping tables |
| Expected Result | `Err(FontParseError::InvalidTableBounds)` for all vectors |

#### STP-001.3: Glyph Count Overflow

| Attribute | Value |
|-----------|-------|
| Test ID | STP-001.3 |
| Target | glyf table glyph count validation |
| Type | Unit test |
| Description | Verify that fonts claiming > 65536 glyphs are rejected without allocation |
| Test Vectors | TV-FONT-004: numGlyphs = u16::MAX + 1 (crafted binary), TV-FONT-005: numGlyphs = 0xFFFFFFFF |
| Expected Result | `Err(FontParseError::GlyphCountExceeded)` |

#### STP-001.4: Font Metadata Sanitization

| Attribute | Value |
|-----------|-------|
| Test ID | STP-001.4 |
| Target | PDF font metadata embedding |
| Type | Unit test |
| Description | Verify that font name strings containing PDF control sequences are sanitized |
| Test Vectors | TV-FONT-006: name table with `(%endobj)` injection, TV-FONT-007: name with `<< /JavaScript >>` |
| Expected Result | Control characters stripped or escaped; emitted PDF does not contain injected objects |

---

### 2.2 STP-002: WASM Sandbox Escape Testing

**Threat:** TM-002 (Resource exhaustion), TM-007 (Channel spoofing), TM-011 (Information disclosure), TM-015 (Privilege escalation)
**Priority:** P1
**Type:** Integration + Manual Penetration

#### STP-002.1: Fuel Limit Enforcement

| Attribute | Value |
|-----------|-------|
| Test ID | STP-002.1 |
| Target | wasmtime fuel consumption (REQ-7.3) |
| Type | Integration test |
| Description | Load a WASM module containing an infinite loop; verify it is terminated after 100,000 instructions |
| Test Vectors | TV-WASM-001: `loop { br 0 }` module, TV-WASM-002: nested loops totaling 200,000 iterations |
| Expected Result | `Err(CompileError::PluginFuelExceeded)` for both vectors |

#### STP-002.2: Memory Limit Enforcement

| Attribute | Value |
|-----------|-------|
| Test ID | STP-002.2 |
| Target | wasmtime linear memory growth limits |
| Type | Integration test |
| Description | Load a WASM module that attempts to grow memory beyond 64MB limit |
| Test Vectors | TV-WASM-003: `memory.grow` to 256MB, TV-WASM-004: allocate in 1MB increments until limit |
| Expected Result | `memory.grow` returns -1 (failure); engine reports `PluginMemoryLimitExceeded` |

#### STP-002.3: Host Import Isolation

| Attribute | Value |
|-----------|-------|
| Test ID | STP-002.3 |
| Target | WASM host import surface |
| Type | Integration test |
| Description | Verify that WASM plugins cannot invoke unregistered host functions |
| Test Vectors | TV-WASM-005: module with `import "env" "read_file"` (not provided), TV-WASM-006: module with `import "env" "socket_connect"` |
| Expected Result | Instantiation fails with `LinkError` for all unregistered imports |

#### STP-002.4: Sandbox Escape Attempt

| Attribute | Value |
|-----------|-------|
| Test ID | STP-002.4 |
| Target | wasmtime sandbox integrity |
| Type | Manual penetration test |
| Description | Attempt known WASM sandbox escape techniques: speculative execution side channels, return-oriented programming via crafted WASM bytecode, integer overflow in memory access |
| Test Vectors | Curated from Wasmtime issue tracker and academic WASM security papers |
| Expected Result | No host memory access, no arbitrary code execution outside sandbox |

#### STP-002.5: Plugin Communication Isolation

| Attribute | Value |
|-----------|-------|
| Test ID | STP-002.5 |
| Target | Inter-plugin communication isolation (TM-007) |
| Type | Integration test |
| Description | Load two plugins; verify Plugin A cannot read Plugin B's memory or impersonate Plugin B's identity |
| Test Vectors | TV-WASM-007: Plugin A passes Plugin B's memory region to host import |
| Expected Result | Host import rejects cross-plugin memory references |

---

### 2.3 STP-003: rkyv Deserialization Bounds Checking

**Threat:** TM-003 (Untrusted rkyv data), TM-008 (Deeply nested structures)
**Priority:** P2
**Type:** Fuzzing + Unit Test

#### STP-003.1: rkyv Fuzzing Harness

| Attribute | Value |
|-----------|-------|
| Test ID | STP-003.1 |
| Target | rkyv deserialization of S-IR |
| Tool | cargo-fuzz (libFuzzer) |
| Input | Mutated rkyv binary data |
| Corpus | 20+ seed S-IR documents from golden master suite |
| Duration | 4 hours continuous |
| Pass Criteria | No panic, no segfault, no OOM; all invalid input produces `Err` |

#### STP-003.2: Malformed S-IR Error Handling

| Attribute | Value |
|-----------|-------|
| Test ID | STP-003.2 |
| Target | COMP-IR-VALIDATOR error paths (IF-VALIDATE-001) |
| Type | Unit test |
| Description | Verify all 7 error codes (ERR-VALID-001 through ERR-VALID-007) are reachable and produce structured diagnostics |
| Test Vectors | One crafted S-IR per error code (TV-SIR-E01 through TV-SIR-E07) |
| Expected Result | Each produces the corresponding `ValidationError` with correct entity ID and byte offset |

#### STP-003.3: Deep Nesting Rejection

| Attribute | Value |
|-----------|-------|
| Test ID | STP-003.3 |
| Target | Maximum nesting depth enforcement (TM-008) |
| Type | Unit test |
| Description | S-IR with 257 nested PUSH_BLOCK instructions |
| Test Vectors | TV-SIR-E08: 257 nested blocks |
| Expected Result | `Err(ValidationError::NestingDepthExceeded { depth: 257, max: 256 })` |

---

### 2.4 STP-004: Integer Overflow Testing (fp26_6)

**Threat:** TM-004 (Integer overflow in fixed-point arithmetic)
**Priority:** P1
**Type:** Property-Based Test + Unit Test

#### STP-004.1: fp26_6 Boundary Value Analysis

| Attribute | Value |
|-----------|-------|
| Test ID | STP-004.1 |
| Target | 26.6 fixed-point arithmetic operations |
| Type | Property-based test (proptest) |
| Description | Verify that all fp26_6 operations (add, sub, mul, div) handle boundary values correctly |
| Test Vectors | Systematically generated: |
| | TV-FP-001: i32::MIN * 64 (underflow), TV-FP-002: i32::MAX * 64 (overflow) |
| | TV-FP-003: (-33554432.0 * 64) exact boundary, TV-FP-004: (33554431.984375 * 64) exact boundary |
| | TV-FP-005: 0.0 (zero), TV-FP-006: -0.0 (negative zero) |
| | TV-FP-007: max + 1 (overflow), TV-FP-008: min - 1 (underflow) |
| Properties | For all a, b in fp26_6 range: a + b clamps to range; a * b clamps to range; no panic |

#### STP-004.2: Quantization Rounding Correctness

| Attribute | Value |
|-----------|-------|
| Test ID | STP-004.2 |
| Target | Floating-point to fp26_6 quantization (REQ-3.2.7) |
| Type | Unit test |
| Description | Verify `round(v * 64)` produces values within ±1/128 error bound |
| Test Vectors | TV-FP-009 through TV-FP-020: values at rounding boundaries (x.0, x.5, x.984375) |
| Expected Result | All quantized values within ±1/128 of original |

#### STP-004.3: Knuth-Plass Penalty Overflow

| Attribute | Value |
|-----------|-------|
| Test ID | STP-004.3 |
| Target | Knuth-Plass badness calculation `b = 100 * (w-t)^3 / s^3` |
| Type | Unit test |
| Description | Verify penalty calculation does not overflow i32 when (w-t) is large and s is small |
| Test Vectors | TV-FP-021: w=1000pt, t=1pt, s=0.001pt (extreme stretch), TV-FP-022: w-t = i32::MAX |
| Expected Result | Penalty clamps to `i32::MAX` or returns `INF_BADNESS` sentinel; no panic |

---

### 2.5 STP-005: Resource Limit Enforcement Testing

**Threat:** TM-001 (Font OOM), TM-002 (WASM resource exhaustion), TM-008 (Document structure DoS)
**Priority:** P1
**Type:** Integration Test

#### STP-005.1: Overall Memory Budget

| Attribute | Value |
|-----------|-------|
| Test ID | STP-005.1 |
| Target | Engine-wide memory consumption |
| Type | Integration test |
| Description | Process a 1GB S-IR document and verify RSS stays within 2GB limit |
| Test Vectors | TV-RES-001: 1GB S-IR with maximum entity count |
| Expected Result | RSS < 2GB; no OOM kill |

#### STP-005.2: Concurrent Plugin Resource Isolation

| Attribute | Value |
|-----------|-------|
| Test ID | STP-005.2 |
| Target | Multiple WASM plugins running concurrently |
| Type | Integration test |
| Description | Load 4 plugins, each consuming near-maximum memory; verify no cross-plugin memory impact |
| Test Vectors | TV-RES-002: 4 plugins each allocating 60MB (within 64MB limit) |
| Expected Result | All plugins execute successfully; total memory < 300MB (4 * 64MB + overhead) |

#### STP-005.3: Compilation Timeout

| Attribute | Value |
|-----------|-------|
| Test ID | STP-005.3 |
| Target | Compilation wall-clock timeout |
| Type | Integration test |
| Description | Verify that a pathological document (wide content + complex constraints) triggers a timeout |
| Test Vectors | TV-RES-003: 10000-character word in 1-inch column with complex floating constraints |
| Expected Result | Compilation completes within 10 seconds or returns `CompileError::Timeout` |

---

### 2.6 STP-006: Path Traversal Testing

**Threat:** TM-005 (Path traversal in font file loading)
**Priority:** P1
**Type:** Unit Test

#### STP-006.1: Basic Path Traversal

| Attribute | Value |
|-----------|-------|
| Test ID | STP-006.1 |
| Target | Font file path resolution |
| Type | Unit test |
| Description | Verify that font paths with `../` are rejected or canonicalized safely |
| Test Vectors | TV-PATH-001: `fonts/../../../etc/passwd`, TV-PATH-002: `fonts/..\\..\\..\\Windows\\System32\\config` |
| Expected Result | `Err(FontLoadError::PathTraversalDetected)` or resolved path is within trusted directory |

#### STP-006.2: Symbolic Link Following

| Attribute | Value |
|-----------|-------|
| Test ID | STP-006.2 |
| Target | Symbolic link resolution in font paths |
| Type | Integration test |
| Description | Create symlink outside trusted dir pointing to sensitive file; attempt to load as font |
| Test Vectors | TV-PATH-003: symlink `fonts/evil.ttf` -> `/etc/shadow` |
| Expected Result | Engine resolves canonical path and rejects if outside trusted directory |

#### STP-006.3: Null Byte Injection

| Attribute | Value |
|-----------|-------|
| Test ID | STP-006.3 |
| Target | Null byte in font path strings |
| Type | Unit test |
| Description | Verify that paths containing null bytes are rejected (C string truncation attack) |
| Test Vectors | TV-PATH-004: `fonts/good.ttf\0../../etc/passwd` |
| Expected Result | `Err(FontLoadError::InvalidPath)` — null byte detected before path resolution |

#### STP-006.4: Configuration Path Validation

| Attribute | Value |
|-----------|-------|
| Test ID | STP-006.4 |
| Target | Configuration file path references (TM-009) |
| Type | Unit test |
| Description | Verify that font path references in YAML/TOML configuration are validated |
| Test Vectors | TV-PATH-005: config with `font_dir: "/tmp/../../etc"` |
| Expected Result | Configuration parsing rejects or canonicalizes to within trusted directory |

---

## 3. Test Execution Schedule

| Phase | Tests | Duration | Gate |
|-------|-------|----------|------|
| Phase 3 (Current) | STP-001.2-001.4, STP-002.1-002.3, STP-003.2-003.3, STP-004.1-004.3, STP-005.1-005.3, STP-006.1-006.4 | Sprint | Must pass before Phase 4 |
| Phase 4 (Implementation) | STP-001.1, STP-003.1 (fuzzing harnesses) | Nightly CI | Continuous from Phase 4 |
| Pre-Release | STP-002.4 (manual penetration), STP-006.2 (symlink test) | Release candidate | Must pass before v1.0.0 |

---

## 4. Test Case Summary

| Test ID | Description | Type | Priority | Threat |
|---------|-------------|------|----------|--------|
| STP-001.1 | Font parser fuzzing (AFL++) | Fuzzing | P1 | TM-001 |
| STP-001.2 | Font table size validation | Unit | P1 | TM-001 |
| STP-001.3 | Glyph count overflow | Unit | P1 | TM-001 |
| STP-001.4 | Font metadata sanitization | Unit | P1 | TM-006 |
| STP-002.1 | WASM fuel limit enforcement | Integration | P1 | TM-002 |
| STP-002.2 | WASM memory limit enforcement | Integration | P1 | TM-002 |
| STP-002.3 | Host import isolation | Integration | P1 | TM-015 |
| STP-002.4 | Sandbox escape attempt | Manual | P1 | TM-011 |
| STP-002.5 | Plugin communication isolation | Integration | P2 | TM-007 |
| STP-003.1 | rkyv deserialization fuzzing | Fuzzing | P2 | TM-003 |
| STP-003.2 | Malformed S-IR error handling | Unit | P2 | TM-003 |
| STP-003.3 | Deep nesting rejection | Unit | P1 | TM-008 |
| STP-004.1 | fp26_6 boundary value analysis | Property | P1 | TM-004 |
| STP-004.2 | Quantization rounding correctness | Unit | P1 | TM-004 |
| STP-004.3 | Knuth-Plass penalty overflow | Unit | P1 | TM-004 |
| STP-005.1 | Overall memory budget | Integration | P1 | TM-001, TM-008 |
| STP-005.2 | Concurrent plugin resource isolation | Integration | P1 | TM-002 |
| STP-005.3 | Compilation timeout | Integration | P1 | TM-008, TM-013 |
| STP-006.1 | Basic path traversal | Unit | P1 | TM-005 |
| STP-006.2 | Symbolic link following | Integration | P1 | TM-005 |
| STP-006.3 | Null byte injection | Unit | P1 | TM-005 |
| STP-006.4 | Configuration path validation | Unit | P2 | TM-009 |
| **Total** | **22 test cases** | | | |

---

*End of SEC-STP-001 v1.0.0*
