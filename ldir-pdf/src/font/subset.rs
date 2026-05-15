//! TrueType font subsetting — reduces font binary to only used glyphs.
//!
//! Takes full font data and a set of glyph IDs, produces a minimal
//! TrueType font containing only the necessary tables and glyphs.
//!
//! Glyph IDs are NOT remapped — they retain their original values so
//! that `CIDToGIDMap: Identity` works correctly in the PDF.

#![allow(dead_code)]

use std::collections::HashSet;

use ttf_parser::Tag;

/// Tracks which glyph IDs are used in a document.
#[derive(Debug, Clone)]
pub struct FontSubset {
    glyph_ids: HashSet<u32>,
}

impl FontSubset {
    /// Create an empty subset.
    pub fn new() -> Self {
        Self {
            glyph_ids: HashSet::new(),
        }
    }

    /// Add a glyph ID to the subset.
    pub fn add_glyph(&mut self, glyph_id: u32) {
        self.glyph_ids.insert(glyph_id);
    }

    /// Check if a glyph ID is in the subset (glyph 0 = .notdef always included).
    pub fn contains(&self, glyph_id: u32) -> bool {
        glyph_id == 0 || self.glyph_ids.contains(&glyph_id)
    }

    /// Number of used glyph IDs (excluding .notdef).
    pub fn len(&self) -> usize {
        self.glyph_ids.len()
    }

    /// Whether the subset is empty.
    pub fn is_empty(&self) -> bool {
        self.glyph_ids.is_empty()
    }

    /// Iterate over used glyph IDs.
    pub fn iter(&self) -> impl Iterator<Item = u32> + '_ {
        self.glyph_ids.iter().copied()
    }

    /// Collect all glyph IDs from a G-IR document.
    pub fn from_gir(gir_doc: &ldir_ir::gir::GIRDocument) -> Self {
        use ldir_ir::gir::GIROpcode;
        let mut subset = Self::new();
        for page in gir_doc.iter() {
            for cmd in page.iter() {
                if cmd.opcode() == GIROpcode::PutGlyph
                    && let Some(gid) = cmd.arg(0)
                {
                    subset.add_glyph(gid as u32);
                }
            }
        }
        subset
    }
}

impl Default for FontSubset {
    fn default() -> Self {
        Self::new()
    }
}

/// Subset a TrueType font to only the specified glyph IDs.
///
/// Glyph IDs are NOT remapped — they retain their original values.
/// Unused glyph slots (between 0 and max used ID) are empty (zero-length).
pub fn subset_font(full_data: &[u8], used_glyphs: &HashSet<u32>) -> Vec<u8> {
    let face = match ttf_parser::Face::parse(full_data, 0) {
        Ok(f) => f,
        Err(_) => return full_data.to_vec(),
    };

    let raw = face.raw_face();

    // Determine max glyph ID we need to cover
    let max_gid = used_glyphs.iter().copied().max().unwrap_or(0) as u16;
    let num_glyphs = (max_gid as u32 + 1) as u16;

    // Get loca format from head table
    let head_data = match raw.table(Tag::from_bytes(b"head")) {
        Some(d) => d,
        None => return full_data.to_vec(),
    };
    let loca_format = u16::from_be_bytes([head_data[50], head_data[51]]);

    let loca_table = raw.table(Tag::from_bytes(b"loca"));
    let glyf_table = raw.table(Tag::from_bytes(b"glyf"));

    // Build new glyf and loca tables
    let mut new_glyf: Vec<u8> = Vec::new();
    let mut new_loca: Vec<u32> = Vec::with_capacity(num_glyphs as usize + 1);
    new_loca.push(0);

    for gid in 0..=max_gid {
        let (start, len) = if let (Some(loca), Some(glyf)) = (loca_table, glyf_table) {
            get_glyph_range(loca, glyf, gid, loca_format).unwrap_or((0, 0))
        } else {
            (0, 0)
        };
        if len > 0
            && let Some(glyf) = glyf_table
        {
            new_glyf.extend_from_slice(&glyf[start..start + len]);
        }
        // Pad to 2-byte boundary (required by short loca format)
        while !new_glyf.len().is_multiple_of(2) {
            new_glyf.push(0);
        }
        new_loca.push(new_glyf.len() as u32);
    }

    // Serialize loca table
    let mut loca_bytes = Vec::new();
    if loca_format == 0 {
        for &off in &new_loca {
            loca_bytes.extend_from_slice(&(off / 2).to_be_bytes());
        }
    } else {
        for &off in &new_loca {
            loca_bytes.extend_from_slice(&off.to_be_bytes());
        }
    }

    // Build new hmtx: for each glyph 0..=max_gid, write advanceWidth + lsb
    let mut hmtx_bytes = Vec::new();
    for gid in 0..=max_gid {
        let advance = face
            .glyph_hor_advance(ttf_parser::GlyphId(gid))
            .unwrap_or(0);
        let lsb = face
            .glyph_hor_side_bearing(ttf_parser::GlyphId(gid))
            .unwrap_or(0);
        hmtx_bytes.extend_from_slice(&advance.to_be_bytes());
        hmtx_bytes.extend_from_slice(&(lsb as u16).to_be_bytes());
    }
    let new_num_hmetrics = num_glyphs;

    // Collect tables
    let mut tables: Vec<([u8; 4], Vec<u8>)> = Vec::new();

    // head: copy raw, zero out checksumAdjustment (fixed at end)
    if let Some(data) = raw.table(Tag::from_bytes(b"head")) {
        let mut h = data.to_vec();
        h[8..12].copy_from_slice(&[0, 0, 0, 0]);
        tables.push((*b"head", h));
    }

    // hhea: copy raw, fix numberOfHMetrics at offset 34
    if let Some(data) = raw.table(Tag::from_bytes(b"hhea")) {
        let mut h = data.to_vec();
        h[34..36].copy_from_slice(&new_num_hmetrics.to_be_bytes());
        tables.push((*b"hhea", h));
    }

    tables.push((*b"hmtx", hmtx_bytes));

    // maxp: copy raw, fix numGlyphs at offset 4
    if let Some(data) = raw.table(Tag::from_bytes(b"maxp")) {
        let mut m = data.to_vec();
        if m.len() >= 6 {
            m[4..6].copy_from_slice(&num_glyphs.to_be_bytes());
        }
        tables.push((*b"maxp", m));
    }

    // name, post, OS/2: copy as-is
    for &tag in [b"name", b"post", b"OS/2"] {
        if let Some(data) = raw.table(Tag::from_bytes(&tag)) {
            tables.push((tag, data.to_vec()));
        }
    }

    // cmap: copy as-is (glyph IDs aren't remapped, so cmap stays valid)
    if let Some(data) = raw.table(Tag::from_bytes(b"cmap")) {
        tables.push((*b"cmap", data.to_vec()));
    }

    tables.push((*b"loca", loca_bytes));
    tables.push((*b"glyf", new_glyf));

    // Sort by tag (required by TrueType spec)
    tables.sort_by_key(|(tag, _)| *tag);

    assemble_font(tables)
}

