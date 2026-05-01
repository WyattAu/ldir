//! Line-breaking algorithm modules.

mod badness;
pub(crate) mod cjk;
mod knuth_plass;
mod rtl;
mod types;

pub use knuth_plass::linebreak;
pub use types::*;
