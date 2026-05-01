# LDIR Deployment Strategy

**Document ID:** DEP-DS-001
**Version:** 1.0.0
**Status:** APPROVED
**Date:** 2026-04-23
**Phase:** 9 — Deployment
**Transition Date:** Phase 8 → Phase 9: 2026-04-23

---

## 1. Overview

This document defines the deployment strategy for the LDIR typesetting engine library, covering Crates.io publication, documentation hosting, release automation, monitoring, rollback, and incident response.

---

## 2. Crates.io Publication

### 2.1 Crate Inventory

| Crate | Description | Publication Priority |
|-------|-------------|---------------------|
| `ldir-ir` | S-IR/G-IR type definitions, opcodes | First (foundation) |
| `ldir-core` | ECS engine, layout optimizer, compiler | Second (core) |
| `ldir-tex` | TeX macro expander frontend | Third |
| `ldir-md` | CommonMark Markdown frontend | Third |
| `ldir-pdf` | PDF/A-4 backend | Fourth |
| `ldir-vello` | GPU/Native rendering backend | Fourth |
| `ldir-wasm` | WASM/WebGL browser bundle | Fifth |
| `ldc` | CLI compiler binary | Fifth |

### 2.2 Semantic Versioning Policy

| Change Type | Version Bump | Example |
|-------------|-------------|---------|
| Public API breakage | MAJOR (X.0.0) | Renaming a struct field |
| New functionality, backward-compatible | MINOR (0.X.0) | New layout algorithm |
| Bug fixes, internal changes | PATCH (0.0.X) | Boundary condition fix |

### 2.3 Workspace Versioning

All crates share a synchronized version via `workspace.package.version` in root `Cargo.toml`:

```toml
[workspace.package]
version = "0.1.0"
edition = "2024"
```

### 2.4 Publication Prerequisites

| Gate | Check | CI Job |
|------|-------|--------|
| G-001 | All tests pass (`cargo test --workspace`) | `test.yml` |
| G-002 | No clippy warnings (`cargo clippy -- -D warnings`) | `lint.yml` |
| G-003 | All docs build (`cargo doc --no-deps`) | `docs.yml` |
| G-004 | No `cargo audit` vulnerabilities | `audit.yml` |
| G-005 | Benchmarks within 2% regression | `bench.yml` |
| G-006 | Fuzzing corpus clean (no crashes) | `fuzz-nightly.yml` |
| G-007 | Determinism: bit-identical G-IR (1/4/16 threads) | `test.yml` |
| G-008 | Lean4 proofs compile with 0 errors | `lean-build.yml` |

---

## 3. Documentation Hosting

### 3.1 docs.rs Auto-Deployment

