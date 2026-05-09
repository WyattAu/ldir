//! # ldir-ir
//!
//! IR type definitions for the LDIR document pipeline: S-IR, L-IR, and G-IR
//! with rkyv serialization support.

#![warn(clippy::unwrap_used, clippy::expect_used)]
//!
//! This crate defines three layers of intermediate representation used
//! throughout the LDIR compilation pipeline:
//!
//! - **S-IR** (Source IR): Tree-structured document representation with
//!   13-byte fixed-cost instruction headers. Instructions reference each other
//!   via entity IDs and parent pointers, forming a tree rooted at the entity
//!   whose `parent_id == ROOT_SENTINEL`.
//! - **L-IR** (Layout IR): Positioned box tree capturing all layout decisions
//!   as explicit 26.6 fixed-point geometry.
//! - **G-IR** (Graphical IR): Flat, per-page command buffer with 36-byte
//!   structs containing an opcode and 8 x i32 argument slots.
//!
//! ## Key Types
//!
//! - [`SIRDocument`] / [`SIRInstruction`] — S-IR document and instruction types
//! - [`SIROpcode`] / [`BlockType`] — S-IR opcodes and block classifications
//! - [`StyleModifier`] — Inline style flags (bold, italic, mono)
//! - [`LIRDocument`] / [`LIRNode`] — L-IR layout tree with 23 node types
//! - [`Fp266`] — 26.6 fixed-point number for geometry
//! - [`GIRDocument`] / [`GIRCommand`] — G-IR rendering command buffer
//!
//! ## Quick Start
//!
//! ```rust
//! use ldir_ir::sir::{SIRDocument, SIRInstruction, SIROpcode, BlockType, ROOT_SENTINEL};
//!
//! let mut doc = SIRDocument::new();
//! let payload_off = doc.payload_mut().append(&[BlockType::Document as u8]);
//! doc.push(SIRInstruction::new(SIROpcode::PushBlock, 0, ROOT_SENTINEL, payload_off));
//! doc.push_with_payload(
//!     SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0),
//!     b"Hello, world.",
//! );
//! ```
//!
//! ## Wire Format
//!
//! S-IR instructions use a 13-byte wire-format header (REQ-3.1.2):
//! - OpCode: 1 byte
//! - EntityID: 4 bytes (u32)
//! - ParentID: 4 bytes (u32, sentinel 0xFFFFFFFF for root)
//! - PayloadOffset: 4 bytes (u32)
//!
//! G-IR commands use a 36-byte struct with 8 x i32 argument slots.
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
//!
//! ## References
//!
//! - [Repository](https://github.com/WyattAu/ldir)
//! - YP-IR-SEMANTICS-001: IR Type Semantics and Well-Formedness
//! - REQ-3.1.x: S-IR specification
//! - REQ-3.2.x: G-IR specification

#![deny(unsafe_code)]

/// S-IR (Source Intermediate Representation) type definitions.
///
/// Tree-structured document representation with entity-based addressing.
/// Wire format: 13-byte header per instruction (REQ-3.1.2).
pub mod sir;

/// G-IR (Graphical Intermediate Representation) type definitions.
///
/// Linearized rendering command sequence with page-based structure.
pub mod gir;

/// L-IR (Layout Intermediate Representation) module.
///
/// Positioned box tree capturing all layout decisions as explicit geometry.
pub mod lir;

/// 26.6 fixed-point arithmetic for L-IR geometry (AX-LIR-001).
///
/// 26.6 format: 26 integer bits + 6 fractional bits.
pub mod fp266;

pub use sir::{EntityId, ROOT_SENTINEL};
