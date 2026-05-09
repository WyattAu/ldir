//! LDIR DOCX Backend — converts S-IR v2 to DOCX.

#![warn(clippy::unwrap_used, clippy::expect_used)]
#![deny(unsafe_code)]

mod builder;

pub use builder::DocxBuilder;
