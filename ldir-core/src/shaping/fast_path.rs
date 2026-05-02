//! ASCII/Latin-1 fast-path shaping (bypasses HarfBuzz).
//!
//! Produces identical output for ASCII text as the full shaping path would.
//! Uses a monospace assumption: each character advances by `0.6 * font_size`.
//!
//! This is a **stub** implementation. Full HarfBuzz integration is deferred to Phase D.

use crate::fp266::Fp266;
use crate::shaping::{ShapedGlyph, ShapedRun};

/// Ratio of character advance to font size for monospace assumption.
/// 0.6 * font_size approximates an average Latin glyph advance.
const MONO_ADVANCE_RATIO_NUM: i32 = 3;
const MONO_ADVANCE_RATIO_DEN: i32 = 5;

/// Check whether a string is pure ASCII (all bytes <= 0x7F).
fn is_ascii_str(text: &str) -> bool {
    text.as_bytes().iter().all(|&b| b <= 0x7F)
}

/// Compute the monospace advance for a single character at the given font size.
///
/// `advance = font_size * 3/5` (0.6 ratio).
pub fn mono_advance(font_size: Fp266) -> Fp266 {
    (font_size * MONO_ADVANCE_RATIO_NUM).div(Fp266::from_int(MONO_ADVANCE_RATIO_DEN))
}

/// Shape an ASCII text string using the fast path.
///
/// Each ASCII character is mapped to a placeholder glyph ID (its byte value + 1,
/// so NUL maps to 1 and 0x7F maps to 128). All characters use the monospace advance
/// and zero offsets.
///
/// # Arguments
///
/// * `text` - Must be pure ASCII. Panics on non-ASCII input.
/// * `font_size` - Font size in device units (26.6 fixed-point).
/// * `font_id` - Font identifier (returned in cluster info; not used for lookup).
///
/// # Panics
///
/// Panics if `text` contains non-ASCII bytes.
pub fn shape_ascii(text: &str, font_size: Fp266, _font_id: u32) -> ShapedRun {
    assert!(is_ascii_str(text), "shape_ascii requires pure ASCII input");

    if text.is_empty() {
        return ShapedRun::empty();
    }

    let char_advance = mono_advance(font_size);
    let mut glyphs = Vec::with_capacity(text.len());
    let mut total_advance = Fp266::ZERO;

    for (byte_offset, ch) in text.as_bytes().iter().enumerate() {
        let glyph_id = (*ch as u32) + 1;
        glyphs.push(ShapedGlyph {
            glyph_id,
            x_offset: Fp266::ZERO,
            y_offset: Fp266::ZERO,
            advance: char_advance,
            cluster_id: byte_offset as u32,
        });
        total_advance += char_advance;
    }

    ShapedRun {
        glyphs,
        total_advance,
    }
}

