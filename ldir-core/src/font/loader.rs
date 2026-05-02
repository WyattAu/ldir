//! ttf-parser based font loading (Phase A-1).
//!
//! Wraps `ttf_parser::Face` with cached metrics and zero-copy data sharing
//! via `Arc<Vec<u8>>`.

use std::sync::Arc;

use ttf_parser::GlyphId;

use crate::fp266::Fp266;

/// A handle to raw font data shared via `Arc`.
#[derive(Clone)]
pub struct FontHandle {
    data: Arc<Vec<u8>>,
}

impl FontHandle {
    /// Creates a new font handle from shared raw font data.
    pub fn new(data: Arc<Vec<u8>>) -> Self {
        Self { data }
    }

    /// Returns a reference to the raw font bytes.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Consumes the handle and returns the underlying `Arc<Vec<u8>>`.
    pub fn into_arc(self) -> Arc<Vec<u8>> {
        self.data
    }

    /// Parses a font face at the given collection index.
    pub fn parse_face(
        &self,
        index: u32,
    ) -> Result<ttf_parser::Face<'_>, ttf_parser::FaceParsingError> {
        ttf_parser::Face::parse(&self.data, index)
    }
}

/// A loaded font with a handle and cached metrics.
pub struct LoadedFont {
    handle: FontHandle,
    upem: u16,
    face_index: u32,
}

impl LoadedFont {
    /// Returns a reference to the underlying font handle.
    pub fn handle(&self) -> &FontHandle {
        &self.handle
    }

    /// Returns a reference to the raw font bytes.
    pub fn data(&self) -> &[u8] {
        self.handle.data()
    }

    /// Returns the parsed `ttf_parser::Face` for this font.
    #[allow(clippy::expect_used)]
    pub fn face(&self) -> ttf_parser::Face<'_> {
        self.handle
            .parse_face(self.face_index)
            .expect("font data should be valid (verified at load time)")
    }

    /// Returns the face index within the font collection.
    pub fn face_index(&self) -> u32 {
        self.face_index
    }
}

/// Horizontal metrics for a single glyph.
#[derive(Clone, Debug, PartialEq)]
pub struct FontMetrics {
    /// Horizontal advance width in font units.
    pub advance_width: Option<u16>,
    /// Left side bearing in font units.
    pub left_side_bearing: Option<i16>,
    /// Bounding box of the glyph outline.
    pub bbox: Option<ttf_parser::Rect>,
}

/// General metadata extracted from a font face.
#[derive(Clone, Debug)]
pub struct FontInfo {
    /// Primary family name of the font.
    pub family_name: Option<String>,
    /// Style variant (e.g., "Normal", "Italic").
    pub style: String,
    /// Units per em (design grid size).
    pub units_per_em: u16,
    /// Whether the font is monospaced.
    pub is_monospace: bool,
    /// PostScript name of the font.
    pub post_script_name: Option<String>,
}

/// Loads a font from raw data (face index 0).
pub fn load_font(data: Arc<Vec<u8>>) -> Result<LoadedFont, String> {
    load_font_with_index(data, 0)
}

/// Loads a font from raw data at the given collection index.
pub fn load_font_with_index(data: Arc<Vec<u8>>, index: u32) -> Result<LoadedFont, String> {
    let handle = FontHandle::new(data);
    let face = handle
        .parse_face(index)
        .map_err(|e| format!("failed to parse font: {e}"))?;

    let upem = face.units_per_em();
    let upem = if upem == 0 { 1000 } else { upem };

    Ok(LoadedFont {
        handle,
        upem,
        face_index: index,
    })
}

/// Returns the glyph ID for a character, if the font supports it.
pub fn glyph_id_for_char(face: &ttf_parser::Face, ch: char) -> Option<GlyphId> {
    face.glyph_index(ch)
}

/// Returns horizontal metrics for a glyph.
pub fn glyph_metrics(face: &ttf_parser::Face, glyph_id: GlyphId) -> FontMetrics {
    let advance_width = face.glyph_hor_advance(glyph_id);
    let lsb = face.glyph_hor_side_bearing(glyph_id);
    let bbox = face.glyph_bounding_box(glyph_id);

    FontMetrics {
        advance_width,
        left_side_bearing: lsb,
        bbox,
    }
}

fn find_name(face: &ttf_parser::Face, name_id: u16) -> Option<String> {
    for name in face.names() {
        if name.name_id == name_id
            && let Some(s) = name.to_string()
        {
            return Some(s);
        }
    }
    None
}

