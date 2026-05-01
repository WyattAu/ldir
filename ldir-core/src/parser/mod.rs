//! S-IR parser module.
//!
//! Deserializes S-IR documents from rkyv-serialized bytes with pre-condition
//! checks for alignment and minimum size.

pub mod sir_parser;

pub use sir_parser::{parse_sir, parse_sir_with_source_map};
