# LDIR Continuous Monitoring Strategy

**Document ID:** MON-CM-001
**Version:** 1.0.0
**Status:** APPROVED
**Date:** 2026-04-23
**Phase:** 11 — Continuous Monitoring
**Transition Date:** Phase 10 → Phase 11: 2026-04-23

---

## 1. Overview

This document defines the continuous monitoring strategy for LDIR, covering dependency health, security vulnerabilities, performance regressions, standard revisions, and supply chain integrity. All monitoring is automated through CI/CD.

---

## 2. Dependency Monitoring

### 2.1 Automated Updates

| Tool | Scope | Frequency |
|------|-------|-----------|
| Dependabot | Rust crates, GitHub Actions | Weekly (Monday 09:00 UTC) |

### 2.2 Dependabot Configuration

```yaml
# .github/dependabot.yml
version: 2
updates:
  - package-ecosystem: "cargo"
    directory: "/"
    schedule:
      interval: "weekly"
      day: "monday"
    open-pull-requests-limit: 10
    labels: ["dependencies", "automated"]
  - package-ecosystem: "github-actions"
    directory: "/"
    schedule:
      interval: "weekly"
```

### 2.3 Review Policy

| Change Type | Action | Approval |
|-------------|--------|----------|
| Patch bump | Auto-merge if CI passes | None |
| Minor bump | Manual review + CI | 1 reviewer |
| Major bump | Full review + breaking change assessment | 2 reviewers |
| New dependency | Security audit + license check | 2 reviewers |

### 2.4 Critical Dependencies

| Crate | Priority | Justification |
|-------|----------|---------------|
| `rkyv` | P1 | Core serialization (REQ-2.3) |
| `wasmtime` | P1 | WASM sandbox (REQ-7.1) |
| `tracing` | P2 | Telemetry (REQ-8.1) |
| `criterion` | P2 | Benchmarking |
| `proptest` | P2 | Property-based testing |

---

## 3. Security Scanning

### 3.1 Cargo Audit

| Parameter | Configuration |
|-----------|---------------|
| Tool | `cargo-audit` v0.21+ |
| Database | `rustsec/advisory-db` (auto-fetched) |
| Frequency | Every PR + weekly (Sunday 03:00 UTC) |
| Failure threshold | Any CVE → CI fails |
| Ignored advisories | None (zero tolerance) |

### 3.2 CI Integration

```yaml
# .github/workflows/audit.yml
name: Security Audit
on:
  pull_request:
  schedule:
    - cron: "0 3 * * 0"
jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-audit
      - run: cargo audit --deny warnings --deny unmaintained --deny unsound --deny yanked
```

### 3.3 GitHub Advisory Database

| Setting | Value |
|---------|-------|
| Dependabot security updates | Enabled |
| Private vulnerability reporting | Enabled |
| Draft security advisories | Enabled |

### 3.4 Additional Checks

| Check | Tool | Frequency |
|-------|------|-----------|
| License compliance | `cargo-deny` | Every PR |
| Supply chain audit | `cargo supply-chain` | Weekly |
| Unsafe code audit | `cargo-geiger` | Weekly |

### 3.5 Vulnerability Response SLA

| Severity | Response | Patch | Advisory |
|----------|----------|-------|----------|
| Critical (CVSS 9+) | < 4h | < 48h | Immediate |
| High (CVSS 7-8.9) | < 24h | < 1 week | Within 48h |
| Medium (CVSS 4-6.9) | < 72h | Next release | Next release |
| Low (CVSS < 4) | Next release | Next release | CHANGELOG |

---

## 4. Performance Regression Detection

### 4.1 Criterion Baseline Comparison

All 42 benchmarks run on every PR to `main`. Criterion compares against stored baseline.

### 4.2 Regression Thresholds

| Category | Threshold | Action |
|----------|-----------|--------|
| Parsing (BM-PARSE) | > 2% latency | Block merge |
| Compilation (BM-COMPILE) | > 2% latency | Block merge |
| Fixed-Point (BM-FIXPT) | > 5ns per op | Warn + review |
| Line Breaking (BM-LAYOUT) | > 2% latency | Block merge |
| Pagination (BM-PAGINATE) | > 2% latency | Block merge |
| PDF Emission (BM-EMIT) | > 2% latency | Block merge |
| ECS (BM-ECS) | > 2% latency | Warn + review |
| Concurrency (BM-CONCURRENCY) | > 5% overhead | Block merge |
| WASM (BM-WASM) | > 5% overhead | Warn + review |

