//! G-IR command type (REQ-3.2.3).
//!
//! A `GIRCommand` is a single rendering command in the G-IR flat command
//! buffer. Each command has an opcode and a fixed-size argument array.
//!
//! Matches Lean 4 `GIRCommand` structure in `ProofIRWellformedness.lean`:
//! ```lean
//! structure GIRCommand where
//!   opcode : GIROpcode
//!   args : List Int
//! ```
//!
//! # Wire Format
//!
//! Per REQ-3.2.2, G-IR commands are aligned to 16-byte boundaries.
//! The command struct is 36 bytes (1 byte opcode + 3 bytes padding + 8×4 bytes args).
//! Wire serialization pads to the next 16-byte boundary.

use crate::gir::opcode::GIROpcode;

/// Number of argument slots in a G-IR command.
pub const GIR_COMMAND_ARGS: usize = 8;

/// Single G-IR rendering command with fixed-size argument array (REQ-3.2.3).
///
/// Layout (C repr):
/// ```text
/// Offset  Size  Field
/// 0       1     opcode (u8)
/// 1       3     padding
/// 4       32    args: [i32; 8]
/// ```
/// Total: 36 bytes. Padded to 16-byte alignment in wire format (REQ-3.2.2).
///
/// All coordinates in args use 26.6 fixed-point format (REQ-3.2.5).
///
/// # Well-Formedness (DEF-005)
///
/// Per DEF-005 cond. 1: Coordinates must be in 26.6 representable range.
/// Per DEF-005 cond. 3: PushStack/PopStack must be balanced per page.
///
/// # Examples
///
/// ```
/// use ldir_ir::gir::{GIRCommand, GIROpcode};
///
/// let cmd = GIRCommand::new_move_xy(640, 1280);
/// assert_eq!(cmd.opcode(), GIROpcode::MoveXY);
/// assert_eq!(cmd.arg(0), Some(640));
/// assert_eq!(cmd.arg(1), Some(1280));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct GIRCommand {
    /// Operation discriminator.
    opcode: GIROpcode,
    /// Padding for alignment.
    _pad: [u8; 3],
    /// Fixed-size argument array (coordinates in 26.6 fixed-point).
    args: [i32; GIR_COMMAND_ARGS],
}

impl GIRCommand {
    /// Create a new command with the given opcode and argument array.
    #[inline]
    pub const fn new(opcode: GIROpcode, args: [i32; GIR_COMMAND_ARGS]) -> Self {
        Self {
            opcode,
            _pad: [0; 3],
            args,
        }
    }

    /// Create a command with the given opcode and zeroed arguments.
    #[inline]
    pub const fn new_zeroed(opcode: GIROpcode) -> Self {
        Self::new(opcode, [0; GIR_COMMAND_ARGS])
    }

    /// Get the operation discriminator.
    #[inline]
    pub const fn opcode(&self) -> GIROpcode {
        self.opcode
    }

    /// Get an argument by index.
    ///
    /// Returns `None` if `index >= 8`.
    #[inline]
    pub const fn arg(&self, index: usize) -> Option<i32> {
        if index < GIR_COMMAND_ARGS {
            Some(self.args[index])
        } else {
            None
        }
    }

    /// Get the full argument array.
    #[inline]
    pub const fn args(&self) -> [i32; GIR_COMMAND_ARGS] {
        self.args
    }

    /// Set an argument by index.
    ///
    /// Panics if `index >= 8`.
    #[inline]
    pub fn set_arg(&mut self, index: usize, value: i32) {
        self.args[index] = value;
    }

    /// Create a `SetFont` command.
    ///
    /// Args: `[font_id, 0, 0, 0, 0, 0, 0, 0]`
    #[inline]
    pub const fn new_set_font(font_id: i32) -> Self {
        let mut args = [0i32; GIR_COMMAND_ARGS];
        args[0] = font_id;
        Self::new(GIROpcode::SetFont, args)
    }

    /// Create a `MoveXY` command.
    ///
    /// Args: `[x_fp26_6, y_fp26_6, 0, 0, 0, 0, 0, 0]`
    /// Coordinates in 26.6 fixed-point format (REQ-3.2.5).
    #[inline]
    pub const fn new_move_xy(x_fp26_6: i32, y_fp26_6: i32) -> Self {
        let mut args = [0i32; GIR_COMMAND_ARGS];
        args[0] = x_fp26_6;
        args[1] = y_fp26_6;
        Self::new(GIROpcode::MoveXY, args)
    }

    /// Create a `PutGlyph` command.
    ///
    /// Args: `[glyph_id, advance_x_fp26_6, 0, 0, 0, 0, 0, 0]`
    #[inline]
    pub const fn new_put_glyph(glyph_id: i32, advance_x_fp26_6: i32) -> Self {
        let mut args = [0i32; GIR_COMMAND_ARGS];
        args[0] = glyph_id;
        args[1] = advance_x_fp26_6;
        Self::new(GIROpcode::PutGlyph, args)
    }

