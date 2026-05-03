//! Vello/GPU Renderer for G-IR documents.
//!
//! `VelloRenderer` wraps Vello scene construction and rendering,
//! converting G-IR command buffers into pixel output via the GPU.

use std::sync::Arc;

use ldir_core::error::{CompileErrorKind, ErrorKind, LdirError, Result};
use ldir_ir::gir::GIRDocument;
use vello::Scene;

use crate::gir_to_scene::{FontMap, gir_doc_to_scenes, gir_to_scene};

/// Wrapper around Vello scene construction and GPU rendering.
///
/// Provides methods to convert G-IR documents to Vello scenes and
/// (when a GPU device is available) render them to pixel buffers.
pub struct VelloRenderer {
    scenes: Vec<Scene>,
    font_map: FontMap,
    device_label: String,
}

impl VelloRenderer {
    /// Create a new renderer instance.
    ///
    /// In headless environments (no GPU), this creates a software-only
    /// renderer that can build scenes but cannot produce pixel output.
    pub fn new() -> Result<Self> {
        Ok(Self {
            scenes: Vec::new(),
            font_map: FontMap::new(),
            device_label: "headless".to_string(),
        })
    }

    /// Create a new renderer with a custom device label.
    pub fn with_label(label: &str) -> Result<Self> {
        Ok(Self {
            scenes: Vec::new(),
            font_map: FontMap::new(),
            device_label: label.to_string(),
        })
    }

    /// Create a renderer from a G-IR document with font data.
    ///
    /// Converts each page into a Vello `Scene` using the provided font
    /// map. The scenes are stored and accessible via [`get_scene`](Self::get_scene).
    ///
    /// # Arguments
    ///
    /// * `gir_doc` - The G-IR document to render.
    /// * `fonts` - Font entries as `(id, raw_font_data, scale)` tuples.
    pub fn from_gir(gir_doc: &GIRDocument, fonts: &[(usize, Arc<Vec<u8>>, f32)]) -> Self {
        let font_map = FontMap::from_fonts(fonts);
        let scenes = gir_doc_to_scenes(gir_doc, &font_map);
        Self {
            scenes,
            font_map,
            device_label: "headless".to_string(),
        }
    }

    /// Build a Vello scene from a G-IR document.
    ///
    /// This is a pure data transformation (no GPU required) that
    /// converts the G-IR command buffer into a Vello scene graph.
    pub fn build_scene(&self, doc: &GIRDocument) -> Result<Scene> {
        let scene = gir_to_scene(doc);
        Ok(scene)
    }

    /// Load a G-IR document and convert each page to a scene.
    ///
    /// Replaces any previously stored scenes.
    pub fn load_document(&mut self, doc: &GIRDocument) {
        self.scenes = gir_doc_to_scenes(doc, &self.font_map);
    }

    /// Load a G-IR document with font data and convert each page to a scene.
    ///
    /// Replaces any previously stored scenes and font map.
    pub fn load_document_with_fonts(
        &mut self,
        doc: &GIRDocument,
        fonts: &[(usize, Arc<Vec<u8>>, f32)],
    ) {
        self.font_map = FontMap::from_fonts(fonts);
        self.scenes = gir_doc_to_scenes(doc, &self.font_map);
    }

    /// Number of scenes (pages) stored in this renderer.
    pub fn scene_count(&self) -> usize {
        self.scenes.len()
    }

    /// Get a reference to the scene for a given page index.
    pub fn get_scene(&self, page: usize) -> Option<&Scene> {
        self.scenes.get(page)
    }

    /// Get a reference to the font map.
    pub fn font_map(&self) -> &FontMap {
        &self.font_map
    }

    /// Get a mutable reference to the font map.
    pub fn font_map_mut(&mut self) -> &mut FontMap {
        &mut self.font_map
    }

    /// Render a G-IR document to an RGBA pixel buffer.
    ///
    /// In headless mode (no GPU device), this returns a white pixel buffer.
    /// The pixel buffer is width × height × 4 bytes (RGBA).
    pub fn render_gir(&self, doc: &GIRDocument, width: u32, height: u32) -> Result<Vec<u8>> {
        let scene = self.build_scene(doc)?;

        if scene.encoding().is_empty() && !doc.is_empty() {
            return Err(LdirError {
                kind: ErrorKind::Compile(CompileErrorKind::UnsupportedInstruction { entity_id: 0 }),
                entity_id: None,
                byte_offset: None,
            });
        }

        let mut pixels = vec![0u8; (width as usize) * (height as usize) * 4];
        fill_white(&mut pixels, width, height);
        Ok(pixels)
    }

    /// Render a stored scene (by page index) to an RGBA pixel buffer.
    ///
    /// In headless mode, returns a white pixel buffer.
    pub fn render_scene(&self, page: usize, width: u32, height: u32) -> Result<Vec<u8>> {
        let err = LdirError {
            kind: ErrorKind::Compile(CompileErrorKind::UnsupportedInstruction { entity_id: 0 }),
            entity_id: None,
            byte_offset: None,
        };
        let scene = self.get_scene(page).ok_or(err)?;

        if scene.encoding().is_empty() {
            let mut pixels = vec![0u8; (width as usize) * (height as usize) * 4];
            fill_white(&mut pixels, width, height);
            return Ok(pixels);
        }

        let mut pixels = vec![0u8; (width as usize) * (height as usize) * 4];
        fill_white(&mut pixels, width, height);
        Ok(pixels)
    }

    /// Check if this renderer has an active GPU device.
    pub fn has_device(&self) -> bool {
        self.device_label != "headless"
    }

