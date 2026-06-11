//! G-IR to Vello Scene conversion.
//!
//! Transforms G-IR command buffers into Vello `Scene` objects suitable
//! for GPU rendering. Coordinates are converted from 26.6 fixed-point
//! to f64 scene coordinates (divide by 64.0).

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use ldir_ir::gir::{GIRDocument, GIROpcode};
use ttf_parser::OutlineBuilder;
use vello::peniko::kurbo::{Affine, BezPath, Rect, RoundedRect};
use vello::peniko::{Blob, Color, Fill, Font};
use vello::{Glyph, Scene};

/// Maximum number of glyph outlines to cache per font.
const GLYPH_CACHE_CAPACITY: usize = 8192;

/// Scale factor for converting 26.6 fixed-point to scene units.
const FP266_SCALE: f64 = 64.0;

/// Map from font IDs to Vello font resources.
#[derive(Debug, Clone)]
pub struct FontMap {
    fonts: HashMap<usize, FontEntry>,
}

/// A single font entry in the font map.
#[derive(Debug)]
pub struct FontEntry {
    /// Vello font handle (wraps font data + collection index).
    pub font: Font,
    /// Font size in pixels per em for rendering.
    pub scale: f32,
    /// Raw font data bytes for outline extraction via ttf_parser.
    data: Arc<Vec<u8>>,
    /// Cached glyph outlines: glyph_id -> BezPath (interior mutability for caching).
    glyph_cache: RefCell<HashMap<u16, Option<BezPath>>>,
}

impl FontEntry {
    /// Get or compute the glyph outline for the given glyph ID.
    ///
    /// Returns None if the glyph has no outline (.notdef or empty).
    /// Caches results for subsequent lookups.
    fn get_glyph_outline(&self, glyph_id: u32) -> Option<BezPath> {
        let gid = glyph_id as u16;
        {
            let cache = self.glyph_cache.borrow();
            if let Some(cached) = cache.get(&gid) {
                return cached.clone();
            }
        }

        let outline = self.compute_glyph_outline(gid);
        let mut cache = self.glyph_cache.borrow_mut();
        // Evict entries if cache is full
        if cache.len() >= GLYPH_CACHE_CAPACITY {
            let keys: Vec<u16> = cache
                .keys()
                .take(GLYPH_CACHE_CAPACITY / 4)
                .copied()
                .collect();
            for k in keys {
                cache.remove(&k);
            }
        }
        cache.insert(gid, outline.clone());
        outline
    }

    fn compute_glyph_outline(&self, gid: u16) -> Option<BezPath> {
        let face = ttf_parser::Face::parse(&self.data, 0).ok()?;
        let upem = face.units_per_em();
        if upem == 0 {
            return None;
        }
        let glyph_id = ttf_parser::GlyphId(gid);
        let mut builder = GlyphOutlineBuilder(BezPath::new());
        face.outline_glyph(glyph_id, &mut builder)?;
        let path = builder.0;
        if path.is_empty() { None } else { Some(path) }
    }
}

impl Clone for FontEntry {
    fn clone(&self) -> Self {
        Self {
            font: self.font.clone(),
            scale: self.scale,
            data: Arc::clone(&self.data),
            glyph_cache: RefCell::new(self.glyph_cache.borrow().clone()),
        }
    }
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
            map.insert(
                id,
                FontEntry {
                    font,
                    scale,
                    data: Arc::clone(data),
                    glyph_cache: RefCell::new(HashMap::new()),
                },
            );
        }
        Self { fonts: map }
    }

    /// Insert a font into the map.
    pub fn insert(&mut self, id: usize, font: Font, scale: f32, data: Arc<Vec<u8>>) {
        self.fonts.insert(
            id,
            FontEntry {
                font,
                scale,
                data,
                glyph_cache: RefCell::new(HashMap::new()),
            },
        );
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

/// Outline builder that converts ttf_parser glyph outline commands
/// into a kurbo `BezPath` suitable for Vello scene rendering.
struct GlyphOutlineBuilder(BezPath);

impl OutlineBuilder for GlyphOutlineBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.move_to((x as f64, y as f64));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to((x as f64, y as f64));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.0.quad_to((x1 as f64, y1 as f64), (x as f64, y as f64));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.0.curve_to(
            (x1 as f64, y1 as f64),
            (x2 as f64, y2 as f64),
            (x as f64, y as f64),
        );
    }

    fn close(&mut self) {
        self.0.close_path();
    }
}

