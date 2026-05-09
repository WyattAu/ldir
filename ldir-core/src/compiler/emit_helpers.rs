//! Helper functions for emitting G-IR commands during compilation.

use ldir_ir::gir::{GIRCommand, GIRPage};

use crate::fp266::Fp266;

/// Emit a PushStack command onto the page.
pub fn emit_push_stack(page: &mut GIRPage) {
    page.push(GIRCommand::new_push_stack());
}

/// Emit a PopStack command onto the page.
pub fn emit_pop_stack(page: &mut GIRPage) {
    page.push(GIRCommand::new_pop_stack());
}

/// Emit a SetFont command onto the page.
pub fn emit_set_font(page: &mut GIRPage, font_id: i32) {
    page.push(GIRCommand::new_set_font(font_id));
}

/// Emit a MoveXY command onto the page.
pub fn emit_move_xy(page: &mut GIRPage, x: Fp266, y: Fp266) {
    page.push(GIRCommand::new_move_xy(x.raw() as i32, y.raw() as i32));
}

/// Emit PutGlyph commands for each character in the content string.
///
/// Each character is emitted as a PutGlyph with its Unicode codepoint as
/// the glyph ID and a default advance width.
pub fn emit_text_content(page: &mut GIRPage, content: &str, glyph_advance: i32) {
    for ch in content.chars() {
        page.push(GIRCommand::new_put_glyph(ch as i32, glyph_advance));
    }
}

/// Emit a DrawRule command (e.g., for horizontal rules, underlines).
pub fn emit_draw_rule(page: &mut GIRPage, x: Fp266, y: Fp266, width: Fp266, thickness: Fp266) {
    page.push(GIRCommand::new_draw_rule(
        x.raw() as i32,
        y.raw() as i32,
        width.raw() as i32,
        thickness.raw() as i32,
    ));
}

/// Emit an AttachMetadata command for hyperlink data.
pub fn emit_attach_metadata(
    page: &mut GIRPage,
    key_offset: i32,
    val_offset: i32,
    key_len: i32,
    val_len: i32,
) {
    page.push(GIRCommand::new_attach_metadata(
        key_offset, val_offset, key_len, val_len,
    ));
}

/// Emit PopStack commands to balance the stack back to the target depth.
///
/// Returns the number of PopStack commands emitted.
pub fn emit_balance_stack(page: &mut GIRPage, current_depth: usize, target_depth: usize) -> usize {
    let to_pop = current_depth.saturating_sub(target_depth);
    for _ in 0..to_pop {
        emit_pop_stack(page);
    }
    to_pop
}

/// Emit paragraph spacing (advance Y and reset X).
pub fn emit_paragraph_spacing(
    page: &mut GIRPage,
    ctx: &mut super::context::CompileContext,
    spacing_pt: i32,
) {
    ctx.advance_y(Fp266::from_int(spacing_pt));
    ctx.reset_x();
    emit_move_xy(page, ctx.x, ctx.y);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::context::CompileContext;
    use ldir_ir::gir::GIROpcode;

    #[test]
    fn test_emit_push_pop() {
        let mut page = GIRPage::new();
        emit_push_stack(&mut page);
        emit_set_font(&mut page, 1);
        emit_pop_stack(&mut page);
        assert!(page.is_stack_balanced());
        assert_eq!(page.len(), 3);
    }

    #[test]
    fn test_emit_move_xy() -> Result<(), Box<dyn std::error::Error>> {
        let mut page = GIRPage::new();
        emit_move_xy(&mut page, Fp266::from_int(100), Fp266::from_int(200));
        assert_eq!(page.len(), 1);
        let cmd = page.get(0).ok_or("no command")?;
        assert_eq!(cmd.opcode(), GIROpcode::MoveXY);
        assert_eq!(cmd.arg(0), Some(100 * 64));
        assert_eq!(cmd.arg(1), Some(200 * 64));
        Ok(())
    }

    #[test]
    fn test_emit_text_content() -> Result<(), Box<dyn std::error::Error>> {
        let mut page = GIRPage::new();
        emit_text_content(&mut page, "AB", 7 * 64);
        assert_eq!(page.len(), 2);
        let cmd0 = page.get(0).ok_or("no command at 0")?;
        assert_eq!(cmd0.opcode(), GIROpcode::PutGlyph);
        assert_eq!(cmd0.arg(0), Some('A' as i32));
        let cmd1 = page.get(1).ok_or("no command at 1")?;
        assert_eq!(cmd1.arg(0), Some('B' as i32));
        Ok(())
    }

    #[test]
    fn test_emit_text_empty() {
        let mut page = GIRPage::new();
        emit_text_content(&mut page, "", 7 * 64);
        assert!(page.is_empty());
    }

    #[test]
    fn test_emit_draw_rule() -> Result<(), Box<dyn std::error::Error>> {
        let mut page = GIRPage::new();
        emit_draw_rule(
            &mut page,
            Fp266::from_int(72),
            Fp266::from_int(100),
            Fp266::from_int(468),
            Fp266::from_int(1),
        );
        assert_eq!(page.len(), 1);
        let cmd = page.get(0).ok_or("no command")?;
        assert_eq!(cmd.opcode(), GIROpcode::DrawRule);
        Ok(())
    }

    #[test]
    fn test_emit_balance_stack() {
        let mut page = GIRPage::new();
        for _ in 0..3 {
            emit_push_stack(&mut page);
        }
        let popped = emit_balance_stack(&mut page, 3, 0);
        assert_eq!(popped, 3);
        assert!(page.is_stack_balanced());
    }

    #[test]
    fn test_emit_attach_metadata() -> Result<(), Box<dyn std::error::Error>> {
        let mut page = GIRPage::new();
        emit_attach_metadata(&mut page, 0, 10, 4, 20);
        assert_eq!(page.len(), 1);
        let cmd = page.get(0).ok_or("no command")?;
        assert_eq!(cmd.opcode(), GIROpcode::AttachMetadata);
        Ok(())
    }

    #[test]
    fn test_emit_paragraph_spacing() -> Result<(), Box<dyn std::error::Error>> {
        let mut page = GIRPage::new();
        let mut ctx = CompileContext::new();
        emit_paragraph_spacing(&mut page, &mut ctx, 12);
        assert_eq!(page.len(), 1);
        let cmd = page.get(0).ok_or("no command")?;
        assert_eq!(cmd.opcode(), GIROpcode::MoveXY);
        Ok(())
    }
}