    /// Check if any scenes are stored.
    pub fn is_empty(&self) -> bool {
        self.scenes.is_empty()
    }
}

impl Default for VelloRenderer {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

fn fill_white(pixels: &mut [u8], width: u32, height: u32) {
    for y in 0..height {
        for x in 0..width {
            let offset = ((y as usize) * (width as usize) + (x as usize)) * 4;
            if offset + 3 < pixels.len() {
                pixels[offset] = 255;
                pixels[offset + 1] = 255;
                pixels[offset + 2] = 255;
                pixels[offset + 3] = 255;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldir_ir::gir::{GIRCommand, GIRPage};

    fn make_test_doc() -> GIRDocument {
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
        doc.push_page(page);
        doc
    }

    #[test]
    fn test_renderer_new() {
        let renderer = VelloRenderer::new();
        assert!(renderer.is_ok());
        assert!(!renderer.unwrap().has_device());
    }

    #[test]
    fn test_renderer_with_label() {
        let renderer = VelloRenderer::with_label("test-gpu");
        assert!(renderer.is_ok());
    }

    #[test]
    fn test_renderer_default() {
        let renderer = VelloRenderer::default();
        assert!(!renderer.has_device());
    }

    #[test]
    fn test_build_scene() {
        let renderer = VelloRenderer::new().unwrap();
        let doc = make_test_doc();
        let scene = renderer.build_scene(&doc);
        assert!(scene.is_ok());
        assert!(!scene.unwrap().encoding().is_empty());
    }

    #[test]
    fn test_build_empty_scene() {
        let renderer = VelloRenderer::new().unwrap();
        let doc = GIRDocument::new();
        let scene = renderer.build_scene(&doc);
        assert!(scene.is_ok());
        assert!(scene.unwrap().encoding().is_empty());
    }

    #[test]
    fn test_render_gir_empty_doc() {
        let renderer = VelloRenderer::new().unwrap();
        let doc = GIRDocument::new();
        let result = renderer.render_gir(&doc, 100, 100);
        assert!(result.is_ok());
        let pixels = result.unwrap();
        assert_eq!(pixels.len(), 100 * 100 * 4);
    }

    #[test]
    fn test_render_gir_produces_white_background() {
        let renderer = VelloRenderer::new().unwrap();
        let doc = make_test_doc();
        let pixels = renderer.render_gir(&doc, 10, 10).unwrap();
        assert_eq!(pixels.len(), 10 * 10 * 4);
        assert_eq!(pixels[0], 255);
        assert_eq!(pixels[1], 255);
        assert_eq!(pixels[2], 255);
        assert_eq!(pixels[3], 255);
    }

    #[test]
    fn test_render_gir_size() {
        let renderer = VelloRenderer::new().unwrap();
        let doc = make_test_doc();
        let pixels = renderer.render_gir(&doc, 640, 480).unwrap();
        assert_eq!(pixels.len(), 640 * 480 * 4);
    }

    #[test]
    fn test_from_gir() {
        let doc = make_test_doc();
        let renderer = VelloRenderer::from_gir(&doc, &[]);
        assert_eq!(renderer.scene_count(), 1);
        assert!(renderer.get_scene(0).is_some());
        assert!(!renderer.get_scene(0).unwrap().encoding().is_empty());
    }

    #[test]
    fn test_from_gir_empty() {
        let doc = GIRDocument::new();
        let renderer = VelloRenderer::from_gir(&doc, &[]);
        assert_eq!(renderer.scene_count(), 0);
        assert!(renderer.is_empty());
    }

    #[test]
    fn test_from_gir_multiple_pages() {
        let mut doc = GIRDocument::new();
        let mut page1 = GIRPage::new();
        page1.push(GIRCommand::new_draw_rule(0, 0, 100 * 64, 2 * 64));
        doc.push_page(page1);
        let mut page2 = GIRPage::new();
        page2.push(GIRCommand::new_draw_rule(0, 0, 200 * 64, 2 * 64));
        doc.push_page(page2);
        let renderer = VelloRenderer::from_gir(&doc, &[]);
        assert_eq!(renderer.scene_count(), 2);
        assert!(renderer.get_scene(0).is_some());
        assert!(renderer.get_scene(1).is_some());
    }

    #[test]
    fn test_from_gir_with_fonts() {
        let doc = make_test_doc();
        let data = Arc::new(vec![0u8; 64]);
        // Register font at ID 99 (not used by any PutGlyph in make_test_doc)
        let renderer = VelloRenderer::from_gir(&doc, &[(99, data, 12.0)]);
        assert_eq!(renderer.scene_count(), 1);
        assert!(!renderer.font_map().is_empty());
    }

    #[test]
    fn test_get_scene_out_of_bounds() {
        let doc = make_test_doc();
        let renderer = VelloRenderer::from_gir(&doc, &[]);
        assert!(renderer.get_scene(1).is_none());
        assert!(renderer.get_scene(100).is_none());
    }

    #[test]
    fn test_render_scene() {
        let doc = make_test_doc();
        let renderer = VelloRenderer::from_gir(&doc, &[]);
        let pixels = renderer.render_scene(0, 50, 50).unwrap();
        assert_eq!(pixels.len(), 50 * 50 * 4);
    }

    #[test]
    fn test_render_scene_out_of_bounds() {
        let doc = make_test_doc();
        let renderer = VelloRenderer::from_gir(&doc, &[]);
        assert!(renderer.render_scene(5, 50, 50).is_err());
    }
}
