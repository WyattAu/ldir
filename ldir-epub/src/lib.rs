//! LDIR EPUB Backend — converts S-IR v2 to EPUB 3.

#![warn(clippy::unwrap_used, clippy::expect_used)]
#![deny(unsafe_code)]

mod builder;

pub use builder::EpubBuilder;
pub use builder::EpubOptions;
