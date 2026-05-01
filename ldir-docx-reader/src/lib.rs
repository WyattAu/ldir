//! LDIR DOCX Reader — converts DOCX (.docx) files to S-IR v2.

#![allow(clippy::collapsible_if)]

mod parser;

pub use parser::parse_docx;
