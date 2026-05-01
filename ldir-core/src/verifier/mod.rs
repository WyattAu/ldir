//! G-IR well-formedness verifier (DEF-005).
//!
//! Verifies that a G-IR document satisfies all well-formedness conditions:
//!
//! 1. Coordinates in 26.6 representable range
//! 2. Font precedence maintained (SetFont before PutGlyph on each page)
//! 3. Coordinate stack balanced (PushStack count == PopStack count per page)
//! 4. Pages have at least one command

use ldir_ir::gir::{GIRDocument, GIROpcode};

use crate::fp266::{MAX_RAW, MIN_RAW};

/// Check G-IR document well-formedness per DEF-005.
///
/// Returns `Ok(())` if all conditions are met, `Err(messages)` with
/// descriptive strings for each violation found.
pub fn check_gir(doc: &GIRDocument) -> Result<(), Vec<String>> {
    let mut errors: Vec<String> = Vec::new();

    if doc.is_empty() {
        errors.push("GIR-WF-001: document has no pages".to_string());
        return Err(errors);
    }

    for (page_idx, page) in doc.iter().enumerate() {
        let prefix = format!("page {}", page_idx);

        if page.is_empty() {
            errors.push(format!("{}: GIR-WF-004: page has no commands", prefix));
        }

        check_coordinate_range(&prefix, page, &mut errors);
        check_font_precedence(&prefix, page, &mut errors);
        check_stack_balance(&prefix, page, &mut errors);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_coordinate_range(prefix: &str, page: &ldir_ir::gir::GIRPage, errors: &mut Vec<String>) {
    for (cmd_idx, cmd) in page.iter().enumerate() {
        let args = cmd.args();
        let coord_indices: &[usize] = match cmd.opcode() {
            GIROpcode::MoveXY => &[0, 1],
            GIROpcode::PutGlyph => &[1],
            GIROpcode::DrawRule => &[0, 1, 2, 3],
            _ => &[],
        };

        for &arg_idx in coord_indices {
            let val = args[arg_idx] as i64;
            if !(MIN_RAW..=MAX_RAW).contains(&val) {
                errors.push(format!(
                    "{} cmd {}: GIR-WF-001: coordinate arg[{}] = {} outside 26.6 range [{}, {}]",
                    prefix, cmd_idx, arg_idx, val, MIN_RAW, MAX_RAW,
                ));
            }
        }
    }
}

fn check_font_precedence(prefix: &str, page: &ldir_ir::gir::GIRPage, errors: &mut Vec<String>) {
    let mut has_font = false;
    for (cmd_idx, cmd) in page.iter().enumerate() {
        match cmd.opcode() {
            GIROpcode::SetFont => {
                has_font = true;
            }
            GIROpcode::PutGlyph if !has_font => {
                errors.push(format!(
                    "{} cmd {}: GIR-WF-002: PutGlyph without preceding SetFont",
                    prefix, cmd_idx
                ));
            }
            _ => {}
        }
    }
}

fn check_stack_balance(prefix: &str, page: &ldir_ir::gir::GIRPage, errors: &mut Vec<String>) {
    let mut depth: i32 = 0;
    for (cmd_idx, cmd) in page.iter().enumerate() {
        let delta = cmd.opcode().stack_delta();
        depth += delta;
        if depth < 0 {
            errors.push(format!(
                "{} cmd {}: GIR-WF-003: stack underflow (depth={})",
                prefix, cmd_idx, depth
            ));
        }
    }
    if depth != 0 {
        errors.push(format!(
            "{}: GIR-WF-003: unbalanced stack (final depth={})",
            prefix, depth
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldir_ir::gir::{GIRCommand, GIRPage};

    fn make_wellformed_doc() -> GIRDocument {
        let mut doc = GIRDocument::new();
        let mut page = GIRPage::new();
        page.push(GIRCommand::new_set_font(1));
        page.push(GIRCommand::new_push_stack());
        page.push(GIRCommand::new_move_xy(100, 200));
        page.push(GIRCommand::new_put_glyph(65, 640));
        page.push(GIRCommand::new_pop_stack());
        doc.push_page(page);
        doc
    }

    #[test]
    fn test_wellformed_doc_passes() {
        let doc = make_wellformed_doc();
        assert!(check_gir(&doc).is_ok());
    }

    #[test]
    fn test_empty_document_fails() {
        let doc = GIRDocument::new();
        let result = check_gir(&doc);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("GIR-WF-001")));
    }

    #[test]
    fn test_empty_page_fails() {
        let mut doc = GIRDocument::new();
        doc.push_page(GIRPage::new());
        let result = check_gir(&doc);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("GIR-WF-004")));
    }

    #[test]
    fn test_stack_unbalanced_fails() {
        let mut doc = GIRDocument::new();
        let mut page = GIRPage::new();
        page.push(GIRCommand::new_set_font(1));
        page.push(GIRCommand::new_push_stack());
        doc.push_page(page);
        let result = check_gir(&doc);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("GIR-WF-003")));
    }

    #[test]
    fn test_stack_underflow_fails() {
        let mut doc = GIRDocument::new();
        let mut page = GIRPage::new();
        page.push(GIRCommand::new_set_font(1));
        page.push(GIRCommand::new_pop_stack());
        doc.push_page(page);
        let result = check_gir(&doc);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("underflow")));
    }

    #[test]
    fn test_font_precedence_fails() {
        let mut doc = GIRDocument::new();
        let mut page = GIRPage::new();
        page.push(GIRCommand::new_put_glyph(65, 640));
        page.push(GIRCommand::new_set_font(1));
        page.push(GIRCommand::new_put_glyph(66, 640));
        doc.push_page(page);
        let result = check_gir(&doc);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("GIR-WF-002")));
    }

    #[test]
    fn test_font_before_glyph_ok() {
        let mut doc = GIRDocument::new();
        let mut page = GIRPage::new();
        page.push(GIRCommand::new_set_font(1));
        page.push(GIRCommand::new_put_glyph(65, 640));
        page.push(GIRCommand::new_put_glyph(66, 640));
        doc.push_page(page);
        assert!(check_gir(&doc).is_ok());
    }

    #[test]
    fn test_coordinate_all_i32_in_range() {
        let mut doc = GIRDocument::new();
        let mut page = GIRPage::new();
        page.push(GIRCommand::new_set_font(1));
        let mut cmd = GIRCommand::new_move_xy(i32::MIN, i32::MAX);
        page.push(cmd);
        page.push(GIRCommand::new_draw_rule(
            i32::MIN,
            i32::MAX,
            i32::MIN,
            i32::MAX,
        ));
        doc.push_page(page);
        assert!(
            check_gir(&doc).is_ok(),
            "all i32 values should be within 26.6 range"
        );
    }

    #[test]
    fn test_multiple_errors_collected() {
        let mut doc = GIRDocument::new();
        let mut page = GIRPage::new();
        page.push(GIRCommand::new_push_stack());
        page.push(GIRCommand::new_pop_stack());
        page.push(GIRCommand::new_pop_stack());
        doc.push_page(page);
        let result = check_gir(&doc);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(
            errors.len() >= 2,
            "expected >= 2 errors, got {}",
            errors.len()
        );
    }

    #[test]
    fn test_multi_page_wellformed() {
        let mut doc = GIRDocument::new();
        for _ in 0..3 {
            let mut page = GIRPage::new();
            page.push(GIRCommand::new_set_font(0));
            page.push(GIRCommand::new_put_glyph(65, 640));
            doc.push_page(page);
        }
        assert!(check_gir(&doc).is_ok());
    }

    #[test]
    fn test_multi_page_one_bad() {
        let mut doc = GIRDocument::new();
        let mut good_page = GIRPage::new();
        good_page.push(GIRCommand::new_set_font(0));
        good_page.push(GIRCommand::new_put_glyph(65, 640));
        doc.push_page(good_page);

        let mut bad_page = GIRPage::new();
        bad_page.push(GIRCommand::new_push_stack());
        doc.push_page(bad_page);

        let result = check_gir(&doc);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("page 1")));
    }

    #[test]
    fn test_draw_rule_coords_checked() {
        let mut doc = GIRDocument::new();
        let mut page = GIRPage::new();
        page.push(GIRCommand::new_set_font(0));
        page.push(GIRCommand::new_draw_rule(100, 200, 300, 10));
        doc.push_page(page);
        assert!(check_gir(&doc).is_ok());
    }

    #[test]
    fn test_attach_metadata_ignored() {
        let mut doc = GIRDocument::new();
        let mut page = GIRPage::new();
        page.push(GIRCommand::new_set_font(0));
        page.push(GIRCommand::new_attach_metadata(0, 10, 4, 8));
        page.push(GIRCommand::new_put_glyph(65, 640));
        doc.push_page(page);
        assert!(check_gir(&doc).is_ok());
    }

    #[test]
    fn test_compiled_doc_passes_verifier() {
        use ldir_ir::sir::{ROOT_SENTINEL, SIRDocument, SIRInstruction, SIROpcode};

        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::ApplyStyle, 1, 0, 0));
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 2, 1, 0));

        let gir = crate::compiler::compile_sir(&doc).unwrap();
        let result = check_gir(&gir);
        assert!(
            result.is_ok(),
            "compiled doc should pass verifier: {:?}",
            result.err()
        );
    }
}
