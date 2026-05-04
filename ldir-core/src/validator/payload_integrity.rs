//! AX-004: Check that payload offsets are within bounds.
//!
//! Per DEF-004: "payload_offset + payload_length <= payload_region.len()"
//!
//! Validates that every `payload_offset` in the document references a valid
//! position in the payload region. Also checks that text content from
//! `SetContent` instructions is valid UTF-8.
//!
//! Note: `ApplyStyle` instructions encode their style data directly in
//! `payload_offset` as a packed value, not as a region offset, so they
//! are excluded from bounds checking.

use ldir_ir::sir::{SIRDocument, SIROpcode};

use crate::error::{LdirError, ValidationErrorKind};

/// Check that payload offsets are within bounds and text content is valid UTF-8.
///
/// For each instruction that references the payload region (PushBlock, SetContent,
/// LinkData, InsertMath), verifies that `payload_offset` points within the region.
/// For `SetContent` instructions, additionally verifies that the payload text is
/// valid UTF-8. `ApplyStyle` instructions are excluded since their `payload_offset`
/// is a packed style value, not a region offset.
pub fn check(doc: &SIRDocument) -> Vec<LdirError> {
    let mut errors = Vec::new();
    let payload_len = doc.payload().len() as u32;

    for instr in doc.iter() {
        let offset = instr.payload_offset();
        match instr.opcode() {
            SIROpcode::ApplyStyle => continue,
            SIROpcode::PushBlock => {
                if offset >= payload_len {
                    errors.push(
                        ValidationErrorKind::PayloadOutOfBounds {
                            entity_id: instr.entity_id(),
                            offset,
                        }
                        .into(),
                    );
                }
            }
            SIROpcode::SetContent => {
                if offset > payload_len {
                    errors.push(
                        ValidationErrorKind::PayloadOutOfBounds {
                            entity_id: instr.entity_id(),
                            offset,
                        }
                        .into(),
                    );
                } else if let Some(bytes) = doc.payload().get_until_nul(offset)
                    && std::str::from_utf8(bytes).is_err()
                {
                    errors.push(
                        ValidationErrorKind::PayloadOutOfBounds {
                            entity_id: instr.entity_id(),
                            offset,
                        }
                        .into(),
                    );
                }
            }
            SIROpcode::LinkData | SIROpcode::InsertMath => {
                if offset >= payload_len {
                    errors.push(
                        ValidationErrorKind::PayloadOutOfBounds {
                            entity_id: instr.entity_id(),
                            offset,
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
    use ldir_ir::sir::{BlockType, ROOT_SENTINEL, SIRInstruction, SIROpcode};

    fn make_valid_doc() -> SIRDocument {
        let mut doc = SIRDocument::new();
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::PushBlock, 0, ROOT_SENTINEL, 0),
            &[BlockType::Document as u8],
        );
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0),
            b"Hello",
        );
        doc.push(SIRInstruction::new(SIROpcode::ApplyStyle, 2, 0, 10));
        doc
    }

    #[test]
    fn test_empty_doc_passes() {
        let doc = SIRDocument::new();
        assert!(check(&doc).is_empty());
    }

    #[test]
    fn test_valid_doc_passes() {
        let doc = make_valid_doc();
        assert!(check(&doc).is_empty());
    }

    #[test]
    fn test_set_content_out_of_bounds() {
        let doc = make_valid_doc();
        let bad = SIRInstruction::new(SIROpcode::SetContent, 3, 0, 999);
        let mut doc2 = doc;
        doc2.push(bad);
        let errors = check(&doc2);
        assert_eq!(errors.len(), 1);
        match &errors[0].kind {
            crate::error::ErrorKind::Validation(ValidationErrorKind::PayloadOutOfBounds {
                entity_id,
                offset,
            }) => {
                assert_eq!(*entity_id, 3);
                assert_eq!(*offset, 999);
            }
            other => panic!("expected PayloadOutOfBounds, got {:?}", other),
        }
    }

    #[test]
    fn test_push_block_out_of_bounds() {
        let doc = make_valid_doc();
        let bad = SIRInstruction::new(SIROpcode::PushBlock, 3, 0, 999);
        let mut doc2 = doc;
        doc2.push(bad);
        let errors = check(&doc2);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_link_data_out_of_bounds() {
        let doc = make_valid_doc();
        let bad = SIRInstruction::new(SIROpcode::LinkData, 3, 0, 999);
        let mut doc2 = doc;
        doc2.push(bad);
        let errors = check(&doc2);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_apply_style_skipped() {
        let doc = make_valid_doc();
        let errors = check(&doc);
        let style_errors: Vec<_> = errors
            .iter()
            .filter(|e| {
                matches!(
                    &e.kind,
                    crate::error::ErrorKind::Validation(
                        ValidationErrorKind::PayloadOutOfBounds { .. }
                    )
                )
            })
            .collect();
        assert!(style_errors.is_empty(), "ApplyStyle should not be checked");
    }

    #[test]
    fn test_invalid_utf8_detected() {
        let mut doc = SIRDocument::new();
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::PushBlock, 0, ROOT_SENTINEL, 0),
            &[BlockType::Document as u8],
        );
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0),
            b"Hello\x00",
        );
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::SetContent, 2, 0, 0),
            &[0xFF, 0xFE],
        );
        let errors = check(&doc);
        assert_eq!(
            errors.len(),
            1,
            "only the invalid UTF-8 content should error"
        );
    }

    #[test]
    fn test_empty_set_content_passes() {
        let mut doc = SIRDocument::new();
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::PushBlock, 0, ROOT_SENTINEL, 0),
            &[BlockType::Document as u8],
        );
        let payload_len = doc.payload().len() as u32;
        doc.push(SIRInstruction::new(
            SIROpcode::SetContent,
            1,
            0,
            payload_len,
        ));
        assert!(check(&doc).is_empty());
    }

    #[test]
    fn test_multiple_errors_collected() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 0, 50));
        doc.push(SIRInstruction::new(SIROpcode::LinkData, 2, 0, 100));
        let errors = check(&doc);
        assert!(errors.len() >= 3);
    }
}
