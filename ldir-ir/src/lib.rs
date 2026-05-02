//! # LDIR IR
//!
//! S-IR, L-IR, and G-IR type definitions with rkyv serialization.
//!
//! This crate defines the intermediate representation types used throughout LDIR:
//! - **S-IR** (Source IR): Tree-structured document representation
//! - **L-IR** (Layout IR): Positioned box tree capturing all layout decisions
//! - **G-IR** (Graphical IR): Linearized rendering commands
//!
//! ## S-IR (Source Intermediate Representation)
//!
//! S-IR is a tree-structured document format where each instruction is a
//! 13-byte fixed-cost header. Instructions reference each other via
//! entity IDs and parent pointers, forming a tree rooted at the entity
//! whose `parent_id == ROOT_SENTINEL`.
//!
//! ## G-IR (Graphical Intermediate Representation)
//!
//! G-IR is a flat, per-page command buffer optimized for rendering. Each
//! command is a 36-byte struct with an opcode and 8 × i32 argument slots.
//!
//! ## Wire Format
//!
//! S-IR instructions use a 13-byte wire-format header (REQ-3.1.2):
//! - OpCode: 1 byte
//! - EntityID: 4 bytes (u32)
//! - ParentID: 4 bytes (u32, sentinel 0xFFFFFFFF for root)
//! - PayloadOffset: 4 bytes (u32)
//!
//! G-IR commands use a 36-byte struct with 8×i32 argument slots.
//! Wire format is 16-byte aligned (REQ-3.2.2).
//!
//! ## Serialization
//!
//! S-IR types implement `rkyv::Archive`, `rkyv::Serialize`, and
//! `rkyv::Deserialize` for zero-copy deserialization. G-IR uses a
//! custom binary format emitted by `ldir_core::emitter::emit_gir`.
//!
//! ## Formal Verification
//!
//! Type well-formedness is specified in Lean 4:
//! - `entityUnique`: All entity IDs are distinct (AX-001)
//! - `parentExists`: Every parent reference is valid (AX-002)
//! - `isAcyclic`: No circular parent chains (AX-003)
//! - `hasSingleRoot`: Exactly one root node exists (DEF-004.5)
//! - `pageStackBalanced`: Coordinate stack is balanced (DEF-005.3)
//!
//! ## References
//!
//! - YP-IR-SEMANTICS-001: IR Type Semantics and Well-Formedness
//! - REQ-3.1.x: S-IR specification
//! - REQ-3.2.x: G-IR specification

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod fp266;
pub mod gir;
pub mod lir;
pub mod sir;

pub use sir::{EntityId, ROOT_SENTINEL};
