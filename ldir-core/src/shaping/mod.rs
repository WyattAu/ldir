//! Text shaping integration (TASK-016, Phase A-2).
//!
//! Provides the interface between text content and glyph layout.
//! Uses HarfBuzz for real font-aware shaping with an ASCII fast-path
//! for simple text (REQ-4.3.1.1).
//!
//! All coordinates use [`Fp266`] (26.6 fixed-point)
//! per REQ-3.2.5 and THM-KP-DETERMINISM.

#![allow(dead_code)]

pub mod cache;
pub mod fast_path;
#[cfg(not(target_arch = "wasm32"))]
pub mod harfbuzz;
pub mod indic;

use crate::fp266::Fp266;
use crate::shaping::cache::ThreadSafeShapeCache;

/// OpenType feature to enable/disable during shaping.
#[cfg(not(target_arch = "wasm32"))]
#[allow(unused_imports)]
#[allow(dead_code)]
pub use harfbuzz::{CALT, Feature, KERN, LIGA, LNUM, ONUM, TNUM};

/// No-op OpenType feature placeholder for WASM targets.
///
/// Features (kerning, ligatures, etc.) are not supported without HarfBuzz.
/// This type allows code to compile on WASM but feature parameters are
/// silently ignored during shaping.
#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Feature {
    pub tag: [u8; 4],
    pub value: u32,
    pub start: u32,
    pub end: u32,
}

#[cfg(target_arch = "wasm32")]
impl Feature {
    pub fn enable(tag: &[u8; 4]) -> Self {
        Self {
            tag: *tag,
            value: 1,
            start: 0,
            end: u32::MAX,
        }
    }
    pub fn disable(tag: &[u8; 4]) -> Self {
        Self {
            tag: *tag,
            value: 0,
            start: 0,
            end: u32::MAX,
        }
    }
}

/// Metrics for a single glyph from font data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlyphMetrics {
    /// Horizontal advance width.
    pub advance_width: Fp266,
    /// Left side bearing.
    pub left_side_bearing: Fp266,
    /// Glyph height (ascent + descent).
    pub height: Fp266,
    /// Bounding box: (x_min, y_min, x_max, y_max).
    pub bbox: (Fp266, Fp266, Fp266, Fp266),
}

/// A single positioned glyph within a shaped run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShapedGlyph {
    /// Font-specific glyph identifier.
    pub glyph_id: u32,
    /// X offset from the pen position.
    pub x_offset: Fp266,
    /// Y offset from the pen position.
    pub y_offset: Fp266,
    /// Advance width contributed by this glyph.
    pub advance: Fp266,
    /// Cluster ID mapping back to source text (byte offset).
    pub cluster_id: u32,
}

/// The result of shaping a contiguous run of text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShapedRun {
    /// Positioned glyphs in visual order.
    pub glyphs: Vec<ShapedGlyph>,
    /// Total advance width of the entire run.
    pub total_advance: Fp266,
}

impl ShapedRun {
    /// Create an empty shaped run.
    pub fn empty() -> Self {
        Self {
            glyphs: Vec::new(),
            total_advance: Fp266::ZERO,
        }
    }
}

fn is_ascii_simple(text: &str) -> bool {
    text.is_ascii() && !text.is_empty()
}

fn shape_ascii_with_metrics(font_data: &[u8], text: &str, font_size: Fp266) -> ShapedRun {
    match crate::font::loader::load_font(std::sync::Arc::new(font_data.to_vec())) {
        Ok(font) => {
            let face = font.face();
            let upem = crate::font::loader::units_per_em(&font);
            let mut glyphs = Vec::with_capacity(text.len());
            let mut total_advance = Fp266::ZERO;

            for (byte_offset, ch) in text.as_bytes().iter().enumerate() {
                let glyph_id = crate::font::loader::glyph_id_for_char(&face, *ch as char)
                    .map(|g| g.0 as u32)
                    .unwrap_or(0);

                let advance = if glyph_id != 0 {
                    crate::font::loader::glyph_advance_fp266(
                        &face,
                        ttf_parser::GlyphId(glyph_id as u16),
                        font_size,
                        upem,
                    )
                } else {
                    Fp266::ZERO
                };

                glyphs.push(ShapedGlyph {
                    glyph_id,
                    x_offset: Fp266::ZERO,
                    y_offset: Fp266::ZERO,
                    advance,
                    cluster_id: byte_offset as u32,
                });
                total_advance += advance;
            }

            ShapedRun {
                glyphs,
                total_advance,
            }
        }
        Err(_) => fast_path::shape_ascii(text, font_size, 0),
    }
}

