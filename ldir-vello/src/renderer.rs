//! Vello/GPU Renderer for G-IR documents.
//!
//! `VelloRenderer` wraps Vello scene construction and rendering,
//! converting G-IR command buffers into pixel output via the GPU.
//!
//! When the `gpu` feature is enabled and a GPU device is available,
//! scenes are rasterized using Vello's compute shader pipeline.
//! Otherwise, the renderer operates in headless mode and returns
//! white pixel buffers.

#[cfg(feature = "gpu")]
use std::num::NonZeroUsize;
use std::sync::Arc;

use ldir_core::error::{CompileErrorKind, ErrorKind, LdirError, Result};
use ldir_ir::gir::GIRDocument;
use vello::Scene;

#[cfg(feature = "gpu")]
use vello::{
    AaConfig, AaSupport, RenderParams, Renderer as VelloRendererImpl, RendererOptions,
    peniko::Color,
};

use crate::gir_to_scene::{FontMap, gir_doc_to_scenes, gir_to_scene};
use crate::viewport::Viewport;

/// Wrapper around Vello scene construction and GPU rendering.
///
/// Provides methods to convert G-IR documents to Vello scenes and
/// (when a GPU device is available) render them to pixel buffers.
///
/// # Feature Gates
///
/// - `gpu` feature: enables GPU rasterization via wgpu + Vello compute shaders.
/// - `software` feature (default): headless mode, white buffer fallback.
pub struct VelloRenderer {
    scenes: Vec<Scene>,
    font_map: FontMap,
    viewport: Viewport,
    device_label: String,
    #[cfg(feature = "gpu")]
    gpu: Option<GpuState>,
}

#[cfg(feature = "gpu")]
struct GpuState {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    renderer: std::cell::RefCell<VelloRendererImpl>,
}

#[cfg(feature = "gpu")]
impl GpuState {
    /// Initialize GPU state with a wgpu device.
    ///
    /// Uses `wgpu::Instance::new()` to discover available adapters.
    /// Prefers high-performance GPUs. Falls back to any available adapter
    /// that supports compute shaders.
    fn new() -> std::result::Result<Self, String> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .ok_or_else(|| "no suitable GPU adapter found".to_string())?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("ldir-vello"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            },
            None,
        ))
        .map_err(|e| format!("failed to create wgpu device: {e}"))?;

        let renderer = VelloRendererImpl::new(
            &device,
            RendererOptions {
                surface_format: None,
                use_cpu: false,
                antialiasing_support: AaSupport::area_only(),
                num_init_threads: NonZeroUsize::new(1),
            },
        )
        .map_err(|e| format!("failed to create Vello renderer: {e}"))?;

        Ok(Self {
            device: Arc::new(device),
            queue: Arc::new(queue),
            renderer: std::cell::RefCell::new(renderer),
        })
    }
}

impl VelloRenderer {
    /// Create a new renderer instance.
    ///
    /// Attempts to initialize GPU rendering when the `gpu` feature is enabled.
    /// Falls back to headless mode if GPU initialization fails.
    ///
    /// # Errors
    ///
    /// Returns an error only if the `gpu` feature is enabled and GPU
    /// initialization is explicitly required but fails. Currently always
    /// falls back gracefully.
    pub fn new() -> Result<Self> {
        #[cfg(feature = "gpu")]
        let gpu = match GpuState::new() {
            Ok(state) => Some(state),
            Err(e) => {
                eprintln!("[ldir-vello] GPU init failed, using headless: {e}");
                None
            }
        };

        Ok(Self {
            scenes: Vec::new(),
            font_map: FontMap::new(),
            viewport: Viewport::default(),
            device_label: if cfg!(feature = "gpu") {
                #[cfg(feature = "gpu")]
                {
                    if gpu.is_some() { "gpu" } else { "headless" }
                }
                #[cfg(not(feature = "gpu"))]
                {
                    "headless"
                }
            } else {
                "headless"
            }
            .to_string(),
            #[cfg(feature = "gpu")]
            gpu,
        })
    }

    /// Create a new renderer with a custom device label.
    pub fn with_label(label: &str) -> Result<Self> {
        let mut r = Self::new()?;
        r.device_label = label.to_string();
        Ok(r)
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
            viewport: Viewport::default(),
            device_label: "headless".to_string(),
            #[cfg(feature = "gpu")]
            gpu: None,
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

    /// Get a reference to the viewport.
    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    /// Get a mutable reference to the viewport.
    pub fn viewport_mut(&mut self) -> &mut Viewport {
        &mut self.viewport
    }

    /// Set the viewport to a new value.
    pub fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
    }

