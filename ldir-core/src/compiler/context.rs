//! Compilation state tracking for the S-IR → G-IR compiler.
//!
//! Manages coordinate position, stack depth, page dimensions,
//! and other mutable state during compilation.

use std::sync::Arc;

use bumpalo::Bump;
use ldir_ir::gir::GIRPage;
use ldir_ir::sir::StyleModifier;

use crate::compiler::templates::PageTemplate;
use crate::font::db::FontDatabase;
use crate::fp266::Fp266;
use crate::shaping::cache::ThreadSafeShapeCache;

/// Maximum allowed nesting depth for PushStack/PopStack.
pub const MAX_STACK_DEPTH: usize = 256;

/// Default US Letter page width in points (612pt = 8.5in).
pub const DEFAULT_PAGE_WIDTH_PT: i32 = 612;

/// Default US Letter page height in points (792pt = 11in).
pub const DEFAULT_PAGE_HEIGHT_PT: i32 = 792;

/// Default paragraph spacing in points.
pub const DEFAULT_PARA_SPACING_PT: i32 = 12;

/// Default font size in points (for body text).
pub const DEFAULT_FONT_SIZE_PT: i32 = 12;

/// Default glyph advance width in points (approximate for monospace).
pub const DEFAULT_GLYPH_ADVANCE_PT: i32 = 7;

/// Default line height as a multiple of font size (1.2×).
pub const DEFAULT_LINE_HEIGHT_FACTOR: i32 = 6;

/// Left margin in points.
pub const LEFT_MARGIN_PT: i32 = 72;

/// Right margin in points.
pub const RIGHT_MARGIN_PT: i32 = 72;

/// Top margin in points.
pub const TOP_MARGIN_PT: i32 = 72;

/// Bottom margin in points.
pub const BOTTOM_MARGIN_PT: i32 = 72;

/// Font ID for regular (upright) weight.
pub const FONT_ID_REGULAR: u32 = 0;
/// Font ID for bold weight.
pub const FONT_ID_BOLD: u32 = 1;
/// Font ID for italic style.
pub const FONT_ID_ITALIC: u32 = 2;
/// Font ID for bold italic.
pub const FONT_ID_BOLD_ITALIC: u32 = 3;
/// Font ID for monospace.
pub const FONT_ID_MONO: u32 = 4;

/// Mutable compilation state.
pub struct CompileContext {
    /// Current cursor X position (26.6 fixed-point).
    pub x: Fp266,
    /// Current cursor Y position (26.6 fixed-point).
    pub y: Fp266,
    /// Current stack depth (for PushStack/PopStack balance).
    pub stack_depth: usize,
    /// Current page width in 26.6 fixed-point.
    pub page_width: Fp266,
    /// Current page height in 26.6 fixed-point.
    pub page_height: Fp266,
    /// Current font size in 26.6 fixed-point.
    pub font_size: Fp266,
    /// Current font ID.
    pub font_id: u32,
    /// Usable content width (page width minus margins).
    pub content_width: Fp266,
    /// Raw font data for real shaping (ttf-parser + HarfBuzz).
    /// When `None`, falls back to ASCII monospace stub.
    pub font_data: Option<Arc<Vec<u8>>>,
    /// Font data per font ID (0=Regular, 1=Bold, 2=Italic, 3=BoldItalic, 4=Mono).
    /// Used for style-aware shaping in `emit_paragraph`.
    pub font_data_variants: Vec<Option<Arc<Vec<u8>>>>,
    /// Stack of active style modifiers (for nested bold/italic/mono).
    pub style_stack: Vec<StyleModifier>,
    /// Currently active combined style modifiers.
    pub active_style: StyleModifier,
    /// Pending link URLs for the current block, pre-scanned from LinkData children.
    pub pending_link_urls: Vec<String>,
    /// Link start positions: (start_x, start_y) recorded before SetContent renders.
    pub link_start_positions: Vec<(f64, f64)>,
    /// Left margin in 26.6 fixed-point.
    pub margin_left: Fp266,
    /// Right margin in 26.6 fixed-point.
    pub margin_right: Fp266,
    /// Top margin in 26.6 fixed-point.
    pub margin_top: Fp266,
    /// Bottom margin in 26.6 fixed-point.
    pub margin_bottom: Fp266,
    /// Additional reserved space at page bottom for footnotes (26.6 fixed-point).
    pub footnote_reserve: i64,
    /// Whether drop caps are enabled for paragraphs after headings.
    pub drop_caps_enabled: bool,
    /// Whether the next paragraph should be rendered as a drop cap.
    pub next_para_is_drop_cap: bool,
    /// Number of pages consumed by the TOC (for page number offset).
    pub toc_page_count: usize,
    /// Arena allocator for paragraph-scoped temporary allocations.
    /// Reset before each paragraph to reuse memory.
    pub bump: Bump,
    /// LRU cache for shaped text runs.
    pub shape_cache: ThreadSafeShapeCache,
    /// Font database for system font discovery and name-based lookup.
    /// When `Some`, the compiler can resolve font families by name.
    pub font_db: Option<Arc<FontDatabase>>,
    /// Primary font family name (e.g., "DejaVu Sans").
    /// Used with `font_db` to resolve fonts by name.
    pub font_family: String,
    /// Monospace font family name (e.g., "DejaVu Sans Mono").
    pub font_mono_family: String,
    /// Page template for headers and footers.
    pub template: PageTemplate,
    /// Current page number (1-indexed).
    pub page_number: usize,
    /// Current chapter title (for template expansion).
    pub chapter_title: String,
    /// Current section title (for template expansion).
    pub section_title: String,
}

