//! AX-001 / `entityUnique`: Check that all entity IDs are distinct.
//!
//! Matches Lean 4 predicate:
//! ```lean
//! def entityUnique (doc : SIRDocument) : Bool :=
//!   decide (List.Nodup (doc.map SIRInstruction.entity_id))
//! ```
//!
//! Implementation uses `HashSet` for O(n) average-case detection.

use ldir_ir::sir::{EntityId, SIRDocument};

use crate::error::{LdirError, ValidationErrorKind};

/// Check that all entity IDs in the document are unique.
///
/// Returns an error for each duplicate entity ID found.
pub fn check(doc: &SIRDocument) -> Vec<LdirError> {
    let mut seen = std::collections::HashSet::<EntityId>::new();
    let mut errors = Vec::new();

    for instr in doc.iter() {
        let eid = instr.entity_id();
        if !seen.insert(eid) {
            errors.push(ValidationErrorKind::DuplicateEntityId { entity_id: eid }.into());
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldir_ir::sir::{ROOT_SENTINEL, SIRInstruction, SIROpcode};

    #[test]
    fn test_unique_ids_pass() {
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
    fn test_duplicate_ids_detected() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0));
        doc.push(SIRInstruction::new(SIROpcode::ApplyStyle, 0, 0, 10));
        let errors = check(&doc);
        assert_eq!(errors.len(), 1);
        match &errors[0].kind {
            crate::error::ErrorKind::Validation(ValidationErrorKind::DuplicateEntityId {
                entity_id,
            }) => assert_eq!(*entity_id, 0),
            other => panic!("expected DuplicateEntityId, got {:?}", other),
        }
    }

    #[test]
    fn test_empty_doc_passes() {
        let doc = SIRDocument::new();
        assert!(check(&doc).is_empty());
    }

    #[test]
    fn test_all_same_id_multiple_errors() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            5,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 5, 0, 0));
        doc.push(SIRInstruction::new(SIROpcode::ApplyStyle, 5, 0, 10));
        let errors = check(&doc);
        assert_eq!(errors.len(), 2);
    }
}
