# LDIR Knowledge Base — Patterns, Anti-Patterns, and Lessons Learned

**Document ID:** KB-LL-001
**Version:** 1.0.0
**Status:** APPROVED
**Date:** 2026-04-23
**Phase:** 12 — Knowledge Transfer
**Transition Date:** Phase 11 → Phase 12: 2026-04-23

---

## 1. Overview

This knowledge base captures reusable patterns, known anti-patterns, and actionable lessons from the LDIR R&D lifecycle. Each entry has a unique ID for cross-referencing.

---

## 2. Patterns

### PAT-001: Lean4 Specification-First Development

| Field | Value |
|-------|-------|
| **Domain** | Formal Verification / Methodology |
| **Confidence** | High (validated across 7 Yellow Papers) |
| **Applicability** | Rust projects with correctness-critical components |

Write Lean4 specifications and proofs before Rust implementation. Lean4 is the authoritative truth; Rust is the executable interpretation.

**Practice:** (1) Define types/invariants in Lean4, (2) state theorems about properties, (3) attempt proofs to validate spec consistency, (4) write Rust mirroring Lean4 structure, (5) use `debug_assert!` for preconditions.

**Evidence:** BP-IR-COMPILER-001 Section 9; 10/13 theorems proven before any Rust code.

---

### PAT-002: TOML Over YAML/JSON for Configuration

| Field | Value |
|-------|-------|
| **Domain** | Tooling / Configuration |
| **Confidence** | High |
| **Applicability** | Rust projects, developer-facing configuration |

Use TOML for all configuration files (registries, test vectors, constraints, CI manifests). TOML's explicit typing eliminates ambiguity; no significant whitespace bugs; first-class Rust support via `toml` crate.

**Evidence:** YAML's type coercion (`1.0` → float, `on` → bool) caused 3 config bugs in prototypes that TOML eliminated entirely. Registry files: `yellow_paper_registry.toml`, `blue_paper_registry.toml`, `test_vectors_ir.toml`, `domain_constraints_typesetting.toml`.

---

### PAT-003: Partial Def with Fuel for Termination Guarantees

| Field | Value |
|-------|-------|
| **Domain** | Lean4 / Formal Verification |
| **Confidence** | High |
| **Applicability** | Lean4 projects with recursive functions |

Use `partial def` with explicit fuel parameter when the kernel can't verify termination. Fuel provides an external termination argument: if the function terminates for fuel = N, it terminates for all inputs of size < N.

```lean
partial def compile (fuel : Nat) (doc : SIRDocument) : GIRDocument :=
  match fuel with
  | 0 => .empty
  | Nat.succ fuel' =>
    match doc.instructions with
    | [] => .empty
    | instr :: rest =>
      processInstruction instr (compile fuel' { doc with instructions := rest })
```

**Trade-off:** `partial def` cannot be used in `theorem` proofs directly. Use `@[simp]` lemmas about the non-partial variant for property proofs.

---

### PAT-004: Fixed-Point Arithmetic for Cross-Platform Determinism

| Field | Value |
|-------|-------|
| **Domain** | Numerical Computing / Typesetting |
| **Confidence** | High (formally verified: YP-NUMERICAL-FIXEDPOINT-001) |
| **Applicability** | Systems requiring bit-identical output across platforms |

Use 26.6 fixed-point integers (Q26.6) instead of IEEE-754 for all geometric calculations. Integer arithmetic is deterministic across x86-64, AArch64, and any architecture.

```rust
type Fp26_6 = i32;
const FP_SCALE: i32 = 64;
fn fp_mul(a: Fp26_6, b: Fp26_6) -> Fp26_6 {
    ((a as i64 * b as i64) >> 6).clamp(i32::MIN, i32::MAX) as i32
}
```

**Benefits:** Addition/subtraction exact; matches FreeType internal format; bounded error ±1/128.

---

## 3. Anti-Patterns

### AP-001: Using f64 for Layout Calculations

| Field | Value |
|-------|-------|
| **Severity** | Critical (breaks REQ-2.6, REQ-11.3.1) |
| **Detection** | Clippy lint: `deny(clippy::float_arithmetic)` |

IEEE-754 is non-deterministic: extended precision registers (x87 80-bit vs SSE 64-bit), FMA availability, compiler flags (`-ffast-math`), and rounding modes differ across platforms.

```rust
// BROKEN: different G-IR on x86 vs ARM
let x: f64 = width * 0.5 + margin;
// CORRECT: deterministic across all platforms
let x: i32 = fp_mul(width, fp_from_float(0.5)) + margin;
```

---

### AP-002: Trusting rkyv Data Without Validation

| Field | Value |
|-------|-------|
| **Severity** | Critical (TM-003) |
| **Detection** | Code review, security audit |

rkyv zero-copy deserialization trusts binary format. Corrupted bytes produce `SIRDocument` with invalid invariants (cyclic refs, OOB offsets).

