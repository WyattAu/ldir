//! Font face loader using `ttf-parser`.
//!
//! Parses TrueType/OpenType font data and extracts metrics needed
//! for PDF font embedding (FontDescriptor, widths, ToUnicode CMap).

use ttf_parser::GlyphId;
use ttf_parser::Rect as TtfRect;

/// Metrics for a single glyph, scaled to PDF units (1/1000 em).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FontMetrics {
    /// Advance width in 1/1000 em units.
    pub advance_width: f32,
    /// Left side bearing in 1/1000 em units.
    pub lsb: f32,
    /// Glyph bounding box in 1/1000 em units.
    pub bbox: Option<GlyphBBox>,
}

/// Glyph bounding box in 1/1000 em units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlyphBBox {
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
}

/// Font information needed for PDF FontDescriptor.
#[derive(Debug, Clone)]
pub struct PdfFontInfo {
    /// PostScript name of the font (e.g. "DejaVuSans").
    pub postscript_name: String,
    /// Family name (e.g. "DejaVu Sans").
    pub family_name: String,
    /// Units per em (typically 1000 or 2048).
    pub units_per_em: u16,
    /// Ascent in font units.
    pub ascent: i16,
    /// Descent in font units.
    pub descent: i16,
    /// Cap height in font units.
    pub cap_height: f32,
    /// X-height in font units.
    pub x_height: f32,
    /// Italic angle in degrees.
    pub italic_angle: f32,
    /// Font bounding box in font units.
    pub bbox: TtfRect,
    /// Whether the font is monospace.
    pub is_monospace: bool,
    /// Number of glyphs in the font.
    pub glyph_count: u16,
}

/// A loaded font face, parsed from TrueType/OpenType data.
pub struct FontFace {
    data: Vec<u8>,
    face: ttf_parser::Face<'static>,
    /// Cached font info for PDF embedding.
    info: PdfFontInfo,
}

// SAFETY: FontFace owns `data` and creates a `Face<'static>` from it.
// The `data` field is never moved or dropped while `face` is alive,
// because both are dropped together when FontFace is dropped.
// This is the standard pattern for self-referential font data in Rust.
#[allow(unsafe_code)]
unsafe impl Send for FontFace {}
#[allow(unsafe_code)]
unsafe impl Sync for FontFace {}

impl Clone for FontFace {
    fn clone(&self) -> Self {
        // Re-parse from the owned data to reconstruct the self-referential struct.
        Self::from_bytes(&self.data).unwrap_or_else(|e| {
            unreachable!("font data should be valid when cloning FontFace: {e}")
        })
    }
}

impl FontFace {
    /// Parse font data and return a loaded font face.
    ///
    /// # Errors
    ///
    /// Returns an error if the font data is empty or invalid.
    pub fn from_bytes(data: &[u8]) -> Result<FontFace, String> {
        if data.is_empty() {
            return Err("font data is empty".into());
        }

        // We need to extend the lifetime to 'static for self-referential struct.
        // This is safe because FontFace owns the data and both are dropped together.
        let data_owned = data.to_vec();
        let face = ttf_parser::Face::parse(&data_owned, 0)
            .map_err(|e| format!("failed to parse font: {e}"))?;

        let info = extract_font_info(&face);

        // SAFETY: transmute the face lifetime from 'a to 'static.
        // This is safe because:
        // 1. `data_owned` is stored in the same struct
        // 2. Both are dropped together
        // 3. Clone re-parses from owned data (see impl Clone above)
        #[allow(unsafe_code)]
        let face: ttf_parser::Face<'static> =
            unsafe { std::mem::transmute::<ttf_parser::Face<'_>, ttf_parser::Face<'static>>(face) };

        Ok(FontFace {
            data: data_owned,
            face,
            info,
        })
    }

    /// Raw font bytes for embedding in PDF.
    pub fn raw_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Font information for PDF FontDescriptor.
    pub fn pdf_info(&self) -> &PdfFontInfo {
        &self.info
    }

    /// Number of glyphs in the font.
    pub fn glyph_count(&self) -> u16 {
        self.info.glyph_count
    }

    /// Map a character to a glyph ID via the font's cmap table.
    pub fn glyph_id_for_char(&self, ch: char) -> Option<u32> {
        self.face.glyph_index(ch).map(|id| id.0 as u32)
    }

