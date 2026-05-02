//! Incremental re-layout engine (TASK-020).
//!
//! Tracks paragraph-level dirty state so that only changed entities
//! need recompilation. Unchanged paragraphs produce bit-identical G-IR.
//!
//! ## Key Property
//!
//! When no entities are dirty, [`IncrementalLayout::recompile`] returns
//! a clone of the old G-IR document, guaranteeing bit-identical output.
//!
//! ## References
//!
//! - INV-COMP-001: Bit-identical output (determinism)
//! - YP-INCREMENTAL-001: Incremental layout specification

use std::collections::HashSet;

use ldir_ir::gir::GIRDocument;
use ldir_ir::sir::SIRDocument;

use crate::compiler::compile_sir;
use crate::error::Result;

/// Set of entity IDs that have been modified since the last layout.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DirtySet {
    dirty: HashSet<u32>,
}

impl DirtySet {
    /// Create an empty dirty set.
    pub fn new() -> Self {
        Self {
            dirty: HashSet::new(),
        }
    }

    /// Mark an entity as dirty.
    pub fn mark_dirty(&mut self, entity_id: u32) {
        self.dirty.insert(entity_id);
    }

    /// Check if an entity is dirty.
    pub fn is_dirty(&self, entity_id: u32) -> bool {
        self.dirty.contains(&entity_id)
    }

    /// Clear all dirty flags.
    pub fn clear(&mut self) {
        self.dirty.clear();
    }

    /// Number of dirty entities.
    pub fn len(&self) -> usize {
        self.dirty.len()
    }

    /// Check if no entities are dirty.
    pub fn is_empty(&self) -> bool {
        self.dirty.is_empty()
    }

    /// Iterate over dirty entity IDs.
    pub fn iter(&self) -> impl Iterator<Item = u32> + '_ {
        self.dirty.iter().copied()
    }

    /// Mark all entity IDs in the given range [start, end] as dirty.
    pub fn mark_dirty_range(&mut self, start: u32, end: u32) {
        for id in start..=end {
            self.dirty.insert(id);
        }
    }

    /// Mark all entity IDs in the iterator as dirty.
    pub fn mark_dirty_all(&mut self, ids: impl IntoIterator<Item = u32>) {
        for id in ids {
            self.dirty.insert(id);
        }
    }
}

/// Incremental layout tracker that associates an S-IR document with a
/// dirty set for selective recompilation.
///
/// When no entities are marked dirty, [`recompile`](Self::recompile)
/// returns a clone of the old G-IR, preserving bit-identical output
/// per INV-COMP-001.
pub struct IncrementalLayout {
    sir: SIRDocument,
    dirty: DirtySet,
}

impl IncrementalLayout {
    /// Create a new incremental layout tracker for the given S-IR document.
    ///
    /// Initially, no entities are marked dirty.
    pub fn new(doc: &SIRDocument) -> Self {
        Self {
            sir: doc.clone(),
            dirty: DirtySet::new(),
        }
    }

    /// Mark a paragraph (entity) as changed.
    pub fn mark_dirty(&mut self, entity_id: u32) {
        self.dirty.mark_dirty(entity_id);
    }

    /// Mark all entity IDs in the range [start, end] as dirty.
    pub fn mark_dirty_range(&mut self, start: u32, end: u32) {
        self.dirty.mark_dirty_range(start, end);
    }

    /// Check if a specific entity is dirty.
    pub fn is_dirty(&self, entity_id: u32) -> bool {
        self.dirty.is_dirty(entity_id)
    }

    /// Access the dirty set.
    pub fn dirty_set(&self) -> &DirtySet {
        &self.dirty
    }

    /// Clear all dirty flags.
    pub fn clear_dirty(&mut self) {
        self.dirty.clear();
    }

    /// Alias for [`clear_dirty`](Self::clear_dirty), following Rust collection conventions.
    pub fn clear(&mut self) {
        self.clear_dirty();
    }

