//! DEF-004.5 / `hasSingleRoot`: Check that exactly one root node exists.
//!
//! Matches Lean 4 predicate:
//! ```lean
//! def hasSingleRoot (doc : SIRDocument) : Bool :=
//!   (doc.filter (fun instr => instr.parent_id == rootSentinel)).length == 1
//! ```

use ldir_ir::sir::SIRDocument;

use crate::error::{LdirError, ValidationErrorKind};

/// Check that the document has exactly one root node.
///
/// Returns errors for:
/// - **ERR-VALID-006**: No root node found.
/// - **ERR-VALID-005**: Multiple root nodes found.
pub fn check(doc: &SIRDocument) -> Vec<LdirError> {
    let root_count = doc.roots().count();

    match root_count {
        0 => vec![ValidationErrorKind::NoRoot.into()],
        1 => Vec::new(),
        count => vec![ValidationErrorKind::MultipleRoots { count }.into()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldir_ir::sir::{ROOT_SENTINEL, SIRInstruction, SIROpcode};

    #[test]
    fn test_single_root_passes() {
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
    fn test_no_root_fails() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0));
        let errors = check(&doc);
        assert_eq!(errors.len(), 1);
        match &errors[0].kind {
            crate::error::ErrorKind::Validation(ValidationErrorKind::NoRoot) => {}
            other => panic!("expected NoRoot, got {:?}", other),
        }
    }

    #[test]
    fn test_multiple_roots_fails() {
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
        let errors = check(&doc);
        assert_eq!(errors.len(), 1);
        match &errors[0].kind {
            crate::error::ErrorKind::Validation(ValidationErrorKind::MultipleRoots { count }) => {
                assert_eq!(*count, 2)
            }
            other => panic!("expected MultipleRoots, got {:?}", other),
        }
    }

    #[test]
    fn test_empty_doc_no_root() {
        let doc = SIRDocument::new();
        let errors = check(&doc);
        assert_eq!(errors.len(), 1);
    }
}
