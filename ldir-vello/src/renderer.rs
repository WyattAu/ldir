//! Vello/GPU Renderer for G-IR documents.
//!
//! `VelloRenderer` wraps Vello scene construction and rendering,
//! converting G-IR command buffers into pixel output via the GPU.

use ldir_core::error::{CompileErrorKind, ErrorKind, LdirError, Result};
use ldir_ir::gir::GIRDocument;

/// Wrapper around Vello scene construction and GPU rendering.
///
/// Provides methods to convert G-IR documents to Vello scenes and
/// (when a GPU device is available) render them to pixel buffers.
pub struct VelloRenderer {
    #[allow(dead_code)]
    device_label: String,
}

impl VelloRenderer {
    /// Create a new renderer instance.
    ///
    /// In headless environments (no GPU), this creates a software-only
    /// renderer that can build scenes but cannot produce pixel output.
    pub fn new() -> Result<Self> {
        Ok(Self {
            device_label: "headless".to_string(),
        })
    }

    /// Create a new renderer with a custom device label.
    pub fn with_label(label: &str) -> Result<Self> {
        Ok(Self {
            device_label: label.to_string(),
        })
    }

    /// Build a Vello scene from a G-IR document.
    ///
    /// This is a pure data transformation (no GPU required) that
    /// converts the G-IR command buffer into a Vello scene graph.
    pub fn build_scene(&self, doc: &GIRDocument) -> Result<vello::Scene> {
        let scene = crate::gir_to_scene::gir_to_scene(doc);
        Ok(scene)
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

    /// Check if this renderer has an active GPU device.
    pub fn has_device(&self) -> bool {
        self.device_label != "headless"
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
}
