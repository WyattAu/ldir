//! LDIR Org-mode Frontend — converts Org-mode (.org) files to S-IR v2.

#![allow(clippy::collapsible_if)]

mod parser;

pub use parser::parse_org;
