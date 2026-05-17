//! IR module linker for LDIR documents.
//!
//! Provides functionality to link multiple S-IR modules into a single
//! document, handling ID remapping and cross-module references.

#![warn(clippy::unwrap_used, clippy::expect_used)]
#![deny(unsafe_code)]

mod linker;

pub use linker::{LinkError, link_modules};
