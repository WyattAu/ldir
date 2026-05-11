//! Line-breaking algorithm modules.

mod badness;
pub mod cjk;
mod knuth_plass;
mod optical_margin;
mod rtl;
mod types;

pub use knuth_plass::linebreak;
pub use types::*;