### 4.3 Determinism Verification (Every PR)

```bash
cargo test --test determinism -- --threads 1
cargo test --test determinism -- --threads 4
cargo test --test determinism -- --threads 16
# All three must produce identical G-IR SHA256
```

---

## 5. Standard Revisions Monitoring

### 5.1 Tracked Standards

| Standard | Version | Review |
|----------|---------|--------|
| IEEE 754-2019 | 2019 | Annual |
| ISO 32000-2:2020 (PDF 2.0) | 2020 | Annual |
| ISO 19005-4:2020 (PDF/A-4) | 2020 | Annual |
| ISO 14496-22 (OpenType) | Current | Annual |
| ISO/IEC 10646:2020 (Unicode) | 2020 | Annual |
| WebAssembly Core Spec 2.0 | 2.0 | Annual |
| NIST SP 800-53 Rev 5 | Rev 5 | Annual |
| OWASP Top 10 | 2021 | Annual (or on new release) |
| CommonMark Spec | 0.31 | Annual |

### 5.2 Revision Impact Process

1. GitHub Issue with `standards-monitoring` label
2. Compare old vs new standard text
3. Identify affected REQ-* entries
4. Update requirements, tests, or implementation
5. Re-run affected compliance checks

---

## 6. Compliance Monitoring

### 6.1 Annual Review Schedule

| Review Item | Frequency | Responsible |
|-------------|-----------|-------------|
| NIST SP 800-53 mapping | Annual | Security lead |
| OWASP Top 10 mapping | Annual | Security lead |
| Threat model review | Annual or post-incident | Security lead |
| Security test plan update | Annual or post-incident | QA lead |
| Lean4 proof completeness | Quarterly | Verification lead |
| Benchmark baseline refresh | Monthly (post-impl) | Performance lead |
| Requirements coverage audit | Semi-annual | Project lead |

### 6.2 Compliance Gaps

| Control | Standard | Status | Remediation |
|---------|----------|--------|-------------|
| Dependency scanning | NIST RA-5 | PLANNED | Add `cargo audit` CI |
| Config management | NIST SR-11 | PLANNED | Cargo.lock integrity |
| Plugin ID signing | OWASP A02 | PLANNED | Cryptographic IDs |
| Plugin auth | OWASP A07 | PLANNED | Identity verification |
| Audit logging | OWASP A09 | PLANNED | WASM lifecycle logs |
| Asset inventory | ISO A.8.1 | PLANNED | Plugin registry |

### 6.3 Fuzzing Monitoring

| Target | Corpus | Crashes (30d) | Coverage Target |
|--------|--------|-------------|-----------------|
| S-IR parser | 20 seeds | 0 | > 90% |
| rkyv deserializer | 20 seeds | 0 | > 90% |
| Font parser | 50 seeds | 0 | > 90% |

---

## 7. Supply Chain Monitoring

| Check | Tool | Frequency |
|-------|------|-----------|
| Cargo.lock in VCS | Git pre-commit | Every commit |
| Cargo.toml consistency | CI | Every PR |
| No duplicate deps | `cargo tree --duplicates` | Every PR |
| License check | `cargo-deny` | Every PR |
| Crate maintenance status | RustSec | Weekly |

---

## 8. Alert Routing

| Alert | Severity | Channel | SLA |
|-------|----------|---------|-----|
| CVE in dependency | Critical | GH Advisory + email | < 4h |
| Benchmark regression | High | PR comment (blocking) | < 24h |
| CI failure | High | PR status check | < 4h |
| Fuzz crash | Critical | GH Issue (P1) | < 4h |
| Standard revision | Low | GH Issue (labeled) | Next sprint |
| Compliance gap | Medium | GH Issue (labeled) | < 1 week |

---

## 9. Phase Transition

| From | To | Date | Gate |
|------|-----|------|------|
| Phase 10 (Closure) | Phase 11 (Monitoring) | 2026-04-23 | Closure report approved |
| Phase 11 (Monitoring) | Steady State | 2026-04-23 | All monitors configured |

---

*End of MON-CM-001 v1.0.0*
