//! TrueType font subsetting — reduces font binary to only used glyphs.
//!
//! Takes full font data and a set of glyph IDs, produces a minimal
//! TrueType font containing only the necessary tables and glyphs.
//!
//! Glyph IDs ARE remapped to a compact sequential range 0..N to
//! eliminate empty slots. A `CIDToGIDMap` stream is provided for the
//! PDF consumer to translate CIDs back to the new glyph IDs.

use std::collections::{HashMap, HashSet, VecDeque};

use ttf_parser::Tag;

/// Result of font subsetting with glyph ID remapping.
pub struct SubsetResult {
    /// The subsetted TrueType font binary (with remapped glyph IDs).
    pub font_data: Vec<u8>,
    /// CID-to-GID map stream for PDF embedding.
    /// `None` if no remapping was needed (identity).
    pub cid_to_gid_map: Option<Vec<u8>>,
    /// Maps original glyph ID -> new (remapped) glyph ID.
    #[allow(dead_code)]
    pub glyph_id_map: HashMap<u32, u32>,
}

/// Tracks which glyph IDs are used in a document.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FontSubset {
    glyph_ids: HashSet<u32>,
}

#[allow(dead_code)]
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

fn resolve_compound_glyphs(face: &ttf_parser::Face, used: &mut HashSet<u32>) {
    const ARG_1_AND_2_ARE_WORDS: u16 = 0x0001;
    const MORE_COMPONENTS: u16 = 0x0020;
    const WE_HAVE_A_SCALE: u16 = 0x0008;
    const WE_HAVE_AN_X_AND_Y_SCALE: u16 = 0x0040;
    const WE_HAVE_A_TWO_BY_TWO: u16 = 0x0080;

    let raw = face.raw_face();

    let Some(head_data) = raw.table(Tag::from_bytes(b"head")) else {
        return;
    };
    let loca_format = u16::from_be_bytes([head_data[50], head_data[51]]);
    let Some(loca_table) = raw.table(Tag::from_bytes(b"loca")) else {
        return;
    };
    let Some(glyf_table) = raw.table(Tag::from_bytes(b"glyf")) else {
        return;
    };

    let mut queue: VecDeque<u16> = used
        .iter()
        .filter_map(|&gid| u16::try_from(gid).ok())
        .collect();
    let mut visited: HashSet<u16> = HashSet::with_capacity(used.len());
    let safety_limit = used.len() * 128 + 1024;

    while let Some(gid) = queue.pop_front() {
        if visited.len() > safety_limit {
            break;
        }
        if !visited.insert(gid) {
            continue;
        }

        let Some((start, len)) = get_glyph_range(loca_table, glyf_table, gid, loca_format) else {
            continue;
        };
        if len < 10 {
            continue;
        }

        let glyph_data = &glyf_table[start..start + len];
        let num_contours = i16::from_be_bytes([glyph_data[0], glyph_data[1]]);
        if num_contours >= 0 {
            continue;
        }

        let mut offset = 10usize;
        loop {
            if offset + 4 > glyph_data.len() {
                break;
            }
            let flags = u16::from_be_bytes([glyph_data[offset], glyph_data[offset + 1]]);
            let component_gid =
                u16::from_be_bytes([glyph_data[offset + 2], glyph_data[offset + 3]]);
            offset += 4;

            if flags & ARG_1_AND_2_ARE_WORDS != 0 {
                offset += 4;
            } else {
                offset += 2;
            }
            if flags & WE_HAVE_A_TWO_BY_TWO != 0 {
                offset += 8;
            } else if flags & WE_HAVE_AN_X_AND_Y_SCALE != 0 {
                offset += 4;
            } else if flags & WE_HAVE_A_SCALE != 0 {
                offset += 2;
            }

            let component_u32 = component_gid as u32;
            if !used.contains(&component_u32) {
                used.insert(component_u32);
                queue.push_back(component_gid);
            }

            if flags & MORE_COMPONENTS == 0 {
                break;
            }
        }
    }
}

