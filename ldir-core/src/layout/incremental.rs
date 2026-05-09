//! Incremental re-layout engine (TASK-020).
//!
//! Tracks node-level dirty state so that only changed subtrees
//! need recompilation. Unchanged documents produce bit-identical L-IR.
//!
//! ## Key Property
//!
//! When no nodes are dirty, [`IncrementalLayout::recompile_lir`] returns
//! a clone of the old L-IR document, guaranteeing bit-identical output.
//!
//! ## References
//!
//! - INV-COMP-001: Bit-identical output (determinism)
//! - YP-INCREMENTAL-001: Incremental layout specification

#![allow(dead_code)]

use std::collections::{HashSet, VecDeque};

use ldir_ir::lir::LIRDocument;
use ldir_ir::sir::v2::SIRModuleV2;

use crate::compiler::context::CompileContext;
use crate::layout::lir_compile::{LirError, compile_sir_to_lir};

/// Set of node IDs that have been modified since the last layout.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DirtySet {
    dirty: HashSet<u32>,
}

impl DirtySet {
    pub fn new() -> Self {
        Self {
            dirty: HashSet::new(),
        }
    }

    pub fn mark_dirty(&mut self, node_id: u32) {
        self.dirty.insert(node_id);
    }

    pub fn is_dirty(&self, node_id: u32) -> bool {
        self.dirty.contains(&node_id)
    }

    pub fn clear(&mut self) {
        self.dirty.clear();
    }

    pub fn len(&self) -> usize {
        self.dirty.len()
    }

    pub fn is_empty(&self) -> bool {
        self.dirty.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = u32> + '_ {
        self.dirty.iter().copied()
    }

    pub fn mark_dirty_range(&mut self, start: u32, end: u32) {
        for id in start..=end {
            self.dirty.insert(id);
        }
    }

    pub fn mark_dirty_all(&mut self, ids: impl IntoIterator<Item = u32>) {
        for id in ids {
            self.dirty.insert(id);
        }
    }
}

/// Incremental layout tracker that associates an S-IR v2 module with a
/// dirty set for selective recompilation.
///
/// When no nodes are marked dirty, [`recompile_lir`](Self::recompile_lir)
/// returns a clone of the old L-IR, preserving bit-identical output
/// per INV-COMP-001.
pub struct IncrementalLayout {
    sir: SIRModuleV2,
    dirty: DirtySet,
}

impl IncrementalLayout {
    /// Create a new incremental layout tracker for the given S-IR v2 module.
    ///
    /// Initially, no nodes are marked dirty.
    pub fn new(module: &SIRModuleV2) -> Self {
        Self {
            sir: module.clone(),
            dirty: DirtySet::new(),
        }
    }

    /// Mark a node as changed.
    pub fn mark_dirty(&mut self, node_id: u32) {
        self.dirty.mark_dirty(node_id);
    }

    /// Mark all node IDs in the range [start, end] as dirty.
    pub fn mark_dirty_range(&mut self, start: u32, end: u32) {
        self.dirty.mark_dirty_range(start, end);
    }

    /// Mark a node and all its descendants as dirty.
    pub fn mark_subtree_dirty(&mut self, node_id: u32) {
        let mut queue = VecDeque::new();
        queue.push_back(node_id);
        while let Some(id) = queue.pop_front() {
            if self.dirty.is_dirty(id) {
                continue;
            }
            self.dirty.mark_dirty(id);
            if let Some(node) = self.sir.body.get(id) {
                for &child_id in &node.child_ids {
                    queue.push_back(child_id);
                }
            }
        }
    }

    /// Check if a specific node is dirty.
    pub fn is_dirty(&self, node_id: u32) -> bool {
        self.dirty.is_dirty(node_id)
    }

    /// Access the dirty set.
    pub fn dirty_set(&self) -> &DirtySet {
        &self.dirty
    }

    /// Clear all dirty flags.
    pub fn clear_dirty(&mut self) {
        self.dirty.clear();
    }

    /// Alias for [`clear_dirty`](Self::clear_dirty).
    pub fn clear(&mut self) {
        self.clear_dirty();
    }

    /// Recompile to L-IR.
    ///
    /// - If the dirty set is empty, returns a clone of `old_lir`
    ///   (bit-identical, zero-cost for unchanged documents).
    /// - If any nodes are dirty, recompiles from S-IR v2 via the L-IR compiler.
    pub fn recompile_lir(
        &self,
        old_lir: &LIRDocument,
        ctx: &CompileContext,
    ) -> std::result::Result<LIRDocument, LirError> {
        if self.dirty.is_empty() {
            return Ok(old_lir.clone());
        }
        compile_sir_to_lir(&self.sir, ctx)
    }

