//! S-IR v2 — The Universal Document Intermediate Representation.
//!
//! Unlike S-IR v1 (instruction-stream based), v2 is a structured module
//! designed to be independently useful as the "LLVM of documents."

#![allow(missing_docs)]

pub mod annotations;
pub mod metadata;
pub mod module;
pub mod nodes;
pub mod resources;
pub mod serialize;
pub mod source_span;
pub mod styles;
pub mod text;

pub use annotations::*;
pub use metadata::*;
pub use module::SIRModuleV2;
pub use nodes::*;
pub use resources::*;
pub use serialize::{SIRBinaryWriter, deserialize_module, serialize_module};
pub use source_span::SourceSpan;
pub use styles::*;
pub use text::{module_to_text, text_to_module};