impl Default for CompileContext {
    fn default() -> Self {
        Self::new()
    }
}

impl CompileContext {
    /// Create a new compilation context with default page dimensions.
    /// Uses ASCII monospace stub shaping (no real font data).
    pub fn new() -> Self {
        Self::with_font(None)
    }

    /// Create a compilation context with real font data.
    ///
    /// When `font_data` is `Some`, the compiler uses HarfBuzz + ttf-parser
    /// for real font-aware shaping. When `None`, falls back to the ASCII
    /// monospace stub.
    pub fn with_font(font_data: Option<Arc<Vec<u8>>>) -> Self {
        let page_width = Fp266::from_int(DEFAULT_PAGE_WIDTH_PT);
        let page_height = Fp266::from_int(DEFAULT_PAGE_HEIGHT_PT);
        let margin_left = Fp266::from_int(LEFT_MARGIN_PT);
        let margin_right = Fp266::from_int(RIGHT_MARGIN_PT);
        let margin_top = Fp266::from_int(TOP_MARGIN_PT);
        let margin_bottom = Fp266::from_int(BOTTOM_MARGIN_PT);

        Self {
            x: margin_left,
            y: margin_top,
            stack_depth: 0,
            page_width,
            page_height,
            font_size: Fp266::from_int(DEFAULT_FONT_SIZE_PT),
            font_id: 0,
            content_width: page_width - margin_left - margin_right,
            font_data: font_data.clone(),
            font_data_variants: vec![font_data; 5],
            style_stack: Vec::new(),
            active_style: StyleModifier::EMPTY,
            pending_link_urls: Vec::new(),
            link_start_positions: Vec::new(),
            margin_left,
            margin_right,
            margin_top,
            margin_bottom,
            footnote_reserve: 0,
            drop_caps_enabled: false,
            next_para_is_drop_cap: false,
            toc_page_count: 0,
            bump: Bump::new(),
            shape_cache: ThreadSafeShapeCache::new(1024),
            font_db: None,
            font_family: String::new(),
            font_mono_family: String::new(),
            template: PageTemplate::default(),
            page_number: 0,
            chapter_title: String::new(),
            section_title: String::new(),
        }
    }

    /// Create a compilation context with real font data and custom margins.
    ///
    /// Margins are specified in points.
    pub fn with_font_and_margins(
        font_data: Option<Arc<Vec<u8>>>,
        margin_left: i32,
        margin_right: i32,
        margin_top: i32,
        margin_bottom: i32,
    ) -> Self {
        Self::with_font_margins_and_page(
            font_data,
            margin_left,
            margin_right,
            margin_top,
            margin_bottom,
            DEFAULT_PAGE_WIDTH_PT,
            DEFAULT_PAGE_HEIGHT_PT,
        )
    }

