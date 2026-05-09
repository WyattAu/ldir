//! LDIR Plain Text Backend — converts S-IR v2 to plain text.

#![warn(clippy::unwrap_used, clippy::expect_used)]
#![deny(unsafe_code)]

mod render;

pub use render::TextOptions;
pub use render::TextRenderer;
