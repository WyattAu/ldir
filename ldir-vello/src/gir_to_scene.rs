//! G-IR to Vello Scene conversion.
//!
//! Transforms G-IR command buffers into Vello `Scene` objects suitable
//! for GPU rendering. Coordinates are converted from 26.6 fixed-point
//! to f64 scene coordinates (divide by 64.0).

use std::collections::HashMap;
use std::sync::Arc;

use ldir_ir::gir::{GIRDocument, GIROpcode};
use vello::peniko::{Blob, Color, Fill, Font};
use vello::peniko::kurbo::{Affine, Rect, RoundedRect};
use vello::{Glyph, Scene};

/// Scale factor for converting 26.6 fixed-point to scene units.
const FP266_SCALE: f64 = 64.0;

/// Map from font IDs to Vello font resources.
#[derive(Debug, Clone)]
pub struct FontMap {
    fonts: HashMap<usize, FontEntry>,
}

/// A single font entry in the font map.
#[derive(Debug, Clone)]
pub struct FontEntry {
    /// Vello font handle (wraps font data + collection index).
    pub font: Font,
    /// Font size in pixels per em for rendering.
    pub scale: f32,
}

impl FontMap {
    /// Create an empty font map.
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
        }
    }

    /// Build a font map from a list of (font_id, data, scale) tuples.
    ///
    /// Each entry maps a font ID to a Vello `Font` with the given raw data
    /// and rendering scale.
    pub fn from_fonts(fonts: &[(usize, Arc<Vec<u8>>, f32)]) -> Self {
        let mut map = HashMap::new();
        for &(id, ref data, scale) in fonts {
            let arc_dyn: Arc<dyn AsRef<[u8]> + Send + Sync> = Arc::clone(data) as _;
            let blob = Blob::new(arc_dyn);
            let font = Font::new(blob, 0);
            map.insert(id, FontEntry { font, scale });
        }
        Self { fonts: map }
    }

    /// Insert a font into the map.
    pub fn insert(&mut self, id: usize, font: Font, scale: f32) {
        self.fonts.insert(id, FontEntry { font, scale });
    }

    /// Look up a font entry by ID.
    pub fn get(&self, id: usize) -> Option<&FontEntry> {
        self.fonts.get(&id)
    }

    /// Look up the raw Vello `Font` by ID.
    pub fn get_font(&self, id: usize) -> Option<&Font> {
        self.fonts.get(&id).map(|e| &e.font)
    }

    /// Return the number of registered fonts.
    pub fn len(&self) -> usize {
        self.fonts.len()
    }

    /// Return true if no fonts are registered.
    pub fn is_empty(&self) -> bool {
        self.fonts.is_empty()
    }
}

impl Default for FontMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a single G-IR page to a Vello scene (without font data).
///
/// Glyph commands render placeholder rectangles. Use
/// [`gir_page_to_scene_with_fonts`] for real glyph outlines.
///
/// Processes all commands in order, building up the scene. Stack
/// operations (PushStack/PopStack) are tracked for transform state.
pub fn gir_page_to_scene(page: &ldir_ir::gir::GIRPage) -> Scene {
    gir_page_to_scene_inner(page, None)
}

/// Convert a single G-IR page to a Vello scene with font data.
///
/// When fonts are provided, `PutGlyph` commands use Vello's
/// `draw_glyphs` API for real glyph outline rendering. Glyphs are
/// batched into runs per font.
pub fn gir_page_to_scene_with_fonts(page: &ldir_ir::gir::GIRPage, fonts: &FontMap) -> Scene {
    gir_page_to_scene_inner(page, Some(fonts))
}

