//! HarfBuzz text shaping integration (Phase A-2).
//!
//! Provides real font-aware text shaping via HarfBuzz FFI.
//! All unsafe code is confined to this module.
//!
//! # Safety
//!
//! All `unsafe` blocks in this module call HarfBuzz C FFI functions via
//! `harfbuzz_sys`. Each call follows HarfBuzz's documented API contracts:
//!
//! - `hb_blob_create`: `font_data` pointer is valid for the duration of the
//!   call (HB_MEMORY_MODE_READONLY does not take ownership). Null destroy
//!   function is correct since we do not allocate the data.
//! - `hb_face_create`/`hb_font_create`: take ownership of blob/face refs;
//!   correctly destroyed via `hb_*_destroy` at end of function.
//! - `hb_buffer_add_utf8`: text pointer and length are valid; text is not
//!   mutated during shaping.
//! - `hb_buffer_get_glyph_infos`/`hb_buffer_get_glyph_positions`: return
//!   pointers valid until buffer is destroyed. We iterate within bounds
//!   (`0..glyph_count`) before destroying the buffer.
//!
//! Resource cleanup is guaranteed: blob, face, font, and buffer are all
//! destroyed in a single `unsafe` block at the end of `shape_harfbuzz`,
//! regardless of intermediate results.

#![allow(unsafe_code)]
#![allow(dead_code)]

use harfbuzz_sys as hb;

use crate::fp266::Fp266;
use crate::shaping::{ShapedGlyph, ShapedRun};

/// OpenType feature to enable/disable during shaping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Feature {
    pub tag: [u8; 4],
    pub value: u32,
    pub start: u32,
    pub end: u32,
}

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

pub const KERN: &[u8; 4] = b"kern";
pub const LIGA: &[u8; 4] = b"liga";
pub const CALT: &[u8; 4] = b"calt";
pub const TNUM: &[u8; 4] = b"tnum";
pub const ONUM: &[u8; 4] = b"onum";
pub const LNUM: &[u8; 4] = b"lnum";

