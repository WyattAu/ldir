//! S-IR v2 — The Universal Document Intermediate Representation.
//!
//! Unlike S-IR v1 (instruction-stream based), v2 is a structured module
//! designed to be independently useful as the "LLVM of documents."

#![allow(missing_docs)]

pub mod module;
pub mod metadata;
pub mod resources;
pub mod styles;
pub mod nodes;
pub mod annotations;
pub mod serialize;
pub mod text;

pub use module::SIRModuleV2;
pub use metadata::*;
pub use resources::*;
pub use styles::*;
pub use nodes::*;
pub use annotations::*;
pub use serialize::{deserialize_module, serialize_module, SIRBinaryWriter};
pub use text::{module_to_text, text_to_module};