fn gir_page_to_scene_inner(page: &ldir_ir::gir::GIRPage, fonts: Option<&FontMap>) -> Scene {
    let mut scene = Scene::new();
    let mut transform_stack: Vec<Affine> = Vec::new();
    let mut current_transform = Affine::IDENTITY;
    let mut current_font_id: i32 = 0;
    let mut cursor_x: f64 = 0.0;
    let mut cursor_y: f64 = 0.0;

    // Glyph batching state
    let mut glyph_run_font_id: Option<i32> = None;
    let mut glyph_run_x_start: f64 = 0.0;
    let mut pending_glyphs: Vec<Glyph> = Vec::new();

    fn flush_glyph_run(
        scene: &mut Scene,
        transform: Affine,
        font_id: i32,
        x_start: f64,
        y: f64,
        glyphs: &mut Vec<Glyph>,
        fonts: Option<&FontMap>,
    ) {
        if glyphs.is_empty() {
            return;
        }
        if let Some(fonts) = fonts {
            if let Some(entry) = fonts.get(font_id as usize) {
                let builder = scene
                    .draw_glyphs(&entry.font)
                    .transform(Affine::translate((x_start, y)))
                    .font_size(entry.scale)
                    .brush(Color::BLACK);
                builder.draw(Fill::NonZero, glyphs.drain(..));
            }
        } else {
            for glyph in glyphs.drain(..) {
                let tx = transform * Affine::translate((x_start + glyph.x as f64, y));
                let glyph_rect = RoundedRect::new(0.0, 0.0, 10.0, 12.0, 0.0);
                scene.fill(Fill::NonZero, tx, Color::BLACK, None, &glyph_rect);
            }
        }
        glyphs.clear();
    }

    for cmd in page.iter() {
        match cmd.opcode() {
            GIROpcode::SetFont => {
                if let Some(font_id) = cmd.arg(0) {
                    if glyph_run_font_id.is_some() {
                        flush_glyph_run(
                            &mut scene,
                            current_transform,
                            glyph_run_font_id.unwrap(),
                            glyph_run_x_start,
                            cursor_y,
                            &mut pending_glyphs,
                            fonts,
                        );
                        glyph_run_font_id = None;
                    }
                    current_font_id = font_id;
                }
            }
            GIROpcode::MoveXY => {
                let x_fp = cmd.arg(0).unwrap_or(0) as f64 / FP266_SCALE;
                let y_fp = cmd.arg(1).unwrap_or(0) as f64 / FP266_SCALE;
                if glyph_run_font_id.is_some() {
                    flush_glyph_run(
                        &mut scene,
                        current_transform,
                        glyph_run_font_id.unwrap(),
                        glyph_run_x_start,
                        cursor_y,
                        &mut pending_glyphs,
                        fonts,
                    );
                    glyph_run_font_id = None;
                }
                cursor_x = x_fp;
                cursor_y = y_fp;
            }
            GIROpcode::PutGlyph => {
                let glyph_id = cmd.arg(0).unwrap_or(0) as u32;
                let advance_x = cmd.arg(1).unwrap_or(0) as f64 / FP266_SCALE;

                if glyph_run_font_id != Some(current_font_id) {
                    flush_glyph_run(
                        &mut scene,
                        current_transform,
                        glyph_run_font_id.unwrap_or(current_font_id),
                        glyph_run_x_start,
                        cursor_y,
                        &mut pending_glyphs,
                        fonts,
                    );
                    glyph_run_font_id = Some(current_font_id);
                    glyph_run_x_start = cursor_x;
                }

                pending_glyphs.push(Glyph {
                    id: glyph_id,
                    x: (cursor_x - glyph_run_x_start) as f32,
                    y: 0.0,
                });

                cursor_x += advance_x;
            }
            GIROpcode::DrawRule => {
                if glyph_run_font_id.is_some() {
                    flush_glyph_run(
                        &mut scene,
                        current_transform,
                        glyph_run_font_id.unwrap(),
                        glyph_run_x_start,
                        cursor_y,
                        &mut pending_glyphs,
                        fonts,
                    );
                    glyph_run_font_id = None;
                }
                let x_fp = cmd.arg(0).unwrap_or(0) as f64 / FP266_SCALE;
                let y_fp = cmd.arg(1).unwrap_or(0) as f64 / FP266_SCALE;
                let w_fp = cmd.arg(2).unwrap_or(0) as f64 / FP266_SCALE;
                let thickness_fp = cmd.arg(3).unwrap_or(0) as f64 / FP266_SCALE;

                let tx = current_transform * Affine::translate((x_fp, y_fp));
                let rect = Rect::new(0.0, 0.0, w_fp, thickness_fp);

                scene.fill(Fill::NonZero, tx, Color::BLACK, None, &rect);
            }
            GIROpcode::PushStack => {
                if glyph_run_font_id.is_some() {
                    flush_glyph_run(
                        &mut scene,
                        current_transform,
                        glyph_run_font_id.unwrap(),
                        glyph_run_x_start,
                        cursor_y,
                        &mut pending_glyphs,
                        fonts,
                    );
                    glyph_run_font_id = None;
                }
                transform_stack.push(current_transform);
            }
            GIROpcode::PopStack => {
                if glyph_run_font_id.is_some() {
                    flush_glyph_run(
                        &mut scene,
                        current_transform,
                        glyph_run_font_id.unwrap(),
                        glyph_run_x_start,
                        cursor_y,
                        &mut pending_glyphs,
                        fonts,
                    );
                    glyph_run_font_id = None;
                }
                if let Some(prev) = transform_stack.pop() {
                    current_transform = prev;
                }
            }
            GIROpcode::AttachMetadata => {
                let _key_offset = cmd.arg(0).unwrap_or(0);
                let _val_offset = cmd.arg(1).unwrap_or(0);
                let _key_len = cmd.arg(2).unwrap_or(0);
                let _val_len = cmd.arg(3).unwrap_or(0);
            }
        }
    }

    // Flush any remaining glyphs
    if let Some(font_id) = glyph_run_font_id {
        flush_glyph_run(
            &mut scene,
            current_transform,
            font_id,
            glyph_run_x_start,
            cursor_y,
            &mut pending_glyphs,
            fonts,
        );
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

/// Convert a full G-IR document to a per-page list of Vello scenes.
///
/// Uses font data for real glyph rendering when available.
pub fn gir_doc_to_scenes(doc: &GIRDocument, fonts: &FontMap) -> Vec<Scene> {
    doc.iter()
        .map(|page| gir_page_to_scene_with_fonts(page, fonts))
        .collect()
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
    fn empty_page_to_scene() {
        let page = GIRPage::new();
        let scene = gir_page_to_scene(&page);
        assert!(scene.encoding().is_empty());
    }

    #[test]
    fn empty_page_to_scene_with_fonts() {
        let page = GIRPage::new();
        let fonts = FontMap::new();
        let scene = gir_page_to_scene_with_fonts(&page, &fonts);
        assert!(scene.encoding().is_empty());
    }

    #[test]
    fn move_xy_command() {
        let mut page = GIRPage::new();
        page.push(GIRCommand::new_move_xy(50 * 64, 100 * 64));
        let scene = gir_page_to_scene(&page);
        assert!(scene.encoding().is_empty());
    }

    #[test]
    fn draw_rule_command() {
        let mut page = GIRPage::new();
        page.push(GIRCommand::new_draw_rule(10 * 64, 20 * 64, 200 * 64, 2 * 64));
        let scene = gir_page_to_scene(&page);
        assert!(!scene.encoding().is_empty());
    }

    #[test]
    fn put_glyph_command() {
        let mut page = GIRPage::new();
        page.push(GIRCommand::new_set_font(1));
        page.push(GIRCommand::new_move_xy(10 * 64, 10 * 64));
        page.push(GIRCommand::new_put_glyph(65, 10 * 64));
        let scene = gir_page_to_scene(&page);
        assert!(!scene.encoding().is_empty());
    }

    #[test]
    fn put_glyph_command_with_fonts() {
        let mut page = GIRPage::new();
        page.push(GIRCommand::new_set_font(1));
        page.push(GIRCommand::new_move_xy(10 * 64, 10 * 64));
        page.push(GIRCommand::new_put_glyph(65, 10 * 64));
        let fonts = FontMap::new();
        let scene = gir_page_to_scene_with_fonts(&page, &fonts);
        // No font registered, so glyphs produce no output
        assert!(scene.encoding().is_empty());
    }

    #[test]
    fn font_map_creation() {
        let fonts = FontMap::new();
        assert!(fonts.is_empty());
        assert_eq!(fonts.len(), 0);
        assert!(fonts.get(0).is_none());
    }

    #[test]
    fn font_map_from_fonts() {
        let data = Arc::new(vec![0u8; 64]);
        let fonts = FontMap::from_fonts(&[(0, Arc::clone(&data), 12.0)]);
        assert!(!fonts.is_empty());
        assert_eq!(fonts.len(), 1);
        assert!(fonts.get(0).is_some());
        assert!(fonts.get_font(0).is_some());
        assert_eq!(fonts.get(0).unwrap().scale, 12.0);
    }

    #[test]
    fn font_map_insert() {
        let mut fonts = FontMap::new();
        let data: Arc<dyn AsRef<[u8]> + Send + Sync> = Arc::new(vec![0u8; 64]);
        let blob = Blob::new(Arc::clone(&data));
        let font = Font::new(blob, 0);
        fonts.insert(42, font, 16.0);
        assert_eq!(fonts.len(), 1);
        assert_eq!(fonts.get(42).unwrap().scale, 16.0);
    }

    #[test]
    fn multi_page_renderer() {
        let mut doc = GIRDocument::new();
        let mut page1 = GIRPage::new();
        page1.push(GIRCommand::new_draw_rule(0, 0, 100 * 64, 2 * 64));
        doc.push_page(page1);
        let mut page2 = GIRPage::new();
        page2.push(GIRCommand::new_draw_rule(0, 0, 200 * 64, 2 * 64));
        doc.push_page(page2);
        let fonts = FontMap::new();
        let scenes = gir_doc_to_scenes(&doc, &fonts);
        assert_eq!(scenes.len(), 2);
        assert!(!scenes[0].encoding().is_empty());
        assert!(!scenes[1].encoding().is_empty());
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

    #[test]
    fn test_consecutive_glyphs_batched() {
        let mut page = GIRPage::new();
        page.push(GIRCommand::new_set_font(1));
        page.push(GIRCommand::new_move_xy(0, 0));
        page.push(GIRCommand::new_put_glyph(65, 10 * 64));
        page.push(GIRCommand::new_put_glyph(66, 10 * 64));
        page.push(GIRCommand::new_put_glyph(67, 10 * 64));
        let scene = gir_page_to_scene(&page);
        assert!(!scene.encoding().is_empty());
    }

    #[test]
    fn test_font_switch_flushes_glyph_run() {
        let mut page = GIRPage::new();
        page.push(GIRCommand::new_set_font(1));
        page.push(GIRCommand::new_move_xy(0, 0));
        page.push(GIRCommand::new_put_glyph(65, 10 * 64));
        page.push(GIRCommand::new_set_font(2));
        page.push(GIRCommand::new_put_glyph(65, 10 * 64));
        let scene = gir_page_to_scene(&page);
        assert!(!scene.encoding().is_empty());
    }

    #[test]
    fn test_gir_doc_to_scenes_empty() {
        let doc = GIRDocument::new();
        let fonts = FontMap::new();
        let scenes = gir_doc_to_scenes(&doc, &fonts);
        assert!(scenes.is_empty());
    }
}
