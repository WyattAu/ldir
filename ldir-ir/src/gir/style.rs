//! G-IR style types.
//!
//! Style definitions for font selection, sizing, and color in the G-IR
//! rendering pipeline. Styles are referenced by S-IR `ApplyStyle`
//! instructions and resolved during compilation.
//!
//! Per BP-IR-COMPILER-001 Section 6.2 (Data Dictionary):
//! ```text
//! StyleEntry {
//!     u32 id
//!     u32 font_id
//!     i32 size_fp26_6
//! }
//! ```

/// A single style entry defining typographic properties.
///
/// Style entries are referenced by S-IR `ApplyStyle` instructions via
/// their `id` field and resolved during S-IR → G-IR compilation.
///
/// # Coordinates
///
/// Font size is stored in 26.6 fixed-point format (REQ-3.2.5).
/// Color components are 8-bit unsigned values (0-255).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct GIRStyle {
    /// Unique style identifier.
    pub id: u32,
    /// Font resource identifier.
    pub font_id: u32,
    /// Font size in 26.6 fixed-point format (REQ-3.2.5).
    /// For example, 12pt = 12 * 64 = 768.
    pub size_fp26_6: i32,
    /// Text color (packed RGBA, 0xRRGGBBAA).
    pub color_rgba: u32,
    /// Text color red component (0-255).
    pub color_r: u8,
    /// Text color green component (0-255).
    pub color_g: u8,
    /// Text color blue component (0-255).
    pub color_b: u8,
    /// Text color alpha component (0-255).
    pub color_a: u8,
}

impl GIRStyle {
    /// Create a new style with the given id, font, and size.
    ///
    /// Color defaults to opaque black (0x000000FF).
    ///
    /// # Arguments
    ///
    /// * `id` - Unique style identifier.
    /// * `font_id` - Font resource identifier.
    /// * `size_fp26_6` - Font size in 26.6 fixed-point format.
    #[inline]
    pub const fn new(id: u32, font_id: u32, size_fp26_6: i32) -> Self {
        Self {
            id,
            font_id,
            size_fp26_6,
            color_rgba: 0x000000FF,
            color_r: 0,
            color_g: 0,
            color_b: 0,
            color_a: 255,
        }
    }

    /// Create a style with full RGBA color specification.
    #[inline]
    pub const fn with_color(
        id: u32,
        font_id: u32,
        size_fp26_6: i32,
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    ) -> Self {
        Self {
            id,
            font_id,
            size_fp26_6,
            color_rgba: ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32),
            color_r: r,
            color_g: g,
            color_b: b,
            color_a: a,
        }
    }

    /// Get the font size as a floating-point value.
    #[inline]
    pub fn size_f64(&self) -> f64 {
        self.size_fp26_6 as f64 / 64.0
    }
}

impl Default for GIRStyle {
    fn default() -> Self {
        Self::new(0, 0, 0)
    }
}

/// A table of style entries indexed by style ID.
///
/// Used during S-IR → G-IR compilation to resolve `ApplyStyle`
/// instructions into `SetFont` commands.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StyleTable {
    entries: Vec<GIRStyle>,
}

impl StyleTable {
    /// Create a new empty style table.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Create a style table with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
        }
    }

    /// Add a style entry to the table.
    pub fn insert(&mut self, style: GIRStyle) {
        self.entries.push(style);
    }

    /// Look up a style by ID.
    ///
    /// Returns `None` if no style with the given ID exists.
    pub fn get(&self, id: u32) -> Option<&GIRStyle> {
        self.entries.iter().find(|s| s.id == id)
    }

    /// Look up a style by ID mutably.
    pub fn get_mut(&mut self, id: u32) -> Option<&mut GIRStyle> {
        self.entries.iter_mut().find(|s| s.id == id)
    }

    /// Number of style entries.
    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the table is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate over style entries.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &GIRStyle> {
        self.entries.iter()
    }
}

impl Default for StyleTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_new() {
        let style = GIRStyle::new(1, 42, 12 * 64);
        assert_eq!(style.id, 1);
        assert_eq!(style.font_id, 42);
        assert_eq!(style.size_fp26_6, 768);
        assert_eq!(style.size_f64(), 12.0);
    }

    #[test]
    fn test_style_with_color() {
        let style = GIRStyle::with_color(1, 0, 10 * 64, 255, 0, 0, 255);
        assert_eq!(style.color_r, 255);
        assert_eq!(style.color_g, 0);
        assert_eq!(style.color_b, 0);
        assert_eq!(style.color_a, 255);
    }

    #[test]
    fn test_style_default() {
        let style = GIRStyle::default();
        assert_eq!(style.id, 0);
        assert_eq!(style.font_id, 0);
        assert_eq!(style.size_fp26_6, 0);
    }

    #[test]
    fn test_style_table() {
        let mut table = StyleTable::new();
        table.insert(GIRStyle::new(1, 10, 12 * 64));
        table.insert(GIRStyle::new(2, 20, 14 * 64));

        assert_eq!(table.len(), 2);
        assert!(table.get(1).is_some());
        assert!(table.get(99).is_none());

        let s = table.get(1).unwrap();
        assert_eq!(s.font_id, 10);
    }

    #[test]
    fn test_style_table_empty() {
        let table = StyleTable::new();
        assert!(table.is_empty());
    }
}
