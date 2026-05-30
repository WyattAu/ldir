#![warn(clippy::unwrap_used, clippy::expect_used)]
#![deny(unsafe_code)]

mod convert;

pub use convert::PandocError;
pub use convert::sir_to_pandoc_json;
