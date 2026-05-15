# ADR-0001: Justified unwrap/expect Exceptions

## Status: Accepted

## Context

LDIR enforces a zero-unwrap/expect policy in production code paths. After a
full audit, three exceptions were identified where unwrap/expect is justified
by invariant guarantees that cannot fail under correct usage.

## Decision

Three unwrap/expect calls are permitted with explicit documentation and
`#[allow(clippy::expect_used)]` or `#[allow(clippy::unwrap_used)]` annotations.

### Exception 1 & 2: rkyv serialization in SIRDocument

**File:** `ldir-ir/src/sir/document.rs`, lines 207 and 233

```rust
let instr_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&self.instructions)
    .expect("INVARIANT: rkyv serialization of well-formed SIRDocument never fails");
```

**Justification:**
- rkyv serialization of `Vec<SIRInstruction>` (a plain struct with primitive fields) cannot fail
- The `SIRInstruction` type contains only `u8`, `u32`, and `BlockType` fields -- all trivially serializable
- The document has already passed validation (`validate_sir`) before serialization is called in normal flows
- This is guarded by the INVARIANT comment and annotated with `#[allow(clippy::expect_used)]`

### Exception 3: len()-guarded unwrap in linker

**File:** `ldir-link/src/linker.rs`, line 37

```rust
let node = &self.nodes[self.nodes.len() - 1];
```

**Justification:**
- The unwrap (array index) is guarded by a preceding `assert!(!self.nodes.is_empty())` check
- The linker only calls this after pushing at least one node onto the stack
- The invariant is that the node stack is never empty when this code path is reached

## Consequences

- **Positive:** Avoids unnecessary `Option`/`Result` wrapping for provably safe operations
- **Positive:** Makes the invariant reasoning explicit in code comments
- **Negative:** Any future change to `SIRInstruction` layout or linker flow must re-verify these invariants
- **Mitigation:** CI pre-commit hook scans for new unwrap/expect calls and fails if any are added without documentation

## Alternatives Considered

1. **Return Result from all methods:** Would propagate rkyv errors through every call site for an error that provably cannot occur. Adds noise without safety benefit.
2. **Use `unsafe` with `unreachable_unchecked()`:** Would save a branch but is unsound if the invariant is ever violated. `expect()` with a message is safer -- it panics with a clear message if the invariant is broken.

## Related Standards

- MISRA-C:2012 Rule 17.7 (return values shall be checked) -- exception documented
- IEC 61508: Software safety integrity -- these are not safety-critical paths

## Date: 2026-05-15
