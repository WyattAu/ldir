//! S-IR (Source Intermediate Representation) type definitions.
//!
//! Tree-structured document representation with entity-based addressing.
//! Wire format: 13-byte header per instruction (REQ-3.1.2).
//!
//! ## Module Structure
//!
//! - `opcode`: S-IR opcode and block type enums (REQ-3.1.3)
//! - `instruction`: S-IR instruction struct (13-byte wire format)
//! - `document`: S-IR document collection
//! - `payload`: Variable-length payload region (REQ-3.1.4)
//!
//! ## Well-Formedness (DEF-004)
//!
//! A document is well-formed iff all 6 WF-SIR conditions hold:
//! 1. **AX-001**: Entity IDs are unique.
//! 2. **AX-002**: Parent references are valid.
//! 3. **AX-003**: Parent graph is acyclic.
//! 4. **AX-004**: Payload offsets are in bounds.
//! 5. **DEF-004.5**: Exactly one root entity.
//! 6. **DEF-004.6**: Block nesting is properly structured.

pub mod v2;

mod document;
mod instruction;
mod opcode;
mod payload;
pub mod serde;
pub mod style;

pub use document::SIRDocument;
pub use instruction::{EntityId, INSTRUCTION_WIRE_SIZE, ROOT_SENTINEL, SIRInstruction};
pub use opcode::{BlockType, SIROpcode};
pub use payload::PayloadRegion;
pub use style::StyleModifier;
pub use v2::SourceSpan;
