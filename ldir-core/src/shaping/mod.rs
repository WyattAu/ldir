//! Text shaping integration (TASK-016, Phase A-2).
//!
//! Provides the interface between text content and glyph layout.
//! Uses HarfBuzz for real font-aware shaping with an ASCII fast-path
//! for simple text (REQ-4.3.1.1).
//!
//! All coordinates use `Fp266` (26.6 fixed-point)
//! per REQ-3.2.5 and THM-KP-DETERMINISM.

pub mod cache;
pub mod fast_path;
#[cfg(not(target_arch = "wasm32"))]
pub mod harfbuzz;
pub mod indic;

use crate::fp266::Fp266;
use crate::shaping::cache::ThreadSafeShapeCache;
use std::sync::Arc;

/// OpenType feature to enable/disable during shaping.
#[cfg(not(target_arch = "wasm32"))]
#[allow(unused_imports)]
#[allow(dead_code)]
pub use harfbuzz::{
    CALT, DLIG, Feature, HLIG, KERN, LIGA, LNUM, ONUM, SALT, SS01, SS02, SS03, SS04, SS05, SS06,
    SS07, SS08, SS09, SS10, SS11, SS12, SS13, SS14, SS15, SS16, SS17, SS18, SS19, SS20, TNUM, ZERO,
};

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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    if features.is_none() && is_ascii_simple(text) {
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
///
/// Returns `Arc<ShapedRun>` -- callers can deref to access glyphs without cloning.
#[inline]
pub fn shape_text_cached(
    cache: &ThreadSafeShapeCache,
    font_data: &[u8],
    text: &str,
    font_size: Fp266,
    font_id: u32,
) -> Arc<ShapedRun> {
    if text.is_empty() {
        return Arc::new(ShapedRun::empty());
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
///
/// Returns `Arc<ShapedRun>` -- callers can deref to access glyphs without cloning.
#[inline]
pub fn shape_text_cached_auto_font(
    cache: &ThreadSafeShapeCache,
    font_data: &[u8],
    text: &str,
    font_size: Fp266,
) -> Arc<ShapedRun> {
    if text.is_empty() {
        return Arc::new(ShapedRun::empty());
    }
    let stable_font_id = font_data_hash(font_data);
    cache.get_or_shape(text, stable_font_id, font_size, |t, _fid, fs| {
        shape_text(font_data, t, fs)
    })
}

/// Shapes text using the shape cache with optional OpenType feature overrides.
///
/// Features are incorporated into the cache key to ensure different feature
/// sets produce distinct cache entries.
///
/// Returns `Arc<ShapedRun>` -- callers can deref to access glyphs without cloning.
#[inline]
pub fn shape_text_cached_with_features(
    cache: &ThreadSafeShapeCache,
    font_data: &[u8],
    text: &str,
    font_size: Fp266,
    font_id: u32,
    features: Option<&[Feature]>,
) -> Arc<ShapedRun> {
    if text.is_empty() {
        return Arc::new(ShapedRun::empty());
    }
    if features.is_none() {
        return shape_text_cached(cache, font_data, text, font_size, font_id);
    }
    let stable_font_id = font_id;
    let feature_list: Vec<Feature> = features.map(|f| f.to_vec()).unwrap_or_default();
    let composite_id = stable_font_id
        .wrapping_mul(31)
        .wrapping_add(feature_list_hash(&feature_list));
    cache.get_or_shape(text, composite_id, font_size, |t, _fid, fs| {
        shape_text_with_features(font_data, t, fs, Some(&feature_list))
    })
}

fn feature_list_hash(features: &[Feature]) -> u32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;
    let mut hasher = DefaultHasher::new();
    for f in features {
        hasher.write_u32(u32::from_be_bytes(f.tag));
        hasher.write_u32(f.value);
    }
    hasher.finish() as u32
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

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn shape_with_features_routes_to_harfbuzz() {
        let data = ldir_test_helpers::test_font_data();
        let features = vec![Feature::enable(KERN), Feature::enable(LIGA)];
        let run = shape_text_with_features(&data, "fi fl", Fp266::from_int(12), Some(&features));
        assert!(!run.glyphs.is_empty());
        assert!(run.total_advance.raw() > 0);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn shape_with_features_deterministic() {
        let data = ldir_test_helpers::test_font_data();
        let features = vec![Feature::enable(KERN)];
        let run1 = shape_text_with_features(&data, "AV", Fp266::from_int(12), Some(&features));
        let run2 = shape_text_with_features(&data, "AV", Fp266::from_int(12), Some(&features));
        assert_eq!(run1, run2);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn shape_cached_with_features() {
        let data = ldir_test_helpers::test_font_data();
        let cache = ThreadSafeShapeCache::new(64);
        let features = vec![Feature::enable(LIGA)];
        let run1 = shape_text_cached_with_features(
            &cache,
            &data,
            "fi",
            Fp266::from_int(12),
            0,
            Some(&features),
        );
        let run2 = shape_text_cached_with_features(
            &cache,
            &data,
            "fi",
            Fp266::from_int(12),
            0,
            Some(&features),
        );
        assert!(run1.total_advance.raw() > 0);
        assert_eq!(run1, run2);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn shape_cached_without_features_uses_existing_path() {
        let data = ldir_test_helpers::test_font_data();
        let cache = ThreadSafeShapeCache::new(64);
        let run1 =
            shape_text_cached_with_features(&cache, &data, "fi", Fp266::from_int(12), 0, None);
        let run2 = shape_text_cached(&cache, &data, "fi", Fp266::from_int(12), 0);
        assert_eq!(run1, run2);
    }
}
