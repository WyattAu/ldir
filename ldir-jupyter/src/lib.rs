#![warn(clippy::unwrap_used, clippy::expect_used)]
#![deny(unsafe_code)]

mod convert;

pub use convert::JupyterError;
pub use convert::sir_to_notebook;
