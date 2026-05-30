# Contributing to ldir

Thank you for your interest in contributing to ldir!

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/ldir.git`
3. Create a branch: `git checkout -b my-feature`
4. Build: `cargo build --workspace`
5. Test: `cargo test --workspace`
6. Lint: `cargo clippy --workspace -- -D warnings && cargo fmt --check`

## Code Style

- Follow Rust idiomatic conventions
- Clippy must pass with `-D warnings` (zero warnings)
- `cargo fmt` must pass with no changes
- All public types and functions must have doc comments
- Use `///` for Rustdoc, `//` for implementation notes
- Unsafe blocks must have SAFETY comments

## Commit Messages

Use Conventional Commits format:
- `feat:` new feature
- `fix:` bug fix
- `docs:` documentation
- `refactor:` code restructuring
- `test:` test additions
- `ci:` CI/CD changes
- `perf:` performance improvements

## Testing

- All new features require tests
- Critical paths require >95% branch coverage
- Use `ldir_test_helpers` for shared test utilities
- Font tests use `test_font_data()` for portable test fonts

## Architecture

ldir uses a three-layer IR pipeline:
- **S-IR (Semantic IR)**: Document structure and semantics
- **L-IR (Layout IR)**: Page-level layout information
- **G-IR (Graphics IR)**: Rendering commands

When adding features, consider which IR layer is affected.

## Pull Request Process

1. Update relevant roadmap files (ROADMAP.md, ROADMAP_NEXT.md, ROADMAP_PRODUCTION.md)
2. Ensure all CI checks pass
3. Update documentation for user-facing changes
4. Keep PRs focused on a single concern

## Questions?

Open an issue for discussion before starting significant work.
