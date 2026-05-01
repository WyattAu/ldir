//! G-IR (Graphical Intermediate Representation) type definitions.
//!
//! Linearized rendering command sequence with page-based structure.
//!
//! ## Module Structure
//!
//! - `opcode`: G-IR opcode enum (REQ-3.2.3)
//! - `command`: G-IR command struct with fixed-size args
//! - `page`: G-IR page with stack balance validation (DEF-002)
//! - `document`: G-IR document (DEF-003)
//! - `style`: Style types for font/size/color
//!
//! ## Well-Formedness (DEF-005)
//!
//! A document is well-formed iff all pages satisfy:
//! 1. Coordinates are in 26.6 representable range.
//! 2. Font precedence is maintained.
//! 3. Coordinate stack is balanced.
//! 4. Coordinates are within page bounds.

mod command;
mod document;
mod opcode;
mod page;
mod style;

pub use command::{GIR_COMMAND_ARGS, GIRCommand};
pub use document::{GIRDocument, GIRImage, ImageFormat};
pub use opcode::GIROpcode;
pub use page::{GIRLink, GIRPage};
pub use style::{GIRStyle, StyleTable};