/// Shapes text using HarfBuzz with optional script, language, direction, and feature overrides.
pub fn shape_harfbuzz(
    font_data: &[u8],
    text: &str,
    font_size: Fp266,
    script: Option<hb::hb_script_t>,
    language: Option<&str>,
    direction: Option<hb::hb_direction_t>,
    features: Option<&[Feature]>,
) -> ShapedRun {
    if text.is_empty() {
        return ShapedRun::empty();
    }

    let blob = unsafe {
        hb::hb_blob_create(
            font_data.as_ptr() as *const i8,
            font_data.len() as u32,
            hb::HB_MEMORY_MODE_READONLY,
            std::ptr::null_mut(),
            None,
        )
    };

    let face = unsafe { hb::hb_face_create(blob, 0) };
    let font = unsafe { hb::hb_font_create(face) };

    let (scale, ptem) = match ttf_parser::Face::parse(font_data, 0) {
        Ok(tp_face) => {
            let upem = tp_face.units_per_em();
            let design_size_pt = font_size.to_f64();
            let s = (design_size_pt * upem as f64 / 72.0) as i32;
            (s, design_size_pt as f32)
        }
        Err(_) => {
            tracing::warn!(
                "harfbuzz: failed to parse font with ttf_parser, falling back to raw scale"
            );
            (font_size.raw() as i32, font_size.to_f64() as f32)
        }
    };
    unsafe {
        hb::hb_font_set_scale(font, scale, scale);
        hb::hb_font_set_ptem(font, ptem);
    }

    unsafe { hb::hb_ot_font_set_funcs(font) };

    let buffer = unsafe { hb::hb_buffer_create() };

    unsafe {
        hb::hb_buffer_add_utf8(
            buffer,
            text.as_ptr() as *const i8,
            text.len() as i32,
            0,
            text.len() as i32,
        );
    }

    if let Some(script) = script {
        unsafe { hb::hb_buffer_set_script(buffer, script) };
    }
    if let Some(lang) = language {
        let lang_cstr = std::ffi::CString::new(lang).unwrap_or_default();
        unsafe {
            hb::hb_buffer_set_language(buffer, hb::hb_language_from_string(lang_cstr.as_ptr(), -1))
        };
    }
    if let Some(dir) = direction {
        unsafe { hb::hb_buffer_set_direction(buffer, dir) };
    } else {
        unsafe { hb::hb_buffer_guess_segment_properties(buffer) };
    }

    let hb_features: Vec<hb::hb_feature_t> = features
        .map(|fs| {
            fs.iter()
                .map(|f| hb::hb_feature_t {
                    tag: u32::from_be_bytes(f.tag),
                    value: f.value,
                    start: f.start,
                    end: f.end,
                })
                .collect()
        })
        .unwrap_or_default();

    let features_ptr = if hb_features.is_empty() {
        std::ptr::null()
    } else {
        hb_features.as_ptr()
    };

    unsafe {
        hb::hb_shape(font, buffer, features_ptr, hb_features.len() as u32);
    }

    let glyph_count = unsafe { hb::hb_buffer_get_length(buffer) } as usize;
    let glyph_infos = unsafe { hb::hb_buffer_get_glyph_infos(buffer, std::ptr::null_mut()) };
    let glyph_positions =
        unsafe { hb::hb_buffer_get_glyph_positions(buffer, std::ptr::null_mut()) };

    let mut glyphs = Vec::with_capacity(glyph_count);
    let mut total_advance = Fp266::ZERO;

    for i in 0..glyph_count {
        let info = unsafe { *glyph_infos.add(i) };
        let pos = unsafe { *glyph_positions.add(i) };

        let x_offset = Fp266::from_raw(pos.x_offset as i64);
        let y_offset = Fp266::from_raw(pos.y_offset as i64);
        let advance = Fp266::from_raw(pos.x_advance as i64);

        total_advance += advance;

        glyphs.push(ShapedGlyph {
            glyph_id: info.codepoint,
            x_offset,
            y_offset,
            advance,
            cluster_id: info.cluster,
        });
    }

    unsafe {
        hb::hb_buffer_destroy(buffer);
        hb::hb_font_destroy(font);
        hb::hb_face_destroy(face);
        hb::hb_blob_destroy(blob);
    }

    ShapedRun {
        glyphs,
        total_advance,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_font_data() -> Vec<u8> {
        ldir_test_helpers::test_font_data()
    }

    #[test]
    fn shape_empty_text() {
        let data = test_font_data();
        let run = shape_harfbuzz(&data, "", Fp266::from_int(12), None, None, None, None);
        assert!(run.glyphs.is_empty());
        assert!(run.total_advance.is_zero());
    }

    #[test]
    fn shape_ascii_text() {
        let data = test_font_data();
        let run = shape_harfbuzz(&data, "hello", Fp266::from_int(12), None, None, None, None);
        assert_eq!(run.glyphs.len(), 5);
        assert!(run.total_advance.raw() > 0);
    }

    #[test]
    fn shape_single_char() {
        let data = test_font_data();
        let run = shape_harfbuzz(&data, "A", Fp266::from_int(12), None, None, None, None);
        assert_eq!(run.glyphs.len(), 1);
        assert!(run.glyphs[0].glyph_id > 0);
    }

    #[test]
    fn shape_with_script() {
        let data = test_font_data();
        let run = shape_harfbuzz(
            &data,
            "hello",
            Fp266::from_int(12),
            Some(hb::HB_SCRIPT_LATIN),
            None,
            None,
            None,
        );
        assert_eq!(run.glyphs.len(), 5);
    }

    #[test]
    fn shape_with_language() {
        let data = test_font_data();
        let run = shape_harfbuzz(
            &data,
            "hello",
            Fp266::from_int(12),
            None,
            Some("en"),
            None,
            None,
        );
        assert_eq!(run.glyphs.len(), 5);
    }

    #[test]
    fn shape_with_rtl_direction() {
        let data = test_font_data();
        let run = shape_harfbuzz(
            &data,
            "hello",
            Fp266::from_int(12),
            None,
            None,
            Some(hb::HB_DIRECTION_RTL),
            None,
        );
        assert_eq!(run.glyphs.len(), 5);
    }

    #[test]
    fn shape_deterministic() {
        let data = test_font_data();
        let run1 = shape_harfbuzz(&data, "test", Fp266::from_int(12), None, None, None, None);
        let run2 = shape_harfbuzz(&data, "test", Fp266::from_int(12), None, None, None, None);
        assert_eq!(run1, run2);
    }

    #[test]
    fn shape_cluster_ids() {
        let data = test_font_data();
        let run = shape_harfbuzz(&data, "ab", Fp266::from_int(12), None, None, None, None);
        assert_eq!(run.glyphs.len(), 2);
        assert_eq!(run.glyphs[0].cluster_id, 0);
        assert_eq!(run.glyphs[1].cluster_id, 1);
    }

    #[test]
    fn shape_total_advance_positive() {
        let data = test_font_data();
        let run = shape_harfbuzz(
            &data,
            "Hello, world!",
            Fp266::from_int(12),
            None,
            None,
            None,
            None,
        );
        assert!(run.total_advance.raw() > 0);
    }

    #[test]
    fn shape_font_size_affects_advance() {
        let data = test_font_data();
        let run1 = shape_harfbuzz(&data, "A", Fp266::from_int(12), None, None, None, None);
        let run2 = shape_harfbuzz(&data, "A", Fp266::from_int(24), None, None, None, None);
        assert!(run2.total_advance.raw() > run1.total_advance.raw());
    }

    #[test]
    fn shape_unicode_text() {
        let data = test_font_data();
        let run = shape_harfbuzz(&data, "café", Fp266::from_int(12), None, None, None, None);
        assert!(!run.glyphs.is_empty());
    }
}
