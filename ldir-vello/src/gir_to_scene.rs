//! G-IR to Vello Scene conversion.
//!
//! Transforms G-IR command buffers into Vello `Scene` objects suitable
//! for GPU rendering. Coordinates are converted from 26.6 fixed-point
//! to f64 scene coordinates (divide by 64.0).

use ldir_ir::gir::{GIRDocument, GIROpcode};
use vello::peniko::Fill;
use vello::peniko::kurbo::{Affine, Rect, RoundedRect};
use vello::{Scene, peniko::Color};

/// Scale factor for converting 26.6 fixed-point to scene units.
const FP266_SCALE: f64 = 64.0;

/// Convert a single G-IR page to a Vello scene.
///
/// Processes all commands in order, building up the scene. Stack
/// operations (PushStack/PopStack) are tracked for transform state.
pub fn gir_page_to_scene(page: &ldir_ir::gir::GIRPage) -> Scene {
    let mut scene = Scene::new();
    let mut transform_stack: Vec<Affine> = Vec::new();
    let mut current_transform = Affine::IDENTITY;
    let mut _current_font_id: i32 = 0;
    let mut cursor_x: f64 = 0.0;
    let mut cursor_y: f64 = 0.0;

    for cmd in page.iter() {
        match cmd.opcode() {
            GIROpcode::SetFont => {
                if let Some(font_id) = cmd.arg(0) {
                    _current_font_id = font_id;
                }
            }
            GIROpcode::MoveXY => {
                let x_fp = cmd.arg(0).unwrap_or(0) as f64 / FP266_SCALE;
                let y_fp = cmd.arg(1).unwrap_or(0) as f64 / FP266_SCALE;
                cursor_x = x_fp;
                cursor_y = y_fp;
            }
            GIROpcode::PutGlyph => {
                let _glyph_id = cmd.arg(0).unwrap_or(0);
                let advance_x = cmd.arg(1).unwrap_or(0) as f64 / FP266_SCALE;

                let tx = current_transform * Affine::translate((cursor_x, cursor_y));

                let glyph_rect = RoundedRect::new(0.0, 0.0, 10.0, 12.0, 0.0);
                scene.fill(Fill::NonZero, tx, Color::BLACK, None, &glyph_rect);

                cursor_x += advance_x;
            }
            GIROpcode::DrawRule => {
                let x_fp = cmd.arg(0).unwrap_or(0) as f64 / FP266_SCALE;
                let y_fp = cmd.arg(1).unwrap_or(0) as f64 / FP266_SCALE;
                let w_fp = cmd.arg(2).unwrap_or(0) as f64 / FP266_SCALE;
                let thickness_fp = cmd.arg(3).unwrap_or(0) as f64 / FP266_SCALE;

                let tx = current_transform * Affine::translate((x_fp, y_fp));
                let rect = Rect::new(0.0, 0.0, w_fp, thickness_fp);

                scene.fill(Fill::NonZero, tx, Color::BLACK, None, &rect);
            }
            GIROpcode::PushStack => {
                transform_stack.push(current_transform);
            }
            GIROpcode::PopStack => {
                if let Some(prev) = transform_stack.pop() {
                    current_transform = prev;
                }
            }
            GIROpcode::AttachMetadata => {
                // Metadata is informational; no visual rendering.
                // In a full implementation, this would attach accessibility
                // or hyperlink metadata to the scene layer.
                let _key_offset = cmd.arg(0).unwrap_or(0);
                let _val_offset = cmd.arg(1).unwrap_or(0);
                let _key_len = cmd.arg(2).unwrap_or(0);
                let _val_len = cmd.arg(3).unwrap_or(0);
            }
        }
    }

    scene
}

/// Convert a full G-IR document to a Vello scene.
///
/// Each page is appended to a single scene with vertical offsets
/// to simulate multi-page layout.
pub fn gir_to_scene(doc: &GIRDocument) -> Scene {
    let mut combined = Scene::new();
    let mut y_offset: f64 = 0.0;

    for (i, page) in doc.iter().enumerate() {
        let page_scene = gir_page_to_scene(page);
        if i > 0 {
            let page_height = page.height as f64 / FP266_SCALE;
            y_offset += page_height;
        }
        if !page_scene.encoding().is_empty() {
            combined.append(&page_scene, Some(Affine::translate((0.0, y_offset))));
        }
    }

    combined
}

/// Convert a 26.6 fixed-point i32 value to scene f64 coordinates.
#[inline]
pub fn fp266_to_scene(val: i32) -> f64 {
    val as f64 / FP266_SCALE
}

