# PATTERN_ADOPTION: ldir

Relationship to the WyattAu pattern canon
(`github.com/WyattAu/pattern-standards`), validated by
`scripts/adoption-check.py` per `references/adoption-format.md`.

ldir is a document compiler ("the LLVM of Documents"): frontends parse
untrusted source text into a typed three-layer IR (S-IR, L-IR, G-IR) and
backends render it. The compiler core is library code; the `ldc` binary is
the clap-driven shell over it.

| Pattern | Status | Evidence |
|---|---|---|
| T1 | adopted | ldir-md/src/lib.rs:99 (Markdown source text parsed once into a typed SIRDocument value) |
| T13 | adopted | ldir-core/src/layout/mod.rs (layout and compiler core as library crates) vs ldc/src/main.rs (CLI shell; see also ldc/src/cli.rs) |
| T17 | partial | ldir-core/src/layout/incremental.rs:28 (dirty-set tracking drives recompile_lir so unchanged subtrees are skipped; a dirty set plus invalidation, not a memoized query graph) |

Domain hints checked and not adopted: P83 (no audit surface; the
ldir-validate crate checks PDF/A conformance, not history integrity).