    /// Get metrics for a glyph, scaled to 1/1000 em units.
    pub fn glyph_metrics(&self, glyph_id: u32) -> Option<FontMetrics> {
        let gid = GlyphId(glyph_id as u16);
        let upem = self.info.units_per_em as f32;
        let scale = 1000.0 / upem;

        let advance = self.face.glyph_hor_advance(gid)? as f32 * scale;

        let lsb = self.face.glyph_hor_side_bearing(gid).unwrap_or(0) as f32 * scale;

        let bbox = self.face.glyph_bounding_box(gid).map(|r| GlyphBBox {
            x_min: r.x_min as f32 * scale,
            y_min: r.y_min as f32 * scale,
            x_max: r.x_max as f32 * scale,
            y_max: r.y_max as f32 * scale,
        });

        Some(FontMetrics {
            advance_width: advance,
            lsb,
            bbox,
        })
    }

    /// Get advance width for a glyph in 1/1000 em units.
    pub fn glyph_advance_width(&self, glyph_id: u32) -> Option<f32> {
        let gid = GlyphId(glyph_id as u16);
        let upem = self.info.units_per_em as f32;
        let scale = 1000.0 / upem;
        self.face.glyph_hor_advance(gid).map(|w| w as f32 * scale)
    }

    /// Map a glyph ID back to a Unicode codepoint (for ToUnicode CMap).
    ///
    /// Returns the first codepoint that maps to this glyph.
    pub fn glyph_to_unicode(&self, glyph_id: u32) -> Option<char> {
        let gid = GlyphId(glyph_id as u16);
        let mut result: Option<char> = None;
        if let Some(cmap) = self.face.tables().cmap {
            for i in 0..cmap.subtables.len() {
                if let Some(subtable) = cmap.subtables.get(i)
                    && subtable.is_unicode()
                {
                    subtable.codepoints(|ch| {
                        if let Some(mapped_gid) = subtable.glyph_index(ch)
                            && mapped_gid == gid
                            && result.is_none()
                        {
                            result = char::from_u32(ch);
                        }
                    });
                    if result.is_some() {
                        return result;
                    }
                }
            }
        }
        result
    }

    /// Internal access to the ttf-parser face (for PDF embedding).
    pub(crate) fn face(&self) -> &ttf_parser::Face<'static> {
        &self.face
    }

    /// Iterate over all glyph-to-unicode mappings.
    pub fn iter_cmap(&self) -> impl Iterator<Item = (u32, char)> + '_ {
        CmapIterator::new(&self.face)
    }
}

/// Iterator over cmap entries (glyph_id → unicode char).
struct CmapIterator<'a> {
    face: &'a ttf_parser::Face<'a>,
    codepoint_ranges: Vec<std::ops::Range<u32>>,
    current_range: usize,
    current_cp: u32,
    emitted: std::collections::HashSet<u32>,
}

impl<'a> CmapIterator<'a> {
    fn new(face: &'a ttf_parser::Face<'a>) -> Self {
        Self {
            face,
            codepoint_ranges: Vec::new(),
            current_range: 0,
            current_cp: 0,
            emitted: std::collections::HashSet::new(),
        }
    }
}

impl Iterator for CmapIterator<'_> {
    type Item = (u32, char);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(_cmap) = self.face.tables().cmap {
            if self.codepoint_ranges.is_empty() {
                self.codepoint_ranges.push(0x20..0x7F);
                self.codepoint_ranges.push(0xA0..0x100);
                self.codepoint_ranges.push(0x100..0x180);
                self.codepoint_ranges.push(0x200..0x300);
                self.codepoint_ranges.push(0x300..0x400);
                self.codepoint_ranges.push(0x400..0x500);
                self.codepoint_ranges.push(0x2000..0x2100);
            }

            while self.current_range < self.codepoint_ranges.len() {
                let range = &self.codepoint_ranges[self.current_range];
                while self.current_cp < range.end {
                    let cp = self.current_cp;
                    self.current_cp += 1;

                    if let Some(ch) = char::from_u32(cp)
                        && let Some(gid) = self.face.glyph_index(ch)
                        && gid.0 != 0
                        && self.emitted.insert(gid.0 as u32)
                    {
                        return Some((gid.0 as u32, ch));
                    }
                }
                self.current_range += 1;
                self.current_cp = self
                    .codepoint_ranges
                    .get(self.current_range)
                    .map(|r| r.start)
                    .unwrap_or(0);
            }
        }
        None
    }
}

