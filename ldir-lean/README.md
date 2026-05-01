# ldir-lean — Formal Verification

Lean4 formal specifications and proofs for the LDIR document IR.

## Building

Requires [Lean 4](https://lean-lang.org/) and [Lake](https://github.com/leanprover/lean4/tree/master/src/lake).

```bash
cd ldir-lean
lake build
```

First build downloads and compiles Mathlib (~5-10 minutes depending on machine).
Subsequent builds are incremental.

## Proof Status

| Proof File | Theorems | Proven | Sorry |
|-----------|----------|--------|-------|
| `ProofIRWellformedness.lean` | 10 + 3 lemmas | 8 | 2 (eraseDups) |

### IR Well-Formedness (`ProofIRWellformedness.lean`)

Formalizes and proves properties of S-IR well-formedness conditions:

- **AX-001** (`entityUnique`): Entity ID uniqueness
- **AX-002** (`parentExists`): Parent reference validity
- **AX-003** (`isAcyclic`): No circular parent chains (fuel-based)
- **DEF-004.5** (`hasSingleRoot`): Exactly one root entity
- **DEF-004.6**: Block nesting structure

Key results:
- `wf_sir_implies_single_root`: Well-formed S-IR has exactly one root
- `not_single_root_implies_not_wf`: Converse direction
- `compile_terminates`: Compilation always terminates (structural recursion)

### Known Gaps

Two `sorry` remain due to Mathlib's `List.eraseDups` being defined via `List.brecOn`,
making structural induction opaque:
- `entityUnique_subset`: eraseDups preserves subset relationship
- `entityUnique_soundness`: eraseDups-based uniqueness check is sound

## Planned Proofs

| Target | Yellow Paper Reference | Priority |
|--------|----------------------|----------|
| Knuth-Plass optimality | YP-LAYOUT-KNUTHPLASS-001 THM-KP-OPTIMALITY | Medium |
| Compiler correctness | BP-IR-COMPILER-001 POST-COMP-001 | High |
| Constraint solver soundness | YP-CONSTRAINT-CASSOWARY-001 | Low |
| Fixed-point arithmetic bounds | YP-NUMERICAL-FIXEDPOINT-001 | Medium |

## Toolchain

```
leanprover/lean4:v4.29.0
Mathlib @ b301d257a1c13bc4e27350c06e5169b8b08a53ed
```
