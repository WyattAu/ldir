//! LDIR Asciidoc Frontend — converts Asciidoc (.adoc) files to S-IR v2.

#![allow(clippy::collapsible_if, clippy::manual_strip, clippy::manual_split_once, clippy::nonminimal_bool, dead_code)]

mod parser;

pub use parser::parse_asciidoc;
