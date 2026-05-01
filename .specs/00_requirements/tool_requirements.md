# LDIR Tool Requirements

## Build Toolchain
| Tool | Minimum Version | Purpose | Priority | Installation |
|------|----------------|---------|----------|-------------|
| Rust | 1.75.0 | Core implementation | Mandatory | rustup |
| Cargo | 1.75.0 | Build system | Mandatory | rustup |
| Lean4 | 4.0.0 | Formal verification | Mandatory | elan or nix |
| Nix | 2.18.0 | Reproducible builds | Mandatory | nix installer |
| CMake | 3.28.0 | C FFI build | Required | System package |
| LLVM | 17.0.0 | SIMD intrinsics | Required | System package |
| Node.js | 20.0.0 | WASM tooling | Required | nvm or nix |
| Wasmtime | 15.0.0 | WASM runtime | Required | cargo install |
| Docker | 24.0.0 | CI/CD | Recommended | System package |
| Python | 3.11.0 | Scripts/tooling | Optional | System package |

## Development Tools
| Tool | Purpose | Priority |
|------|---------|----------|
| rust-analyzer | IDE support | Mandatory |
| Lean4 VSCode Extension | Proof development | Mandatory |
| cargo-fuzz | S-IR fuzzing | Mandatory |
| cargo-flamegraph | Profiling | Required |
| Tracy | Frame profiling | Required |
| Valgrind | Memory leak detection | Required |
| AddressSanitizer | Memory safety | Required |
| criterion | Benchmarking | Required |

## CI/CD Tools
| Tool | Purpose | Priority |
|------|---------|----------|
| GitHub Actions | CI/CD pipeline | Required |
| cargo-nextest | Test runner | Required |
| grcov | Coverage | Required |
| Trivy | Supply chain scanning | Required |
| cargo-deny | License/dependency audit | Required |
