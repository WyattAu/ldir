//! # ldir-core
//!
//! Compiler, validator, emitter, and font management for the LDIR document
//! pipeline. This crate contains the central compilation pipeline:
//!
//! 1. S-IR deserialization and validation
//! 2. S-IR to L-IR layout compilation
//! 3. L-IR to G-IR rendering
//! 4. G-IR binary emission
//!
//! ## Key Types
//!
//! - [`compiler::compile_sir`] — S-IR to G-IR compilation entry point
//! - [`validator::validate_sir`] — S-IR well-formedness validation
//! - [`emitter::emit_gir`] — G-IR binary serialization
//! - [`parser::parse_sir`] — S-IR deserialization from rkyv bytes
//! - [`verifier::check_gir`] — G-IR well-formedness verification
//! - [`font::FontDatabase`] — Font loading and management
//! - [`error::LdirError`] — Structured error hierarchy
//!
//! ## Quick Start
//!
//! ```rust
//! use ldir_core::compiler::compile_sir;
//! use ldir_core::validator::validate_sir;
//! use ldir_ir::sir::{SIRDocument, SIRInstruction, SIROpcode, ROOT_SENTINEL, BlockType};
//!
//! let mut doc = SIRDocument::new();
//! doc.push_with_payload(
//!     SIRInstruction::new(SIROpcode::PushBlock, 0, ROOT_SENTINEL, 0),
//!     &[BlockType::Document as u8],
//! );
//! doc.push_with_payload(
//!     SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0),
//!     b"Hello",
//! );
//!
//! validate_sir(&doc).expect("document should be well-formed");
//! let gir = compile_sir(&doc).expect("compilation should succeed");
//! assert!(gir.is_well_formed());
//! ```
//!
//! ## Key Modules
//!
//! - [`fp266`] — 26.6 fixed-point arithmetic (THM-FP-ADD-EXACT, THM-FP-MUL-ROUND)
//! - [`compiler`] — S-IR to G-IR compilation (IF-COMPILE-001, ALG-COMPILE-001)
//! - [`validator`] — S-IR well-formedness validation (IF-VALIDATE-001)
//! - [`emitter`] — G-IR binary serialization (IF-EMIT-001)
//! - [`parser`] — S-IR deserialization from rkyv bytes (IF-PARSE-001)
//! - [`verifier`] — G-IR well-formedness verification (DEF-005)
//! - [`error`] — Structured error hierarchy
//! - [`font`] — Font loading via ttf-parser and fontdb
//! - [`plugin`] — Plugin system for custom frontends and backends
//!
//! ## Formal Verification
//!
//! The well-formedness predicates for S-IR and G-IR are formally specified in
//! Lean 4. Key theorems:
//! - `THM-WF-SIR-DECIDABLE`: S-IR well-formedness is decidable
//! - `THM-WF-GIR-DECIDABLE`: G-IR well-formedness is decidable
//! - `THM-COMPILE-TERMINATES`: Compilation terminates
//!
//! ## References
//!
//! - [Repository](https://github.com/WyattAu/ldir)
//! - YP-IR-SEMANTICS-001: IR Type Semantics and Well-Formedness
//! - BP-IR-COMPILER-001: S-IR to G-IR Compiler Component

#![deny(unsafe_code)]
#![warn(clippy::unwrap_used, clippy::expect_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

/// S-IR to G-IR compiler with Knuth-Plass line breaking and font shaping.
pub mod compiler;

pub(crate) mod ecs;
pub mod interner;

/// Typed arena allocator and string arena for compilation hot paths.
pub mod arena;

/// Cross-reference resolution for document labels and citations.
pub mod cross_ref;

/// G-IR binary emitter and parser (IF-EMIT-001).
pub mod emitter;

/// Structured error hierarchy for the compilation pipeline.
pub mod error;

/// Font loading, metrics, and database (ttf-parser + fontdb).
pub mod font;

/// 26.6 fixed-point arithmetic for coordinates (REQ-3.2.5).
pub mod fp266;

/// Page number formatting: arabic, roman, alphabetic.
pub mod page_numbers;

/// Table of contents and document outline generation.
pub mod toc;

pub mod layout;

pub use layout::lir_compile::compile_sir_to_lir;

/// S-IR parser — deserialization from rkyv bytes (IF-PARSE-001).
pub mod parser;

/// Plugin system for custom frontends and backends.
pub mod plugin;

pub(crate) mod profiling;

/// Text shaping: HarfBuzz FFI, LRU cache, fast ASCII path.
pub mod shaping;

/// Cassowary constraint solver for float placement and layout.
pub mod solver;

#[doc(hidden)]
pub mod source_map;

pub(crate) mod trace;

/// S-IR well-formedness validator (IF-VALIDATE-001).
pub mod validator;

/// G-IR well-formedness verifier (DEF-005).
pub mod verifier;

/// Rust-native test plugins for the Wasm plugin ABI.
pub mod wasm_plugins;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
