//! Source-to-SIR entity mapping (REQ-5.3.1, REQ-5.3.2).
//!
//! Bidirectional mapping between source byte offsets and `EntityId` values
//! in an S-IR document. Enables LSP features like hover (offset→entity)
//! and diagnostics (entity→source location).
//!
//! # Performance
//!
//! - `entity_at_offset`: O(1) amortized via `Vec` index (main LSP path).
//! - `offset_of_entity` / `source_location`: O(n) scan; hot only during
//!   diagnostics, can be upgraded to a `HashMap` later.

use ldir_ir::sir::{EntityId, INSTRUCTION_WIRE_SIZE, SIRDocument};

/// Source location tied to an entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLocation {
    /// 1-based source line number.
    pub line: u32,
    /// 0-based source column (byte offset within the line).
    pub col: u32,
    /// Absolute byte offset in the source file.
    pub byte_offset: u32,
}

/// Bidirectional mapping between source byte offsets and S-IR entity IDs.
///
/// Forward direction (offset → entity) uses a dense `Vec<Option<EntityId>>`
/// indexed directly by byte offset for O(1) lookup. Reverse direction
/// (entity → offset) uses a linear scan over a flat entry list.
///
/// # Examples
///
/// ```
/// use ldir_core::source_map::SourceMap;
///
/// let mut map = SourceMap::new();
/// map.insert(1, 10, 1, 0);
/// map.insert(2, 20, 2, 4);
///
/// assert_eq!(map.entity_at_offset(10), Some(1));
/// assert_eq!(map.offset_of_entity(2), Some((20, 2, 4)));
/// ```
#[derive(Debug, Clone)]
pub struct SourceMap {
    /// Dense Vec indexed by byte_offset; each slot holds the entity at that
    /// offset, or `None` if no entity starts there.
    offset_to_entity: Vec<Option<EntityId>>,

    /// Flat list of `(entity_id, byte_offset, line, col)`. Insertion order is
    /// preserved for deterministic iteration. Used for reverse lookups.
    entity_entries: Vec<(EntityId, u32, u32, u32)>,
}

impl SourceMap {
    /// Create an empty source map.
    #[inline]
    pub fn new() -> Self {
        Self {
            offset_to_entity: Vec::new(),
            entity_entries: Vec::new(),
        }
    }

    /// Create a source map with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            offset_to_entity: Vec::with_capacity(capacity),
            entity_entries: Vec::with_capacity(capacity),
        }
    }

    /// Record a mapping from `entity_id` to the given source position.
    ///
    /// Multiple calls with the same `byte_offset` will overwrite the previous
    /// entity at that offset. Multiple calls with the same `entity_id` will
    /// each add a new entry (the reverse lookup returns the first match).
    pub fn insert(
        &mut self,
        entity_id: EntityId,
        byte_offset: u32,
        source_line: u32,
        source_col: u32,
    ) {
        let idx = byte_offset as usize;
        if idx >= self.offset_to_entity.len() {
            self.offset_to_entity.resize(idx + 1, None);
        }
        self.offset_to_entity[idx] = Some(entity_id);
        self.entity_entries
            .push((entity_id, byte_offset, source_line, source_col));
    }

    /// Look up the entity that starts at exactly `offset`.
    ///
    /// O(1) amortized via direct `Vec` index — this is the main LSP path
    /// for cursor-hover resolution (REQ-5.3.1).
    #[inline]
    pub fn entity_at_offset(&self, offset: u32) -> Option<EntityId> {
        self.offset_to_entity
            .get(offset as usize)
            .copied()
            .flatten()
    }

    /// Find the byte offset, line, and column for `entity_id`.
    ///
    /// Returns the first entry whose entity matches. O(n) scan — acceptable
    /// for diagnostic reporting; can be upgraded to a `HashMap` later.
    pub fn offset_of_entity(&self, entity_id: EntityId) -> Option<(u32, u32, u32)> {
        self.entity_entries
            .iter()
            .find(|(eid, _, _, _)| *eid == entity_id)
            .map(|(_, off, line, col)| (*off, *line, *col))
    }

    /// Find the [`SourceLocation`] for `entity_id`.
    ///
    /// Convenience wrapper around [`Self::offset_of_entity`].
    pub fn source_location(&self, entity_id: EntityId) -> Option<SourceLocation> {
        self.offset_of_entity(entity_id)
            .map(|(byte_offset, line, col)| SourceLocation {
                line,
                col,
                byte_offset,
            })
    }

    /// Iterate over all `(EntityId, byte_offset, line, col)` entries in
    /// insertion order. Guaranteed deterministic for a given build sequence.
    pub fn iter(&self) -> impl Iterator<Item = &(EntityId, u32, u32, u32)> {
        self.entity_entries.iter()
    }

    /// Number of mapped entities.
    #[inline]
    pub fn len(&self) -> usize {
        self.entity_entries.len()
    }

    /// Check if the source map is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entity_entries.is_empty()
    }

    /// Auto-build a source map from a [`SIRDocument`] instruction stream.
    ///
    /// Each instruction is assigned a byte offset starting at `base_offset`,
    /// spaced by [`INSTRUCTION_WIRE_SIZE`] (13 bytes per REQ-3.1.2).
    /// Line and column are set to 0 since wire-format byte offsets do not
    /// carry original source coordinates — use [`Self::insert`] from a
    /// frontend parser to record real source locations.
    pub fn build_from_document(doc: &SIRDocument, base_offset: u32) -> Self {
        let mut map = Self::with_capacity(doc.len());
        let wire = INSTRUCTION_WIRE_SIZE as u32;
        for (i, instr) in doc.iter().enumerate() {
            let offset = base_offset
                .checked_add((i as u32).checked_mul(wire).unwrap())
                .unwrap();
            map.insert(instr.entity_id(), offset, 0, 0);
        }
        map
    }
}