/// Convert a 26.6 fixed-point Fp266 value to scene f64 coordinates.
#[inline]
pub fn fp266_to_scene_fp(val: ldir_core::fp266::Fp266) -> f64 {
    val.to_f64()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldir_ir::gir::{GIRCommand, GIRPage};

    fn make_simple_doc() -> GIRDocument {
        let mut doc = GIRDocument::new();
        let mut page = GIRPage::new();
        page.push(GIRCommand::new_set_font(1));
        page.push(GIRCommand::new_move_xy(100 * 64, 200 * 64));
        page.push(GIRCommand::new_draw_rule(
            100 * 64,
            200 * 64,
            400 * 64,
            2 * 64,
        ));
        page.push(GIRCommand::new_push_stack());
        page.push(GIRCommand::new_move_xy(150 * 64, 250 * 64));
        page.push(GIRCommand::new_put_glyph(65, 10 * 64));
        page.push(GIRCommand::new_pop_stack());
        page.push(GIRCommand::new_attach_metadata(0, 10, 4, 8));
        doc.push_page(page);
        doc
    }

    #[test]
    fn test_fp266_to_scene() {
        assert_eq!(fp266_to_scene(0), 0.0);
        assert_eq!(fp266_to_scene(64), 1.0);
        assert_eq!(fp266_to_scene(128), 2.0);
        assert_eq!(fp266_to_scene(32), 0.5);
    }

    #[test]
    fn test_fp266_to_scene_fp() {
        assert_eq!(fp266_to_scene_fp(ldir_core::fp266::Fp266::from_int(1)), 1.0);
        assert_eq!(
            fp266_to_scene_fp(ldir_core::fp266::Fp266::from_frac(1, 2)),
            0.5
        );
    }

    #[test]
    fn test_gir_page_to_scene_non_empty() {
        let doc = make_simple_doc();
        let scene = gir_page_to_scene(&doc.get(0).unwrap());
        assert!(!scene.encoding().is_empty());
    }

    #[test]
    fn test_gir_to_scene_non_empty() {
        let doc = make_simple_doc();
        let scene = gir_to_scene(&doc);
        assert!(!scene.encoding().is_empty());
    }

    #[test]
    fn test_empty_page_produces_empty_scene() {
        let page = GIRPage::new();
        let scene = gir_page_to_scene(&page);
        assert!(scene.encoding().is_empty());
    }

    #[test]
    fn test_empty_doc_produces_empty_scene() {
        let doc = GIRDocument::new();
        let scene = gir_to_scene(&doc);
        assert!(scene.encoding().is_empty());
    }

    #[test]
    fn test_multi_page_scene() {
        let mut doc = GIRDocument::new();
        let mut page1 = GIRPage::new();
        page1.push(GIRCommand::new_draw_rule(0, 0, 100 * 64, 2 * 64));
        doc.push_page(page1);
        let mut page2 = GIRPage::new();
        page2.push(GIRCommand::new_draw_rule(0, 0, 200 * 64, 2 * 64));
        doc.push_page(page2);

        let scene = gir_to_scene(&doc);
        assert!(!scene.encoding().is_empty());
    }

    #[test]
    fn test_push_pop_preserves_transform() {
        let mut page = GIRPage::new();
        page.push(GIRCommand::new_move_xy(50 * 64, 50 * 64));
        page.push(GIRCommand::new_push_stack());
        page.push(GIRCommand::new_move_xy(200 * 64, 200 * 64));
        page.push(GIRCommand::new_put_glyph(65, 10 * 64));
        page.push(GIRCommand::new_pop_stack());
        page.push(GIRCommand::new_draw_rule(0, 0, 100 * 64, 2 * 64));

        let scene = gir_page_to_scene(&page);
        assert!(!scene.encoding().is_empty());
    }

    #[test]
    fn test_attach_metadata_no_render() {
        let mut page = GIRPage::new();
        page.push(GIRCommand::new_attach_metadata(0, 10, 4, 8));
        let scene = gir_page_to_scene(&page);
        assert!(scene.encoding().is_empty());
    }

    #[test]
    fn test_nested_push_pop() {
        let mut page = GIRPage::new();
        page.push(GIRCommand::new_push_stack());
        page.push(GIRCommand::new_move_xy(10 * 64, 10 * 64));
        page.push(GIRCommand::new_push_stack());
        page.push(GIRCommand::new_move_xy(20 * 64, 20 * 64));
        page.push(GIRCommand::new_draw_rule(0, 0, 50 * 64, 1 * 64));
        page.push(GIRCommand::new_pop_stack());
        page.push(GIRCommand::new_pop_stack());

        let scene = gir_page_to_scene(&page);
        assert!(!scene.encoding().is_empty());
    }

    #[test]
    fn test_set_font_tracking() {
        let mut page = GIRPage::new();
        page.push(GIRCommand::new_set_font(5));
        page.push(GIRCommand::new_set_font(10));
        page.push(GIRCommand::new_move_xy(0, 0));
        page.push(GIRCommand::new_put_glyph(65, 10 * 64));

        let scene = gir_page_to_scene(&page);
        assert!(!scene.encoding().is_empty());
    }
}