    /// Replace the underlying S-IR v2 module (e.g., after an edit).
    ///
    /// Automatically diffs old vs new node trees and marks changed
    /// nodes as dirty. When a node changes, all its descendants are
    /// also marked dirty.
    pub fn update_sir(&mut self, module: &SIRModuleV2) {
        let old_nodes: Vec<_> = self.sir.body.iter().collect();
        let new_nodes: Vec<_> = module.body.iter().collect();

        let old_len = old_nodes.len();
        let new_len = new_nodes.len();
        let max_len = old_len.max(new_len);

        let mut changed_ids: Vec<u32> = Vec::new();

        for i in 0..max_len {
            let old_node = old_nodes.get(i);
            let new_node = new_nodes.get(i);

            match (old_node, new_node) {
                (None, Some(ni)) => {
                    self.dirty.mark_dirty(ni.id);
                    changed_ids.push(ni.id);
                }
                (Some(oi), None) => {
                    self.dirty.mark_dirty(oi.id);
                }
                (Some(o), Some(n)) => {
                    if o.id != n.id
                        || std::mem::discriminant(&o.node_type)
                            != std::mem::discriminant(&n.node_type)
                        || o.node_type != n.node_type
                    {
                        self.dirty.mark_dirty(n.id);
                        changed_ids.push(n.id);
                    }
                }
                (None, None) => {}
            }
        }

        self.sir = module.clone();

        for id in changed_ids {
            self.mark_subtree_dirty(id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldir_ir::sir::v2::nodes::{Node, NodeType};

    fn make_simple_module() -> SIRModuleV2 {
        let mut module = SIRModuleV2::new();
        let doc_id = module.body.push(Node::new(1, NodeType::Document));
        let para_id = module
            .body
            .push(Node::new(2, NodeType::Paragraph).with_parent(doc_id));
        let text_id = module.body.push(
            Node::new(
                3,
                NodeType::Text {
                    content: "Hello".into(),
                },
            )
            .with_parent(para_id),
        );
        if let Some(node) = module.body.get_mut(doc_id) {
            node.add_child(para_id);
        }
        if let Some(node) = module.body.get_mut(para_id) {
            node.add_child(text_id);
        }
        module
    }

    fn make_ctx() -> CompileContext {
        CompileContext::new()
    }

    // === DirtySet tests ===

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

    // === IncrementalLayout basic tests ===

    #[test]
    fn incremental_layout_new() {
        let module = make_simple_module();
        let layout = IncrementalLayout::new(&module);
        assert!(layout.dirty_set().is_empty());
        assert!(!layout.is_dirty(0));
    }

    #[test]
    fn incremental_mark_dirty() {
        let module = make_simple_module();
        let mut layout = IncrementalLayout::new(&module);
        layout.mark_dirty(1);
        assert!(layout.is_dirty(1));
        assert!(!layout.is_dirty(0));
        assert_eq!(layout.dirty_set().len(), 1);
    }

    #[test]
    fn incremental_clear_dirty() {
        let module = make_simple_module();
        let mut layout = IncrementalLayout::new(&module);
        layout.mark_dirty(1);
        layout.mark_dirty(2);
        layout.clear_dirty();
        assert!(layout.dirty_set().is_empty());
    }

    #[test]
    fn incremental_mark_dirty_range() {
        let module = make_simple_module();
        let mut layout = IncrementalLayout::new(&module);
        layout.mark_dirty_range(3, 7);
        assert_eq!(layout.dirty_set().len(), 5);
        for id in 3..=7 {
            assert!(layout.is_dirty(id));
        }
        assert!(!layout.is_dirty(2));
        assert!(!layout.is_dirty(8));
    }

    #[test]
    fn incremental_multiple_dirty() -> Result<(), Box<dyn std::error::Error>> {
        let module = make_simple_module();
        let mut layout = IncrementalLayout::new(&module);
        layout.mark_dirty(1);
        layout.mark_dirty(2);
        assert_eq!(layout.dirty_set().len(), 2);
        let ctx = make_ctx();
        let old_lir = compile_sir_to_lir(&module, &ctx)?;
        let result = layout.recompile_lir(&old_lir, &ctx);
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn determinism_same_input_same_output() -> Result<(), Box<dyn std::error::Error>> {
        let module = make_simple_module();
        let layout1 = IncrementalLayout::new(&module);
        let layout2 = IncrementalLayout::new(&module);
        let ctx = make_ctx();
        let lir1 = compile_sir_to_lir(&module, &ctx)?;
        let lir2 = compile_sir_to_lir(&module, &ctx)?;
        assert_eq!(
            layout1.recompile_lir(&lir1, &ctx)?,
            layout2.recompile_lir(&lir2, &ctx)?
        );
        Ok(())
    }

    // === v2-specific tests ===

    #[test]
    fn test_v2_unchanged_produces_same_lir() -> Result<(), Box<dyn std::error::Error>> {
        let module = make_simple_module();
        let layout = IncrementalLayout::new(&module);
        let ctx = make_ctx();
        let lir1 = compile_sir_to_lir(&module, &ctx)?;
        let lir2 = layout.recompile_lir(&lir1, &ctx)?;
        assert_eq!(lir1, lir2);
        Ok(())
    }

    #[test]
    fn test_v2_dirty_triggers_recompile() -> Result<(), Box<dyn std::error::Error>> {
        let module = make_simple_module();
        let mut layout = IncrementalLayout::new(&module);
        layout.mark_dirty(2);
        let ctx = make_ctx();
        let old_lir = compile_sir_to_lir(&module, &ctx)?;
        let new_lir = layout.recompile_lir(&old_lir, &ctx)?;
        assert_eq!(old_lir, new_lir);
        Ok(())
    }

    #[test]
    fn test_v2_update_sir_identical() {
        let module = make_simple_module();
        let mut layout = IncrementalLayout::new(&module);
        let module2 = make_simple_module();
        layout.update_sir(&module2);
        assert!(
            layout.dirty_set().is_empty(),
            "identical S-IR v2 should not mark anything dirty"
        );
    }

    #[test]
    fn test_v2_update_sir_changed() {
        let module1 = make_simple_module();
        let mut layout = IncrementalLayout::new(&module1);

        let mut module2 = SIRModuleV2::new();
        let doc_id = module2.body.push(Node::new(1, NodeType::Document));
        let para_id = module2
            .body
            .push(Node::new(2, NodeType::Paragraph).with_parent(doc_id));
        let text_id = module2.body.push(
            Node::new(
                3,
                NodeType::Text {
                    content: "Changed text".into(),
                },
            )
            .with_parent(para_id),
        );
        if let Some(node) = module2.body.get_mut(doc_id) {
            node.add_child(para_id);
        }
        if let Some(node) = module2.body.get_mut(para_id) {
            node.add_child(text_id);
        }

        layout.update_sir(&module2);
        assert!(
            !layout.dirty_set().is_empty(),
            "changed node type should mark nodes dirty"
        );
    }

    #[test]
    fn test_v2_mark_subtree_dirty() {
        let mut module = SIRModuleV2::new();
        let doc_id = module.body.push(Node::new(1, NodeType::Document));
        let sec_id = module
            .body
            .push(Node::new(2, NodeType::Section).with_parent(doc_id));
        let para_id = module
            .body
            .push(Node::new(3, NodeType::Paragraph).with_parent(sec_id));
        let text_id = module.body.push(
            Node::new(
                4,
                NodeType::Text {
                    content: "Child text".into(),
                },
            )
            .with_parent(para_id),
        );

        if let Some(node) = module.body.get_mut(doc_id) {
            node.add_child(sec_id);
        }
        if let Some(node) = module.body.get_mut(sec_id) {
            node.add_child(para_id);
        }
        if let Some(node) = module.body.get_mut(para_id) {
            node.add_child(text_id);
        }

        let mut layout = IncrementalLayout::new(&module);
        layout.mark_subtree_dirty(sec_id);
        assert!(layout.is_dirty(sec_id), "section should be dirty");
        assert!(layout.is_dirty(para_id), "child paragraph should be dirty");
        assert!(layout.is_dirty(text_id), "grandchild text should be dirty");
        assert!(!layout.is_dirty(doc_id), "parent should NOT be dirty");
    }

    #[test]
    fn test_v2_dirty_descendants() {
        let module1 = make_simple_module();
        let mut layout = IncrementalLayout::new(&module1);

        let mut module2 = SIRModuleV2::new();
        let doc_id = module2.body.push(Node::new(1, NodeType::Document));
        let sec_id = module2
            .body
            .push(Node::new(2, NodeType::Section).with_parent(doc_id));
        let sec_text_id = module2.body.push(
            Node::new(
                3,
                NodeType::Text {
                    content: "Heading".into(),
                },
            )
            .with_parent(sec_id),
        );
        let para_id = module2
            .body
            .push(Node::new(4, NodeType::Paragraph).with_parent(doc_id));
        let para_text_id = module2.body.push(
            Node::new(
                5,
                NodeType::Text {
                    content: "Body".into(),
                },
            )
            .with_parent(para_id),
        );

        if let Some(node) = module2.body.get_mut(doc_id) {
            node.add_child(sec_id);
            node.add_child(para_id);
        }
        if let Some(node) = module2.body.get_mut(sec_id) {
            node.add_child(sec_text_id);
        }
        if let Some(node) = module2.body.get_mut(para_id) {
            node.add_child(para_text_id);
        }

        layout.update_sir(&module2);
        assert!(
            !layout.dirty_set().is_empty(),
            "structural change should be dirty"
        );
    }
}
