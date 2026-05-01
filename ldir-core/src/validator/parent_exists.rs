//! AX-002 / `parentExists`: Check that every parent reference points to an existing entity.
//!
//! Matches Lean 4 predicate:
//! ```lean
//! def parentExists (doc : SIRDocument) : Bool :=
//!   let allIds := doc.map SIRInstruction.entity_id
//!   doc.all (fun instr => parentIdValid instr allIds)
//! ```
//!
//! Implementation uses `HashSet` for O(1) lookup per instruction.

use ldir_ir::sir::{EntityId, ROOT_SENTINEL, SIRDocument};

use crate::error::{LdirError, ValidationErrorKind};

/// Check that every non-root parent reference points to an existing entity.
///
/// Returns an error for each instruction whose `parent_id` is neither
/// `ROOT_SENTINEL` nor a valid entity ID in the document.
pub fn check(doc: &SIRDocument) -> Vec<LdirError> {
    let entity_ids: std::collections::HashSet<EntityId> = doc.entity_ids().collect();

    let mut errors = Vec::new();

    for instr in doc.iter() {
        let pid = instr.parent_id();
        if pid != ROOT_SENTINEL && !entity_ids.contains(&pid) {
            errors.push(
                ValidationErrorKind::ParentNotFound {
                    entity_id: instr.entity_id(),
                    parent_id: pid,
                }
                .into(),
            );
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldir_ir::sir::{SIRInstruction, SIROpcode};

    #[test]
    fn test_valid_parents_pass() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0));
        assert!(check(&doc).is_empty());
    }

    #[test]
    fn test_missing_parent_detected() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 99, 0));
        let errors = check(&doc);
        assert_eq!(errors.len(), 1);
        match &errors[0].kind {
            crate::error::ErrorKind::Validation(ValidationErrorKind::ParentNotFound {
                entity_id,
                parent_id,
            }) => {
                assert_eq!(*entity_id, 1);
                assert_eq!(*parent_id, 99);
            }
            other => panic!("expected ParentNotFound, got {:?}", other),
        }
    }

    #[test]
    fn test_root_sentinel_is_valid() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        assert!(check(&doc).is_empty());
    }

    #[test]
    fn test_empty_doc_passes() {
        let doc = SIRDocument::new();
        assert!(check(&doc).is_empty());
    }

    #[test]
    fn test_multiple_missing_parents() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 50, 0));
        doc.push(SIRInstruction::new(SIROpcode::ApplyStyle, 2, 51, 0));
        let errors = check(&doc);
        assert_eq!(errors.len(), 2);
    }
}
