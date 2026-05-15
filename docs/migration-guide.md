# LDIR Migration Guide

## Version History

| Version | Date       | Summary                                      |
|---------|------------|----------------------------------------------|
| 0.1.0   | 2026-04-23 | Initial project structure and specifications  |

## Breaking Changes

No breaking changes yet. The project is in active pre-release development (version 0.1.0).

The following APIs are currently available:

- `ldir_ir::sir` -- S-IR types (`SIRDocument`, `SIRInstruction`, `SIROpcode`, `BlockType`, `ROOT_SENTINEL`)
- `ldir_ir::sir::v2` -- S-IR v2 types (`SIRModuleV2`, `SIRNodeV2`, `NodeType`)
- `ldir_ir::lir` -- L-IR types (`LIRDocument`, `LIRNode`, `LIRGeometry`)
- `ldir_ir::gir` -- G-IR types (`GIRDocument`, `GIRPage`, `GIRCommand`, `GIROpcode`, `GIRStyle`, `StyleTable`)
- `ldir_core::compiler::compile_sir` -- S-IR to G-IR compilation
- `ldir_core::validator::validate_sir` -- S-IR well-formedness validation
- `ldir_core::emitter::{emit_gir, parse_gir}` -- G-IR binary serialization
- `ldir_core::plugin::{FrontendPlugin, BackendPlugin, PluginRegistry}` -- Plugin system
- `ldir_core::fp266::Fp266` -- 26.6 fixed-point arithmetic
- `ldir_core::ecs` -- Entity Component System
- `ldir_core::error` -- Error types (`LdirError`, `ErrorKind`)

### Planned deprecations (0.x -> 1.0)

The following may change before 1.0:

- `SIRInstruction` repr alignment (currently 16 bytes, wire format 13 bytes)
- G-IR command argument layout may expand for additional rendering features
- `ldir-md`, `ldir-tex`, `ldir-pdf` crate APIs are not yet stable

## Upgrade Instructions

### Minimum Rust version

LDIR requires **Rust 1.88** or later (edition 2024). Update with:

```bash
rustup update stable
```

### Workspace setup

If you are using LDIR as a workspace dependency, update your `Cargo.toml`:

```toml
[workspace.dependencies]
ldir-core = "0.1"
ldir-ir = "0.1"
```

Or for path dependencies during development:

```toml
[workspace.dependencies]
ldir-core = { path = "ldir-core" }
ldir-ir = { path = "ldir-ir" }
```

## Feature Flags

| Flag | Default | Description |
|------|---------|-------------|
| `wasm-plugins` | off | Wasmtime-based Wasm plugin sandbox for frontends/backends |

## Semantic Versioning Policy

LDIR follows [Semantic Versioning 2.0.0](https://semver.org/):

- **Major versions**: Breaking API changes
- **Minor versions**: New functionality, backward-compatible
- **Patch versions**: Bug fixes, backward-compatible

The `ldir_core::Result<T>` error type and `ldir_core::error` module are
considered part of the public API and will not introduce breaking changes
without a major version bump.
