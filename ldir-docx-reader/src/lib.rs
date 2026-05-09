//! LDIR DOCX Reader — converts DOCX (.docx) files to S-IR v2.

#![warn(clippy::unwrap_used, clippy::expect_used)]
#![deny(unsafe_code)]

mod parser;

pub use parser::parse_docx;
