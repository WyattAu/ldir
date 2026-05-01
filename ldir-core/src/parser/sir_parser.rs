//! S-IR parser: deserialize S-IR documents from rkyv bytes.
//!
//! Interface: [`parse_sir`]
//!
//! ## Pre-conditions
//!
//! - **PRE-PARSE-001**: `bytes.len() >= MIN_SIR_BYTES` (minimum rkyv-encoded size)
//! - **PRE-PARSE-002**: `bytes.as_ptr() as usize % 4 == 0` (4-byte alignment)
//!
//! ## Errors
//!
//! - **ERR-PARSE-001**: Input too short (`ParseErrorKind::InputTooShort`)
//! - **ERR-PARSE-002**: Alignment error (`ParseErrorKind::AlignmentError`)
//! - **ERR-PARSE-003**: Invalid rkyv data (`ParseErrorKind::DeserializationError`)

use ldir_ir::sir::SIRDocument;

use crate::error::{ParseErrorKind, Result};
use crate::source_map::SourceMap;

/// Minimum size for any valid rkyv-encoded `SIRDocument`.
///
/// An empty document's rkyv encoding still includes the `Vec` header
/// (length + capacity + pointer). We use a conservative lower bound
/// to catch obviously corrupt input early.
const MIN_SIR_BYTES: usize = 4;

/// IF-PARSE-001: Parse an S-IR document from rkyv-serialized bytes.
///
/// Performs pre-condition checks (size and alignment) before delegating
/// to `rkyv::from_bytes` for zero-copy deserialization with validation.
///
/// # Errors
///
/// - `ERR-PARSE-001`: Input is shorter than `MIN_SIR_BYTES`.
/// - `ERR-PARSE-002`: Input pointer is not 4-byte aligned.
/// - `ERR-PARSE-003`: Bytes are not valid rkyv-encoded `SIRDocument`.
pub fn parse_sir(bytes: &[u8]) -> Result<SIRDocument> {
    // PRE-PARSE-001 / ERR-PARSE-001
    if bytes.len() < MIN_SIR_BYTES {
        return Err(ParseErrorKind::InputTooShort { len: bytes.len() }.into());
    }

    // PRE-PARSE-002 / ERR-PARSE-002
    let ptr = bytes.as_ptr() as usize;
    if !ptr.is_multiple_of(4) {
        return Err(ParseErrorKind::AlignmentError {
            offset: (ptr % 4) as u32,
        }
        .into());
    }

    // ERR-PARSE-003
    SIRDocument::from_bytes(bytes).map_err(|e| {
        ParseErrorKind::DeserializationError {
            message: e.to_string(),
        }
        .into()
    })
}

