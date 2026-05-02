//! LIR text style and layout enumeration types.
//!
//! Defines style references, text alignment, flow direction,
//! list types, and other enumerations used by LIR node types.

use crate::fp266::Fp266;

/// Text alignment within a line or paragraph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextAlign {
    /// Align text to the left margin.
    Left,
    /// Align text to the right margin.
    Right,
    /// Center text between margins.
    Center,
    /// Spread text to fill the full line width.
    Justify,
}

/// Flow direction for block-level stacking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlowDirection {
    /// Stack blocks from top to bottom (normal reading order).
    TopToBottom,
    /// Stack blocks from bottom to top.
    BottomToTop,
}

/// List marker type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ListType {
    /// Numbered list (1, 2, 3, ...).
    Ordered,
    /// Bulleted list.
    Unordered,
}

/// Figure placement strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Placement {
    /// Place figure at its current position in the flow.
    Here,
    /// Place figure at the top of the next page.
    Top,
    /// Place figure at the bottom of the current page.
    Bottom,
    /// Let the layout engine decide placement.
    Float,
}

/// Math block display type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MathType {
    /// Inline math within text flow.
    Inline,
    /// Display math, centered on its own line.
    Display,
    /// Display math with an equation number.
    Numbered,
}

/// Padding (inset) on all four sides, in scaled points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Padding {
    /// Top padding.
    pub top: Fp266,
    /// Right padding.
    pub right: Fp266,
    /// Bottom padding.
    pub bottom: Fp266,
    /// Left padding.
    pub left: Fp266,
}

impl Padding {
    /// Zero padding on all sides.
    pub const ZERO: Self = Self {
        top: Fp266::ZERO,
        right: Fp266::ZERO,
        bottom: Fp266::ZERO,
        left: Fp266::ZERO,
    };

    /// Create uniform padding on all sides.
    #[inline]
    pub const fn uniform(value: Fp266) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Create padding with individual values per side.
    #[inline]
    pub const fn new(top: Fp266, right: Fp266, bottom: Fp266, left: Fp266) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Total horizontal padding (left + right).
    #[inline]
    pub fn horizontal(&self) -> Fp266 {
        self.left + self.right
    }

    /// Total vertical padding (top + bottom).
    #[inline]
    pub fn vertical(&self) -> Fp266 {
        self.top + self.bottom
    }
}

/// Resolved text style properties for a glyph run.
///
/// Referenced by `style_id: u32` on LIR nodes. The style table
/// on `LIRDocument` holds the full set of resolved styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LIRTextStyle {
    /// Unique style identifier.
    pub id: u32,
    /// Font resource identifier.
    pub font_id: u32,
    /// Font size in 26.6 fixed-point.
    pub font_size: Fp266,
    /// Text color as packed RGBA (0xRRGGBBAA).
    pub color_rgba: u32,
    /// Bold weight.
    pub bold: bool,
    /// Italic slant.
    pub italic: bool,
    /// Underline decoration.
    pub underline: bool,
    /// Strikethrough decoration.
    pub strikethrough: bool,
    /// Small capitals variant.
    pub small_caps: bool,
    /// Extra spacing between letters.
    pub letter_spacing: Fp266,
    /// Extra spacing between words.
    pub word_spacing: Fp266,
    /// Line height (baseline-to-baseline distance).
    pub line_height: Fp266,
}

impl LIRTextStyle {
    /// Create a new style with default color (opaque black) and no decorations.
    pub fn new(id: u32, font_id: u32, font_size: Fp266) -> Self {
        Self {
            id,
            font_id,
            font_size,
            color_rgba: 0x000000FF,
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            small_caps: false,
            letter_spacing: Fp266::ZERO,
            word_spacing: Fp266::ZERO,
            line_height: Fp266::from_int(12),
        }
    }

    /// Get the font size as a floating-point value.
    pub fn font_size_f64(&self) -> f64 {
        self.font_size.to_f64()
    }
}

/// A table of resolved text styles indexed by ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LIRStyleTable {
    entries: Vec<LIRTextStyle>,
}

impl LIRStyleTable {
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
    pub fn insert(&mut self, style: LIRTextStyle) {
        self.entries.push(style);
    }

    /// Look up a style by ID.
    pub fn get(&self, id: u32) -> Option<&LIRTextStyle> {
        self.entries.iter().find(|s| s.id == id)
    }

    /// Look up a style by ID mutably.
    pub fn get_mut(&mut self, id: u32) -> Option<&mut LIRTextStyle> {
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
    pub fn iter(&self) -> impl Iterator<Item = &LIRTextStyle> {
        self.entries.iter()
    }
}

impl Default for LIRStyleTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_style_new() {
        let style = LIRTextStyle::new(1, 42, Fp266::from_int(12));
        assert_eq!(style.id, 1);
        assert_eq!(style.font_id, 42);
        assert_eq!(style.font_size, Fp266::from_int(12));
        assert!(!style.bold);
        assert!(!style.italic);
    }

    #[test]
    fn test_text_style_font_size_f64() {
        let style = LIRTextStyle::new(0, 0, Fp266::from_int(12));
        assert_eq!(style.font_size_f64(), 12.0);
    }

    #[test]
    fn test_padding_uniform() {
        let p = Padding::uniform(Fp266::from_int(10));
        assert_eq!(p.top, Fp266::from_int(10));
        assert_eq!(p.right, Fp266::from_int(10));
        assert_eq!(p.bottom, Fp266::from_int(10));
        assert_eq!(p.left, Fp266::from_int(10));
        assert_eq!(p.horizontal(), Fp266::from_int(20));
        assert_eq!(p.vertical(), Fp266::from_int(20));
    }

    #[test]
    fn test_style_table() {
        let mut table = LIRStyleTable::new();
        table.insert(LIRTextStyle::new(1, 10, Fp266::from_int(12)));
        table.insert(LIRTextStyle::new(2, 20, Fp266::from_int(14)));
        assert_eq!(table.len(), 2);
        assert!(table.get(1).is_some());
        assert!(table.get(99).is_none());
    }
}