    /// Create a compilation context with real font data, custom margins, and custom page size.
    ///
    /// All dimensions are specified in points.
    pub fn with_font_margins_and_page(
        font_data: Option<Arc<Vec<u8>>>,
        margin_left: i32,
        margin_right: i32,
        margin_top: i32,
        margin_bottom: i32,
        page_width_pt: i32,
        page_height_pt: i32,
    ) -> Self {
        let ml = Fp266::from_int(margin_left);
        let mr = Fp266::from_int(margin_right);
        let mt = Fp266::from_int(margin_top);
        let mb = Fp266::from_int(margin_bottom);
        let page_width = Fp266::from_int(page_width_pt);
        let page_height = Fp266::from_int(page_height_pt);
        Self {
            x: ml,
            y: mt,
            stack_depth: 0,
            page_width,
            page_height,
            font_size: Fp266::from_int(DEFAULT_FONT_SIZE_PT),
            font_id: 0,
            content_width: page_width - ml - mr,
            font_data: font_data.clone(),
            font_data_variants: vec![font_data; 5],
            style_stack: Vec::new(),
            active_style: StyleModifier::EMPTY,
            pending_link_urls: Vec::new(),
            link_start_positions: Vec::new(),
            margin_left: ml,
            margin_right: mr,
            margin_top: mt,
            margin_bottom: mb,
            footnote_reserve: 0,
            drop_caps_enabled: false,
            next_para_is_drop_cap: false,
            toc_page_count: 0,
            bump: Bump::new(),
            shape_cache: ThreadSafeShapeCache::new(1024),
            font_db: None,
            font_family: String::new(),
            font_mono_family: String::new(),
            template: PageTemplate::default(),
            page_number: 0,
            chapter_title: String::new(),
            section_title: String::new(),
        }
    }

    /// Create a new page with the current dimensions.
    pub fn new_page(&self) -> GIRPage {
        GIRPage::with_dimensions(self.page_width.raw() as i32, self.page_height.raw() as i32)
    }

    /// Advance Y by the given amount (typically line height).
    pub fn advance_y(&mut self, amount: Fp266) {
        self.y += amount;
    }

    /// Advance X by the given amount (typically glyph advance).
    pub fn advance_x(&mut self, amount: Fp266) {
        self.x += amount;
    }

    /// Reset X to left margin.
    pub fn reset_x(&mut self) {
        self.x = self.margin_left;
    }

    /// Calculate line height based on current font size.
    pub fn line_height(&self) -> Fp266 {
        self.font_size
            + self
                .font_size
                .mul(Fp266::from_frac(DEFAULT_LINE_HEIGHT_FACTOR, 10))
    }

    /// Check if Y position exceeds page height (minus bottom margin and footnote reserve).
    pub fn exceeds_page(&self) -> bool {
        let max_y = self.page_height - self.margin_bottom - Fp266::from_raw(self.footnote_reserve);
        self.y > max_y
    }

    /// Push stack and check for overflow.
    ///
    /// Returns `Ok(())` on success, error if depth exceeds `MAX_STACK_DEPTH`.
    pub fn push_stack(&mut self) -> Result<(), crate::error::LdirError> {
        self.stack_depth += 1;
        if self.stack_depth > MAX_STACK_DEPTH {
            return Err(crate::error::CompileErrorKind::StackOverflow {
                depth: self.stack_depth,
            }
            .into());
        }
        Ok(())
    }

    /// Pop stack and check for underflow.
    pub fn pop_stack(&mut self) -> Result<(), crate::error::LdirError> {
        if self.stack_depth == 0 {
            return Err(crate::error::CompileErrorKind::StackOverflow { depth: 0 }.into());
        }
        self.stack_depth -= 1;
        Ok(())
    }

    /// Get the current font size.
    #[inline]
    pub fn font_size(&self) -> Fp266 {
        self.font_size
    }

    /// Get the current font ID.
    #[inline]
    pub fn font_id(&self) -> u32 {
        self.font_id
    }

    /// Push a style modifier onto the style stack.
    ///
    /// Updates `active_style` and `font_id` based on the combined modifiers.
    pub fn push_style(&mut self, modifier: StyleModifier) {
        self.style_stack.push(modifier);
        self.recompute_style();
    }

    /// Pop the last style modifier from the style stack.
    ///
    /// Updates `active_style` and `font_id` based on remaining modifiers.
    pub fn pop_style(&mut self) {
        self.style_stack.pop();
        self.recompute_style();
    }

