# ldir-core

Compiler, validator, emitter, and font management for the LDIR document
pipeline. Transforms S-IR documents through layout into G-IR rendering
commands with real font shaping and deterministic layout.

## Features

- **S-IR validation**: Well-formedness checks (unique IDs, acyclic, single root)
- **S-IR to G-IR compilation**: Knuth-Plass line breaking, font shaping
- **S-IR to L-IR compilation**: Layout tree generation
- **G-IR emission**: Binary serialization and parsing
- **G-IR verification**: Coordinate range, font precedence, stack balance
- **Font management**: Loading via ttf-parser/fontdb, HarfBuzz shaping
- **26.6 fixed-point**: Exact arithmetic for coordinates
- **Plugin system**: Custom frontend and backend plugins
- **Source mapping**: Bidirectional source-to-S-IR entity mapping (LSP)

## API Overview

| Function / Type | Description |
|-----------------|-------------|
| `compiler::compile_sir` | S-IR to G-IR compilation |
| `compile_sir_to_lir` | S-IR to L-IR layout compilation |
| `validator::validate_sir` | S-IR well-formedness validation |
| `emitter::emit_gir` | G-IR binary serialization |
| `parser::parse_sir` | S-IR deserialization from rkyv |
| `verifier::check_gir` | G-IR well-formedness verification |
| `font::FontDatabase` | Font loading and management |
| `error::LdirError` | Structured error hierarchy |
| `plugin::PluginRegistry` | Plugin registration |

## Usage

```rust
use ldir_core::compiler::compile_sir;
use ldir_core::validator::validate_sir;
use ldir_ir::sir::{SIRDocument, SIRInstruction, SIROpcode, ROOT_SENTINEL};

let mut doc = SIRDocument::new();
doc.push(SIRInstruction::new(SIROpcode::PushBlock, 0, ROOT_SENTINEL, 0));
doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0));

validate_sir(&doc).expect("well-formed document");
let gir = compile_sir(&doc).expect("compilation succeeds");
assert!(gir.is_well_formed());
```

## License

MIT OR Apache-2.0

## Repository

[https://github.com/WyattAu/ldir](https://github.com/WyattAu/ldir)