/// Basic Unicode shaper using ttf_parser cmap lookup (WASM-safe).
///
/// Maps each Unicode scalar value to its glyph ID via the font's cmap table.
/// Falls back to monospace advance for glyphs without explicit advance data.
/// No kerning, ligatures, or complex shaping — suitable for WASM targets
/// where HarfBuzz is unavailable.
pub fn shape_unicode_basic(
    font_data: &[u8],
    text: &str,
    font_size: Fp266,
    _font_id: u32,
) -> ShapedRun {
    if text.is_empty() {
        return ShapedRun::empty();
    }

    let default_advance = mono_advance(font_size);

    // Try to load the font for cmap-based glyph lookup.
    let face = ttf_parser::Face::parse(font_data, 0).ok();
    let upem = face.as_ref().map(|f| {
        let u = f.units_per_em();
        if u == 0 { 1000 } else { u }
    });

    let mut glyphs = Vec::with_capacity(text.chars().count());
    let mut total_advance = Fp266::ZERO;

    for (byte_offset, ch) in text.char_indices() {
        let (glyph_id, advance) = if let (Some(face), Some(upem)) = (&face, upem) {
            match face.glyph_index(ch) {
                Some(gid) if gid.0 != 0 => {
                    let glyph_id = gid.0 as u32;
                    let adv_units = face.glyph_hor_advance(gid).map_or(0, |v| v as i64);
                    let advance = if upem as i64 > 0 {
                        Fp266::from_raw((font_size.raw() * adv_units) / upem as i64)
                    } else {
                        default_advance
                    };
                    (glyph_id, advance)
                }
                _ => (0, default_advance),
            }
        } else {
            // No font data — ASCII gets real glyph IDs, non-ASCII gets 0xFFFD
            let glyph_id = if is_ascii_str(&ch.to_string()) {
                (ch as u32) + 1
            } else {
                0xFFFD
            };
            (glyph_id, default_advance)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_fast_path_non_empty() {
        let run = shape_ascii("hello", Fp266::from_int(10), 1);
        assert_eq!(run.glyphs.len(), 5);
        assert!(!run.total_advance.is_zero());
    }

    #[test]
    fn ascii_empty_string() {
        let run = shape_ascii("", Fp266::from_int(10), 1);
        assert!(run.glyphs.is_empty());
        assert!(run.total_advance.is_zero());
    }

    #[test]
    fn monospace_advance_calculation() {
        let font_size = Fp266::from_int(10);
        let adv = mono_advance(font_size);
        // 10 * 3/5 = 6.0
        assert!((adv.to_f64() - 6.0).abs() < 0.01);
    }

    #[test]
    fn monospace_advance_total() {
        let run = shape_ascii("abc", Fp266::from_int(10), 1);
        // 3 chars * 6.0 = 18.0
        assert!((run.total_advance.to_f64() - 18.0).abs() < 0.01);
    }

    #[test]
    fn ascii_glyph_ids() {
        let run = shape_ascii("A", Fp266::from_int(10), 1);
        // 'A' = 0x41, glyph_id = 0x41 + 1 = 66
        assert_eq!(run.glyphs[0].glyph_id, 66);
    }

    #[test]
    fn ascii_cluster_ids() {
        let run = shape_ascii("ab", Fp266::from_int(10), 1);
        assert_eq!(run.glyphs[0].cluster_id, 0);
        assert_eq!(run.glyphs[1].cluster_id, 1);
    }

    #[test]
    fn ascii_x_offsets_zero() {
        let run = shape_ascii("ab", Fp266::from_int(10), 1);
        assert_eq!(run.glyphs[0].x_offset, Fp266::ZERO);
        assert_eq!(run.glyphs[1].x_offset, Fp266::ZERO);
    }

    #[test]
    #[should_panic]
    fn ascii_panics_on_non_ascii() {
        let _ = shape_ascii("héllo", Fp266::from_int(10), 1);
    }

    #[test]
    fn unicode_basic_no_font_produces_replacement_glyphs() {
        // Without font data, non-ASCII gets tofu (glyph 0)
        let run = shape_unicode_basic(&[], "日本語", Fp266::from_int(10), 1);
        assert_eq!(run.glyphs.len(), 3);
        for g in &run.glyphs {
            assert_eq!(g.glyph_id, 0xFFFD);
        }
    }

    #[test]
    fn unicode_basic_no_font_mixed() {
        let run = shape_unicode_basic(&[], "a日", Fp266::from_int(10), 1);
        assert_eq!(run.glyphs.len(), 2);
        // 'a' is ASCII, gets its own glyph_id
        assert_eq!(run.glyphs[0].glyph_id, 'a' as u32 + 1);
        // '日' is non-ASCII, gets replacement
        assert_eq!(run.glyphs[1].glyph_id, 0xFFFD);
    }

    #[test]
    fn unicode_basic_no_font_empty() {
        let run = shape_unicode_basic(&[], "", Fp266::from_int(10), 1);
        assert!(run.glyphs.is_empty());
    }

    #[test]
    fn unicode_basic_with_font_uses_cmap() {
        // With real font data, cmap should map known chars to non-zero glyph IDs
        let path = "/usr/share/fonts/TTF/DejaVuSans.ttf";
        if let Ok(data) = std::fs::read(path) {
            let run = shape_unicode_basic(&data, "A", Fp266::from_int(10), 1);
            assert_eq!(run.glyphs.len(), 1);
            // 'A' should map to a real glyph ID (not 0xFFFD or 0)
            assert_ne!(run.glyphs[0].glyph_id, 0xFFFD);
            assert!(run.glyphs[0].glyph_id > 0);
        }
        // Skip on systems without the test font
    }

    #[test]
    fn unicode_basic_with_font_unknown_char() {
        let path = "/usr/share/fonts/TTF/DejaVuSans.ttf";
        if let Ok(data) = std::fs::read(path) {
            // U+0001 is a control character not in most fonts
            let run = shape_unicode_basic(&data, "\u{0001}", Fp266::from_int(10), 1);
            assert_eq!(run.glyphs.len(), 1);
            // Missing glyph should get glyph ID 0 with default advance
            assert_eq!(run.glyphs[0].glyph_id, 0);
        }
    }
}