    /// Set font data for a specific font ID variant.
    pub fn set_font_variant(&mut self, font_id: usize, data: Option<Arc<Vec<u8>>>) {
        if font_id < self.font_data_variants.len() {
            self.font_data_variants[font_id] = data;
        }
    }

    /// Recompute active style and font_id from the style stack.
    fn recompute_style(&mut self) {
        let mut combined = StyleModifier::EMPTY;
        for &style in &self.style_stack {
            combined = StyleModifier(combined.0 | style.0);
        }
        self.active_style = combined;
        self.font_id = style_to_font_id(combined);
    }
}

/// Map combined style modifiers to a font ID.
///
/// Font ID convention:
/// - 0 = Regular (no modifiers)
/// - 1 = Bold
/// - 2 = Italic
/// - 3 = Bold + Italic
/// - 4 = Monospace
///
/// MONO takes priority over BOLD/ITALIC (monospace fonts rarely have variants).
fn style_to_font_id(style: StyleModifier) -> u32 {
    if style.contains(StyleModifier::MONO) {
        return FONT_ID_MONO;
    }
    let bold = style.contains(StyleModifier::BOLD);
    let italic = style.contains(StyleModifier::ITALIC);
    match (bold, italic) {
        (false, false) => FONT_ID_REGULAR,
        (true, false) => FONT_ID_BOLD,
        (false, true) => FONT_ID_ITALIC,
        (true, true) => FONT_ID_BOLD_ITALIC,
    }
}

