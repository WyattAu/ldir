//! G-IR opcode definitions (REQ-3.2.3).
//!
//! Maps to the Lean 4 `GIROpcode` inductive type in
//! `ProofIRWellformedness.lean` Section 2:
//! ```lean
//! inductive GIROpcode where
//!   | setFont | moveXY | putGlyph | drawRule
//!   | pushStack | popStack | attachMetadata
//! ```

/// G-IR operation discriminator (1 byte, REQ-3.2.3).
///
/// The G-IR opcode set comprises rendering commands for the layout engine:
/// font selection, glyph placement, rule drawing, coordinate stack
/// manipulation, and metadata attachment.
///
/// Per REQ-3.2.1: G-IR compiles into a flat command buffer per page,
/// optimized for direct GPU upload or PDF stream generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum GIROpcode {
    /// Select a font for subsequent glyph commands.
    /// Args: `[font_id]`.
    ///
    /// Maps to Lean 4: `GIROpcode.setFont`.
    SetFont = 0x00,

    /// Move the drawing cursor to absolute coordinates.
    /// Args: `[x_fp26_6, y_fp26_6]`.
    /// Coordinates in 26.6 fixed-point format (REQ-3.2.5).
    ///
    /// Maps to Lean 4: `GIROpcode.moveXY`.
    MoveXY = 0x01,

    /// Place a glyph at the current cursor position.
    /// Args: `[glyph_id, advance_x_fp26_6]`.
    ///
    /// Maps to Lean 4: `GIROpcode.putGlyph`.
    PutGlyph = 0x02,

    /// Draw a horizontal or vertical rule.
    /// Args: `[x_fp26_6, y_fp26_6, width_fp26_6, thickness_fp26_6]`.
    ///
    /// Maps to Lean 4: `GIROpcode.drawRule`.
    DrawRule = 0x03,

    /// Push the current coordinate system onto the stack.
    /// Used for nested blocks (indentation, columns).
    /// Args: `[]`.
    ///
    /// Maps to Lean 4: `GIROpcode.pushStack`.
    PushStack = 0x04,

    /// Pop the most recent coordinate system from the stack.
    /// Args: `[]`.
    ///
    /// Maps to Lean 4: `GIROpcode.popStack`.
    PopStack = 0x05,

    /// Attach metadata (accessibility tags, hyperlinks, etc.).
    /// Args: `[key_offset, val_offset, key_len, val_len]` into metadata region.
    ///
    /// Maps to Lean 4: `GIROpcode.attachMetadata`.
    AttachMetadata = 0x06,
}

impl GIROpcode {
    /// Try to convert a raw byte to a `GIROpcode`.
    ///
    /// Returns `None` for bytes outside the valid discriminant range.
    pub fn from_u8(byte: u8) -> Option<Self> {
        match byte {
            0x00 => Some(Self::SetFont),
            0x01 => Some(Self::MoveXY),
            0x02 => Some(Self::PutGlyph),
            0x03 => Some(Self::DrawRule),
            0x04 => Some(Self::PushStack),
            0x05 => Some(Self::PopStack),
            0x06 => Some(Self::AttachMetadata),
            _ => None,
        }
    }

    /// Get the stack delta for this opcode.
    ///
    /// Used by `pageStackBalanced` (DEF-005 cond. 3) to verify stack balance.
    ///
    /// Matches Lean 4 `stackDelta` in `ProofIRWellformedness.lean` Section 4:
    /// ```lean
    /// def stackDelta (op : GIROpcode) : Int :=
    ///   match op with
    ///   | .pushStack => 1 | .popStack => -1 | _ => 0
    /// ```
    #[inline]
    pub const fn stack_delta(&self) -> i32 {
        match self {
            Self::PushStack => 1,
            Self::PopStack => -1,
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_discriminants() {
        assert_eq!(GIROpcode::SetFont as u8, 0x00);
        assert_eq!(GIROpcode::MoveXY as u8, 0x01);
        assert_eq!(GIROpcode::PutGlyph as u8, 0x02);
        assert_eq!(GIROpcode::DrawRule as u8, 0x03);
        assert_eq!(GIROpcode::PushStack as u8, 0x04);
        assert_eq!(GIROpcode::PopStack as u8, 0x05);
        assert_eq!(GIROpcode::AttachMetadata as u8, 0x06);
    }

    #[test]
    fn test_from_u8_roundtrip() {
        for byte in 0..=255u8 {
            if let Some(op) = GIROpcode::from_u8(byte) {
                assert_eq!(op as u8, byte);
            }
        }
    }

    #[test]
    fn test_stack_delta() {
        assert_eq!(GIROpcode::PushStack.stack_delta(), 1);
        assert_eq!(GIROpcode::PopStack.stack_delta(), -1);
        assert_eq!(GIROpcode::SetFont.stack_delta(), 0);
        assert_eq!(GIROpcode::MoveXY.stack_delta(), 0);
        assert_eq!(GIROpcode::PutGlyph.stack_delta(), 0);
        assert_eq!(GIROpcode::DrawRule.stack_delta(), 0);
        assert_eq!(GIROpcode::AttachMetadata.stack_delta(), 0);
    }
}
