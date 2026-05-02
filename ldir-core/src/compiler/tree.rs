//! Build a parent→children tree from flat S-IR instructions.
//!
//! Converts the flat `SIRDocument` instruction list into an adjacency list
//! representation suitable for DFS traversal during compilation.

#![allow(clippy::unwrap_used)]

use std::collections::{HashMap, HashSet};

use ldir_ir::sir::{EntityId, SIRDocument, SIRInstruction};

/// A tree node representing an S-IR instruction and its children.
#[derive(Debug, Clone)]
pub struct TreeNode<'a> {
    /// The S-IR instruction this node represents.
    pub instruction: &'a SIRInstruction,
    /// Indices of child nodes in the `InstructionTree`.
    pub children: Vec<usize>,
}

/// An adjacency-list tree built from a flat S-IR document.
///
/// Maps instruction index → node with children indices.
/// Ensures acyclic structure per AX-003.
pub struct InstructionTree<'a> {
    nodes: Vec<TreeNode<'a>>,
    index_by_entity: HashMap<EntityId, usize>,
    root_index: Option<usize>,
}

impl<'a> InstructionTree<'a> {
    /// Build a tree from a flat S-IR document.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Multiple root nodes are found (ERR-VALID-005)
    /// - No root node is found (ERR-VALID-006)
    /// - A parent reference doesn't exist (ERR-VALID-002)
    /// - A cycle is detected (ERR-VALID-003)
    pub fn build(doc: &'a SIRDocument) -> Result<Self, crate::error::LdirError> {
        let mut index_by_entity: HashMap<EntityId, usize> = HashMap::new();
        let mut root_index: Option<usize> = None;
        let mut roots_found = 0usize;

        for (idx, instr) in doc.iter().enumerate() {
            if index_by_entity.contains_key(&instr.entity_id()) {
                return Err(crate::error::LdirError::from(
                    crate::error::ValidationErrorKind::DuplicateEntityId {
                        entity_id: instr.entity_id(),
                    },
                )
                .with_entity(instr.entity_id()));
            }
            index_by_entity.insert(instr.entity_id(), idx);

            if instr.is_root() {
                roots_found += 1;
                root_index = Some(idx);
            }
        }

        match roots_found {
            0 => return Err(crate::error::ValidationErrorKind::NoRoot.into()),
            1 => {}
            _ => {
                return Err(crate::error::ValidationErrorKind::MultipleRoots {
                    count: roots_found,
                }
                .into());
            }
        }

        let mut nodes: Vec<TreeNode<'a>> = doc
            .iter()
            .map(|instr| TreeNode {
                instruction: instr,
                children: Vec::new(),
            })
            .collect();

        for (idx, instr) in doc.iter().enumerate() {
            if instr.is_root() {
                continue;
            }
            let parent_id = instr.parent_id();
            if parent_id == instr.entity_id() {
                return Err(crate::error::LdirError::from(
                    crate::error::ValidationErrorKind::CircularParentChain {
                        entity_id: instr.entity_id(),
                    },
                )
                .with_entity(instr.entity_id()));
            }
            let parent_idx = *index_by_entity.get(&parent_id).ok_or_else(|| {
                crate::error::LdirError::from(crate::error::ValidationErrorKind::ParentNotFound {
                    entity_id: instr.entity_id(),
                    parent_id,
                })
                .with_entity(instr.entity_id())
            })?;
            nodes[parent_idx].children.push(idx);
        }

        let root_idx = root_index.unwrap();
        let mut on_path = HashSet::new();
        detect_cycle(&nodes, root_idx, &mut on_path)?;

        let mut reachable = HashSet::new();
        collect_reachable(&nodes, root_idx, &mut reachable);

        if reachable.len() != nodes.len() {
            return Err(
                crate::error::ValidationErrorKind::CircularParentChain { entity_id: 0 }.into(),
            );
        }

        Ok(Self {
            nodes,
            index_by_entity,
            root_index: Some(root_idx),
        })
    }

    /// Get the root node index.
    pub fn root_index(&self) -> usize {
        self.root_index.unwrap()
    }

    /// Get a node by index.
    pub fn node(&self, index: usize) -> &TreeNode<'a> {
        &self.nodes[index]
    }

    /// Get the number of nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the tree is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Look up instruction index by entity ID.
    pub fn index_by_entity(&self, entity_id: EntityId) -> Option<usize> {
        self.index_by_entity.get(&entity_id).copied()
    }
}

fn detect_cycle(
    nodes: &[TreeNode],
    current: usize,
    on_path: &mut HashSet<usize>,
) -> Result<(), crate::error::LdirError> {
    if !on_path.insert(current) {
        return Err(crate::error::LdirError::from(
            crate::error::ValidationErrorKind::CircularParentChain {
                entity_id: nodes[current].instruction.entity_id(),
            },
        )
        .with_entity(nodes[current].instruction.entity_id()));
    }
    for &child in &nodes[current].children {
        detect_cycle(nodes, child, on_path)?;
    }
    on_path.remove(&current);
    Ok(())
}

fn collect_reachable(nodes: &[TreeNode], current: usize, reachable: &mut HashSet<usize>) {
    if reachable.insert(current) {
        for &child in &nodes[current].children {
            collect_reachable(nodes, child, reachable);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldir_ir::sir::{ROOT_SENTINEL, SIRInstruction, SIROpcode};

    fn make_doc() -> SIRDocument {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0));
        doc.push(SIRInstruction::new(SIROpcode::PushBlock, 2, 0, 0));
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 3, 2, 0));
        doc
    }

    #[test]
    fn test_build_tree() {
        let doc = make_doc();
        let tree = InstructionTree::build(&doc).unwrap();
        assert_eq!(tree.len(), 4);
        assert_eq!(tree.root_index(), 0);
        let root = tree.node(0);
        assert_eq!(root.children, vec![1, 2]);
        let child2 = tree.node(2);
        assert_eq!(child2.children, vec![3]);
    }

    #[test]
    fn test_no_root() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 0, 0, 0));
        let result = InstructionTree::build(&doc);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_roots() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            1,
            ROOT_SENTINEL,
            0,
        ));
        let result = InstructionTree::build(&doc);
        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_entity_id() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 0, 0, 0));
        let result = InstructionTree::build(&doc);
        assert!(result.is_err());
    }

    #[test]
    fn test_parent_not_found() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 99, 0));
        let result = InstructionTree::build(&doc);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_doc() {
        let doc = SIRDocument::new();
        let result = InstructionTree::build(&doc);
        assert!(result.is_err());
    }

    #[test]
    fn test_index_by_entity() {
        let doc = make_doc();
        let tree = InstructionTree::build(&doc).unwrap();
        assert_eq!(tree.index_by_entity(0), Some(0));
        assert_eq!(tree.index_by_entity(3), Some(3));
        assert_eq!(tree.index_by_entity(99), None);
    }
}
