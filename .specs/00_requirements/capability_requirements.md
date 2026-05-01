# LDIR Capability Requirements

**Date:** 2026-04-23
**Environment:** Linux (x86-64), Nix-managed

---

## 1. Available Capabilities

| Tool | Version | Status | Purpose |
|---|---|---|---|
| **Lean4** | 4.30.0-rc2 | Available | Formal verification of IR well-formedness, algorithm correctness proofs |
| **Rust (rustc)** | 1.94.1 (stable) | Available | Primary implementation language for all crates |
| **Cargo** | 1.94.1 | Available | Build system and dependency management |
| **Rustfmt** | 1.8.0 | Available | Code formatting enforcement |
| **Clippy** | 0.1.94 | Available | Static analysis and lint enforcement |
| **Nix** | 2.34.4 | Available | Reproducible build environment, tool version pinning |
| **Docker** | 29.4.0 | Available | Containerized CI/CD, cross-platform build environments |
| **Git** | 2.54.0 | Available | Version control |
| **CMake** | 4.3.2 | Available | Build system for native dependencies (HarfBuzz, FreeType if built from source) |
| **Python** | 3.14.4 | Available | Build scripts, test orchestration, tooling automation |
| **Node.js** | 25.9.0 | Available | WASM tooling, browser backend development, LSP server |
| **LLVM** | Available | Available | Potential JIT compilation, WASM toolchain dependency |

---

## 2. Required Capabilities (Missing)

| Tool | Purpose | Install Method | Priority |
|---|---|---|---|
| **Wasmtime** | WASM runtime for plugin sandboxing (REQ-7.1) | `cargo install wasmtime-cli` or Nix: `wasmtime` | **Critical** |
| **cargo-fuzz** | Continuous fuzzing of S-IR parser (REQ-9.1) | `cargo install cargo-fuzz` | **High** |
| **wasm-pack** | Building WASM bundles for browser backend | `cargo install wasm-pack` or Nix: `wasm-pack` | **High** |
| **HarfBuzz** (dev) | Font shaping library (system dependency for `rust-harfbuzz`) | Nix: `harfbuzz` with `pkg-config` | **Critical** |
| **FreeType** (dev) | Font rasterization (backend dependency) | Nix: `freetype` with `pkg-config` | **High** |
| **Tracy** | Frame profiling and flame-graph analysis (REQ-8.2) | System package or source build | **Medium** |
| **lcov / llvm-cov** | Code coverage reporting for CI | Nix: `llvmPackages_19.llvm` or `lcov` | **Medium** |
| **nextest** | Faster test runner for large test suites | `cargo install cargo-nextest` | **Low** |
| **Grcov** | Coverage report generation | `cargo install grcov` | **Low** |
| **just** | Task runner for development commands | `cargo install just` or Nix: `just` | **Low** |
| **protoc** | Protocol Buffers compiler (if FlatBuffers/protobuf used for cross-language ABI) | Nix: `protobuf` | **Low** |

---

## 3. Capability Matrix

### 3.1 Development Toolchain

| Capability | Required | Available | Gap | Severity |
|---|---|---|---|---|
| Rust compiler | Yes | Yes (1.94.1) | None | — |
| Cargo build system | Yes | Yes (1.94.1) | None | — |
| Lean4 proof assistant | Yes | Yes (4.30.0-rc2) | None | — |
| Clippy linter | Yes | Yes (0.1.94) | None | — |
| Rustfmt formatter | Yes | Yes (1.8.0) | None | — |
| Nix reproducibility | Yes | Yes (2.34.4) | None | — |
| Git VCS | Yes | Yes (2.54.0) | None | — |
| C/C++ build (CMake) | Yes | Yes (4.3.2) | None | — |

### 3.2 Runtime Dependencies

| Capability | Required | Available | Gap | Severity |
|---|---|---|---|---|
| WASM runtime (wasmtime) | Yes | No | Install wasmtime | **Critical** |
| Font shaping (HarfBuzz) | Yes | No | Install harfbuzz dev | **Critical** |
| Font parsing (FreeType) | Yes | No | Install freetype dev | **High** |