#[inline]
fn get_glyph_range(
    loca: &[u8],
    glyf: &[u8],
    glyph_id: u16,
    loca_format: u16,
) -> Option<(usize, usize)> {
    let idx = glyph_id as usize;

    let (start, end) = if loca_format == 0 {
        if idx * 2 + 4 > loca.len() {
            return None;
        }
        let s = u16::from_be_bytes([loca[idx * 2], loca[idx * 2 + 1]]) as usize * 2;
        let e = u16::from_be_bytes([loca[idx * 2 + 2], loca[idx * 2 + 3]]) as usize * 2;
        (s, e)
    } else {
        if idx * 4 + 8 > loca.len() {
            return None;
        }
        let s = u32::from_be_bytes([
            loca[idx * 4],
            loca[idx * 4 + 1],
            loca[idx * 4 + 2],
            loca[idx * 4 + 3],
        ]) as usize;
        let e = u32::from_be_bytes([
            loca[idx * 4 + 4],
            loca[idx * 4 + 5],
            loca[idx * 4 + 6],
            loca[idx * 4 + 7],
        ]) as usize;
        (s, e)
    };

    if start == end {
        return Some((0, 0));
    }
    if end > glyf.len() {
        return Some((0, 0));
    }
    Some((start, end - start))
}

fn assemble_font(tables: Vec<([u8; 4], Vec<u8>)>) -> Vec<u8> {
    let num_tables = tables.len() as u16;
    let header_size = 12usize;
    let dir_size = tables.len() * 16;
    let data_start = (header_size + dir_size + 3) & !3;

    // Calculate table offsets
    let mut current_offset = data_start;
    let mut table_offsets: Vec<(u32, u32)> = Vec::with_capacity(tables.len());
    for (_, data) in &tables {
        table_offsets.push((current_offset as u32, data.len() as u32));
        current_offset = (current_offset + data.len() + 3) & !3;
    }

    let total_size = current_offset;
    let mut out = Vec::with_capacity(total_size);

    // Offset table (12 bytes)
    out.extend_from_slice(&0x00010000u32.to_be_bytes());
    out.extend_from_slice(&num_tables.to_be_bytes());
    let sr = highest_power_of_2_u32(num_tables as u32) * 16;
    let es = (sr.trailing_zeros() as u16) - 4;
    let rs = num_tables as u32 * 16 - sr;
    out.extend_from_slice(&(sr as u16).to_be_bytes());
    out.extend_from_slice(&es.to_be_bytes());
    out.extend_from_slice(&(rs as u16).to_be_bytes());

    // Table directory
    for (i, (tag, data)) in tables.iter().enumerate() {
        out.extend_from_slice(tag);
        out.extend_from_slice(&calc_checksum(data).to_be_bytes());
        out.extend_from_slice(&table_offsets[i].0.to_be_bytes());
        out.extend_from_slice(&table_offsets[i].1.to_be_bytes());
    }

    // Pad to data_start
    while out.len() < data_start {
        out.push(0);
    }

    // Table data
    for (_, data) in &tables {
        out.extend_from_slice(data);
        while out.len() % 4 != 0 {
            out.push(0);
        }
    }

    // Fix head checksumAdjustment
    for (i, (tag, _)) in tables.iter().enumerate() {
        if tag == b"head" {
            let ho = table_offsets[i].0 as usize;
            out[ho + 8..ho + 12].copy_from_slice(&[0, 0, 0, 0]);
            let adj = 0xB1B0AFBAu32.wrapping_sub(calc_checksum(&out));
            out[ho + 8..ho + 12].copy_from_slice(&adj.to_be_bytes());
            break;
        }
    }

    out
}

