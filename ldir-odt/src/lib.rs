//! LDIR ODT Backend -- converts S-IR v2 to ODT (OpenDocument Text, ISO/IEC 26300).

#![warn(clippy::unwrap_used, clippy::expect_used)]
#![deny(unsafe_code)]

mod builder;

pub use builder::OdtBuilder;
pub use builder::OdtError;
