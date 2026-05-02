//! Font loading, metrics, and embedding for the PDF backend.
//!
//! Per REQ-6.2.4: parse TrueType/OpenType font tables and subset fonts
//! exactly to the glyphs used in the G-IR.
//!
//! Uses `ttf-parser` for real TrueType/OpenType table parsing.

mod loader;
mod subset;
mod tables;

pub use loader::FontFace;
#[allow(unused_imports)]
pub(crate) use loader::{FontMetrics, PdfFontInfo};
#[allow(unused_imports)]
pub(crate) use subset::{FontSubset, subset_font};
#[allow(unused_imports)]
pub(crate) use tables::{CmapTable, HeadTable, HheaTable, HmtxTable};