/// Extract font information needed for PDF embedding.
fn extract_font_info(face: &ttf_parser::Face) -> PdfFontInfo {
    let postscript_name = face
        .names()
        .into_iter()
        .find(|name| name.name_id == ttf_parser::name_id::POST_SCRIPT_NAME && name.is_unicode())
        .or_else(|| {
            face.names()
                .into_iter()
                .find(|name| name.name_id == ttf_parser::name_id::POST_SCRIPT_NAME)
        })
        .and_then(|name| name.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let family_name = face
        .names()
        .into_iter()
        .find(|name| name.name_id == ttf_parser::name_id::FAMILY && name.is_unicode())
        .or_else(|| {
            face.names()
                .into_iter()
                .find(|name| name.name_id == ttf_parser::name_id::FAMILY)
        })
        .and_then(|name| name.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let units_per_em = face.units_per_em();
    let global_bbox = face.global_bounding_box();

    let ascent = face.ascender();
    let descent = face.descender();

    // Estimate cap_height from 'H' glyph bbox, or use ascent
    let cap_height = face
        .glyph_bounding_box(face.glyph_index('H').unwrap_or(GlyphId(0)))
        .map(|r| r.y_max as f32)
        .unwrap_or(ascent as f32);

    // Estimate x_height from 'x' glyph bbox, or use 0.5 * ascent
    let x_height = face
        .glyph_bounding_box(face.glyph_index('x').unwrap_or(GlyphId(0)))
        .map(|r| r.y_max as f32)
        .unwrap_or(ascent as f32 * 0.5);

    // Extract italic angle from Face API
    let italic_angle = face.italic_angle();

    PdfFontInfo {
        postscript_name,
        family_name,
        units_per_em,
        ascent,
        descent,
        cap_height,
        x_height,
        italic_angle,
        bbox: global_bbox,
        is_monospace: face.is_monospaced(),
        glyph_count: face.number_of_glyphs(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Use a small TrueType font fixture for tests.
    // We generate a minimal valid TTF header + required tables.
    fn make_minimal_ttf() -> Vec<u8> {
        ldir_test_helpers::test_font_data()
    }

    #[test]
    fn test_font_face_from_system_font() -> Result<(), Box<dyn std::error::Error>> {
        let data = make_minimal_ttf();
        if data.is_empty() {
            return Ok(());
        }
        let face = FontFace::from_bytes(&data)?;
        assert!(face.glyph_count() > 0);
        assert!(face.pdf_info().units_per_em > 0);
        assert!(!face.pdf_info().postscript_name.is_empty());
        Ok(())
    }

    #[test]
    fn test_glyph_id_for_char() -> Result<(), Box<dyn std::error::Error>> {
        let data = make_minimal_ttf();
        if data.is_empty() {
            return Ok(());
        }
        let face = FontFace::from_bytes(&data)?;
        let gid = face.glyph_id_for_char('A');
        assert!(gid.is_some());
        let g = gid.ok_or("no glyph for 'A'")?;
        assert!(g > 0);
        Ok(())
    }

    #[test]
    fn test_glyph_metrics() -> Result<(), Box<dyn std::error::Error>> {
        let data = make_minimal_ttf();
        if data.is_empty() {
            return Ok(());
        }
        let face = FontFace::from_bytes(&data)?;
        let gid = face.glyph_id_for_char('A').ok_or("no glyph for 'A'")?;
        let metrics = face.glyph_metrics(gid).ok_or("no metrics for glyph")?;
        assert!(metrics.advance_width > 0.0);
        Ok(())
    }

    #[test]
    fn test_glyph_to_unicode_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let data = make_minimal_ttf();
        if data.is_empty() {
            return Ok(());
        }
        let face = FontFace::from_bytes(&data)?;
        let gid = face.glyph_id_for_char('A').ok_or("no glyph for 'A'")?;
        let back = face.glyph_to_unicode(gid);
        assert_eq!(back, Some('A'));
        Ok(())
    }

    #[test]
    fn test_pdf_font_info() -> Result<(), Box<dyn std::error::Error>> {
        let data = make_minimal_ttf();
        if data.is_empty() {
            return Ok(());
        }
        let face = FontFace::from_bytes(&data)?;
        let info = face.pdf_info();
        assert!(info.ascent > 0);
        assert!(info.descent <= 0);
        assert!(info.units_per_em >= 16);
        assert!(info.glyph_count > 0);
        Ok(())
    }

    #[test]
    fn test_raw_bytes_accessible() -> Result<(), Box<dyn std::error::Error>> {
        let data = make_minimal_ttf();
        if data.is_empty() {
            return Ok(());
        }
        let face = FontFace::from_bytes(&data)?;
        assert!(!face.raw_bytes().is_empty());
        Ok(())
    }

    #[test]
    fn test_empty_data_rejected() {
        let result = FontFace::from_bytes(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_data_rejected() {
        let result = FontFace::from_bytes(&[0x00, 0x01, 0x02]);
        assert!(result.is_err());
    }
}
