## Description
<!-- What does this PR do? -->

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation

## Checklist
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo test --workspace --exclude ldir-wasm` passes
- [ ] Lean4 proofs compile (0 errors) -- if .lean files changed
- [ ] WASM build passes (`cargo build -p ldir-wasm --target wasm32-unknown-unknown`) -- if ldir-wasm changed
- [ ] No new production unwrap/expect calls (3 justified exceptions exist)
- [ ] Updated CHANGELOG.md
