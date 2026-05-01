//! S-IR opcode definitions (REQ-3.1.3).
//!
//! Maps to the Lean 4 `SIROpcode` inductive type in
//! `ProofIRWellformedness.lean` Section 1.
//!
//! Each opcode is assigned a 1-byte discriminant for the wire format.

/// Block type parameter for `PushBlock` instructions.
///
/// Matches Lean 4 `BlockType` inductive:
/// `document | paragraph | heading | list | math | code`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[repr(u8)]
#[rkyv(attr(derive(Debug, Clone, Copy, PartialEq, Eq, Hash)))]
pub enum BlockType {
    /// Top-level document root node.
    Document = 0x00,
    /// Paragraph block.
    Paragraph = 0x01,
    /// Heading block (with level in payload).
    Heading = 0x02,
    /// Ordered or unordered list block.
    List = 0x03,
    /// Mathematical expression block (deferred to post-MVP, REQ-1.2.3).
    Math = 0x04,
    /// Code / verbatim block.
    Code = 0x05,
    /// Blockquote block.
    BlockQuote = 0x06,
    /// Thematic break (horizontal rule).
    ThematicBreak = 0x07,
    /// Image block (embedded or linked image).
    Image = 0x08,
    /// Table block (grid of cells).
    Table = 0x09,
}

impl BlockType {
    /// Try to convert a raw byte to a `BlockType`.
    ///
    /// Returns `None` for bytes outside the valid discriminant range.
    pub fn from_u8(byte: u8) -> Option<Self> {
        match byte {
            0x00 => Some(Self::Document),
            0x01 => Some(Self::Paragraph),
            0x02 => Some(Self::Heading),
            0x03 => Some(Self::List),
            0x04 => Some(Self::Math),
            0x05 => Some(Self::Code),
            0x06 => Some(Self::BlockQuote),
            0x07 => Some(Self::ThematicBreak),
            0x08 => Some(Self::Image),
            0x09 => Some(Self::Table),
            _ => None,
        }
    }
}

/// S-IR operation discriminator (1 byte, REQ-3.1.3).
///
/// Wire format assigns each variant a fixed byte value:
/// - `0x00`..`0x05`: `PushBlock` with `BlockType` encoded as lower nibble
/// - `0x06`..`0x09`: Parameterless opcodes
///
/// Matches Lean 4 `SIROpcode` inductive in `ProofIRWellformedness.lean`:
/// `pushBlock | setContent | applyStyle | insertMath | linkData`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[repr(u8)]
#[rkyv(attr(derive(Debug, Clone, Copy, PartialEq, Eq, Hash)))]
pub enum SIROpcode {
    /// Push a structural block onto the document tree.
    /// Carries a `BlockType` parameter in the payload region.
    ///
    /// Maps to Lean 4: `SIROpcode.pushBlock(bt : BlockType)`.
    PushBlock = 0x00,

    /// Set text content for the current entity.
    /// Payload contains a UTF-8 text blob (REQ-3.1.4).
    ///
    /// Maps to Lean 4: `SIROpcode.setContent`.
    SetContent = 0x01,

    /// Apply a style (font, size, color) to the current entity.
    /// Payload contains style parameters.
    ///
    /// Maps to Lean 4: `SIROpcode.applyStyle`.
    ApplyStyle = 0x02,

    /// Insert a mathematical expression placeholder.
    /// Deferred to post-MVP per REQ-1.2.3.
    /// Payload contains MathML or TeX reference.
    ///
    /// Maps to Lean 4: `SIROpcode.insertMath`.
    InsertMath = 0x03,

    /// Attach a hyperlink or data reference.
    /// Payload contains the link target pointer.
    ///
    /// Maps to Lean 4: `SIROpcode.linkData`.
    LinkData = 0x04,
}

impl SIROpcode {
    /// Try to convert a raw byte to a `SIROpcode`.
    ///
    /// Returns `None` for bytes outside the valid discriminant range.
    pub fn from_u8(byte: u8) -> Option<Self> {
        match byte {
            0x00 => Some(Self::PushBlock),
            0x01 => Some(Self::SetContent),
            0x02 => Some(Self::ApplyStyle),
            0x03 => Some(Self::InsertMath),
            0x04 => Some(Self::LinkData),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_discriminants() {
        assert_eq!(SIROpcode::PushBlock as u8, 0x00);
        assert_eq!(SIROpcode::SetContent as u8, 0x01);
        assert_eq!(SIROpcode::ApplyStyle as u8, 0x02);
        assert_eq!(SIROpcode::InsertMath as u8, 0x03);
        assert_eq!(SIROpcode::LinkData as u8, 0x04);
    }

    #[test]
    fn test_block_type_discriminants() {
        assert_eq!(BlockType::Document as u8, 0x00);
        assert_eq!(BlockType::Code as u8, 0x05);
        assert_eq!(BlockType::Image as u8, 0x08);
        assert_eq!(BlockType::Table as u8, 0x09);
    }

    #[test]
    fn test_opcode_from_u8_roundtrip() {
        for byte in 0..=255u8 {
            if let Some(op) = SIROpcode::from_u8(byte) {
                assert_eq!(op as u8, byte);
            }
        }
    }

    #[test]
    fn test_block_type_from_u8_roundtrip() {
        for byte in 0..=255u8 {
            if let Some(bt) = BlockType::from_u8(byte) {
                assert_eq!(bt as u8, byte);
            }
        }
    }
}
