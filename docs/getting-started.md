# Getting Started with LDIR

A 5-minute guide to compiling your first document with the LDIR typesetting pipeline.

## Prerequisites

- **Rust 1.88+** (edition 2024). No other dependencies are needed for basic use.
- Verify: `rustc --version` should show `1.88` or later.

## Installation

### Option A: Cargo dependencies (recommended for libraries)

```toml
# Cargo.toml
[dependencies]
ldir-core = "0.1"
ldir-ir = "0.1"
```

### Option B: Workspace clone (for development)

```bash
git clone https://github.com/WyattAu/ldir.git
cd ldir
cargo build
```

## Quick Start

This example builds a minimal document tree, validates it, compiles to G-IR,
and emits binary output. Create a new binary project and paste this into
`src/main.rs`:

```rust
use ldir_core::compiler::compile_sir;
use ldir_core::emitter::emit_gir;
use ldir_core::validator::validate_sir;
use ldir_ir::sir::{ROOT_SENTINEL, SIRDocument, SIRInstruction, SIROpcode};

fn main() {
    let mut doc = SIRDocument::new();

    // Document root (parent = ROOT_SENTINEL = 0xFFFFFFFF)
    doc.push(SIRInstruction::new(SIROpcode::PushBlock, 0, ROOT_SENTINEL, 0));

    // Paragraph block (child of document)
    doc.push(SIRInstruction::new(SIROpcode::PushBlock, 1, 0, 0));

    // Text content (child of paragraph)
    doc.push(SIRInstruction::new(SIROpcode::SetContent, 2, 1, 0));

    // Validate, compile, emit
    validate_sir(&doc).expect("well-formed");
    let gir = compile_sir(&doc).expect("compiles");
    let bytes = emit_gir(&gir);

    println!("{} bytes, {} commands", bytes.len(), gir.total_commands());
}
```

Run with `cargo run`.

## Step-by-Step Walkthrough

### Step 1: Create S-IR instructions

S-IR (Source Intermediate Representation) is a tree-structured document
represented as a flat list of instructions. Each instruction has four fields:

| Field            | Type | Description                                    |
|------------------|------|------------------------------------------------|
| `opcode`         | `SIROpcode` | Operation: `PushBlock`, `SetContent`, `ApplyStyle`, `InsertMath`, `LinkData` |
| `entity_id`      | `u32` | Unique identifier (must be distinct per document) |
| `parent_id`      | `u32` | Parent entity ID, or `ROOT_SENTINEL` for the root |
| `payload_offset` | `u32` | Offset into the payload region (text, style data) |

```rust
use ldir_ir::sir::{ROOT_SENTINEL, SIRDocument, SIRInstruction, SIROpcode};

let mut doc = SIRDocument::new();

// Document root — the single top-level node
doc.push(SIRInstruction::new(SIROpcode::PushBlock, 0, ROOT_SENTINEL, 0));

// Paragraph block — child of the document root (entity_id 0)
doc.push(SIRInstruction::new(SIROpcode::PushBlock, 1, 0, 0));

// Text content — child of the paragraph (entity_id 1)
doc.push(SIRInstruction::new(SIROpcode::SetContent, 2, 1, 0));
```

The `parent_id` field encodes the tree structure:
```
entity 0 (Document, root)
└── entity 1 (Paragraph)
    └── entity 2 (Content)
```

### Step 2: Validate the S-IR document

The validator runs six well-formedness checks (matching the Lean 4 formal
specification in `ProofIRWellformedness.lean`):

1. **AX-001**: Entity IDs are unique
2. **AX-002**: Parent references exist
3. **AX-003**: No circular parent chains
4. **AX-004**: Payload offsets in bounds
5. **DEF-004.5**: Exactly one root entity
6. **DEF-004.6**: Valid block nesting

```rust
use ldir_core::validator::validate_sir;

match validate_sir(&doc) {
    Ok(()) => println!("Document is well-formed"),
    Err(errors) => {
        for e in &errors {
            println!("  Error: {}", e);
        }
    }
}
```

### Step 3: Compile S-IR to G-IR

The compiler traverses the S-IR tree (DFS) and emits G-IR (Graphical
Intermediate Representation) rendering commands:

- `PushBlock` → `PushStack`, `MoveXY`, `PopStack`
- `SetContent` → `PutGlyph` per character
- `ApplyStyle` → `SetFont`
- `InsertMath` → `AttachMetadata` (placeholder)
- `LinkData` → `AttachMetadata`

```rust
use ldir_core::compiler::compile_sir;

let gir = compile_sir(&doc)?;

// G-IR is a sequence of pages, each containing rendering commands
println!("Pages: {}", gir.page_count());
println!("Total commands: {}", gir.total_commands());
println!("Well-formed: {}", gir.is_well_formed());
```

### Step 4: Emit G-IR to bytes

The binary format starts with a 4-byte magic (`GIR0`) followed by page
and command data in little-endian format:

```rust
use ldir_core::emitter::emit_gir;

let bytes = emit_gir(&gir);

// Header: 4 bytes magic + 4 bytes page count
assert_eq!(&bytes[0..4], b"GIR0");
let page_count = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
println!("Binary: {} bytes, {} pages", bytes.len(), page_count);
```

### Step 5: Verify the output

Round-trip verification ensures the binary format is correct:

```rust
use ldir_core::emitter::parse_gir;

let restored = parse_gir(&bytes)?;
assert_eq!(gir, restored, "round-trip must be identical");
```

You can also serialize S-IR to rkyv bytes for storage or transport:

```rust
let sir_bytes = doc.to_bytes();
let restored_doc = SIRDocument::from_bytes(&sir_bytes)?;
assert_eq!(doc, restored_doc);
```

## Next Steps

- **API Reference**: See `.docs/api_reference.md` for complete type documentation.
- **Examples**: Run `cargo run --example <name> --package ldir-core`:
  - `hello-world` — minimal document compilation
  - `custom-style` — style application with fonts and colors
  - `incremental-edit` — modifying and recompiling documents
  - `markdown-to-pdf` -- Markdown to PDF pipeline
  - `tex-basic` -- TeX input support
- **26.6 Fixed-Point**: See `ldir_core::fp266::Fp266` for coordinate arithmetic.
- **ECS**: See `ldir_core::ecs::World` for the entity component system.

## Architecture Overview

```
Input (MD/TeX)  ->  S-IR  ->  [validate]  ->  [layout]  ->  L-IR  ->  [link]  ->  G-IR  ->  [emit]  ->  Binary
                              |                                                      |
                         Lean 4 proofs                                           PDF/Vello
                        (well-formedness)                                    (rendering backends)
```

| Crate     | Role                              |
|-----------|-----------------------------------|
| `ldir-ir` | S-IR, L-IR, and G-IR type definitions |
| `ldir-core` | Validator, layout compiler, emitter |
| `ldir-link` | L-IR to G-IR linking              |
| `ldir-md` | Markdown parser                   |
| `ldir-tex` | TeX parser                       |
| `ldir-pdf` | PDF/A-4 backend                  |
| `ldir-vello` | GPU renderer                    |