/// Subset a TrueType font to only the specified glyph IDs.
///
/// Glyph IDs are remapped to a compact sequential range 0..N.
/// The returned `SubsetResult` includes a `CIDToGIDMap` for the PDF
/// and a mapping from old to new glyph IDs.
pub fn subset_font(full_data: &[u8], used_glyphs: &HashSet<u32>) -> SubsetResult {
    let face = match ttf_parser::Face::parse(full_data, 0) {
        Ok(f) => f,
        Err(_) => {
            let map: HashMap<u32, u32> = used_glyphs.iter().map(|&g| (g, g)).collect();
            return SubsetResult {
                font_data: full_data.to_vec(),
                cid_to_gid_map: None,
                glyph_id_map: map,
            };
        }
    };

    let mut used = used_glyphs.clone();
    resolve_compound_glyphs(&face, &mut used);

    let raw = face.raw_face();

    // Ensure .notdef (glyph 0) is included
    used.insert(0);

    // Sort used glyph IDs to build remapping
    let mut used_sorted: Vec<u32> = used.iter().copied().collect();
    used_sorted.sort();

    let mut old_to_new: HashMap<u32, u32> = HashMap::with_capacity(used_sorted.len());
    for (new_id, &old_id) in used_sorted.iter().enumerate() {
        old_to_new.insert(old_id, new_id as u32);
    }

    let num_glyphs = used_sorted.len() as u16;
    let needs_remapping = used_sorted
        .last()
        .is_some_and(|&max| max as usize >= used_sorted.len());

    // Get loca format from head table
    let head_data = match raw.table(Tag::from_bytes(b"head")) {
        Some(d) => d,
        None => {
            return SubsetResult {
                font_data: full_data.to_vec(),
                cid_to_gid_map: None,
                glyph_id_map: old_to_new,
            };
        }
    };
    let loca_format = u16::from_be_bytes([head_data[50], head_data[51]]);

    let loca_table = raw.table(Tag::from_bytes(b"loca"));
    let glyf_table = raw.table(Tag::from_bytes(b"glyf"));

    // Build new glyf and loca tables — only used glyphs, contiguous
    let mut new_glyf: Vec<u8> = Vec::new();
    let mut new_loca: Vec<u32> = Vec::with_capacity(used_sorted.len() + 1);
    new_loca.push(0);

    for &old_gid in &used_sorted {
        let old_gid_u16 = old_gid as u16;
        let (start, len) = if let (Some(loca), Some(glyf)) = (loca_table, glyf_table) {
            get_glyph_range(loca, glyf, old_gid_u16, loca_format).unwrap_or((0, 0))
        } else {
            (0, 0)
        };
        if len > 0
            && let Some(glyf) = glyf_table
        {
            new_glyf.extend_from_slice(&glyf[start..start + len]);
        }
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

    // Build new hmtx: full 4-byte records for all used glyphs
    let mut hmtx_bytes = Vec::with_capacity(used_sorted.len() * 4);
    for &old_gid in &used_sorted {
        let advance = face
            .glyph_hor_advance(ttf_parser::GlyphId(old_gid as u16))
            .unwrap_or(0);
        let lsb = face
            .glyph_hor_side_bearing(ttf_parser::GlyphId(old_gid as u16))
            .unwrap_or(0);
        hmtx_bytes.extend_from_slice(&advance.to_be_bytes());
        hmtx_bytes.extend_from_slice(&(lsb as u16).to_be_bytes());
    }
    let new_num_hmetrics = num_glyphs;

    // Build cmap table with remapped glyph IDs
    let cmap_bytes = build_subset_cmap(&face, &used_sorted, &old_to_new);

    // Collect tables
    let mut tables: Vec<([u8; 4], Vec<u8>)> = Vec::new();

    // head: copy raw, zero out checksumAdjustment
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

    // name, post, OS/2, GSUB, GPOS: copy as-is
    for &tag in [b"name", b"post", b"OS/2", b"GSUB", b"GPOS"] {
        if let Some(data) = raw.table(Tag::from_bytes(&tag)) {
            tables.push((tag, data.to_vec()));
        }
    }

    for &tag in [b"VORG", b"vmtx", b"vhea"] {
        if let Some(data) = raw.table(Tag::from_bytes(&tag)) {
            tables.push((tag, data.to_vec()));
        }
    }

    // cmap: rebuilt with remapped glyph IDs
    tables.push((*b"cmap", cmap_bytes));

    tables.push((*b"loca", loca_bytes));
    tables.push((*b"glyf", new_glyf));

    tables.sort_by_key(|(tag, _)| *tag);

    let font_data = assemble_font(tables);

    // Build CIDToGIDMap stream if remapping occurred
    let cid_to_gid_map = if needs_remapping {
        let mut gid_map = Vec::with_capacity(1 + used_sorted.len() * 2);
        gid_map.push(1u8); // format: 1 = word-aligned
        for &old_id in &used_sorted {
            let new_id = old_to_new[&old_id] as u16;
            gid_map.extend_from_slice(&new_id.to_be_bytes());
        }
        Some(gid_map)
    } else {
        None
    };

    SubsetResult {
        font_data,
        cid_to_gid_map,
        glyph_id_map: old_to_new,
    }
}

/// Build a minimal format 4 cmap subtable containing only the used codepoints,
/// mapping them to their new (remapped) glyph IDs.
fn build_subset_cmap(
    face: &ttf_parser::Face,
    used_sorted: &[u32],
    old_to_new: &HashMap<u32, u32>,
) -> Vec<u8> {
    let mut mappings: Vec<(u32, u16)> = Vec::new();
    let used_set: HashSet<u32> = used_sorted.iter().copied().collect();

    if let Some(cmap) = face.tables().cmap {
        for i in 0..cmap.subtables.len() {
            if let Some(subtable) = cmap.subtables.get(i)
                && subtable.is_unicode()
            {
                subtable.codepoints(|ch| {
                    if let Some(gid) = subtable.glyph_index(ch) {
                        let gid_u32 = gid.0 as u32;
                        if gid_u32 != 0
                            && used_set.contains(&gid_u32)
                            && let Some(&new_gid) = old_to_new.get(&gid_u32)
                        {
                            mappings.push((ch, new_gid as u16));
                        }
                    }
                });
            }
        }
    }

    mappings.sort_by_key(|&(ch, _)| ch);
    mappings.dedup_by(|a, b| a.0 == b.0);

    if mappings.is_empty() {
        // Empty cmap: format 0 subtable with no mappings
        let mut out = Vec::new();
        out.extend_from_slice(&0u16.to_be_bytes()); // version
        out.extend_from_slice(&1u16.to_be_bytes()); // numTables
        // One subtable record: platform 3 (Windows), encoding 1 (Unicode BMP), offset 12
        out.extend_from_slice(&3u16.to_be_bytes());
        out.extend_from_slice(&1u16.to_be_bytes());
        out.extend_from_slice(&12u32.to_be_bytes());
        // Format 0 subtable
        out.extend_from_slice(&0u16.to_be_bytes()); // format
        out.extend_from_slice(&262u16.to_be_bytes()); // length (6 header + 256)
        out.extend_from_slice(&0u16.to_be_bytes()); // language
        out.extend(std::iter::repeat_n(0, 256));
        return out;
    }

    // Build format 4 subtable
    build_format4_cmap(&mappings)
}

/// Build a TrueType cmap format 4 subtable from sorted (codepoint, glyph_id) pairs.
fn build_format4_cmap(mappings: &[(u32, u16)]) -> Vec<u8> {
    let min_char = mappings[0].0 as u16;
    let max_char = mappings[mappings.len() - 1].0 as u16;
    // segCount includes the required 0xFFFF sentinel segment
    let real_seg_count = 1u16;
    let seg_count = real_seg_count + 1u16; // +1 for sentinel

    let seg_count_x2 = seg_count * 2;
    let search_range = highest_power_of_2_u16(seg_count_x2);
    let entry_selector = (search_range as u32 / 2).trailing_zeros() as u16;
    let range_shift = seg_count_x2 - search_range;

    let end_codes: Vec<u16> = vec![max_char, 0xFFFF];
    let start_codes: Vec<u16> = vec![min_char, 0xFFFF];

    let first_delta = mappings[0].1 as i16 - mappings[0].0 as i16;
    let can_use_delta = mappings.len() == (max_char as usize - min_char as usize + 1)
        && mappings
            .iter()
            .all(|&(ch, gid)| gid as i16 - ch as i16 == first_delta);

    let (id_deltas, id_range_offsets, glyph_id_array) = if can_use_delta {
        (vec![first_delta, 1], vec![0, 0], Vec::new())
    } else {
        let sc = seg_count as usize; // already includes sentinel

        let offset_to_glyph_array = 14u32
            + (sc * 2) as u32 // endCode
            + 2u32 // reservedPad
            + (sc * 2) as u32 // startCode
            + (sc * 2) as u32 // idDelta
            + (sc * 2) as u32; // idRangeOffset

        let mut gids: Vec<u16> = Vec::new();
        let mut mapping_idx = 0usize;
        for ch in min_char..=max_char {
            if mapping_idx < mappings.len() && mappings[mapping_idx].0 as u16 == ch {
                gids.push(mappings[mapping_idx].1);
                mapping_idx += 1;
            } else {
                gids.push(0);
            }
        }

        let id_range_offset_addr = 14u32
            + (sc * 2) as u32 // endCode
            + 2u32 // reservedPad
            + (sc * 2) as u32 // startCode
            + (sc * 2) as u32; // idDelta

        let offset_val = (offset_to_glyph_array - id_range_offset_addr) as u16;
        (vec![0, 1], vec![offset_val, 0], gids)
    };

    let glyph_array_bytes: usize = glyph_id_array.len() * 2;
    let subtable_length = 14u32
        + (seg_count * 2) as u32 // endCode
        + 2u32 // reservedPad
        + (seg_count * 2) as u32 // startCode
        + (seg_count * 2) as u32 // idDelta
        + (seg_count * 2) as u32 // idRangeOffset
        + glyph_array_bytes as u32;

    let mut out = Vec::with_capacity(12 + subtable_length as usize);

    // cmap header: version=0, numTables=1
    out.extend_from_slice(&0u16.to_be_bytes());
    out.extend_from_slice(&1u16.to_be_bytes());
    // Encoding record: platform 3 (Windows), encoding 1 (Unicode BMP), offset=12
    out.extend_from_slice(&3u16.to_be_bytes());
    out.extend_from_slice(&1u16.to_be_bytes());
    out.extend_from_slice(&12u32.to_be_bytes());

    // Format 4 subtable
    out.extend_from_slice(&4u16.to_be_bytes()); // format
    out.extend_from_slice(&(subtable_length as u16).to_be_bytes()); // length
    out.extend_from_slice(&0u16.to_be_bytes()); // language
    out.extend_from_slice(&seg_count_x2.to_be_bytes());
    out.extend_from_slice(&search_range.to_be_bytes());
    out.extend_from_slice(&entry_selector.to_be_bytes());
    out.extend_from_slice(&range_shift.to_be_bytes());

    // endCode
    for &ec in &end_codes {
        out.extend_from_slice(&ec.to_be_bytes());
    }
    // reservedPad
    out.extend_from_slice(&0u16.to_be_bytes());
    // startCode
    for &sc in &start_codes {
        out.extend_from_slice(&sc.to_be_bytes());
    }
    // idDelta
    for &d in &id_deltas {
        out.extend_from_slice(&d.to_be_bytes());
    }
    // idRangeOffset
    for &r in &id_range_offsets {
        out.extend_from_slice(&r.to_be_bytes());
    }
    // glyphIdArray
    for &g in &glyph_id_array {
        out.extend_from_slice(&g.to_be_bytes());
    }

    out
}

fn highest_power_of_2_u16(v: u16) -> u16 {
    if v == 0 {
        return 0;
    }
    1 << (15 - v.leading_zeros())
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

        let result = subset_font(&data, &glyphs);
        let subsetted = result.font_data;

        assert!(
            subsetted.len() < data.len(),
            "subset {} should be smaller than original {}",
            subsetted.len(),
            data.len()
        );

        let parse_result = ttf_parser::Face::parse(&subsetted, 0);
        assert!(
            parse_result.is_ok(),
            "subset font should be valid: {:?}",
            parse_result.err()
        );

        let subset_face = parse_result.unwrap();

        // Codepoints should still map to valid glyph IDs (remapped)
        for ch in "Hello World".chars() {
            let original_gid = face.glyph_index(ch).unwrap();
            let subset_gid = subset_face.glyph_index(ch).unwrap();
            // The subset glyph ID should be the remapped one
            let expected_new = result
                .glyph_id_map
                .get(&(original_gid.0 as u32))
                .copied()
                .unwrap_or(original_gid.0 as u32);
            assert_eq!(
                subset_gid.0 as u32, expected_new,
                "glyph mapping for '{ch}' should be remapped correctly"
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

        let result = subset_font(&data, &glyphs);
        let subsetted = result.font_data;

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

    #[test]
    fn test_subset_compound_glyphs_included() {
        let Some(data) = get_font_data() else { return };

        let face = ttf_parser::Face::parse(&data, 0).unwrap();
        let raw = face.raw_face();

        // Find a compound glyph: numberOfContours < 0 in glyf table
        let head_data = raw.table(Tag::from_bytes(b"head")).unwrap();
        let loca_format = u16::from_be_bytes([head_data[50], head_data[51]]);
        let loca_table = raw.table(Tag::from_bytes(b"loca")).unwrap();
        let glyf_table = raw.table(Tag::from_bytes(b"glyf")).unwrap();

        let num_glyphs = face.number_of_glyphs();
        let mut compound_gid: Option<u16> = None;
        let mut component_gids: Vec<u16> = Vec::new();

        for gid in 1..num_glyphs {
            if let Some((start, len)) = get_glyph_range(loca_table, glyf_table, gid, loca_format) {
                if len >= 10 {
                    let gd = &glyf_table[start..start + len];
                    let nc = i16::from_be_bytes([gd[0], gd[1]]);
                    if nc < 0 {
                        compound_gid = Some(gid);
                        // Collect first-level component glyph IDs
                        let mut off = 10usize;
                        loop {
                            if off + 4 > gd.len() {
                                break;
                            }
                            let flags = u16::from_be_bytes([gd[off], gd[off + 1]]);
                            let cg = u16::from_be_bytes([gd[off + 2], gd[off + 3]]);
                            off += 4;
                            if flags & 0x0001 != 0 {
                                off += 4;
                            } else {
                                off += 2;
                            }
                            if flags & 0x0080 != 0 {
                                off += 8;
                            } else if flags & 0x0040 != 0 {
                                off += 4;
                            } else if flags & 0x0008 != 0 {
                                off += 2;
                            }
                            component_gids.push(cg);
                            if flags & 0x0020 == 0 {
                                break;
                            }
                        }
                        break;
                    }
                }
            }
        }

        let Some(compound) = compound_gid else {
            return; // No compound glyphs in this font, skip test
        };

        // Subset with only the compound glyph and .notdef
        let mut glyphs = HashSet::new();
        glyphs.insert(0);
        glyphs.insert(compound as u32);

        let result = subset_font(&data, &glyphs);
        let sub_face = ttf_parser::Face::parse(&result.font_data, 0).unwrap();

        // Verify compound glyph data is present (glyph 0 = .notdef, glyph 1 = compound)
        // With remapping, the compound is at new ID = result.glyph_id_map[compound]
        let compound_new_id = result.glyph_id_map[&(compound as u32)] as u16;
        assert!(compound_new_id > 0, "compound glyph should not be .notdef");

        // Verify numGlyphs equals the number of unique used glyphs
        assert_eq!(
            sub_face.number_of_glyphs(),
            result.glyph_id_map.len() as u16,
            "numGlyphs should equal used glyph count"
        );
    }

    #[test]
    fn test_subset_compact_cjk() {
        let Some(data) = get_font_data() else { return };

        let face = ttf_parser::Face::parse(&data, 0).unwrap();

        // Simulate CJK-style sparsity: pick glyphs with large IDs spread far apart
        let num_glyphs_total = face.number_of_glyphs();
        let mut glyphs = HashSet::new();
        glyphs.insert(0); // .notdef
        let glyph_count = 50usize;
        if num_glyphs_total > 200 {
            // Pick sparse glyph IDs to simulate CJK fonts
            for i in 0..glyph_count {
                let gid = (i as u32 * (num_glyphs_total as u32 / glyph_count as u32))
                    .min(num_glyphs_total as u32 - 1);
                glyphs.insert(gid);
            }
        } else {
            // Not enough glyphs for sparse test, pick whatever we have
            for ch in "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars() {
                if let Some(gid) = face.glyph_index(ch) {
                    glyphs.insert(gid.0 as u32);
                }
            }
        }

        let result = subset_font(&data, &glyphs);
        let subsetted = result.font_data;

        // Subset should have numGlyphs == total used glyph count (including .notdef and resolved compounds)
        let sub_face = ttf_parser::Face::parse(&subsetted, 0).unwrap();
        assert_eq!(
            sub_face.number_of_glyphs() as usize,
            result.glyph_id_map.len(),
            "numGlyphs should match total used glyph count"
        );

        // Subset size should be proportional to used glyphs, not max_gid
        let max_old_gid = *glyphs.iter().max().unwrap_or(&0) as usize;
        let ratio = subsetted.len() as f64 / data.len() as f64;
        // With remapping, even if max_old_gid is large, subset should be small
        if max_old_gid > 100 {
            assert!(
                ratio < 0.5,
                "sparse subset should be much smaller: original={}, subset={}, ratio={:.2}",
                data.len(),
                subsetted.len(),
                ratio
            );
        }

        // CIDToGIDMap should be present when remapping occurred
        if max_old_gid >= glyphs.len() {
            assert!(
                result.cid_to_gid_map.is_some(),
                "CIDToGIDMap should be present when glyph IDs are remapped"
            );
            let map = result.cid_to_gid_map.as_ref().unwrap();
            // Format byte
            assert_eq!(map[0], 1, "CIDToGIDMap format should be 1 (word-aligned)");
            // Map length: 1 format byte + numGlyphs * 2 bytes per entry
            assert_eq!(
                map.len(),
                1 + (sub_face.number_of_glyphs() as usize) * 2,
                "CIDToGIDMap length should be correct"
            );
        }
    }
}
