//! AX-003 / `isAcyclic`: Check that no circular parent chains exist.
//!
//! Matches Lean 4 predicate:
//! ```lean
//! def isAcyclic (doc : SIRDocument) : Bool :=
//!   doc.all fun instr =>
//!     instr.parentId = rootSentinel ∨
//!     isAcyclicAux doc [instr.entityId] doc.length instr.parentId
//! ```
//!
//! Uses **iterative DFS** with a fuel parameter to avoid stack overflow
//! on deeply nested or malicious documents.

use ldir_ir::sir::{EntityId, ROOT_SENTINEL, SIRDocument};

use crate::error::{LdirError, ValidationErrorKind};

/// Check that no circular parent chains exist in the document.
///
/// Builds a parent map, then for each non-root instruction walks the
/// parent chain iteratively. If an entity ID is revisited, a cycle exists.
///
/// Fuel limit is `doc.len()` (matching Lean4's `isAcyclicAux`).
pub fn check(doc: &SIRDocument) -> Vec<LdirError> {
    let mut parent_map = std::collections::HashMap::new();
    for instr in doc.iter() {
        parent_map.insert(instr.entity_id(), instr.parent_id());
    }

    let fuel = doc.len();
    let mut errors = Vec::new();
    let mut reported_cycles = std::collections::HashSet::new();

    for instr in doc.iter() {
        let pid = instr.parent_id();
        if pid == ROOT_SENTINEL {
            continue;
        }

        if walk_parent_chain(pid, &parent_map, fuel) && reported_cycles.insert(instr.entity_id()) {
            errors.push(
                ValidationErrorKind::CircularParentChain {
                    entity_id: instr.entity_id(),
                }
                .into(),
            );
        }
    }

    errors
}

/// Iterative DFS following parent references.
///
/// Returns `true` if a cycle is detected (entity revisited),
/// `false` if the chain reaches ROOT_SENTINEL or a dead end within fuel.
fn walk_parent_chain(
    start: EntityId,
    parent_map: &std::collections::HashMap<EntityId, EntityId>,
    fuel: usize,
) -> bool {
    let mut visited = std::collections::HashSet::new();
    let mut current = start;
    let mut remaining = fuel;

    loop {
        if remaining == 0 {
            return false;
        }

        if !visited.insert(current) {
            return true;
        }

        match parent_map.get(&current) {
            None => return false,
            Some(&pid) => {
                if pid == ROOT_SENTINEL {
                    return false;
                }
                current = pid;
                remaining -= 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldir_ir::sir::{SIRInstruction, SIROpcode};

    #[test]
    fn test_acyclic_doc_passes() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0));
        doc.push(SIRInstruction::new(SIROpcode::ApplyStyle, 2, 0, 10));
        assert!(check(&doc).is_empty());
    }

    #[test]
    fn test_direct_cycle_detected() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::PushBlock, 1, 2, 0));
        doc.push(SIRInstruction::new(SIROpcode::PushBlock, 2, 1, 0));
        let errors = check(&doc);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_indirect_cycle_detected() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::PushBlock, 1, 2, 0));
        doc.push(SIRInstruction::new(SIROpcode::PushBlock, 2, 3, 0));
        doc.push(SIRInstruction::new(SIROpcode::PushBlock, 3, 1, 0));
        let errors = check(&doc);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_self_loop_detected() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 1, 0));
        let errors = check(&doc);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_empty_doc_passes() {
        let doc = SIRDocument::new();
        assert!(check(&doc).is_empty());
    }

    #[test]
    fn test_deep_chain_no_cycle() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        for i in 1..50 {
            doc.push(SIRInstruction::new(SIROpcode::SetContent, i, i - 1, 0));
        }
        assert!(check(&doc).is_empty());
    }

    #[test]
    fn test_cycle_matches_lean4_cyclic_doc() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::PushBlock, 1, 2, 0));
        doc.push(SIRInstruction::new(SIROpcode::PushBlock, 2, 1, 0));
        let errors = check(&doc);
        assert!(!errors.is_empty());
    }
}
