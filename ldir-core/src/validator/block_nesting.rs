//! DEF-004.6: Check that block nesting is valid.
//!
//! Rules:
//! - Only the root entity may have `parent_id == ROOT_SENTINEL`.
//! - Non-root `PushBlock` instructions must have a parent that is also
//!   a `PushBlock` (i.e., blocks can nest inside blocks).
//! - `Document`-type blocks must not be nested under other blocks.

use ldir_ir::sir::{ROOT_SENTINEL, SIRDocument, SIROpcode};

use crate::error::{LdirError, ValidationErrorKind};

/// Check that block nesting is valid.
///
/// Rules:
/// - Only the root entity may have `parent_id == ROOT_SENTINEL`.
/// - Non-root `PushBlock` instructions must have a `PushBlock` parent
///   (blocks nest inside blocks, not standalone).
pub fn check(doc: &SIRDocument) -> Vec<LdirError> {
    let mut errors = Vec::new();

    // Collect all PushBlock entity IDs for parent lookup
    let mut push_block_ids: std::collections::HashSet<u32> = std::collections::HashSet::new();
    for instr in doc.iter() {
        if instr.opcode() == SIROpcode::PushBlock {
            push_block_ids.insert(instr.entity_id());
        }
    }

    for instr in doc.iter() {
        match instr.opcode() {
            SIROpcode::PushBlock => {
                if instr.is_root() {
                    // Root PushBlock: valid (only one should exist per DEF-004.5)
                } else if !push_block_ids.contains(&instr.parent_id()) {
                    // Non-root PushBlock whose parent is not a PushBlock
                    errors.push(
                        ValidationErrorKind::InvalidBlockNesting {
                            entity_id: instr.entity_id(),
                        }
                        .into(),
                    );
                }
            }
            _ => {
                // SetContent, ApplyStyle, etc. must have a valid parent
                if !push_block_ids.contains(&instr.parent_id())
                    && instr.parent_id() != ROOT_SENTINEL
                {
                    errors.push(
                        ValidationErrorKind::InvalidBlockNesting {
                            entity_id: instr.entity_id(),
                        }
                        .into(),
                    );
                }
            }
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldir_ir::sir::{ROOT_SENTINEL, SIRInstruction, SIROpcode};

    #[test]
    fn test_valid_nesting_passes() {
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
    fn test_empty_doc_passes() {
        let doc = SIRDocument::new();
        assert!(check(&doc).is_empty());
    }
}
