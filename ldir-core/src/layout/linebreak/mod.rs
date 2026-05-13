//! Line-breaking algorithm modules.

mod badness;
pub mod cjk;
mod knuth_plass;
mod optical_margin;
mod rtl;
mod types;

pub use badness::{
    compute_adjustment_ratio, compute_badness, compute_demerits,
    compute_demerits_batch, compute_demerits_batch_simd,
};
pub use knuth_plass::linebreak;
pub use types::*;
