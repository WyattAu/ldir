//! # LDIR Core
//!
//! fp26_6 arithmetic, ECS, S-IR validator, compiler, and G-IR emitter.
//!
//! This crate contains the central compilation pipeline:
//! 1. S-IR deserialization and validation
//! 2. S-IR → G-IR compilation
//! 3. G-IR emission
//!
//! ## Quick Start
//!
//! ```rust
//! use ldir_core::compiler::compile_sir;
//! use ldir_core::validator::validate_sir;
//! use ldir_ir::sir::{SIRDocument, SIRInstruction, SIROpcode, ROOT_SENTINEL};
//!
//! // Build a simple S-IR document
//! let mut doc = SIRDocument::new();
//! doc.push(SIRInstruction::new(SIROpcode::PushBlock, 0, ROOT_SENTINEL, 0));
//! doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0));
//!
//! // Validate well-formedness
//! validate_sir(&doc).expect("document should be well-formed");
//!
//! // Compile to G-IR
//! let gir = compile_sir(&doc).expect("compilation should succeed");
//! assert!(gir.is_well_formed());
//! ```
//!
//! ## Key Modules
//!
//! - [`fp266`] — 26.6 fixed-point arithmetic (THM-FP-ADD-EXACT, THM-FP-MUL-ROUND)
//! - [`compiler`] — S-IR → G-IR compilation (IF-COMPILE-001, ALG-COMPILE-001)
//! - [`validator`] — S-IR well-formedness validation (IF-VALIDATE-001)
//! - [`emitter`] — G-IR binary serialization (IF-EMIT-001)
//! - [`parser`] — S-IR deserialization from rkyv bytes (IF-PARSE-001)
//! - [`verifier`] — G-IR well-formedness verification (DEF-005)
//! - [`error`] — Structured error hierarchy
//!
//! ## Formal Verification
//!
//! The well-formedness predicates for S-IR and G-IR are formally specified in
//! Lean 4 at `.specs/02_architecture/proofs/LDIRProofs/ProofIRWellformedness.lean`.
//! Key theorems:
//! - `THM-WF-SIR-DECIDABLE`: S-IR well-formedness is decidable
//! - `THM-WF-GIR-DECIDABLE`: G-IR well-formedness is decidable
//! - `THM-COMPILE-TERMINATES`: Compilation terminates
//! - `THM-ROOT-UNIQUENESS`: wellFormedSIR implies exactly one root
//!
//! ## References
//!
//! - YP-IR-SEMANTICS-001: IR Type Semantics and Well-Formedness
//! - BP-IR-COMPILER-001: S-IR to G-IR Compiler Component
//! - REQ-3.1.x: S-IR specification
//! - REQ-3.2.x: G-IR specification

#![deny(unsafe_code)] // No unsafe code until Phase B optimization
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

pub mod compiler;
pub(crate) mod ecs;
pub mod emitter;
pub mod error;
pub mod font;
pub mod fp266;
pub(crate) mod layout;

pub use layout::lir_compile::compile_sir_to_lir;
pub mod parser;
pub(crate) mod profiling;
pub(crate) mod shaping;
pub(crate) mod solver;
#[doc(hidden)]
pub mod source_map;
pub(crate) mod trace;
pub mod validator;
pub mod verifier;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