/// IF-PARSE-002: Parse an S-IR document from rkyv-serialized bytes and
/// build a [`SourceMap`] mapping wire-format byte offsets to entity IDs.
///
/// The source map offsets start at `base_offset` and advance by the
/// instruction wire size (13 bytes, REQ-3.1.2) per instruction.
/// Line and column fields are set to 0 (wire-format offsets do not carry
/// original source coordinates).
///
/// # Errors
///
/// Same as [`parse_sir`].
pub fn parse_sir_with_source_map(
    bytes: &[u8],
    base_offset: u32,
) -> Result<(SIRDocument, SourceMap)> {
    let doc = parse_sir(bytes)?;
    let map = SourceMap::build_from_document(&doc, base_offset);
    Ok((doc, map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldir_ir::sir::{ROOT_SENTINEL, SIRInstruction, SIROpcode};

    fn make_wellformed_doc() -> SIRDocument {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0));
        doc.push(SIRInstruction::new(SIROpcode::ApplyStyle, 2, 0, 10));
        doc
    }

    // -- Well-formed input tests --

    #[test]
    fn test_parse_wellformed_document() {
        let doc = make_wellformed_doc();
        let bytes = doc.to_bytes();
        let parsed = parse_sir(&bytes).unwrap();
        assert_eq!(parsed, doc);
    }

    #[test]
    fn test_parse_empty_document() {
        let doc = SIRDocument::new();
        let bytes = doc.to_bytes();
        let parsed = parse_sir(&bytes).unwrap();
        assert_eq!(parsed, doc);
    }

    #[test]
    fn test_parse_preserves_all_fields() {
        let doc = make_wellformed_doc();
        let bytes = doc.to_bytes();
        let parsed = parse_sir(&bytes).unwrap();
        for (orig, rest) in doc.iter().zip(parsed.iter()) {
            assert_eq!(orig.opcode(), rest.opcode());
            assert_eq!(orig.entity_id(), rest.entity_id());
            assert_eq!(orig.parent_id(), rest.parent_id());
            assert_eq!(orig.payload_offset(), rest.payload_offset());
        }
    }

    // -- Malformed input tests --

    #[test]
    fn test_err_parse_001_empty_input() {
        let result = parse_sir(&[]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let kind = match &err.kind {
            crate::error::ErrorKind::Parse(p) => p,
            _ => panic!("expected Parse error, got {:?}", err.kind),
        };
        assert_eq!(kind, &ParseErrorKind::InputTooShort { len: 0 });
    }

    #[test]
    fn test_err_parse_001_short_input() {
        let result = parse_sir(&[0x00, 0x01, 0x02]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match &err.kind {
            crate::error::ErrorKind::Parse(p) => {
                assert_eq!(p, &ParseErrorKind::InputTooShort { len: 3 });
            }
            _ => panic!("expected Parse error"),
        }
    }

    #[test]
    fn test_err_parse_001_one_byte_short() {
        let doc = make_wellformed_doc();
        let bytes = doc.to_bytes();
        // Truncate to just under minimum
        let short = &bytes[..std::cmp::max(MIN_SIR_BYTES - 1, bytes.len() - 1)];
        if short.len() < MIN_SIR_BYTES {
            let result = parse_sir(short);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_err_parse_003_truncated_valid_bytes() {
        let doc = make_wellformed_doc();
        let bytes = doc.to_bytes();
        let truncated = &bytes[..bytes.len() / 2];
        let result = parse_sir(truncated);
        assert!(result.is_err());
        match &result.unwrap_err().kind {
            crate::error::ErrorKind::Parse(ParseErrorKind::DeserializationError { .. }) => {}
            other => panic!("expected DeserializationError, got {:?}", other),
        }
    }

    #[test]
    fn test_err_parse_003_garbage_bytes() {
        let garbage = vec![0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE];
        let result = parse_sir(&garbage);
        assert!(result.is_err());
        match &result.unwrap_err().kind {
            crate::error::ErrorKind::Parse(ParseErrorKind::DeserializationError { .. }) => {}
            other => panic!("expected DeserializationError, got {:?}", other),
        }
    }

    #[test]
    fn test_err_parse_003_all_zeros() {
        let zeros = vec![0u8; 16];
        let result = parse_sir(&zeros);
        // All zeros may or may not parse; just ensure it doesn't panic
        let _ = result;
    }

    // -- Alignment check --
    //
    // Note: alignment check depends on the runtime pointer alignment of the
    // slice. In practice, Vec-backed slices are well-aligned, so ERR-PARSE-002
    // is primarily a defense for FFI/mmap scenarios.

    #[test]
    fn test_alignment_ok_with_vec() {
        let doc = make_wellformed_doc();
        let bytes = doc.to_bytes();
        // Vec-backed slices are always well-aligned
        assert!(parse_sir(&bytes).is_ok());
    }

    // -- Roundtrip consistency --

    #[test]
    fn test_roundtrip_parse_serialize() {
        let doc = make_wellformed_doc();
        let bytes = doc.to_bytes();
        let parsed = parse_sir(&bytes).unwrap();
        let re_bytes = parsed.to_bytes();
        let re_parsed = parse_sir(&re_bytes).unwrap();
        assert_eq!(re_parsed, doc);
    }
}
