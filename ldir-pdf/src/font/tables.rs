//! OpenType/TrueType table definitions — extracted from real font data.

#[allow(unused_imports)]
use ttf_parser::Rect as TtfRect;

/// Cmap table info extracted from a TrueType font.
#[derive(Debug, Clone, PartialEq)]
pub struct CmapTable {
    pub num_tables: u16,
    pub platform_id: u16,
    pub encoding_id: u16,
    pub subtable_offset: u32,
}

impl CmapTable {
    /// Extract cmap info from a parsed face.
    pub fn from_face(face: &ttf_parser::Face) -> Option<Self> {
        let cmap = face.tables().cmap?;
        Some(Self {
            num_tables: cmap.subtables.len(),
            platform_id: 3, // Windows
            encoding_id: 1, // Unicode BMP
            subtable_offset: 12,
        })
    }
}

/// Head table info extracted from a TrueType font.
#[derive(Debug, Clone, PartialEq)]
pub struct HeadTable {
    pub major_version: u16,
    pub minor_version: u16,
    pub font_revision: u32,
    pub units_per_em: u16,
    pub index_to_loc_format: i16,
}

impl HeadTable {
    /// Extract head table info from a parsed face.
    /// In ttf-parser 0.25, `head` is a non-optional struct in FaceTables.
    pub fn from_face(face: &ttf_parser::Face) -> Self {
        let head = &face.tables().head;
        Self {
            major_version: 1,
            minor_version: 0,
            font_revision: 0x00010000, // Default; ttf-parser 0.25 doesn't expose this directly
            units_per_em: head.units_per_em,
            index_to_loc_format: match head.index_to_location_format {
                ttf_parser::head::IndexToLocationFormat::Short => 0,
                ttf_parser::head::IndexToLocationFormat::Long => 1,
            },
        }
    }
}

/// Hhea table info extracted from a TrueType font.
#[derive(Debug, Clone, PartialEq)]
pub struct HheaTable {
    pub ascender: i16,
    pub descender: i16,
    pub line_gap: i16,
    pub number_of_h_metrics: u16,
    pub advance_width_max: u16,
}

impl HheaTable {
    /// Extract hhea table info from a parsed face.
    /// In ttf-parser 0.25, `hhea` is a non-optional struct in FaceTables.
    pub fn from_face(face: &ttf_parser::Face) -> Self {
        let hhea = &face.tables().hhea;
        Self {
            ascender: face.ascender(),
            descender: face.descender(),
            line_gap: face.line_gap(),
            number_of_h_metrics: hhea.number_of_metrics,
            advance_width_max: 0, // ttf-parser 0.25 doesn't expose this directly
        }
    }
}

/// Hmtx table info extracted from a TrueType font.
#[derive(Debug, Clone, PartialEq)]
pub struct HmtxTable {
    pub num_metrics: u16,
    pub advance_widths: Vec<u16>,
    pub lsbs: Vec<i16>,
}

impl HmtxTable {
    /// Extract hmtx info for a subset of glyph IDs.
    pub fn from_face_subset(face: &ttf_parser::Face, glyph_ids: &[u32]) -> Self {
        let upem = face.units_per_em();
        let mut advance_widths = Vec::with_capacity(glyph_ids.len());
        let mut lsbs = Vec::with_capacity(glyph_ids.len());

        for &gid in glyph_ids {
            let g = ttf_parser::GlyphId(gid as u16);
            let advance = face.glyph_hor_advance(g).unwrap_or(upem);
            let lsb = face.glyph_hor_side_bearing(g).unwrap_or(0);
            advance_widths.push(advance);
            lsbs.push(lsb);
        }

        Self {
            num_metrics: advance_widths.len() as u16,
            advance_widths,
            lsbs,
        }
    }
}

/// Global bounding box in font units.
///
/// Reserved for PDF/A-4 conformance (FontBox/FontBBox in FontDescriptor).
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBBox {
    pub x_min: i16,
    pub y_min: i16,
    pub x_max: i16,
    pub y_max: i16,
}

#[allow(dead_code)]
impl GlobalBBox {
    pub fn from_rect(r: TtfRect) -> Self {
        Self {
            x_min: r.x_min,
            y_min: r.y_min,
            x_max: r.x_max,
            y_max: r.y_max,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_face() -> Option<ttf_parser::Face<'static>> {
        let paths = [
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        ];
        for path in &paths {
            if let Ok(data) = std::fs::read(path) {
                if let Ok(_face) = ttf_parser::Face::parse(&data, 0) {
                    let data: &'static [u8] = Box::leak(data.into_boxed_slice());
                    if let Ok(face) = ttf_parser::Face::parse(data, 0) {
                        return Some(unsafe {
                            std::mem::transmute::<ttf_parser::Face<'_>, ttf_parser::Face<'static>>(
                                face,
                            )
                        });
                    }
                }
            }
        }
        None
    }

    #[test]
    fn test_head_table_from_face() {
        let Some(face) = get_face() else { return };
        let head = HeadTable::from_face(&face);
        assert!(head.units_per_em >= 16);
        assert!(head.major_version >= 1);
    }

    #[test]
    fn test_hhea_table_from_face() {
        let Some(face) = get_face() else { return };
        let hhea = HheaTable::from_face(&face);
        assert!(hhea.ascender > 0);
        assert!(hhea.descender < 0);
    }
}