    /// Create a `DrawRule` command.
    ///
    /// Args: `[x_fp26_6, y_fp26_6, width_fp26_6, thickness_fp26_6, 0, 0, 0, 0]`
    /// All values in 26.6 fixed-point format.
    #[inline]
    pub const fn new_draw_rule(
        x_fp26_6: i32,
        y_fp26_6: i32,
        width_fp26_6: i32,
        thickness_fp26_6: i32,
    ) -> Self {
        let mut args = [0i32; GIR_COMMAND_ARGS];
        args[0] = x_fp26_6;
        args[1] = y_fp26_6;
        args[2] = width_fp26_6;
        args[3] = thickness_fp26_6;
        Self::new(GIROpcode::DrawRule, args)
    }

    /// Create a `PushStack` command.
    ///
    /// Args: `[]` (all zeros).
    #[inline]
    pub const fn new_push_stack() -> Self {
        Self::new_zeroed(GIROpcode::PushStack)
    }

    /// Create a `PopStack` command.
    ///
    /// Args: `[]` (all zeros).
    #[inline]
    pub const fn new_pop_stack() -> Self {
        Self::new_zeroed(GIROpcode::PopStack)
    }

    /// Create an `AttachMetadata` command.
    ///
    /// Args: `[key_offset, val_offset, key_len, val_len, 0, 0, 0, 0]`
    #[inline]
    pub const fn new_attach_metadata(
        key_offset: i32,
        val_offset: i32,
        key_len: i32,
        val_len: i32,
    ) -> Self {
        let mut args = [0i32; GIR_COMMAND_ARGS];
        args[0] = key_offset;
        args[1] = val_offset;
        args[2] = key_len;
        args[3] = val_len;
        Self::new(GIROpcode::AttachMetadata, args)
    }
}

impl Default for GIRCommand {
    fn default() -> Self {
        Self::new_zeroed(GIROpcode::SetFont)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_size() {
        assert_eq!(std::mem::size_of::<GIRCommand>(), 36);
    }

    #[test]
    fn test_new_set_font() {
        let cmd = GIRCommand::new_set_font(42);
        assert_eq!(cmd.opcode(), GIROpcode::SetFont);
        assert_eq!(cmd.arg(0), Some(42));
        assert_eq!(cmd.arg(1), Some(0));
    }

    #[test]
    fn test_new_move_xy() {
        let cmd = GIRCommand::new_move_xy(640, 1280);
        assert_eq!(cmd.opcode(), GIROpcode::MoveXY);
        assert_eq!(cmd.arg(0), Some(640));
        assert_eq!(cmd.arg(1), Some(1280));
    }

    #[test]
    fn test_new_put_glyph() {
        let cmd = GIRCommand::new_put_glyph(65, 640);
        assert_eq!(cmd.opcode(), GIROpcode::PutGlyph);
        assert_eq!(cmd.arg(0), Some(65));
        assert_eq!(cmd.arg(1), Some(640));
    }

    #[test]
    fn test_new_draw_rule() {
        let cmd = GIRCommand::new_draw_rule(100, 200, 300, 10);
        assert_eq!(cmd.opcode(), GIROpcode::DrawRule);
        assert_eq!(cmd.arg(0), Some(100));
        assert_eq!(cmd.arg(1), Some(200));
        assert_eq!(cmd.arg(2), Some(300));
        assert_eq!(cmd.arg(3), Some(10));
    }

    #[test]
    fn test_new_push_pop_stack() {
        let push = GIRCommand::new_push_stack();
        let pop = GIRCommand::new_pop_stack();
        assert_eq!(push.opcode(), GIROpcode::PushStack);
        assert_eq!(pop.opcode(), GIROpcode::PopStack);
        for i in 0..8 {
            assert_eq!(push.arg(i), Some(0));
            assert_eq!(pop.arg(i), Some(0));
        }
    }

    #[test]
    fn test_new_attach_metadata() {
        let cmd = GIRCommand::new_attach_metadata(0, 10, 4, 8);
        assert_eq!(cmd.opcode(), GIROpcode::AttachMetadata);
        assert_eq!(cmd.arg(0), Some(0));
        assert_eq!(cmd.arg(1), Some(10));
        assert_eq!(cmd.arg(2), Some(4));
        assert_eq!(cmd.arg(3), Some(8));
    }

    #[test]
    fn test_arg_out_of_bounds() {
        let cmd = GIRCommand::new_set_font(0);
        assert_eq!(cmd.arg(8), None);
        assert_eq!(cmd.arg(100), None);
    }

    #[test]
    fn test_set_arg() {
        let mut cmd = GIRCommand::new_zeroed(GIROpcode::MoveXY);
        cmd.set_arg(0, 100);
        cmd.set_arg(1, 200);
        assert_eq!(cmd.arg(0), Some(100));
        assert_eq!(cmd.arg(1), Some(200));
    }

    #[test]
    fn test_copy_semantics() {
        let a = GIRCommand::new_move_xy(10, 20);
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn test_args_slice() {
        let cmd = GIRCommand::new_set_font(42);
        let args = cmd.args();
        assert_eq!(args[0], 42);
        assert_eq!(args.len(), 8);
    }
}