/// Shapes text into a run of positioned glyphs, choosing ASCII fast-path or HarfBuzz as needed.
pub fn shape_text(font_data: &[u8], text: &str, font_size: Fp266) -> ShapedRun {
    shape_text_with_features(font_data, text, font_size, None)
}

/// Shapes text with optional OpenType feature overrides.
pub fn shape_text_with_features(
    font_data: &[u8],
    text: &str,
    font_size: Fp266,
    features: Option<&[Feature]>,
) -> ShapedRun {
    if text.is_empty() {
        return ShapedRun::empty();
    }
    if is_ascii_simple(text) {
        shape_ascii_with_metrics(font_data, text, font_size)
    } else {
        #[cfg(not(target_arch = "wasm32"))]
        {
            harfbuzz::shape_harfbuzz(font_data, text, font_size, None, None, None, features)
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = features;
            fast_path::shape_unicode_basic(font_data, text, font_size, 0)
        }
    }
}

fn font_data_hash(font_data: &[u8]) -> u32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    font_data.hash(&mut hasher);
    hasher.finish() as u32
}

/// Shapes text using the shape cache when available.
///
/// Uses the cache to avoid re-shaping identical (text, font_id, font_size) triples.
/// Falls back to uncached shaping when font_data is not available.
#[inline]
pub fn shape_text_cached(
    cache: &ThreadSafeShapeCache,
    font_data: &[u8],
    text: &str,
    font_size: Fp266,
    font_id: u32,
) -> ShapedRun {
    if text.is_empty() {
        return ShapedRun::empty();
    }
    let stable_font_id = font_id;
    cache.get_or_shape(text, stable_font_id, font_size, |t, _fid, fs| {
        shape_text(font_data, t, fs)
    })
}

/// Shapes text using the shape cache with a stable font hash.
///
/// Computes a font_id from font_data bytes so the same font data always
/// produces the same cache key, regardless of the logical font_id.
#[inline]
pub fn shape_text_cached_auto_font(
    cache: &ThreadSafeShapeCache,
    font_data: &[u8],
    text: &str,
    font_size: Fp266,
) -> ShapedRun {
    if text.is_empty() {
        return ShapedRun::empty();
    }
    let stable_font_id = font_data_hash(font_data);
    cache.get_or_shape(text, stable_font_id, font_size, |t, _fid, fs| {
        shape_text(font_data, t, fs)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shaped_run_empty() {
        let run = ShapedRun::empty();
        assert!(run.glyphs.is_empty());
        assert!(run.total_advance.is_zero());
    }

    #[test]
    fn glyph_metrics_default_bbox() {
        let m = GlyphMetrics {
            advance_width: Fp266::ZERO,
            left_side_bearing: Fp266::ZERO,
            height: Fp266::ZERO,
            bbox: (Fp266::ZERO, Fp266::ZERO, Fp266::ZERO, Fp266::ZERO),
        };
        assert_eq!(m.advance_width, Fp266::ZERO);
    }

    #[test]
    fn shaped_glyph_fields() {
        let g = ShapedGlyph {
            glyph_id: 42,
            x_offset: Fp266::from_int(1),
            y_offset: Fp266::ZERO,
            advance: Fp266::from_int(10),
            cluster_id: 5,
        };
        assert_eq!(g.glyph_id, 42);
        assert_eq!(g.advance, Fp266::from_int(10));
    }
}