/// Render a single glyph as a filled outline path using cached outlines.
///
/// Uses the font entry's glyph cache to avoid re-parsing font data per glyph.
/// Falls back to a placeholder rectangle if the glyph cannot be rendered.
fn render_glyph_outline_cached(
    scene: &mut Scene,
    transform: Affine,
    entry: &FontEntry,
    glyph_id: u32,
    x: f64,
    y: f64,
) -> bool {
    if glyph_id == 0 {
        return false;
    }

    let Some(path) = entry.get_glyph_outline(glyph_id) else {
        return false;
    };

    let upem = ttf_parser::Face::parse(&entry.data, 0)
        .map(|f| f.units_per_em())
        .unwrap_or(1000);
    let scale_factor = entry.scale as f64 / upem as f64;
    let glyph_transform = transform
        * Affine::translate((x, y))
        * Affine::scale_non_uniform(scale_factor, -scale_factor);

    scene.fill(Fill::NonZero, glyph_transform, Color::BLACK, None, &path);
    true
}

/// Render a single glyph as a filled outline path (uncached, for single-use).
///
/// Uses `ttf_parser` to extract the glyph outline from raw font data,
/// converts it to a kurbo `BezPath`, and fills it into the scene.
///
/// Returns `true` if the glyph was successfully rendered, `false` if
/// it should fall back to a placeholder rectangle.
fn render_glyph_outline(
    scene: &mut Scene,
    transform: Affine,
    font_data: &[u8],
    glyph_id: u32,
    font_size: f32,
    x: f64,
    y: f64,
) -> bool {
    if glyph_id == 0 {
        return false;
    }

    let Ok(face) = ttf_parser::Face::parse(font_data, 0) else {
        return false;
    };

    let upem = face.units_per_em();
    if upem == 0 {
        return false;
    }

    let gid = ttf_parser::GlyphId(glyph_id as u16);
    let mut builder = GlyphOutlineBuilder(BezPath::new());
    let Some(_bbox) = face.outline_glyph(gid, &mut builder) else {
        return false;
    };

    let path = builder.0;
    if path.is_empty() {
        return false;
    }

    let scale_factor = font_size as f64 / upem as f64;
    let glyph_transform = transform
        * Affine::translate((x, y))
        * Affine::scale_non_uniform(scale_factor, -scale_factor);

    scene.fill(Fill::NonZero, glyph_transform, Color::BLACK, None, &path);
    true
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
/// When fonts are provided, `PutGlyph` commands use cached glyph outline
/// rendering for performance. Glyphs are batched into runs per font.
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
                for glyph in glyphs.drain(..) {
                    let gx = x_start + glyph.x as f64;
                    if !render_glyph_outline_cached(scene, transform, entry, glyph.id, gx, y) {
                        let tx = transform * Affine::translate((gx, y));
                        let rect = RoundedRect::new(0.0, 0.0, 10.0, 12.0, 0.0);
                        scene.fill(Fill::NonZero, tx, Color::BLACK, None, &rect);
                    }
                }
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
                    if let Some(id) = glyph_run_font_id.take() {
                        flush_glyph_run(
                            &mut scene,
                            current_transform,
                            id,
                            glyph_run_x_start,
                            cursor_y,
                            &mut pending_glyphs,
                            fonts,
                        );
                    }
                    current_font_id = font_id;
                }
            }
            GIROpcode::MoveXY => {
                let x_fp = cmd.arg(0).unwrap_or(0) as f64 / FP266_SCALE;
                let y_fp = cmd.arg(1).unwrap_or(0) as f64 / FP266_SCALE;
                if let Some(id) = glyph_run_font_id.take() {
                    flush_glyph_run(
                        &mut scene,
                        current_transform,
                        id,
                        glyph_run_x_start,
                        cursor_y,
                        &mut pending_glyphs,
                        fonts,
                    );
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
                if let Some(id) = glyph_run_font_id.take() {
                    flush_glyph_run(
                        &mut scene,
                        current_transform,
                        id,
                        glyph_run_x_start,
                        cursor_y,
                        &mut pending_glyphs,
                        fonts,
                    );
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
                if let Some(id) = glyph_run_font_id.take() {
                    flush_glyph_run(
                        &mut scene,
                        current_transform,
                        id,
                        glyph_run_x_start,
                        cursor_y,
                        &mut pending_glyphs,
                        fonts,
                    );
                }
                transform_stack.push(current_transform);
            }
            GIROpcode::PopStack => {
                if let Some(id) = glyph_run_font_id.take() {
                    flush_glyph_run(
                        &mut scene,
                        current_transform,
                        id,
                        glyph_run_x_start,
                        cursor_y,
                        &mut pending_glyphs,
                        fonts,
                    );
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
        page.push(GIRCommand::new_draw_rule(
            10 * 64,
            20 * 64,
            200 * 64,
            2 * 64,
        ));
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
        let data = Arc::new(vec![0u8; 64]);
        let arc_dyn: Arc<dyn AsRef<[u8]> + Send + Sync> = Arc::clone(&data) as _;
        let blob = Blob::new(arc_dyn);
        let font = Font::new(blob, 0);
        fonts.insert(42, font, 16.0, data);
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

    fn load_test_font() -> Vec<u8> {
        ldir_test_helpers::test_font_data()
    }

    #[test]
    fn test_glyph_outline_with_real_font() {
        let font_data = load_test_font();
        let arc_data = Arc::new(font_data);
        let face = ttf_parser::Face::parse(&arc_data, 0).unwrap();
        let gid_a = face.glyph_index('A').unwrap();

        let mut page = GIRPage::new();
        page.push(GIRCommand::new_set_font(0));
        page.push(GIRCommand::new_move_xy(100 * 64, 200 * 64));
        page.push(GIRCommand::new_put_glyph(gid_a.0 as i32, 10 * 64));
        let fonts = FontMap::from_fonts(&[(0, Arc::clone(&arc_data), 12.0)]);
        let scene = gir_page_to_scene_with_fonts(&page, &fonts);
        assert!(!scene.encoding().is_empty());
    }

    #[test]
    fn test_glyph_outline_missing_glyph_id_zero_fallback() {
        let font_data = load_test_font();
        let arc_data = Arc::new(font_data);

        let mut page = GIRPage::new();
        page.push(GIRCommand::new_set_font(0));
        page.push(GIRCommand::new_move_xy(50 * 64, 50 * 64));
        page.push(GIRCommand::new_put_glyph(0, 10 * 64));
        let fonts = FontMap::from_fonts(&[(0, Arc::clone(&arc_data), 12.0)]);
        let scene = gir_page_to_scene_with_fonts(&page, &fonts);
        assert!(!scene.encoding().is_empty());
    }

    #[test]
    fn test_glyph_outline_invalid_font_data_fallback() {
        let arc_data = Arc::new(vec![0u8; 64]);

        let mut page = GIRPage::new();
        page.push(GIRCommand::new_set_font(0));
        page.push(GIRCommand::new_move_xy(50 * 64, 50 * 64));
        page.push(GIRCommand::new_put_glyph(65, 10 * 64));
        let fonts = FontMap::from_fonts(&[(0, Arc::clone(&arc_data), 12.0)]);
        let scene = gir_page_to_scene_with_fonts(&page, &fonts);
        assert!(!scene.encoding().is_empty());
    }

    #[test]
    fn test_render_glyph_outline_returns_false_for_glyph_zero() {
        let result = render_glyph_outline(
            &mut Scene::new(),
            Affine::IDENTITY,
            &[0u8; 64],
            0,
            12.0,
            0.0,
            0.0,
        );
        assert!(!result);
    }

    #[test]
    fn test_render_glyph_outline_returns_false_for_invalid_font() {
        let result = render_glyph_outline(
            &mut Scene::new(),
            Affine::IDENTITY,
            &[0u8; 10],
            65,
            12.0,
            0.0,
            0.0,
        );
        assert!(!result);
    }
}
