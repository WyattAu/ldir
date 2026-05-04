//! S-IR validator module.
//!
//! Validates S-IR document well-formedness per the Lean 4 predicates in
//! `ProofIRWellformedness.lean`. Each check is a separate function:
//!
//! - [`entity_unique`]: AX-001 / `entityUnique` — all entity IDs distinct
//! - [`parent_exists`]: AX-002 / `parentExists` — parent references valid
//! - [`acyclicity`]: AX-003 / `isAcyclic` — no circular parent chains
//! - [`root_count`]: DEF-004.5 / `hasSingleRoot` — exactly one root
//! - [`block_nesting`]: DEF-004.6 — valid block nesting
//! - [`payload_integrity`]: AX-004 — payload offsets in bounds
//!
//! The main entry point is [`validate_sir`].

pub mod acyclicity;
pub mod block_nesting;
pub mod entity_unique;
pub mod parent_exists;
pub mod payload_integrity;
pub mod root_count;

use ldir_ir::sir::SIRDocument;

use crate::error::{LdirError, ValidationResult};

/// IF-VALIDATE-001: Validate S-IR well-formedness.
///
/// Runs all well-formedness checks and collects **all** errors
/// (does not stop at the first violation).
///
/// # Checks
///
/// 1. Entity uniqueness (AX-001, Lean4 `entityUnique`)
/// 2. Parent references exist (AX-002, Lean4 `parentExists`)
/// 3. No circular parent chains (AX-003, Lean4 `isAcyclic`)
/// 4. Exactly one root (DEF-004.5, Lean4 `hasSingleRoot`)
/// 5. Valid block nesting (DEF-004.6)
/// 6. Payload offsets in bounds (AX-004)
///
/// # Returns
///
/// - `Ok(())` if all checks pass.
/// - `Err(Vec<LdirError>)` with all violations found.
pub fn validate_sir(doc: &SIRDocument) -> ValidationResult {
    let mut errors: Vec<LdirError> = Vec::new();

    errors.extend(entity_unique::check(doc));
    errors.extend(parent_exists::check(doc));
    errors.extend(acyclicity::check(doc));
    errors.extend(root_count::check(doc));
    errors.extend(block_nesting::check(doc));
    errors.extend(payload_integrity::check(doc));

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldir_ir::sir::{ROOT_SENTINEL, SIRInstruction, SIROpcode};

    fn make_wellformed_doc() -> SIRDocument {
        let mut doc = SIRDocument::new();
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::PushBlock, 0, ROOT_SENTINEL, 0),
            &[ldir_ir::sir::BlockType::Document as u8],
        );
        doc.push_with_payload(SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0), b"text");
        doc.push(SIRInstruction::new(SIROpcode::ApplyStyle, 2, 0, 10));
        doc
    }

    #[test]
    fn test_wellformed_doc_passes() {
        let doc = make_wellformed_doc();
        assert!(validate_sir(&doc).is_ok());
    }

    #[test]
    fn test_empty_doc_fails_no_root() {
        let doc = SIRDocument::new();
        assert!(validate_sir(&doc).is_err());
    }

    #[test]
    fn test_duplicate_entity_ids_fails() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 0, 0, 0));
        assert!(validate_sir(&doc).is_err());
    }

    #[test]
    fn test_cyclic_doc_fails() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::PushBlock, 1, 2, 0));
        doc.push(SIRInstruction::new(SIROpcode::PushBlock, 2, 1, 0));
        assert!(validate_sir(&doc).is_err());
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
        assert!(validate_sir(&doc).is_err());
    }

    #[test]
    fn test_missing_parent_fails() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 99, 0));
        assert!(validate_sir(&doc).is_err());
    }

    #[test]
    fn test_collects_all_errors() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        // duplicate entity_id (0) + multiple roots
        let result = validate_sir(&doc);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(
            errors.len() >= 2,
            "expected at least 2 errors, got {}",
            errors.len()
        );
    }

    #[test]
    fn test_large_tree_passes() {
        let mut doc = SIRDocument::new();
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::PushBlock, 0, ROOT_SENTINEL, 0),
            &[ldir_ir::sir::BlockType::Document as u8],
        );
        for i in 1..100 {
            doc.push_with_payload(
                SIRInstruction::new(SIROpcode::SetContent, i, 0, 0),
                format!("t{}", i).as_bytes(),
            );
        }
        assert!(validate_sir(&doc).is_ok());
    }
}
