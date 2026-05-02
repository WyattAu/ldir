//! Binary serialization and deserialization for G-IR documents.
//!
//! Wire format per REQ-3.2.2:
//! - Header: magic "GIR0" (4 bytes) + page count u32 LE (4 bytes)
//! - Per page: command count u32 LE + width i32 LE + height i32 LE
//! - Per command: opcode u8 + 3 padding + 8 × i32 LE args = 36 bytes

#![allow(clippy::expect_used)]

use ldir_ir::gir::{GIR_COMMAND_ARGS, GIRCommand, GIRDocument, GIROpcode, GIRPage};

use crate::error::{EmitErrorKind, Result};

/// Magic bytes for the G-IR binary format.
pub const GIR_MAGIC: &[u8; 4] = b"GIR0";

/// Total size of the binary header (magic + page count).
pub const HEADER_SIZE: usize = 8;

/// Size of the per-page header (command count + width + height).
pub const PAGE_HEADER_SIZE: usize = 12;

/// Size of a single command in the binary format.
pub const COMMAND_SIZE: usize = 1 + 3 + GIR_COMMAND_ARGS * 4;

/// Emit a G-IR document to binary bytes.
///
/// Implements IF-EMIT-001.
///
/// # Format
///
/// ```text
/// [GIR0] [page_count:u32]
/// For each page:
///   [cmd_count:u32] [width:i32] [height:i32]
///   For each command:
///     [opcode:u8] [pad:3] [args:8×i32]
/// ```
pub fn emit_gir(doc: &GIRDocument) -> Vec<u8> {
    let page_count = doc.page_count() as u32;
    let mut buf = Vec::with_capacity(HEADER_SIZE + page_count as usize * (PAGE_HEADER_SIZE + 256));

    buf.extend_from_slice(GIR_MAGIC);
    buf.extend_from_slice(&page_count.to_le_bytes());

    for page in doc.iter() {
        let cmd_count = page.len() as u32;
        buf.extend_from_slice(&cmd_count.to_le_bytes());
        buf.extend_from_slice(&page.width.to_le_bytes());
        buf.extend_from_slice(&page.height.to_le_bytes());

        for cmd in page.iter() {
            buf.push(cmd.opcode() as u8);
            buf.extend_from_slice(&[0u8; 3]);
            let args = cmd.args();
            for arg in &args {
                buf.extend_from_slice(&arg.to_le_bytes());
            }
        }
    }

    buf
}