/// Extracts general metadata from a font face.
pub fn font_info(face: &ttf_parser::Face) -> FontInfo {
    let family_name = find_name(face, ttf_parser::name_id::FAMILY)
        .or_else(|| find_name(face, ttf_parser::name_id::TYPOGRAPHIC_FAMILY));

    let style = format!("{:?}", face.style());

    let post_script_name = find_name(face, ttf_parser::name_id::POST_SCRIPT_NAME);

    FontInfo {
        family_name,
        style,
        units_per_em: face.units_per_em(),
        is_monospace: face.is_monospaced(),
        post_script_name,
    }
}

/// Returns the units-per-em of a loaded font.
pub fn units_per_em(font: &LoadedFont) -> u16 {
    font.upem
}

/// Computes a glyph's advance width in 26.6 fixed-point at the given font size.
pub fn glyph_advance_fp266(
    face: &ttf_parser::Face,
    glyph_id: GlyphId,
    font_size: Fp266,
    upem: u16,
) -> Fp266 {
    let advance = match face.glyph_hor_advance(glyph_id) {
        Some(v) => v as i64,
        None => 0,
    };
    let upem = upem as i64;
    if upem == 0 {
        return Fp266::ZERO;
    }
    let scale = (font_size.raw() * advance) / upem;
    Fp266::from_raw(scale)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_font_data() -> Arc<Vec<u8>> {
        let path = "/usr/share/fonts/TTF/DejaVuSans.ttf";
        Arc::new(std::fs::read(path).expect("test font should exist"))
    }

    #[test]
    fn load_font_success() {
        let data = test_font_data();
        let result = load_font(data);
        assert!(result.is_ok());
    }

    #[test]
    fn load_font_invalid_data() {
        let data = Arc::new(vec![0u8; 10]);
        let result = load_font(data);
        assert!(result.is_err());
    }

    #[test]
    fn font_info_returns_metadata() {
        let font = load_font(test_font_data()).unwrap();
        let face = font.face();
        let info = font_info(&face);
        assert!(info.units_per_em > 0);
        assert!(info.family_name.is_some());
    }

    #[test]
    fn glyph_id_for_known_char() {
        let font = load_font(test_font_data()).unwrap();
        let face = font.face();
        let gid = glyph_id_for_char(&face, 'A');
        assert!(gid.is_some());
        assert!(gid.unwrap().0 > 0);
    }

    #[test]
    fn glyph_id_for_missing_char() {
        let font = load_font(test_font_data()).unwrap();
        let face = font.face();
        let gid = glyph_id_for_char(&face, '\u{0001}');
        assert!(gid.is_none());
    }

    #[test]
    fn glyph_metrics_returns_data() {
        let font = load_font(test_font_data()).unwrap();
        let face = font.face();
        let gid = glyph_id_for_char(&face, 'A').unwrap();
        let metrics = glyph_metrics(&face, gid);
        assert!(metrics.advance_width.is_some());
    }

    #[test]
    fn glyph_advance_fp266_positive() {
        let font = load_font(test_font_data()).unwrap();
        let face = font.face();
        let gid = glyph_id_for_char(&face, 'A').unwrap();
        let advance = glyph_advance_fp266(&face, gid, Fp266::from_int(12), font.upem);
        assert!(advance.raw() > 0);
    }

    #[test]
    fn units_per_em_positive() {
        let font = load_font(test_font_data()).unwrap();
        assert!(units_per_em(&font) > 0);
    }

    #[test]
    fn font_handle_data_access() {
        let data = test_font_data();
        let handle = FontHandle::new(data.clone());
        assert_eq!(handle.data().len(), data.len());
    }

    #[test]
    fn font_handle_into_arc() {
        let data = Arc::new(vec![1u8, 2, 3]);
        let handle = FontHandle::new(data);
        let arc = handle.into_arc();
        assert_eq!(arc.len(), 3);
    }

    #[test]
    fn face_reparse_consistent() {
        let font = load_font(test_font_data()).unwrap();
        let face1 = font.face();
        let face2 = font.face();
        let gid1 = glyph_id_for_char(&face1, 'B');
        let gid2 = glyph_id_for_char(&face2, 'B');
        assert_eq!(gid1, gid2);
    }

    #[test]
    fn dejavu_sans_not_monospace() {
        let font = load_font(test_font_data()).unwrap();
        let face = font.face();
        let info = font_info(&face);
        assert!(!info.is_monospace);
    }

    #[test]
    fn post_script_name_present() {
        let font = load_font(test_font_data()).unwrap();
        let face = font.face();
        let info = font_info(&face);
        assert!(info.post_script_name.is_some());
    }
}