### 3.3 Testing & Quality

| Capability | Required | Available | Gap | Severity |
|---|---|---|---|---|
| Fuzzer (cargo-fuzz) | Yes | No | Install cargo-fuzz | **High** |
| WASM build (wasm-pack) | Yes | No | Install wasm-pack | **High** |
| Profiler (Tracy) | Desired | No | Install Tracy | **Medium** |
| Coverage (llvm-cov/grcov) | Desired | No | Install coverage tool | **Medium** |
| Fast test runner | Desired | No | Install cargo-nextest | **Low** |

### 3.4 Cross-Platform Build

| Capability | Required | Available | Gap | Severity |
|---|---|---|---|---|
| Docker containerization | Yes | Yes (29.4.0) | None | — |
| Node.js (WASM tooling) | Yes | Yes (25.9.0) | None | — |
| Python (automation) | Yes | Yes (3.14.4) | None | — |

---

## 4. Tool Version Requirements

### 4.1 Minimum Versions

| Tool | Minimum Version | Current Version | Notes |
|---|---|---|---|
| Rust (rustc) | 1.85.0 | 1.94.1 | Requires `async fn` in trait, `gen` blocks, stabilized `let_chains` |
| Cargo | 1.85.0 | 1.94.1 | Must match rustc version |
| Lean4 | 4.8.0 | 4.30.0-rc2 | Requires `Lake` build system support, `ProofWidget` compatibility |
| Nix | 2.18.0 | 2.34.4 | Requires `flake` support, `nix develop` shells |
| Docker | 24.0.0 | 29.4.0 | Required for CI build cache and cross-compilation |
| Git | 2.40.0 | 2.54.0 | Required for `scalar` partial clone support in large repos |
| Node.js | 20.0.0 | 25.9.0 | Required for WASM tooling and LSP client |
| Python | 3.11.0 | 3.14.4 | Required for build script orchestration |
| CMake | 3.25.0 | 4.3.2 | Required for native C/C++ dependency builds |

### 4.2 Critical Dependency Versions (Rust Crates)

| Crate | Minimum Version | Purpose |
|---|---|---|
| `rkyv` | 0.8.0 | Zero-copy S-IR serialization |
| `wasmtime` | 15.0.0 | WASM plugin sandboxing runtime |
| `harfbuzz-sys` | 0.5.0 | HarfBuzz Rust bindings for font shaping |
| `freetype-sys` | 0.20.0 | FreeType Rust bindings for font parsing |
| `tracing` | 0.1.40 | Nanosecond-resolution instrumentation |
| `rayon` | 1.10.0 | Parallel work-stealing scheduler |
| `crossbeam-epoch` | 0.9.18 | Lock-free epoch-based reclamation |
| `wgpu` | 0.20.0 | GPU compute shader dispatch (Vello backend) |
| `flate2` | 1.0.30 | Parallel Deflate compression for PDF streams |
| `clap` | 4.5.0 | CLI argument parsing for `ldc` |

### 4.3 Installation Commands

```bash
# Critical — WASM runtime
cargo install wasmtime-cli

# High — Fuzzing
cargo install cargo-fuzz
cargo install cargo-nextest

# High — WASM toolchain for browser backend
cargo install wasm-pack
rustup target add wasm32-unknown-unknown

# High — System font libraries (via Nix)
nix profile install nixpkgs#harfbuzz
nix profile install nixpkgs#freetype
nix profile install nixpkgs#pkg-config

# Medium — Profiling
# Tracy: download from https://github.com/wolfpld/tracy/releases

# Low — Task runner
cargo install just
```

### 4.4 Nix Shell Integration

All missing dependencies should be added to a `flake.nix` devShell to ensure reproducibility:

```nix
devShells.default = pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    rustc cargo clippy rustfmt
    lean4
    wasmtime
    harfbuzz freetype pkg-config
    cmake python3 nodejs
    git nix docker
  ];
};
```