Documentation builds automatically on [docs.rs](https://docs.rs) per crate publish:

```toml
[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]
targets = ["x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu", "wasm32-unknown-unknown"]
```

### 3.2 Documentation Standards

| Requirement | Standard |
|-------------|----------|
| Every public item | Must have `///` doc comment |
| Error types | Must document all variants with recovery |
| `unsafe` blocks | Must have `# Safety` section |
| Code examples | Must pass `cargo test --doc` |

---

## 4. Release Process

### 4.1 Release Flow

```
Developer merges PR → Git Tag v0.1.0 → CI Build & Test → Publish crates.io → docs.rs auto-build
```

### 4.2 Step-by-Step Procedure

1. Verify `main` branch is green — all CI passes
2. Bump `workspace.package.version` in root `Cargo.toml`
3. Update `CHANGELOG.md` with all changes since last release
4. Tag: `git tag -s v0.1.0 -m "Release v0.1.0"`
5. Push tag: `git push origin v0.1.0`
6. CI `release.yml` triggers: builds all targets, runs full test suite, runs benchmark regression, publishes crates in dependency order (`ldir-ir` → `ldir-core` → frontends → backends), creates GitHub Release
7. docs.rs builds automatically after crates.io publication
8. Verify: docs.rs renders, `cargo add ldir-core` resolves, minimal example compiles

### 4.3 Release Channels

| Channel | Tag Pattern | Purpose |
|---------|------------|---------|
| Stable | `v0.1.0` | Production releases |
| Release Candidate | `v0.1.0-rc.1` | Pre-release testing |
| Nightly/Dev | `main` branch | Ongoing development |

---

## 5. Monitoring

### 5.1 CI Status Badges (README.md)

| Badge | Source |
|-------|--------|
| Build Status | GitHub Actions `ci.yml` |
| Test Coverage | `cargo-tarpaulin` |
| Docs | docs.rs build status |
| Crates.io Version | crates.io API |
| Security | `cargo audit` badge |

### 5.2 Release Metrics

| Metric | Alert Threshold |
|--------|-----------------|
| Build time per target | > 15 minutes |
| Test suite duration | > 10 minutes |
| Benchmark regression | > 2% degradation |
| Issue open time | > 14 days without triage |
| Dependabot PR backlog | > 5 open PRs |

---

## 6. Rollback Procedure

### 6.1 Crate Yanking

If a critical bug or vulnerability is discovered post-release:

```bash
cargo yank --version 0.1.0 ldir-core
```

Yanked versions remain available to existing dependents but won't be installed by new `cargo add` or `cargo update`.

### 6.2 Rollback Decision Matrix

| Severity | Response Time | Action |
|----------|--------------|--------|
| Critical (data corruption, security) | < 4 hours | Yank + patch release |
| High (crash on valid input) | < 24 hours | Yank + patch release |
| Medium (incorrect output, edge case) | < 1 week | Advisory + patch release |
| Low (cosmetic, documentation) | Next release | CHANGELOG note |

### 6.3 Re-Publish Guidelines

- Never re-publish the same version number
- Always bump at least PATCH for any code change
- Yanked versions cannot be un-yanked

---

## 7. Incident Response

### 7.1 Security Advisory Process (RUSTSEC)

| Step | Action | Timeline |
|------|--------|----------|
| 1 | Report received via GitHub Security Advisory (private) | T+0 |
| 2 | Triage: confirm vulnerability, assess severity (CVSS) | T+24h |
| 3 | Develop fix in private fork | T+48h |
| 4 | Coordinated disclosure to downstream users | T+72h |
| 5 | Publish fix + RUSTSEC advisory + request CVE | T+96h |

### 7.2 Advisory Severity Classification

| Severity | CVSS Range | Example |
|----------|-----------|---------|
| Critical | 9.0 - 10.0 | Sandbox escape, arbitrary code execution |
| High | 7.0 - 8.9 | Memory corruption via malformed input |
| Medium | 4.0 - 6.9 | Denial of service, resource exhaustion |
| Low | 0.1 - 3.9 | Information disclosure |

### 7.3 Threat-to-Advisory Mapping

| Threat ID | Type | Advisory |
|-----------|------|----------|
| TM-001 | Font parser OOM | RUSTSEC-YYYY-NNNN |
| TM-002 | WASM resource exhaustion | RUSTSEC-YYYY-NNNN |
| TM-004 | fp26_6 overflow | RUSTSEC-YYYY-NNNN |
| TM-005 | Path traversal | RUSTSEC-YYYY-NNNN |
| TM-006 | PDF content injection | RUSTSEC-YYYY-NNNN |
| TM-011 | WASM info disclosure | RUSTSEC-YYYY-NNNN |
| TM-015 | WASM privilege escalation | RUSTSEC-YYYY-NNNN |

---

## 8. Phase Transition

| From | To | Date | Gate |
|------|-----|------|------|
| Phase 8 (Integration) | Phase 9 (Deployment) | 2026-04-23 | All CI gates passing |
| Phase 9 (Deployment) | Phase 10 (Closure) | 2026-04-23 | First successful dry-run publish |

---

*End of DEP-DS-001 v1.0.0*
