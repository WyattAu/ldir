# ldir-core

Compiler, validator, and emitter for the LDIR document pipeline.

Transforms S-IR documents into G-IR rendering commands through a multi-stage compilation pipeline with real font shaping and deterministic layout.

## Modules

- **compiler** -- S-IR to G-IR compilation with font context
- **validator** -- S-IR well-formedness validation (unique IDs, acyclic, single root)
- **emitter** -- G-IR binary serialization
- **parser** -- S-IR deserialization from rkyv bytes
- **verifier** -- G-IR well-formedness verification
- **fp266** -- 26.6 fixed-point arithmetic (exact add, rounding multiply)
- **ecs** -- Entity Component System
- **font** -- Font loading via ttf-parser and fontdb
- **shaping** -- Text shaping via HarfBuzz
- **layout** -- Knuth-Plass line breaking
- **solver** -- Cassowary constraint solver

## Example

```rust
use ldir_core::{compiler::compile_sir, validator::validate_sir};
use ldir_ir::sir::{SIRDocument, SIRInstruction, SIROpcode, ROOT_SENTINEL};

// Build an S-IR document
let mut doc = SIRDocument::new();
doc.push(SIRInstruction::new(SIROpcode::PushBlock, 0, ROOT_SENTINEL, 0));
doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0));

// Validate
validate_sir(&doc).expect("well-formed document");

// Compile S-IR -> G-IR
let gir = compile_sir(&doc).expect("compilation succeeds");
assert!(gir.is_well_formed());
assert_eq!(gir.page_count(), 1);

// Emit binary
let bytes = ldir_core::emitter::binary::emit_gir(&gir);
```

## License

MIT OR Apache-2.0
