# ldir-ir

IR type definitions for the LDIR document pipeline: S-IR, L-IR, and G-IR
with rkyv serialization support.

S-IR is a tree-structured document format with 13-byte fixed-cost instruction
headers. L-IR is a positioned box tree with 26.6 fixed-point geometry. G-IR
is a flat per-page command buffer with 36-byte structs.

## Features

- **S-IR**: Tree-structured document representation with entity-based addressing
- **L-IR**: Positioned box tree with 23 node types for layout decisions
- **G-IR**: Linearized per-page rendering command buffer
- **Fp266**: 26.6 fixed-point arithmetic for geometry (exact add, rounding mul)
- **rkyv serialization**: Zero-copy deserialization for S-IR types
- **Well-formedness validation**: Single root, acyclic parent graph, unique entity IDs

## API Overview

| Type | Description |
|------|-------------|
| `SIRDocument` / `SIRInstruction` | S-IR document and instruction types |
| `SIROpcode` / `BlockType` | S-IR opcodes and block classifications |
| `StyleModifier` | Inline style flags (bold, italic, mono) |
| `LIRDocument` / `LIRNode` | L-IR layout tree with 23 node variants |
| `Fp266` | 26.6 fixed-point number for geometry |
| `GIRDocument` / `GIRCommand` | G-IR rendering command buffer |
| `PayloadRegion` | Variable-length payload storage for S-IR |

## Usage

```rust
use ldir_ir::sir::{SIRDocument, SIRInstruction, SIROpcode, BlockType, ROOT_SENTINEL};

let mut doc = SIRDocument::new();

// Root block
let payload_off = doc.payload_mut().append(&[BlockType::Document as u8]);
doc.push(SIRInstruction::new(SIROpcode::PushBlock, 0, ROOT_SENTINEL, payload_off));

// Paragraph with content
let para_off = doc.payload_mut().append(&[BlockType::Paragraph as u8]);
doc.push(SIRInstruction::new(SIROpcode::PushBlock, 1, 0, para_off));
doc.push_with_payload(
    SIRInstruction::new(SIROpcode::SetContent, 2, 1, 0),
    b"Hello, world.",
);
```

## Wire Format

- **S-IR**: 13-byte header (OpCode:1 + EntityID:4 + ParentID:4 + PayloadOffset:4)
- **G-IR**: 36-byte struct (OpCode:1 + Padding:3 + 8 x i32 args)

## License

MIT OR Apache-2.0

## Repository

[https://github.com/WyattAu/ldir](https://github.com/WyattAu/ldir)
