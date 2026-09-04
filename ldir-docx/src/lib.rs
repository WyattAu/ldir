//! LDIR DOCX Backend — converts S-IR v2 to DOCX.

#![deny(missing_docs)]
#![warn(clippy::unwrap_used, clippy::expect_used)]
#![deny(unsafe_code)]

mod builder;

pub use builder::DocxBuilder;