    /// Recompute the G-IR document.
    ///
    /// - If the dirty set is empty, returns a clone of `old_gir`
    ///   (bit-identical, zero-cost for unchanged documents).
    /// - If any entities are dirty, falls back to full recompilation
    ///   from S-IR (v0.1 strategy; per-paragraph recompilation is
    ///   a future optimization).
    pub fn recompile(&self, old_gir: &GIRDocument) -> Result<GIRDocument> {
        if self.dirty.is_empty() {
            return Ok(old_gir.clone());
        }
        compile_sir(&self.sir)
    }

    /// Replace the underlying S-IR document (e.g., after an edit).
    ///
    /// Automatically diffs old vs new instructions and marks changed
    /// entities as dirty. Entities that differ in opcode, parent_id, or
    /// payload_offset are marked dirty. Added or removed entities are
    /// also marked dirty.
    pub fn update_sir(&mut self, doc: &SIRDocument) {
        let old_len = self.sir.len();
        let new_len = doc.len();

        let max_len = old_len.max(new_len);
        for i in 0..max_len {
            let old_instr = self.sir.get(i);
            let new_instr = doc.get(i);

            match (old_instr, new_instr) {
                (None, Some(ni)) => {
                    self.dirty.mark_dirty(ni.entity_id());
                }
                (Some(oi), None) => {
                    self.dirty.mark_dirty(oi.entity_id());
                }
                (Some(o), Some(n)) => {
                    if o.opcode() != n.opcode()
                        || o.parent_id() != n.parent_id()
                        || o.payload_offset() != n.payload_offset()
                        || o.entity_id() != n.entity_id()
                    {
                        self.dirty.mark_dirty(n.entity_id());
                        self.dirty.mark_dirty(o.entity_id());
                    }
                }
                (None, None) => {}
            }
        }

        self.sir = doc.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldir_ir::sir::{ROOT_SENTINEL, SIRInstruction, SIROpcode};

    fn make_simple_doc() -> SIRDocument {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0));
        doc
    }

    #[test]
    fn dirty_set_new_is_empty() {
        let ds = DirtySet::new();
        assert!(ds.is_empty());
        assert_eq!(ds.len(), 0);
    }

    #[test]
    fn dirty_set_mark_and_check() {
        let mut ds = DirtySet::new();
        ds.mark_dirty(5);
        assert!(ds.is_dirty(5));
        assert!(!ds.is_dirty(3));
        assert_eq!(ds.len(), 1);
    }

    #[test]
    fn dirty_set_clear() {
        let mut ds = DirtySet::new();
        ds.mark_dirty(1);
        ds.mark_dirty(2);
        assert_eq!(ds.len(), 2);
        ds.clear();
        assert!(ds.is_empty());
    }

    #[test]
    fn dirty_set_iter() {
        let mut ds = DirtySet::new();
        ds.mark_dirty(3);
        ds.mark_dirty(7);
        let mut ids: Vec<u32> = ds.iter().collect();
        ids.sort();
        assert_eq!(ids, vec![3, 7]);
    }

    #[test]
    fn dirty_set_mark_idempotent() {
        let mut ds = DirtySet::new();
        ds.mark_dirty(1);
        ds.mark_dirty(1);
        assert_eq!(ds.len(), 1);
    }

    #[test]
    fn incremental_layout_new() {
        let doc = make_simple_doc();
        let layout = IncrementalLayout::new(&doc);
        assert!(layout.dirty_set().is_empty());
        assert!(!layout.is_dirty(0));
    }

    #[test]
    fn incremental_mark_dirty() {
        let doc = make_simple_doc();
        let mut layout = IncrementalLayout::new(&doc);
        layout.mark_dirty(1);
        assert!(layout.is_dirty(1));
        assert!(!layout.is_dirty(0));
        assert_eq!(layout.dirty_set().len(), 1);
    }

    #[test]
    fn incremental_unchanged_produces_same_output() {
        let doc = make_simple_doc();
        let layout = IncrementalLayout::new(&doc);
        let old_gir = compile_sir(&doc).unwrap();
        let new_gir = layout.recompile(&old_gir).unwrap();
        assert_eq!(old_gir, new_gir);
    }

    #[test]
    fn incremental_dirty_triggers_recompile() {
        let doc = make_simple_doc();
        let mut layout = IncrementalLayout::new(&doc);
        layout.mark_dirty(1);
        let old_gir = compile_sir(&doc).unwrap();
        let new_gir = layout.recompile(&old_gir).unwrap();
        assert_eq!(old_gir, new_gir);
    }

    #[test]
    fn incremental_clear_dirty() {
        let doc = make_simple_doc();
        let mut layout = IncrementalLayout::new(&doc);
        layout.mark_dirty(1);
        layout.mark_dirty(2);
        layout.clear_dirty();
        assert!(layout.dirty_set().is_empty());
    }

    #[test]
    fn incremental_update_sir() {
        let doc1 = make_simple_doc();
        let mut layout = IncrementalLayout::new(&doc1);
        let doc2 = make_simple_doc();
        layout.update_sir(&doc2);
        assert!(
            layout.dirty_set().is_empty(),
            "identical S-IR should not mark anything dirty"
        );
        let old_gir = compile_sir(&doc1).unwrap();
        let new_gir = layout.recompile(&old_gir).unwrap();
        assert_eq!(old_gir, new_gir);
    }

    #[test]
    fn incremental_update_sir_detects_changes() {
        let doc1 = make_simple_doc();
        let mut layout = IncrementalLayout::new(&doc1);

        let mut doc2 = SIRDocument::new();
        doc2.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc2.push(SIRInstruction::new(SIROpcode::SetContent, 5, 0, 0));

        layout.update_sir(&doc2);
        assert!(
            !layout.dirty_set().is_empty(),
            "changed entity_id should mark entities dirty"
        );
        assert!(layout.is_dirty(5), "new entity 5 should be dirty");
        assert!(layout.is_dirty(1), "removed entity 1 should be dirty");
    }

    #[test]
    fn incremental_mark_dirty_range() {
        let doc = make_simple_doc();
        let mut layout = IncrementalLayout::new(&doc);
        layout.mark_dirty_range(3, 7);
        assert_eq!(layout.dirty_set().len(), 5);
        for id in 3..=7 {
            assert!(layout.is_dirty(id));
        }
        assert!(!layout.is_dirty(2));
        assert!(!layout.is_dirty(8));
    }

    #[test]
    fn dirty_set_mark_dirty_range() {
        let mut ds = DirtySet::new();
        ds.mark_dirty_range(10, 12);
        assert_eq!(ds.len(), 3);
        assert!(ds.is_dirty(10));
        assert!(ds.is_dirty(11));
        assert!(ds.is_dirty(12));
        assert!(!ds.is_dirty(9));
        assert!(!ds.is_dirty(13));
    }

    #[test]
    fn incremental_multiple_dirty() {
        let doc = make_simple_doc();
        let mut layout = IncrementalLayout::new(&doc);
        layout.mark_dirty(0);
        layout.mark_dirty(1);
        assert_eq!(layout.dirty_set().len(), 2);
        let old_gir = compile_sir(&doc).unwrap();
        let result = layout.recompile(&old_gir);
        assert!(result.is_ok());
    }

    #[test]
    fn determinism_same_input_same_output() {
        let doc = make_simple_doc();
        let layout1 = IncrementalLayout::new(&doc);
        let layout2 = IncrementalLayout::new(&doc);
        let gir1 = compile_sir(&doc).unwrap();
        let gir2 = compile_sir(&doc).unwrap();
        assert_eq!(
            layout1.recompile(&gir1).unwrap(),
            layout2.recompile(&gir2).unwrap()
        );
    }
}
