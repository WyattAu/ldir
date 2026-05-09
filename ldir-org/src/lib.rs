//! LDIR Org-mode Frontend — converts Org-mode (.org) files to S-IR v2.

#![warn(clippy::unwrap_used, clippy::expect_used)]
#![deny(unsafe_code)]

mod parser;

pub use parser::parse_org;