/// Parse a G-IR document from binary bytes.
///
/// Implements round-trip deserialization for testing.
///
/// # Errors
///
/// Returns an error if:
/// - Input is too short for the header
/// - Magic bytes don't match
/// - Page or command data is truncated
/// - An invalid opcode byte is encountered
pub fn parse_gir(bytes: &[u8]) -> Result<GIRDocument> {
    if bytes.len() < HEADER_SIZE {
        return Err(EmitErrorKind::BufferOverflow {
            required: HEADER_SIZE,
            available: bytes.len(),
        }
        .into());
    }

    if &bytes[0..4] != GIR_MAGIC {
        return Err(EmitErrorKind::BufferOverflow {
            required: 4,
            available: 0,
        }
        .into());
    }

    let page_count = u32::from_le_bytes(bytes[4..8].try_into().expect("header slice")) as usize;
    let mut doc = GIRDocument::with_capacity(page_count);
    let mut offset = HEADER_SIZE;

    for _ in 0..page_count {
        if offset + PAGE_HEADER_SIZE > bytes.len() {
            return Err(EmitErrorKind::BufferOverflow {
                required: offset + PAGE_HEADER_SIZE,
                available: bytes.len(),
            }
            .into());
        }

        let cmd_count = u32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .expect("page header slice"),
        ) as usize;
        let width = i32::from_le_bytes(
            bytes[offset + 4..offset + 8]
                .try_into()
                .expect("width slice"),
        );
        let height = i32::from_le_bytes(
            bytes[offset + 8..offset + 12]
                .try_into()
                .expect("height slice"),
        );
        offset += PAGE_HEADER_SIZE;

        let page_data_end = offset + cmd_count * COMMAND_SIZE;
        if page_data_end > bytes.len() {
            return Err(EmitErrorKind::BufferOverflow {
                required: page_data_end,
                available: bytes.len(),
            }
            .into());
        }

        let mut page = GIRPage::with_dimensions(width, height);

        for _ in 0..cmd_count {
            let opcode_byte = bytes[offset];
            let opcode =
                GIROpcode::from_u8(opcode_byte).ok_or_else(|| EmitErrorKind::BufferOverflow {
                    required: offset + 1,
                    available: bytes.len(),
                })?;

            let mut args = [0i32; GIR_COMMAND_ARGS];
            let args_start = offset + 4;
            for (i, arg) in args.iter_mut().enumerate() {
                let arg_offset = args_start + i * 4;
                *arg = i32::from_le_bytes(
                    bytes[arg_offset..arg_offset + 4]
                        .try_into()
                        .expect("arg slice"),
                );
            }

            page.push(GIRCommand::new(opcode, args));
            offset += COMMAND_SIZE;
        }

        doc.push_page(page);
    }

    Ok(doc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldir_ir::gir::GIRCommand;

    fn make_test_doc() -> GIRDocument {
        let mut doc = GIRDocument::new();
        let mut page = GIRPage::new();
        page.push(GIRCommand::new_push_stack());
        page.push(GIRCommand::new_set_font(1));
        page.push(GIRCommand::new_move_xy(100, 200));
        page.push(GIRCommand::new_put_glyph(65, 640));
        page.push(GIRCommand::new_pop_stack());
        doc.push_page(page);
        doc
    }

    #[test]
    fn test_emit_and_parse_roundtrip() {
        let doc = make_test_doc();
        let bytes = emit_gir(&doc);
        let parsed = parse_gir(&bytes).unwrap();
        assert_eq!(doc, parsed);
    }

    #[test]
    fn test_emit_magic() {
        let doc = make_test_doc();
        let bytes = emit_gir(&doc);
        assert_eq!(&bytes[0..4], b"GIR0");
    }

    #[test]
    fn test_emit_page_count() {
        let doc = make_test_doc();
        let bytes = emit_gir(&doc);
        let count = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
        assert_eq!(count, 1);
    }

    #[test]
    fn test_emit_multi_page() {
        let mut doc = GIRDocument::new();
        doc.push_page(GIRPage::new());
        doc.push_page(GIRPage::new());
        doc.push_page(GIRPage::new());
        let bytes = emit_gir(&doc);
        let count = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
        assert_eq!(count, 3);

        let parsed = parse_gir(&bytes).unwrap();
        assert_eq!(parsed.page_count(), 3);
    }

    #[test]
    fn test_parse_empty_input() {
        let result = parse_gir(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_short_input() {
        let result = parse_gir(&[0x47, 0x49, 0x52]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_bad_magic() {
        let mut bytes = vec![0x00; 8];
        let result = parse_gir(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_truncated_page() {
        let mut bytes = b"GIR0".to_vec();
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(&612i32.to_le_bytes());
        bytes.extend_from_slice(&792i32.to_le_bytes());
        let result = parse_gir(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_command_size() {
        assert_eq!(COMMAND_SIZE, 36);
    }

    #[test]
    fn test_deterministic_output() {
        let doc = make_test_doc();
        let bytes1 = emit_gir(&doc);
        let bytes2 = emit_gir(&doc);
        assert_eq!(bytes1, bytes2);
    }

    #[test]
    fn test_empty_doc_emit_parse() {
        let doc = GIRDocument::new();
        let bytes = emit_gir(&doc);
        let parsed = parse_gir(&bytes).unwrap();
        assert_eq!(doc, parsed);
    }

    #[test]
    fn test_all_opcodes_roundtrip() {
        let mut doc = GIRDocument::new();
        let mut page = GIRPage::new();
        page.push(GIRCommand::new_set_font(0));
        page.push(GIRCommand::new_move_xy(100, 200));
        page.push(GIRCommand::new_put_glyph(65, 640));
        page.push(GIRCommand::new_draw_rule(0, 0, 468, 64));
        page.push(GIRCommand::new_push_stack());
        page.push(GIRCommand::new_pop_stack());
        page.push(GIRCommand::new_attach_metadata(0, 10, 4, 8));
        page.push(GIRCommand::new_push_stack());
        page.push(GIRCommand::new_pop_stack());
        doc.push_page(page);

        let bytes = emit_gir(&doc);
        let parsed = parse_gir(&bytes).unwrap();
        assert_eq!(doc, parsed);
        assert!(parsed.is_well_formed());
    }

    #[test]
    fn test_large_args_roundtrip() {
        let mut doc = GIRDocument::new();
        let mut page = GIRPage::new();
        let big_arg = [i32::MAX, i32::MIN, 0, -1, 1, 999999, -999999, 42];
        page.push(GIRCommand::new(GIROpcode::DrawRule, big_arg));
        doc.push_page(page);

        let bytes = emit_gir(&doc);
        let parsed = parse_gir(&bytes).unwrap();
        assert_eq!(doc, parsed);
    }
}
