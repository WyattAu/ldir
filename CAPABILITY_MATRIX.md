# LDIR Capability Matrix

| Capability | Tool | Required Version | Available Version | Status | Notes |
|-----------|------|-----------------|-------------------|--------|-------|
| Formal Verification | Lean4 | >= 4.0.0 | 4.30.0-rc2 | Available | Core verification tool |
| Implementation | Rust | >= 1.75.0 | 1.94.1 | Available | Stable channel |
| Build System | Cargo | >= 1.75.0 | 1.94.1 | Available | |
| Linting | Clippy | >= 1.75.0 | 0.1.94 | Available | |
| Formatting | Rustfmt | >= 1.75.0 | 1.8.0 | Available | |
| Reproducible Builds | Nix | >= 2.18.0 | 2.34.4 | Available | |
| Containerization | Docker | >= 24.0 | 29.4.0 | Available | CI/CD |
| Version Control | Git | >= 2.40.0 | 2.54.0 | Available | |
| Build Orchestration | CMake | >= 3.28.0 | 4.3.2 | Available | C FFI |
| Scripting | Python | >= 3.11.0 | 3.14.4 | Available | Tooling/scripts |
| WASM Tooling | Node.js | >= 20.0.0 | 25.9.0 | Available | WASM build |
| LLVM Toolchain | LLVM | >= 17.0.0 | Available | Available | SIMD intrinsics |
| WASM Runtime | Wasmtime | >= 15.0.0 | 28.x | Available | Optional `wasm-plugins` feature |
| Lean4 IDE | Lean4 VSCode | >= 4.0.0 | Unknown | Unknown | Development ergonomics |
