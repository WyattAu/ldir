//! AX-004: Check that payload offsets are within bounds.
//!
//! Per DEF-004: "payload_offset + payload_length <= payload_region.len()"
//!
//! This check validates that every `payload_offset` in the document
//! references a valid position. Since `SIRDocument` does not currently
//! carry a `PayloadRegion`, this check is a placeholder that validates
//! the invariant can be checked when payload data is available.

use ldir_ir::sir::SIRDocument;

use crate::error::LdirError;

/// Check that payload offsets are within bounds.
///
/// Currently a no-op since `SIRDocument` does not carry a `PayloadRegion`.
/// When payload data is attached to documents, this check will validate
/// that all `payload_offset` values reference valid positions.
pub fn check(_doc: &SIRDocument) -> Vec<LdirError> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_doc_passes() {
        let doc = SIRDocument::new();
        assert!(check(&doc).is_empty());
    }

    #[test]
    fn test_nonempty_doc_passes() {
        let mut doc = SIRDocument::new();
        use ldir_ir::sir::{ROOT_SENTINEL, SIRInstruction, SIROpcode};
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 0, 100));
        assert!(check(&doc).is_empty());
    }
}
