//! Font loading and management (Phase A-1).
//!
//! Provides a zero-copy font handle, ttf-parser-based font introspection,
//! a font database backed by `fontdb`, and a glyph outline cache for
//! the HarfBuzz shaping pipeline.

pub mod db;
pub mod loader;
pub mod outline_cache;

pub use db::FontDatabase;
pub use loader::{
    FontHandle, FontInfo, FontMetrics, LoadedFont, font_info, glyph_id_for_char, glyph_metrics,
    load_font, units_per_em,
};
pub use outline_cache::{Contour, GlyphOutline, GlyphOutlineCache, OutlinePoint, OutlinePointKind};