fn calc_checksum(data: &[u8]) -> u32 {
    let mut sum: u32 = 0;
    let mut i = 0;
    while i + 4 <= data.len() {
        sum = sum.wrapping_add(u32::from_be_bytes([
            data[i],
            data[i + 1],
            data[i + 2],
            data[i + 3],
        ]));
        i += 4;
    }
    if i < data.len() {
        let mut pad = [0u8; 4];
        pad[..data.len() - i].copy_from_slice(&data[i..]);
        sum = sum.wrapping_add(u32::from_be_bytes(pad));
    }
    sum
}

fn highest_power_of_2_u32(v: u32) -> u32 {
    if v == 0 {
        return 0;
    }
    1 << (31 - v.leading_zeros())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_font_data() -> Option<Vec<u8>> {
        Some(ldir_test_helpers::test_font_data())
    }

    #[test]
    fn test_add_and_contains() {
        let mut subset = FontSubset::new();
        subset.add_glyph(42);
        assert!(subset.contains(42));
        assert!(!subset.contains(43));
    }

    #[test]
    fn test_notdef_always_included() {
        let subset = FontSubset::new();
        assert!(subset.contains(0));
    }

    #[test]
    fn test_len_and_is_empty() {
        let mut subset = FontSubset::new();
        assert!(subset.is_empty());
        subset.add_glyph(1);
        subset.add_glyph(2);
        assert_eq!(subset.len(), 2);
        assert!(!subset.is_empty());
    }

    #[test]
    fn test_iter() {
        let mut subset = FontSubset::new();
        subset.add_glyph(10);
        subset.add_glyph(20);
        let ids: Vec<u32> = subset.iter().collect();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&10));
        assert!(ids.contains(&20));
    }

    #[test]
    fn test_subset_font_produces_valid_ttf() {
        let Some(data) = get_font_data() else { return };

        let face = ttf_parser::Face::parse(&data, 0).unwrap();
        let mut glyphs = HashSet::new();
        glyphs.insert(0);
        for ch in "Hello World".chars() {
            if let Some(gid) = face.glyph_index(ch) {
                glyphs.insert(gid.0 as u32);
            }
        }

        let subsetted = subset_font(&data, &glyphs);

        assert!(
            subsetted.len() < data.len(),
            "subset {} should be smaller than original {}",
            subsetted.len(),
            data.len()
        );

        let result = ttf_parser::Face::parse(&subsetted, 0);
        assert!(
            result.is_ok(),
            "subset font should be valid: {:?}",
            result.err()
        );

        let subset_face = result.unwrap();

        // Glyph IDs should still map correctly (not remapped)
        for ch in "Hello World".chars() {
            let original_gid = face.glyph_index(ch).unwrap();
            let subset_gid = subset_face.glyph_index(ch).unwrap();
            assert_eq!(
                original_gid.0, subset_gid.0,
                "glyph mapping for '{ch}' should be preserved"
            );
        }
    }

    #[test]
    fn test_subset_significant_size_reduction() {
        let Some(data) = get_font_data() else { return };

        let face = ttf_parser::Face::parse(&data, 0).unwrap();
        let mut glyphs = HashSet::new();
        glyphs.insert(0);
        for ch in "ABCDEFabcdef0123456789 .,:;!?()-".chars() {
            if let Some(gid) = face.glyph_index(ch) {
                glyphs.insert(gid.0 as u32);
            }
        }

        let subsetted = subset_font(&data, &glyphs);

        let reduction = 1.0 - (subsetted.len() as f64 / data.len() as f64);
        assert!(
            reduction > 0.5,
            "subset should be >50% smaller: original={}, subset={}, reduction={:.1}%",
            data.len(),
            subsetted.len(),
            reduction * 100.0
        );
    }

    #[test]
    fn test_checksum_calculation() {
        let data = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05];
        let sum = calc_checksum(&data);
        let expected = 0x00010203u32.wrapping_add(0x04050000);
        assert_eq!(sum, expected);
    }

    #[test]
    fn test_highest_power_of_2() {
        assert_eq!(highest_power_of_2_u32(1), 1);
        assert_eq!(highest_power_of_2_u32(2), 2);
        assert_eq!(highest_power_of_2_u32(3), 2);
        assert_eq!(highest_power_of_2_u32(4), 4);
        assert_eq!(highest_power_of_2_u32(5), 4);
        assert_eq!(highest_power_of_2_u32(16), 16);
        assert_eq!(highest_power_of_2_u32(0), 0);
    }
}