```rust
// BROKEN: trusts rkyv directly
let doc: SIRDocument = rkyv::from_bytes(bytes).unwrap();
compile_sir(&doc); // may panic on cyclic parent refs
// CORRECT: validate before compilation
validate_sir(&doc)?;
compile_sir(&doc);
```

---

### AP-003: Mathlib Lemma Discovery Before Proof Strategy

| Field | Value |
|-------|-------|
| **Severity** | Medium (wastes 4-8h per occurrence) |
| **Detection** | Time on `simp` without progress |

Searching Mathlib for lemmas before formulating a strategy leads to "lemma hunting." The correct approach: (1) write proof on paper, (2) identify logical steps, (3) search Mathlib for specific lemmas per step, (4) if missing, prove locally or restructure.

---

## 4. Lessons Learned

### LL-001: Mathlib's List.eraseDups Lacks Lemmas in Pinned Versions

| Field | Value |
|-------|-------|
| **Impact** | Blocked 3 proofs (23.1% of total) |
| **Resolution** | Upgrade Mathlib or prove locally |

`List.eraseDups` in the pinned Mathlib lacks `List.eraseDups_length_le` and `List.Sublist.eraseDups`. **Action:** Before pinning Mathlib, run `lake exe mathlib doc` to check lemma availability. If upgrading isn't feasible, prove missing lemmas locally.

```lean
lemma eraseDups_length_le (l : List α) : (l.eraseDups).length ≤ l.length := by
  induction l with | nil => simp | cons x xs ih => simp [List.eraseDups_cons]; split <;> omega
```

---

### LL-002: Bool.and_eq_true_iff Is the Key to Bool Conjunction Proofs

| Field | Value |
|-------|-------|
| **Impact** | Unblocks 5+ proof goals |
| **Resolution** | Use as first tactic on `&&` goals |

Many Lean4 goals reduce to `a && b = true`. The lemma `Bool.and_eq_true_iff : (a && b = true) ↔ (a = true ∧ b = true)` is the single most useful tactic.

```lean
-- Stuck: `a && b = true`
rw [Bool.and_eq_true_iff]  -- becomes: `a = true ∧ b = true`
-- Now `simp` or `omega` handles it
```

For `||`, use `Bool.or_eq_true_iff`. For negation, use `Bool.not_eq_true`.

---

### LL-003: Lake Toolchain Mismatch Between System and Project

| Field | Value |
|-------|-------|
| **Impact** | Build failures, wasted 2-3 hours |
| **Resolution** | Use `elan`; respect `lean-toolchain` |

Installing a different Lean4 version system-wide than the project's `lean-toolchain` causes opaque Lake failures.

**Action:** (1) Never install Lean4 globally — use `elan`, (2) `elan` reads `lean-toolchain` and auto-switches, (3) Verify: `lean --version` matches `lean-toolchain`, (4) After changing `lean-toolchain`, run `lake exe cache get`.

```bash
lean --version && cat lean-toolchain  # must match
elan install $(cat lean-toolchain) && lake exe cache get && lake build
```

---

### LL-004: brecOn-Based Definitions Are Opaque to Standard Tactics

| Field | Value |
|-------|-------|
| **Impact** | Proof goals become unprovable with standard tactics |
| **Resolution** | Use explicit `match` or `termination_by` |

Definitions using `Nat.brecOn` or `WellFounded.fix` produce goals where `simp`, `induction`, `omega`, and `aesop` cannot progress. The recursion structure is hidden inside the brecOn motive.

```lean
-- BROKEN: brecOn makes proofs opaque
def foo (n : Nat) : Nat := Nat.brecOn n (fun ⟨_, ih⟩ => ...)
-- CORRECT: explicit match, standard tactics work
def foo (n : Nat) : Nat := match n with | 0 => 0 | Nat.succ n' => foo n' + 1
```

**Action:** Prefer `match` with `termination_by`. Only use `brecOn` for genuinely complex termination arguments.

---

## 5. Cross-Reference Index

| ID | Type | Related YPs | Related BPs |
|----|------|-------------|-------------|
| PAT-001 | Pattern | All 7 YPs | BP-IR-COMPILER-001 |
| PAT-002 | Pattern | — | — |
| PAT-003 | Pattern | YP-IR-SEMANTICS-001 | BP-IR-COMPILER-001 |
| PAT-004 | Pattern | YP-NUMERICAL-FIXEDPOINT-001 | BP-IR-COMPILER-001 |
| AP-001 | Anti-pattern | YP-NUMERICAL-FIXEDPOINT-001 | BP-IR-COMPILER-001 |
| AP-002 | Anti-pattern | YP-IR-SEMANTICS-001 | BP-IR-COMPILER-001 |
| AP-003 | Anti-pattern | YP-IR-SEMANTICS-001 | — |
| LL-001 | Lesson | YP-IR-SEMANTICS-001 | BP-IR-COMPILER-001 §9.3 |
| LL-002 | Lesson | — | — |
| LL-003 | Lesson | — | — |
| LL-004 | Lesson | YP-IR-SEMANTICS-001 | — |

---

*End of KB-LL-001 v1.0.0*