/// Parse a page size preset name into (width_pt, height_pt).
///
/// Returns `None` for unrecognized names.
pub fn parse_page_size(name: &str) -> Option<(i32, i32)> {
    match name.to_lowercase().as_str() {
        "a4" => Some((595, 842)),
        "letter" => Some((612, 792)),
        "legal" => Some((612, 1008)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_context() {
        let ctx = CompileContext::new();
        assert_eq!(ctx.x, Fp266::from_int(LEFT_MARGIN_PT));
        assert_eq!(ctx.y, Fp266::from_int(TOP_MARGIN_PT));
        assert_eq!(ctx.stack_depth, 0);
    }

    #[test]
    fn test_advance_y() {
        let mut ctx = CompileContext::new();
        ctx.advance_y(Fp266::from_int(20));
        assert_eq!(ctx.y, Fp266::from_int(TOP_MARGIN_PT + 20));
    }

    #[test]
    fn test_advance_x() {
        let mut ctx = CompileContext::new();
        ctx.advance_x(Fp266::from_int(10));
        assert_eq!(ctx.x, Fp266::from_int(LEFT_MARGIN_PT + 10));
    }

    #[test]
    fn test_reset_x() {
        let mut ctx = CompileContext::new();
        ctx.advance_x(Fp266::from_int(100));
        ctx.reset_x();
        assert_eq!(ctx.x, Fp266::from_int(LEFT_MARGIN_PT));
    }

    #[test]
    fn test_push_pop_stack() {
        let mut ctx = CompileContext::new();
        ctx.push_stack().unwrap();
        assert_eq!(ctx.stack_depth, 1);
        ctx.pop_stack().unwrap();
        assert_eq!(ctx.stack_depth, 0);
    }

    #[test]
    fn test_pop_stack_underflow() {
        let mut ctx = CompileContext::new();
        assert!(ctx.pop_stack().is_err());
    }

    #[test]
    fn test_exceeds_page() {
        let mut ctx = CompileContext::new();
        assert!(!ctx.exceeds_page());
        ctx.y = Fp266::from_int(DEFAULT_PAGE_HEIGHT_PT);
        assert!(ctx.exceeds_page());
    }

    #[test]
    fn test_line_height() {
        let ctx = CompileContext::new();
        let lh = ctx.line_height();
        assert!(lh.to_f64() > ctx.font_size.to_f64());
    }

    #[test]
    fn test_new_page() {
        let ctx = CompileContext::new();
        let page = ctx.new_page();
        assert_eq!(page.width, DEFAULT_PAGE_WIDTH_PT * 64);
        assert_eq!(page.height, DEFAULT_PAGE_HEIGHT_PT * 64);
    }

    #[test]
    fn test_content_width() {
        let ctx = CompileContext::new();
        let expected = DEFAULT_PAGE_WIDTH_PT - LEFT_MARGIN_PT - RIGHT_MARGIN_PT;
        assert_eq!(ctx.content_width, Fp266::from_int(expected));
    }

    #[test]
    fn test_style_push_pop_bold() {
        let mut ctx = CompileContext::new();
        assert_eq!(ctx.font_id, FONT_ID_REGULAR);
        ctx.push_style(StyleModifier::BOLD_STYLE);
        assert_eq!(ctx.font_id, FONT_ID_BOLD);
        ctx.pop_style();
        assert_eq!(ctx.font_id, FONT_ID_REGULAR);
    }

    #[test]
    fn test_style_push_pop_italic() {
        let mut ctx = CompileContext::new();
        ctx.push_style(StyleModifier::ITALIC_STYLE);
        assert_eq!(ctx.font_id, FONT_ID_ITALIC);
        ctx.pop_style();
        assert_eq!(ctx.font_id, FONT_ID_REGULAR);
    }

    #[test]
    fn test_style_bold_italic() {
        let mut ctx = CompileContext::new();
        ctx.push_style(StyleModifier::BOLD_STYLE);
        ctx.push_style(StyleModifier::ITALIC_STYLE);
        assert_eq!(ctx.font_id, FONT_ID_BOLD_ITALIC);
        ctx.pop_style();
        assert_eq!(ctx.font_id, FONT_ID_BOLD);
        ctx.pop_style();
        assert_eq!(ctx.font_id, FONT_ID_REGULAR);
    }

    #[test]
    fn test_style_mono() {
        let mut ctx = CompileContext::new();
        ctx.push_style(StyleModifier::MONO_STYLE);
        assert_eq!(ctx.font_id, FONT_ID_MONO);
    }

    #[test]
    fn test_style_mono_takes_priority() {
        let mut ctx = CompileContext::new();
        ctx.push_style(StyleModifier::BOLD_STYLE);
        ctx.push_style(StyleModifier::MONO_STYLE);
        assert_eq!(
            ctx.font_id, FONT_ID_MONO,
            "MONO should take priority over BOLD"
        );
    }

    #[test]
    fn test_custom_margins() {
        let ctx = CompileContext::with_font_and_margins(None, 36, 36, 50, 50);
        assert_eq!(ctx.margin_left, Fp266::from_int(36));
        assert_eq!(ctx.margin_right, Fp266::from_int(36));
        assert_eq!(ctx.margin_top, Fp266::from_int(50));
        assert_eq!(ctx.margin_bottom, Fp266::from_int(50));
        assert_eq!(ctx.x, Fp266::from_int(36));
        assert_eq!(ctx.y, Fp266::from_int(50));
        let expected_width = DEFAULT_PAGE_WIDTH_PT - 36 - 36;
        assert_eq!(ctx.content_width, Fp266::from_int(expected_width));
    }

    #[test]
    fn test_custom_margins_reset_x() {
        let mut ctx = CompileContext::with_font_and_margins(None, 90, 72, 72, 72);
        ctx.advance_x(Fp266::from_int(100));
        ctx.reset_x();
        assert_eq!(ctx.x, Fp266::from_int(90));
    }

    #[test]
    fn test_custom_margins_exceeds_page() {
        let mut ctx = CompileContext::with_font_and_margins(None, 72, 72, 72, 100);
        assert!(!ctx.exceeds_page());
        ctx.y = Fp266::from_int(DEFAULT_PAGE_HEIGHT_PT - 100);
        assert!(!ctx.exceeds_page());
        ctx.y = Fp266::from_int(DEFAULT_PAGE_HEIGHT_PT - 99);
        assert!(ctx.exceeds_page());
    }

    #[test]
    fn test_custom_page_size() {
        let ctx = CompileContext::with_font_margins_and_page(None, 36, 36, 36, 36, 595, 842);
        assert_eq!(ctx.page_width, Fp266::from_int(595));
        assert_eq!(ctx.page_height, Fp266::from_int(842));
        let expected = 595 - 36 - 36;
        assert_eq!(ctx.content_width, Fp266::from_int(expected));
    }

    #[test]
    fn test_parse_page_size_a4() {
        assert_eq!(parse_page_size("a4"), Some((595, 842)));
        assert_eq!(parse_page_size("A4"), Some((595, 842)));
        assert_eq!(parse_page_size("letter"), Some((612, 792)));
        assert_eq!(parse_page_size("legal"), Some((612, 1008)));
        assert_eq!(parse_page_size("unknown"), None);
    }
}
