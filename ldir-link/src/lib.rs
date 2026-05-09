#![warn(clippy::unwrap_used, clippy::expect_used)]
#![deny(unsafe_code)]

mod linker;

pub use linker::{LinkError, link_modules};