    /// Render a G-IR document to an RGBA pixel buffer.
    ///
    /// When GPU rendering is available, uses Vello's compute shader pipeline.
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

        self.render_scene_impl(&scene, width, height)
    }

    /// Render a stored scene (by page index) to an RGBA pixel buffer.
    ///
    /// Applies the current viewport transform (pan/zoom) to the scene
    /// before rendering.
    ///
    /// In headless mode, returns a white pixel buffer.
    pub fn render_scene(&self, page: usize, width: u32, height: u32) -> Result<Vec<u8>> {
        let err = LdirError {
            kind: ErrorKind::Compile(CompileErrorKind::UnsupportedInstruction { entity_id: 0 }),
            entity_id: None,
            byte_offset: None,
        };
        let scene = self.get_scene(page).ok_or(err)?;

        // Apply viewport transform (pan + zoom) by creating a transformed scene.
        let transformed = self.apply_viewport(scene, width, height);
        self.render_scene_impl(&transformed, width, height)
    }

    /// Apply the viewport pan/zoom transform to a scene.
    ///
    /// Creates a new scene with the viewport's affine transform applied.
    /// The transform maps from G-IR 26.6 fixed-point coordinates to screen
    /// pixel coordinates, accounting for pan offset and zoom factor.
    fn apply_viewport(&self, scene: &Scene, _width: u32, _height: u32) -> Scene {
        use vello::peniko::kurbo::Affine;

        let vp = &self.viewport;
        let pan_x = vp.x.to_f64();
        let pan_y = vp.y.to_f64();
        let zoom = vp.zoom;

        // If viewport is at default (identity), skip transform for performance.
        if (pan_x - 0.0).abs() < 0.01 && (pan_y - 0.0).abs() < 0.01 && (zoom - 1.0).abs() < 0.001 {
            return scene.clone();
        }

        // Apply: translate by negative pan (move content), then scale by zoom.
        let transform = Affine::translate((-pan_x, -pan_y)) * Affine::scale(zoom);
        let mut transformed = Scene::new();
        transformed.append(scene, Some(transform));
        transformed
    }

    /// Internal rendering implementation.
    ///
    /// Dispatches to GPU or software path based on feature flag and device availability.
    fn render_scene_impl(&self, scene: &Scene, width: u32, height: u32) -> Result<Vec<u8>> {
        if scene.encoding().is_empty() {
            // Empty scene — return white buffer immediately
            let mut pixels = vec![0u8; (width as usize) * (height as usize) * 4];
            fill_white(&mut pixels, width, height);
            return Ok(pixels);
        }

        #[cfg(feature = "gpu")]
        if let Some(ref gpu) = self.gpu {
            return self.render_gpu(gpu, scene, width, height);
        }

        // Headless fallback: white buffer
        let mut pixels = vec![0u8; (width as usize) * (height as usize) * 4];
        fill_white(&mut pixels, width, height);
        Ok(pixels)
    }

    /// GPU rendering path: render scene to texture, read back to CPU.
    #[cfg(feature = "gpu")]
    fn render_gpu(
        &self,
        gpu: &GpuState,
        scene: &Scene,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>> {
        let device = &gpu.device;
        let queue = &gpu.queue;

        // Create target texture (Rgba8Unorm, STORAGE_BINDING for Vello)
        let target_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ldir-vello-target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
            format: wgpu::TextureFormat::Rgba8Unorm,
            view_formats: &[],
        });
        let target_view = target_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create readback buffer
        let bytes_per_row = align_to(width as usize * 4, 256);
        let buffer_size = bytes_per_row * height as usize;
        let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ldir-vello-readback"),
            size: buffer_size as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // Render scene to texture
        let params = RenderParams {
            base_color: Color::WHITE,
            width,
            height,
            antialiasing_method: AaConfig::Area,
        };

        let mut renderer = gpu.renderer.borrow_mut();
        renderer
            .render_to_texture(device, queue, scene, &target_view, &params)
            .map_err(|_e| LdirError {
                kind: ErrorKind::Compile(CompileErrorKind::UnsupportedInstruction { entity_id: 0 }),
                entity_id: None,
                byte_offset: None,
            })?;

        // Copy texture to readback buffer
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ldir-vello-copy"),
        });
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &target_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &readback_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row as u32),
                    rows_per_image: None,
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        queue.submit(std::iter::once(encoder.finish()));

        // Map buffer and read pixels
        let buffer_slice = readback_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        device.poll(wgpu::Maintain::Wait);
        rx.recv()
            .map_err(|_| LdirError {
                kind: ErrorKind::Compile(CompileErrorKind::UnsupportedInstruction { entity_id: 0 }),
                entity_id: None,
                byte_offset: None,
            })?
            .map_err(|_e| LdirError {
                kind: ErrorKind::Compile(CompileErrorKind::UnsupportedInstruction { entity_id: 0 }),
                entity_id: None,
                byte_offset: None,
            })?;

        let data = buffer_slice.get_mapped_range();
        let mut pixels = vec![0u8; (width as usize) * (height as usize) * 4];
        for y in 0..height as usize {
            let src_offset = y * bytes_per_row;
            let dst_offset = y * (width as usize) * 4;
            let copy_len = (width as usize) * 4;
            if src_offset + copy_len <= data.len() && dst_offset + copy_len <= pixels.len() {
                pixels[dst_offset..dst_offset + copy_len]
                    .copy_from_slice(&data[src_offset..src_offset + copy_len]);
            }
        }
        drop(data);
        readback_buffer.unmap();

        Ok(pixels)
    }

    /// Check if this renderer has an active GPU device.
    pub fn has_device(&self) -> bool {
        #[cfg(feature = "gpu")]
        return self.gpu.is_some();
        #[cfg(not(feature = "gpu"))]
        return false;
    }

    /// Check if any scenes are stored.
    pub fn is_empty(&self) -> bool {
        self.scenes.is_empty()
    }
}

