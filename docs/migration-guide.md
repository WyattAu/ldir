# LDIR Migration Guide

## Version History

| Version | Date       | Summary                                      |
|---------|------------|----------------------------------------------|
| 1.0.0   | 2026-04-23 | Core pipeline complete (S-IR, G-IR, compiler, emitter, validator) |
| 0.2.0   | 2026-04-23 | Yellow papers, Lean 4 proofs, test vectors    |
| 0.1.0   | 2026-04-23 | Initial project structure and specifications  |

## Breaking Changes

### 1.0.0

No breaking changes — this is the initial public release of the Rust crates.

The following APIs are stable for the 1.x series:

- `ldir_ir::sir` — S-IR types (`SIRDocument`, `SIRInstruction`, `SIROpcode`, `BlockType`, `ROOT_SENTINEL`)
- `ldir_ir::gir` — G-IR types (`GIRDocument`, `GIRPage`, `GIRCommand`, `GIROpcode`, `GIRStyle`, `StyleTable`)
- `ldir_core::compiler::compile_sir` — S-IR → G-IR compilation
- `ldir_core::validator::validate_sir` — S-IR well-formedness validation
- `ldir_core::emitter::{emit_gir, parse_gir}` — G-IR binary serialization
- `ldir_core::fp266::Fp266` — 26.6 fixed-point arithmetic
- `ldir_core::ecs` — Entity Component System
- `ldir_core::error` — Error types (`LdirError`, `ErrorKind`)

### Planned deprecations (1.x → 2.x)

The following may change in 2.0:

- `SIRInstruction` repr alignment (currently 16 bytes, wire format 13 bytes)
- G-IR command argument layout may expand for additional rendering features
- `ldir-md`, `ldir-tex`, `ldir-pdf` crate APIs are not yet stable

## Upgrade Instructions

### Upgrading to 1.0.0

No action needed if you are using a previous version — 1.0.0 is the first
release with published Rust crate APIs.

### Minimum Rust version

LDIR requires **Rust 1.85** or later (edition 2024). Update with:

```bash
rustup update stable
```

### Workspace setup

If you are using LDIR as a workspace dependency, update your `Cargo.toml`:

```toml
[workspace.dependencies]
ldir-core = "1.0"
ldir-ir = "1.0"
```

Or for path dependencies during development:

```toml
[workspace.dependencies]
ldir-core = { path = "ldir-core" }
ldir-ir = { path = "ldir-ir" }
```

## Feature Flags

There are currently no feature flags. All functionality is included by default.

## Semantic Versioning Policy

LDIR follows [Semantic Versioning 2.0.0](https://semver.org/):

- **Major versions**: Breaking API changes
- **Minor versions**: New functionality, backward-compatible
- **Patch versions**: Bug fixes, backward-compatible

The `ldir_core::Result<T>` error type and `ldir_core::error` module are
considered part of the public API and will not introduce breaking changes
without a major version bump.
