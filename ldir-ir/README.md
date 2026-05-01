# ldir-ir

S-IR (Source IR) and G-IR (Graphic IR) data structures for the LDIR document pipeline.

S-IR is a tree-structured document format where each instruction uses a fixed 13-byte wire-format header (opcode, entity ID, parent ID, payload offset). G-IR is a flat per-page command buffer with 36-byte structs and 8 x i32 argument slots.

## Features

- 13-byte wire format for S-IR instructions
- rkyv serialization with round-trip fidelity
- Well-formedness validation: single root, acyclic parent graph, unique entity IDs, bounded payloads
- Style modifiers for inline formatting (bold, italic, monospace)

## Example

```rust
use ldir_ir::sir::{SIRDocument, SIRInstruction, SIROpcode, BlockType, StyleModifier, ROOT_SENTINEL};

let mut doc = SIRDocument::new();

// Root block
let root_id = 0;
let payload_off = doc.payload_mut().append(&[BlockType::Document as u8]);
doc.push(SIRInstruction::new(SIROpcode::PushBlock, root_id, ROOT_SENTINEL, payload_off));

// Paragraph with content
let para_off = doc.payload_mut().append(&[BlockType::Paragraph as u8]);
doc.push(SIRInstruction::new(SIROpcode::PushBlock, 1, root_id, para_off));
doc.push_with_payload(
    SIRInstruction::new(SIROpcode::SetContent, 2, 1, 0),
    b"Hello, world.",
);

// Bold style modifier
let packed = StyleModifier::push(StyleModifier::BOLD);
doc.push(SIRInstruction::new(SIROpcode::ApplyStyle, 3, 1, packed));

// Validate
let result = ldir_core::validator::validate_sir(&doc);
assert!(result.is_ok());
```

## License

MIT OR Apache-2.0