impl Default for VelloRenderer {
    fn default() -> Self {
        #[allow(clippy::expect_used)]
        Self::new().expect("VelloRenderer::new() should not fail in headless mode")
    }
}

/// Align `val` up to the next multiple of `align`.
#[cfg(feature = "gpu")]
fn align_to(val: usize, align: usize) -> usize {
    val.div_ceil(align) * align
}

fn fill_white(pixels: &mut [u8], width: u32, _height: u32) {
    let len = pixels.len();
    let stride = (width as usize) * 4;
    // Write one row
    for x in 0..width as usize {
        let offset = x * 4;
        if offset + 3 < stride {
            pixels[offset] = 255;
            pixels[offset + 1] = 255;
            pixels[offset + 2] = 255;
            pixels[offset + 3] = 255;
        }
    }
    // Copy first row to remaining rows using split_at_mut
    let mut offset = stride;
    while offset + stride <= len {
        let (src, dst) = pixels.split_at_mut(offset);
        dst[..stride].copy_from_slice(&src[..stride]);
        offset += stride;
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
    fn test_renderer_new() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let _renderer = VelloRenderer::new()?;
        // May or may not have GPU depending on feature and environment
        Ok(())
    }

    #[test]
    fn test_renderer_with_label() {
        let renderer = VelloRenderer::with_label("test-gpu");
        assert!(renderer.is_ok());
    }

    #[test]
    fn test_renderer_default() {
        let renderer = VelloRenderer::default();
        assert!(!renderer.has_device() || cfg!(feature = "gpu"));
    }

    #[test]
    fn test_build_scene() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let renderer = VelloRenderer::new()?;
        let doc = make_test_doc();
        let scene = renderer.build_scene(&doc)?;
        assert!(!scene.encoding().is_empty());
        Ok(())
    }

    #[test]
    fn test_build_empty_scene() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let renderer = VelloRenderer::new()?;
        let doc = GIRDocument::new();
        let scene = renderer.build_scene(&doc)?;
        assert!(scene.encoding().is_empty());
        Ok(())
    }

    #[test]
    fn test_render_gir_empty_doc() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let renderer = VelloRenderer::new()?;
        let doc = GIRDocument::new();
        let pixels = renderer.render_gir(&doc, 100, 100)?;
        assert_eq!(pixels.len(), 100 * 100 * 4);
        Ok(())
    }

    #[test]
    fn test_render_gir_white_background() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let renderer = VelloRenderer::new()?;
        let doc = make_test_doc();
        let pixels = renderer.render_gir(&doc, 10, 10)?;
        assert_eq!(pixels.len(), 10 * 10 * 4);
        // In headless mode, all pixels are white
        if !renderer.has_device() {
            assert_eq!(pixels[0], 255);
            assert_eq!(pixels[1], 255);
            assert_eq!(pixels[2], 255);
            assert_eq!(pixels[3], 255);
        }
        Ok(())
    }

    #[test]
    fn test_render_gir_size() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let renderer = VelloRenderer::new()?;
        let doc = make_test_doc();
        let pixels = renderer.render_gir(&doc, 640, 480)?;
        assert_eq!(pixels.len(), 640 * 480 * 4);
        Ok(())
    }

    #[test]
    fn test_from_gir() {
        let doc = make_test_doc();
        let renderer = VelloRenderer::from_gir(&doc, &[]);
        assert_eq!(renderer.scene_count(), 1);
        if let Some(scene) = renderer.get_scene(0) {
            assert!(!scene.encoding().is_empty());
        }
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
    fn test_render_scene() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let doc = make_test_doc();
        let renderer = VelloRenderer::from_gir(&doc, &[]);
        let pixels = renderer.render_scene(0, 50, 50)?;
        assert_eq!(pixels.len(), 50 * 50 * 4);
        Ok(())
    }

    #[test]
    fn test_render_scene_out_of_bounds() {
        let doc = make_test_doc();
        let renderer = VelloRenderer::from_gir(&doc, &[]);
        assert!(renderer.render_scene(5, 50, 50).is_err());
    }

    #[test]
    fn test_fill_white() {
        let mut pixels = vec![0u8; 8 * 2 * 4];
        fill_white(&mut pixels, 8, 2);
        // Every pixel should be (255, 255, 255, 255)
        for i in 0..pixels.len() {
            assert_eq!(pixels[i], 255, "pixel byte {i} should be 255");
        }
    }

    #[test]
    fn test_fill_white_single_pixel() {
        let mut pixels = vec![0u8; 4];
        fill_white(&mut pixels, 1, 1);
        assert_eq!(pixels, [255, 255, 255, 255]);
    }

    #[test]
    fn test_viewport_default() {
        let renderer = VelloRenderer::default();
        let vp = renderer.viewport();
        assert_eq!(vp.zoom, 1.0);
        assert_eq!(vp.x, ldir_core::fp266::Fp266::ZERO);
        assert_eq!(vp.y, ldir_core::fp266::Fp266::ZERO);
    }

    #[test]
    fn test_viewport_mut() {
        let mut renderer = VelloRenderer::default();
        renderer.viewport_mut().zoom(2.0);
        assert_eq!(renderer.viewport().zoom, 2.0);
    }

    #[test]
    fn test_set_viewport() {
        let mut renderer = VelloRenderer::default();
        use crate::viewport::Viewport;
        use ldir_core::fp266::Fp266;
        let vp = Viewport::new(
            Fp266::from_int(10),
            Fp266::from_int(20),
            Fp266::from_int(612),
            Fp266::from_int(792),
        );
        renderer.set_viewport(vp);
        assert_eq!(renderer.viewport().x, Fp266::from_int(10));
    }

    #[test]
    fn test_render_scene_with_viewport() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let doc = make_test_doc();
        let mut renderer = VelloRenderer::from_gir(&doc, &[]);
        // Zoom in 2x -- should still render without errors
        renderer.viewport_mut().zoom(2.0);
        let pixels = renderer.render_scene(0, 100, 100)?;
        assert_eq!(pixels.len(), 100 * 100 * 4);
        Ok(())
    }

    #[test]
    fn test_render_scene_with_pan() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let doc = make_test_doc();
        let mut renderer = VelloRenderer::from_gir(&doc, &[]);
        renderer.viewport_mut().pan_f64(50.0, 100.0);
        let pixels = renderer.render_scene(0, 100, 100)?;
        assert_eq!(pixels.len(), 100 * 100 * 4);
        Ok(())
    }

    #[cfg(feature = "gpu")]
    #[test]
    fn test_align_to() {
        assert_eq!(align_to(0, 256), 0);
        assert_eq!(align_to(1, 256), 256);
        assert_eq!(align_to(255, 256), 256);
        assert_eq!(align_to(256, 256), 256);
        assert_eq!(align_to(257, 256), 512);
        assert_eq!(align_to(1024, 256), 1024);
    }
}
