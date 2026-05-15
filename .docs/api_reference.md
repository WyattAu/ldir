# LDIR API Reference

## Crate Overview

| Crate | Description |
|-------|-------------|
| `ldir_ir` | S-IR, L-IR, and G-IR type definitions with rkyv serialization |
| `ldir_core` | fp26_6 arithmetic, ECS, S-IR validator, compiler, and G-IR emitter |

---

## `ldir_ir` — Intermediate Representation Types

### S-IR Module (`ldir_ir::sir`)

#### Types

| Type | Description |
|------|-------------|
| `SIRDocument` | Ordered collection of S-IR instructions representing a document |
| `SIRInstruction` | Atomic S-IR operation with 13-byte wire-format header (REQ-3.1.2) |
| `SIROpcode` | S-IR operation discriminator: `PushBlock`, `SetContent`, `ApplyStyle`, `InsertMath`, `LinkData` |
| `BlockType` | Block type for `PushBlock`: `Document`, `Paragraph`, `Heading`, `List`, `Math`, `Code` |
| `PayloadRegion` | Contiguous region of variable-length payload data |
| `EntityId` | `u32` — 32-bit entity identifier |
| `ROOT_SENTINEL` | `0xFFFF_FFFF` — sentinel parent ID indicating root entity |
| `INSTRUCTION_WIRE_SIZE` | `13` — bytes per S-IR instruction in wire format |

#### Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `serialize_sir` | `(doc: &SIRDocument) -> Vec<u8>` | Serialize to rkyv bytes |
| `deserialize_sir` | `(bytes: &[u8]) -> Result<SIRDocument, rkyv::rancor::Error>` | Deserialize from rkyv bytes |

### G-IR Module (`ldir_ir::gir`)

#### Types

| Type | Description |
|------|-------------|
| `GIRDocument` | Ordered sequence of G-IR pages representing a compiled document |
| `GIRPage` | Ordered sequence of G-IR rendering commands for one page |
| `GIRCommand` | Single G-IR rendering command with opcode and 8 x i32 args (36 bytes) |
| `GIROpcode` | G-IR operation discriminator: `SetFont`, `MoveXY`, `PutGlyph`, `DrawRule`, `PushStack`, `PopStack`, `AttachMetadata` |
| `GIRStyle` | Style entry defining font, size, and color |
| `StyleTable` | Table of style entries indexed by style ID |
| `GIR_COMMAND_ARGS` | `8` — number of argument slots per G-IR command |

### L-IR Module (`ldir_ir::lir`)

#### Types

| Type | Description |
|------|-------------|
| `LIRDocument` | Positioned layout document with pages and style table |
| `LIRPage` | Single page containing positioned L-IR nodes |
| `LIRNode` | Layout node enum: `Glyph`, `Line`, `Paragraph`, `Heading`, `List`, `Table`, etc. |
| `LIRGeometry` | Positioned geometry with x, y, width, height, baseline |
| `LIRStyleTable` | Table of text styles indexed by style ID |
| `LIRTextStyle` | Text style entry with font ID, size, weight |

---

## `ldir_core` — Compilation Pipeline

### `fp266` Module — 26.6 Fixed-Point Arithmetic

#### Constants

| Constant | Type | Value | Description |
|----------|------|-------|-------------|
| `FRACTIONAL_BITS` | `u32` | `6` | Number of fractional bits |
| `SCALE` | `i64` | `64` | 2^6 scale factor |
| `MIN_RAW` | `i64` | `i32::MIN * 64` | Minimum raw value |
| `MAX_RAW` | `i64` | `i32::MAX * 64 + 63` | Maximum raw value |
| `MAX_VALUE` | `f64` | ~524287.99 | Maximum representable float |
| `MIN_VALUE` | `f64` | ~-524288.0 | Minimum representable float |
| `ERROR_BOUND` | `f64` | `1/128` | Error bound per operation (THM-FP-MUL-ROUND) |

#### `Fp266` Struct

A 26.6 fixed-point number stored as an `i64`. Range: [-524288.0, 524287.9921875].

**Associated constants:** `ZERO`, `ONE`, `HALF`

#### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `from_raw` | `(raw: i64) -> Self` | Create from raw 26.6 value |
| `from_int` | `(value: i32) -> Self` | Create from integer |
| `from_frac` | `(num: i32, den: i32) -> Self` | Create from fraction (truncation) |
| `from_f64` | `(value: f64) -> Self` | Quantize from f64 (round-to-nearest, REQ-3.2.7) |
| `raw` | `() -> i64` | Get raw 26.6 value |
| `to_f64` | `() -> f64` | Convert to f64 |
| `to_int` | `() -> i32` | Convert to integer (truncation) |
| `fractional` | `() -> i32` | Get fractional part (0..63) |
| `saturating` | `(raw: i64) -> Self` | Saturating construction (THM-FP-SATURATION) |
| `mul` | `(other: Self) -> Self` | Fixed-point multiply (ALG-FP-MUL, <=0.5 ULP error) |
| `div` | `(other: Self) -> Self` | Fixed-point divide |
| `sqrt` | `() -> Self` | Integer square root (Newton's method) |
| `abs` | `() -> Self` | Absolute value |
| `min` / `max` | `(other: Self) -> Self` | Min/max |
| `clamp` | `(lo: Self, hi: Self) -> Self` | Clamp to range |
| `is_zero` | `() -> bool` | Check if zero |

**Operator impls:** `Add`, `Sub`, `Neg`, `Mul<i32>`, `AddAssign`, `SubAssign`, `Display`, `PartialOrd`, `Ord`, `Hash`

---

### `compiler` Module — S-IR to G-IR Compilation

#### Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `compile_sir` | `(doc: &SIRDocument) -> Result<GIRDocument>` | Compile well-formed S-IR to G-IR (IF-COMPILE-001) |

#### `CompileContext` Struct

Mutable compilation state tracking cursor position, stack depth, page dimensions.

**Constants:** `MAX_STACK_DEPTH` (256), `DEFAULT_PAGE_WIDTH_PT` (612), `DEFAULT_PAGE_HEIGHT_PT` (792), `DEFAULT_FONT_SIZE_PT` (12), `DEFAULT_GLYPH_ADVANCE_PT` (7), `DEFAULT_LINE_HEIGHT_FACTOR` (6), margins (72pt each)

#### `InstructionTree` Struct

Adjacency-list tree built from flat S-IR for DFS traversal.

#### `emit_helpers` Functions

| Function | Description |
|----------|-------------|
| `emit_push_stack` | Emit PushStack command |
| `emit_pop_stack` | Emit PopStack command |
| `emit_set_font` | Emit SetFont command |
| `emit_move_xy` | Emit MoveXY command |
| `emit_text_content` | Emit PutGlyph per character |
| `emit_draw_rule` | Emit DrawRule command |
| `emit_attach_metadata` | Emit AttachMetadata command |
| `balance_stack` | Emit PopStack to balance stack depth |
| `emit_paragraph_spacing` | Advance Y and reset X for paragraph spacing |

---

### `validator` Module — S-IR Well-Formedness

#### Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `validate_sir` | `(doc: &SIRDocument) -> ValidationResult` | Run all 6 well-formedness checks (IF-VALIDATE-001) |
| `entity_unique::check` | `(doc: &SIRDocument) -> Vec<LdirError>` | AX-001: all entity IDs distinct |
| `parent_exists::check` | `(doc: &SIRDocument) -> Vec<LdirError>` | AX-002: parent references valid |
| `acyclicity::check` | `(doc: &SIRDocument) -> Vec<LdirError>` | AX-003: no circular parent chains |
| `root_count::check` | `(doc: &SIRDocument) -> Vec<LdirError>` | DEF-004.5: exactly one root |
| `block_nesting::check` | `(doc: &SIRDocument) -> Vec<LdirError>` | DEF-004.6: valid block nesting |
| `payload_integrity::check` | `(doc: &SIRDocument) -> Vec<LdirError>` | AX-004: payload offsets in bounds |

---

### `parser` Module — S-IR Deserialization

#### Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `parse_sir` | `(bytes: &[u8]) -> Result<SIRDocument>` | Parse S-IR from rkyv bytes (IF-PARSE-001) |

---

### `emitter` Module — G-IR Binary Emission

#### Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `emit_gir` | `(doc: &GIRDocument) -> Vec<u8>` | Serialize G-IR to binary (IF-EMIT-001) |
| `parse_gir` | `(bytes: &[u8]) -> Result<GIRDocument>` | Deserialize G-IR from binary |

#### Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `GIR_MAGIC` | `b"GIR0"` | Magic bytes for G-IR binary format |
| `HEADER_SIZE` | `8` | Binary header size |
| `PAGE_HEADER_SIZE` | `12` | Per-page header size |
| `COMMAND_SIZE` | `36` | Per-command size in binary format |

---

### `verifier` Module — G-IR Well-Formedness

#### Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `check_gir` | `(doc: &GIRDocument) -> Result<(), Vec<String>>` | Check G-IR well-formedness (DEF-005) |

---

### `error` Module — Error Types

#### Types

| Type | Description |
|------|-------------|
| `LdirError` | Structured error with `kind`, `entity_id`, `byte_offset` |
| `ErrorKind` | Top-level category: `Parse`, `Validation`, `Compile`, `Emit` |
| `ParseErrorKind` | `InputTooShort`, `AlignmentError`, `InvalidOpcode`, `DeserializationError` |
| `ValidationErrorKind` | `DuplicateEntityId`, `ParentNotFound`, `CircularParentChain`, `PayloadOutOfBounds`, `MultipleRoots`, `NoRoot`, `InvalidBlockNesting` |
| `CompileErrorKind` | `StackOverflow`, `UnsupportedInstruction` |
| `EmitErrorKind` | `BufferOverflow` |
| `Result<T>` | `std::result::Result<T, LdirError>` |
| `ValidationResult` | `Result<(), Vec<LdirError>>` |

---

### `ecs` Module — Entity Component System

#### Types

| Type | Description |
|------|-------------|
| `Arena<T>` | Bump allocator for contiguous value storage |
| `SparseSet<K>` | O(1) entity-to-index mapping via HashMap + dense Vec |
| `ComponentStore<T>` | Typed SoA component storage with sparse set lookup |
| `Entity` | Versioned entity: `(index: u32, generation: u32)` |
| `EntityAllocator` | Bump allocator with generation tracking and free list |
| `World` | Top-level ECS container with type-erased component stores |
| `ComponentId` | `u8` — component type identifier (up to 256 types) |
| `EntityId` | `u32` — raw entity slot index |

---

## Code Examples

### 1. Create and compile an S-IR document

```rust
use ldir_core::compiler::compile_sir;
use ldir_core::validator::validate_sir;
use ldir_ir::sir::{SIRDocument, SIRInstruction, SIROpcode, ROOT_SENTINEL};

let mut doc = SIRDocument::new();
doc.push(SIRInstruction::new(SIROpcode::PushBlock, 0, ROOT_SENTINEL, 0));
doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0));
doc.push(SIRInstruction::new(SIROpcode::ApplyStyle, 2, 0, 0));

validate_sir(&doc).expect("well-formed");
let gir = compile_sir(&doc).expect("compiles");
assert!(gir.is_well_formed());
```

### 2. 26.6 fixed-point arithmetic

```rust
use ldir_core::fp266::Fp266;

let a = Fp266::from_int(3);
let b = Fp266::from_frac(1, 2);
let sum = a + b;           // 3.5 (exact, THM-FP-ADD-EXACT)
let product = a.mul(b);    // 1.5 (<=0.5 ULP error, THM-FP-MUL-ROUND)
assert_eq!(sum.to_f64(), 3.5);
```

### 3. G-IR binary round-trip

```rust
use ldir_core::emitter::{emit_gir, parse_gir};
use ldir_ir::gir::{GIRDocument, GIRPage, GIRCommand};

let mut doc = GIRDocument::new();
let mut page = GIRPage::new();
page.push(GIRCommand::new_push_stack());
page.push(GIRCommand::new_set_font(0));
page.push(GIRCommand::new_pop_stack());
doc.push_page(page);

let bytes = emit_gir(&doc);
let restored = parse_gir(&bytes).unwrap();
assert_eq!(doc, restored);
```

### 4. ECS world with typed components

```rust
use ldir_core::ecs::World;

#[derive(Debug, PartialEq)]
struct Health(i32);

let mut world = World::new();
world.register::<Health>();

let entity = world.allocate_entity();
world.insert_component(entity, Health(100));

assert_eq!(world.get_component::<Health>(entity), Some(&Health(100)));
```

### 5. S-IR rkyv serialization round-trip

```rust
use ldir_ir::sir::{SIRDocument, SIRInstruction, SIROpcode, ROOT_SENTINEL};

let mut doc = SIRDocument::new();
doc.push(SIRInstruction::new(SIROpcode::PushBlock, 0, ROOT_SENTINEL, 0));
doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0));

let bytes = doc.to_bytes();
let restored = SIRDocument::from_bytes(&bytes).unwrap();
assert_eq!(doc, restored);
```