impl Default for SourceMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldir_ir::sir::{ROOT_SENTINEL, SIRInstruction, SIROpcode};

    fn sample_document() -> SIRDocument {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0));
        doc.push(SIRInstruction::new(SIROpcode::ApplyStyle, 2, 0, 10));
        doc
    }

    // -- build_from_document --

    #[test]
    fn test_build_from_document_basic() {
        let doc = sample_document();
        let map = SourceMap::build_from_document(&doc, 0);

        assert_eq!(map.len(), 3);
        assert_eq!(map.entity_at_offset(0), Some(0));
        assert_eq!(map.entity_at_offset(13), Some(1));
        assert_eq!(map.entity_at_offset(26), Some(2));
    }

    #[test]
    fn test_build_from_document_with_base_offset() {
        let doc = sample_document();
        let map = SourceMap::build_from_document(&doc, 100);

        assert_eq!(map.entity_at_offset(100), Some(0));
        assert_eq!(map.entity_at_offset(113), Some(1));
        assert_eq!(map.entity_at_offset(126), Some(2));
    }

    #[test]
    fn test_build_from_empty_document() {
        let doc = SIRDocument::new();
        let map = SourceMap::build_from_document(&doc, 0);

        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    // -- entity_at_offset --

    #[test]
    fn test_entity_at_offset_exact_match() {
        let mut map = SourceMap::new();
        map.insert(42, 100, 1, 0);
        assert_eq!(map.entity_at_offset(100), Some(42));
    }

    #[test]
    fn test_entity_at_offset_no_match() {
        let mut map = SourceMap::new();
        map.insert(42, 100, 1, 0);
        assert_eq!(map.entity_at_offset(99), None);
        assert_eq!(map.entity_at_offset(101), None);
    }

    #[test]
    fn test_entity_at_offset_empty_map() {
        let map = SourceMap::new();
        assert_eq!(map.entity_at_offset(0), None);
    }

    // -- offset_of_entity --

    #[test]
    fn test_offset_of_entity_found() {
        let mut map = SourceMap::new();
        map.insert(7, 50, 3, 12);
        assert_eq!(map.offset_of_entity(7), Some((50, 3, 12)));
    }

    #[test]
    fn test_offset_of_entity_missing() {
        let map = SourceMap::new();
        assert_eq!(map.offset_of_entity(999), None);
    }

    #[test]
    fn test_offset_of_entity_after_build() {
        let doc = sample_document();
        let map = SourceMap::build_from_document(&doc, 0);
        assert_eq!(map.offset_of_entity(0), Some((0, 0, 0)));
        assert_eq!(map.offset_of_entity(1), Some((13, 0, 0)));
        assert_eq!(map.offset_of_entity(2), Some((26, 0, 0)));
        assert_eq!(map.offset_of_entity(99), None);
    }

    // -- source_location --

    #[test]
    fn test_source_location_found() {
        let mut map = SourceMap::new();
        map.insert(10, 200, 5, 8);
        let loc = map.source_location(10).unwrap();
        assert_eq!(loc.line, 5);
        assert_eq!(loc.col, 8);
        assert_eq!(loc.byte_offset, 200);
    }

    #[test]
    fn test_source_location_missing() {
        let map = SourceMap::new();
        assert!(map.source_location(1).is_none());
    }

    // -- Deterministic iteration order --

    #[test]
    fn test_iteration_order_deterministic() {
        let doc = sample_document();
        let map = SourceMap::build_from_document(&doc, 0);

        let entries: Vec<_> = map.iter().copied().collect();
        assert_eq!(entries, vec![(0, 0, 0, 0), (1, 13, 0, 0), (2, 26, 0, 0)]);
    }

    #[test]
    fn test_iteration_manual_inserts() {
        let mut map = SourceMap::new();
        map.insert(10, 5, 1, 0);
        map.insert(20, 15, 2, 4);
        map.insert(30, 25, 3, 8);

        let ids: Vec<_> = map.iter().map(|e| e.0).collect();
        assert_eq!(ids, vec![10, 20, 30]);
    }

    // -- overwrite semantics --

    #[test]
    fn test_insert_overwrites_offset() {
        let mut map = SourceMap::new();
        map.insert(1, 10, 1, 0);
        map.insert(2, 10, 2, 5);
        assert_eq!(map.entity_at_offset(10), Some(2));
    }

    // -- default --

    #[test]
    fn test_default() {
        let map = SourceMap::default();
        assert!(map.is_empty());
    }

    // -- with_capacity --

    #[test]
    fn test_with_capacity() {
        let map = SourceMap::with_capacity(10);
        assert!(map.is_empty());
    }
}
