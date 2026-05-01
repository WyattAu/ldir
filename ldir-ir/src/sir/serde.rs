//! rkyv serialization/deserialization utilities for S-IR types.
//!
//! Provides helper functions for serialization beyond the inherent methods
//! on `SIRDocument`. These are useful for testing and custom pipelines.

use crate::sir::document::SIRDocument;

/// Serialize an `SIRDocument` to rkyv bytes.
pub fn serialize_sir(doc: &SIRDocument) -> Vec<u8> {
    doc.to_bytes()
}

/// Deserialize an `SIRDocument` from rkyv bytes.
pub fn deserialize_sir(bytes: &[u8]) -> Result<SIRDocument, rkyv::rancor::Error> {
    SIRDocument::from_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sir::instruction::{ROOT_SENTINEL, SIRInstruction};
    use crate::sir::opcode::SIROpcode;

    fn make_test_doc() -> SIRDocument {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0));
        doc.push(SIRInstruction::new(SIROpcode::ApplyStyle, 2, 0, 10));
        doc.push(SIRInstruction::new(SIROpcode::InsertMath, 3, 1, 20));
        doc.push(SIRInstruction::new(SIROpcode::LinkData, 4, 1, 40));
        doc
    }

    #[test]
    fn test_roundtrip_small_document() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        let bytes = doc.to_bytes();
        let restored = SIRDocument::from_bytes(&bytes).unwrap();
        assert_eq!(restored, doc);
    }

    #[test]
    fn test_roundtrip_multi_instruction() {
        let doc = make_test_doc();
        let bytes = doc.to_bytes();
        let restored = SIRDocument::from_bytes(&bytes).unwrap();
        assert_eq!(restored, doc);
    }

    #[test]
    fn test_roundtrip_preserves_entity_ids() {
        let doc = make_test_doc();
        let bytes = doc.to_bytes();
        let restored = SIRDocument::from_bytes(&bytes).unwrap();
        assert_eq!(
            restored.entity_ids().collect::<Vec<_>>(),
            doc.entity_ids().collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_roundtrip_preserves_opcodes() {
        let doc = make_test_doc();
        let bytes = doc.to_bytes();
        let restored = SIRDocument::from_bytes(&bytes).unwrap();
        for (orig, rest) in doc.iter().zip(restored.iter()) {
            assert_eq!(orig.opcode(), rest.opcode());
            assert_eq!(orig.entity_id(), rest.entity_id());
            assert_eq!(orig.parent_id(), rest.parent_id());
            assert_eq!(orig.payload_offset(), rest.payload_offset());
        }
    }

    #[test]
    fn test_serialize_deserialize_helpers() {
        let doc = make_test_doc();
        let bytes = serialize_sir(&doc);
        let restored = deserialize_sir(&bytes).unwrap();
        assert_eq!(restored, doc);
    }

    #[test]
    fn test_empty_document_roundtrip() {
        let doc = SIRDocument::new();
        let bytes = doc.to_bytes();
        let restored = SIRDocument::from_bytes(&bytes).unwrap();
        assert_eq!(restored, doc);
    }

    #[test]
    fn test_truncated_bytes_fails() {
        let doc = make_test_doc();
        let bytes = doc.to_bytes();
        let truncated = &bytes[..bytes.len() / 2];
        assert!(SIRDocument::from_bytes(truncated).is_err());
    }

    #[test]
    fn test_garbage_bytes_fails() {
        let garbage = vec![0xDE, 0xAD, 0xBE, 0xEF];
        assert!(SIRDocument::from_bytes(&garbage).is_err());
    }
}
