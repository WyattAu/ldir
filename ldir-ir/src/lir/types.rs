//! LIR node type definitions (YP-LAYOUT-LIR-001 Section 5).
//!
//! All 23 LIR box types plus supporting types (LIRGeometry, LIRNode enum,
//! LIRDocumentMeta, TOCEntry). Each node carries resolved positions and
//! sizes in 26.6 fixed-point per AX-LIR-001.

use crate::fp266::Fp266;
use crate::lir::position::Rect;
use crate::lir::style::{
    FlowDirection, LIRStyleTable, ListType, MathType, Padding, Placement, TextAlign,
};

use std::fmt;

// ---------------------------------------------------------------------------
// Common geometry (DEF-LIR-GEOM)
// ---------------------------------------------------------------------------

/// Resolved geometry for every LIR node (DEF-LIR-GEOM).
///
/// All values are 26.6 fixed-point scaled points. For non-text nodes,
/// `baseline` is 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct LIRGeometry {
    /// Absolute x position from page content-area left edge.
    pub x: Fp266,
    /// Absolute y position from page content-area top edge.
    pub y: Fp266,
    /// Content width.
    pub width: Fp266,
    /// Content height.
    pub height: Fp266,
    /// Baseline offset from top of node (text nodes only; 0 otherwise).
    pub baseline: Fp266,
}

impl LIRGeometry {
    /// Zero geometry (all fields zero).
    pub const ZERO: Self = Self {
        x: Fp266::ZERO,
        y: Fp266::ZERO,
        width: Fp266::ZERO,
        height: Fp266::ZERO,
        baseline: Fp266::ZERO,
    };

    /// Create geometry with zero baseline.
    #[inline]
    pub const fn new(x: Fp266, y: Fp266, width: Fp266, height: Fp266) -> Self {
        Self {
            x,
            y,
            width,
            height,
            baseline: Fp266::ZERO,
        }
    }

    /// Create geometry with an explicit baseline.
    #[inline]
    pub const fn with_baseline(
        x: Fp266,
        y: Fp266,
        width: Fp266,
        height: Fp266,
        baseline: Fp266,
    ) -> Self {
        Self {
            x,
            y,
            width,
            height,
            baseline,
        }
    }

    /// Create geometry from integer values with zero baseline.
    #[inline]
    pub const fn from_int(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self {
            x: Fp266::from_int(x),
            y: Fp266::from_int(y),
            width: Fp266::from_int(w),
            height: Fp266::from_int(h),
            baseline: Fp266::ZERO,
        }
    }

    /// Convert to a `Rect`.
    pub fn to_rect(&self) -> Rect {
        Rect::new(self.x, self.y, self.width, self.height)
    }
}

// ---------------------------------------------------------------------------
// Document metadata (DEF-LIR-DOCUMENT)
// ---------------------------------------------------------------------------

/// Top-level document metadata (page geometry, language, direction).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LIRDocumentMeta {
    /// Document language (BCP-47 tag, e.g. "en").
    pub language: String,
    /// Page width in scaled points.
    pub page_width: Fp266,
    /// Page height in scaled points.
    pub page_height: Fp266,
    /// Top margin.
    pub margin_top: Fp266,
    /// Bottom margin.
    pub margin_bottom: Fp266,
    /// Left margin.
    pub margin_left: Fp266,
    /// Right margin.
    pub margin_right: Fp266,
}

impl Default for LIRDocumentMeta {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            page_width: Fp266::from_int(612),
            page_height: Fp266::from_int(792),
            margin_top: Fp266::from_int(72),
            margin_bottom: Fp266::from_int(72),
            margin_left: Fp266::from_int(72),
            margin_right: Fp266::from_int(72),
        }
    }
}

impl LIRDocumentMeta {
    /// Create US Letter metadata (612×792 sp, 1in margins).
    pub fn us_letter() -> Self {
        Self::default()
    }

    /// Content width (page width minus left and right margins).
    pub fn content_width(&self) -> Fp266 {
        self.page_width - self.margin_left - self.margin_right
    }

    /// Content height (page height minus top and bottom margins).
    pub fn content_height(&self) -> Fp266 {
        self.page_height - self.margin_top - self.margin_bottom
    }
}

// ---------------------------------------------------------------------------
// LIRNode — tagged union of all box types (DEF-LIR-NODE)
// ---------------------------------------------------------------------------

/// The L-IR node enum: a tagged union of all 23 box types.
///
/// Each variant wraps a concrete struct carrying geometry, optional
/// children, and node-specific fields.
#[derive(Debug, Clone, PartialEq)]
pub enum LIRNode {
    /// Top-level document container.
    Document(LIRDocument),
    /// A single page.
    Page(LIRPage),
    /// Vertical stack of block-level boxes.
    Flow(LIRFlow),
    /// Line-broken paragraph of text.
    Paragraph(LIRParagraph),
    /// Single horizontal line within a paragraph or heading.
    Line(LIRLine),
    /// Positioned glyph (leaf).
    Glyph(LIRGlyph),
    /// Inter-word or inter-glyph spacing (leaf).
    Space(LIRSpace),
    /// Numbered section heading.
    Heading(LIRHeading),
    /// Ordered or unordered list.
    List(LIRList),
    /// Single list item.
    ListItem(LIRListItem),
    /// Grid layout table.
    Table(LIRTable),
    /// Single table row.
    TableRow(LIRTableRow),
    /// Single table cell.
    TableCell(LIRTableCell),
    /// Figure with optional caption.
    Figure(LIRFigure),
    /// Figure or table caption.
    Caption(LIRCaption),
    /// In-flow footnote marker (leaf).
    Footnote(LIRFootnote),
    /// Collected footnote block.
    FootnoteBlock(LIRFootnoteBlock),
    /// Block quotation.
    BlockQuote(LIRBlockQuote),
    /// Monospace code block.
    CodeBlock(LIRCodeBlock),
    /// Mathematical equation.
    MathBlock(LIRMathBlock),
    /// Horizontal rule (leaf).
    ThematicBreak(LIRThematicBreak),
    /// Auto-generated table of contents.
    TableOfContents(LIRTableOfContents),
    /// Explicit page break (leaf sentinel).
    PageBreak(LIRPageBreak),
}

impl LIRNode {
    /// Get the unique node ID.
    pub fn id(&self) -> u32 {
        match self {
            Self::Document(n) => n.id,
            Self::Page(n) => n.id,
            Self::Flow(n) => n.id,
            Self::Paragraph(n) => n.id,
            Self::Line(n) => n.id,
            Self::Glyph(n) => n.id,
            Self::Space(n) => n.id,
            Self::Heading(n) => n.id,
            Self::List(n) => n.id,
            Self::ListItem(n) => n.id,
            Self::Table(n) => n.id,
            Self::TableRow(n) => n.id,
            Self::TableCell(n) => n.id,
            Self::Figure(n) => n.id,
            Self::Caption(n) => n.id,
            Self::Footnote(n) => n.id,
            Self::FootnoteBlock(n) => n.id,
            Self::BlockQuote(n) => n.id,
            Self::CodeBlock(n) => n.id,
            Self::MathBlock(n) => n.id,
            Self::ThematicBreak(n) => n.id,
            Self::TableOfContents(n) => n.id,
            Self::PageBreak(n) => n.id,
        }
    }

    /// Get the node's resolved geometry.
    pub fn geometry(&self) -> &LIRGeometry {
        match self {
            Self::Document(n) => &n.geometry,
            Self::Page(n) => &n.geometry,
            Self::Flow(n) => &n.geometry,
            Self::Paragraph(n) => &n.geometry,
            Self::Line(n) => &n.geometry,
            Self::Glyph(n) => &n.geometry,
            Self::Space(n) => &n.geometry,
            Self::Heading(n) => &n.geometry,
            Self::List(n) => &n.geometry,
            Self::ListItem(n) => &n.geometry,
            Self::Table(n) => &n.geometry,
            Self::TableRow(n) => &n.geometry,
            Self::TableCell(n) => &n.geometry,
            Self::Figure(n) => &n.geometry,
            Self::Caption(n) => &n.geometry,
            Self::Footnote(n) => &n.geometry,
            Self::FootnoteBlock(n) => &n.geometry,
            Self::BlockQuote(n) => &n.geometry,
            Self::CodeBlock(n) => &n.geometry,
            Self::MathBlock(n) => &n.geometry,
            Self::ThematicBreak(n) => &n.geometry,
            Self::TableOfContents(n) => &n.geometry,
            Self::PageBreak(n) => &n.geometry,
        }
    }

    /// Get the S-IR source node ID that produced this LIR node.
    pub fn source_node_id(&self) -> Option<u32> {
        match self {
            Self::Document(n) => n.source_node_id,
            Self::Page(n) => n.source_node_id,
            Self::Flow(n) => n.source_node_id,
            Self::Paragraph(n) => n.source_node_id,
            Self::Line(n) => n.source_node_id,
            Self::Glyph(n) => n.source_node_id,
            Self::Space(n) => n.source_node_id,
            Self::Heading(n) => n.source_node_id,
            Self::List(n) => n.source_node_id,
            Self::ListItem(n) => n.source_node_id,
            Self::Table(n) => n.source_node_id,
            Self::TableRow(n) => n.source_node_id,
            Self::TableCell(n) => n.source_node_id,
            Self::Figure(n) => n.source_node_id,
            Self::Caption(n) => n.source_node_id,
            Self::Footnote(n) => n.source_node_id,
            Self::FootnoteBlock(n) => n.source_node_id,
            Self::BlockQuote(n) => n.source_node_id,
            Self::CodeBlock(n) => n.source_node_id,
            Self::MathBlock(n) => n.source_node_id,
            Self::ThematicBreak(n) => n.source_node_id,
            Self::TableOfContents(n) => n.source_node_id,
            Self::PageBreak(n) => n.source_node_id,
        }
    }

    /// Get the variant name as a static string.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Document(_) => "Document",
            Self::Page(_) => "Page",
            Self::Flow(_) => "Flow",
            Self::Paragraph(_) => "Paragraph",
            Self::Line(_) => "Line",
            Self::Glyph(_) => "Glyph",
            Self::Space(_) => "Space",
            Self::Heading(_) => "Heading",
            Self::List(_) => "List",
            Self::ListItem(_) => "ListItem",
            Self::Table(_) => "Table",
            Self::TableRow(_) => "TableRow",
            Self::TableCell(_) => "TableCell",
            Self::Figure(_) => "Figure",
            Self::Caption(_) => "Caption",
            Self::Footnote(_) => "Footnote",
            Self::FootnoteBlock(_) => "FootnoteBlock",
            Self::BlockQuote(_) => "BlockQuote",
            Self::CodeBlock(_) => "CodeBlock",
            Self::MathBlock(_) => "MathBlock",
            Self::ThematicBreak(_) => "ThematicBreak",
            Self::TableOfContents(_) => "TableOfContents",
            Self::PageBreak(_) => "PageBreak",
        }
    }
}

impl fmt::Display for LIRNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LIRNode::{}(id={})", self.type_name(), self.id())
    }
}

// ---------------------------------------------------------------------------
// LIRDocument (DEF-LIR-DOCUMENT)
// ---------------------------------------------------------------------------

/// Top-level laid-out document (DEF-LIR-DOCUMENT).
#[derive(Debug, Clone, PartialEq)]
pub struct LIRDocument {
    /// Unique node ID.
    pub id: u32,
    /// Resolved geometry.
    pub geometry: LIRGeometry,
    /// S-IR source node ID.
    pub source_node_id: Option<u32>,
    /// Resolved style ID.
    pub style_id: Option<u32>,
    /// Document metadata (page geometry, language).
    pub metadata: LIRDocumentMeta,
    /// Ordered list of page boxes.
    pub pages: Vec<LIRPage>,
    /// Collected footnote blocks.
    pub footnotes: Vec<LIRFootnoteBlock>,
    /// Optional auto-generated table of contents.
    pub toc: Option<LIRTableOfContents>,
    /// Resolved style table.
    pub style_table: LIRStyleTable,
}

impl Default for LIRDocument {
    fn default() -> Self {
        Self {
            id: 0,
            geometry: LIRGeometry::ZERO,
            source_node_id: None,
            style_id: None,
            metadata: LIRDocumentMeta::default(),
            pages: Vec::new(),
            footnotes: Vec::new(),
            toc: None,
            style_table: LIRStyleTable::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// LIRPage (DEF-LIR-PAGE)
// ---------------------------------------------------------------------------

/// A single page with absolute dimensions and margins (DEF-LIR-PAGE).
#[derive(Debug, Clone, PartialEq)]
pub struct LIRPage {
    /// Unique node ID.
    pub id: u32,
    /// Resolved geometry.
    pub geometry: LIRGeometry,
    /// S-IR source node ID.
    pub source_node_id: Option<u32>,
    /// Resolved style ID.
    pub style_id: Option<u32>,
    /// Page width in scaled points.
    pub page_width: Fp266,
    /// Page height in scaled points.
    pub page_height: Fp266,
    /// Top margin.
    pub margin_top: Fp266,
    /// Bottom margin.
    pub margin_bottom: Fp266,
    /// Left margin.
    pub margin_left: Fp266,
    /// Right margin.
    pub margin_right: Fp266,
    /// 1-based page number.
    pub page_number: u32,
    /// Flow content for this page.
    pub children: Vec<LIRNode>,
}

impl LIRPage {
    /// Create a new page from metadata.
    pub fn new(page_number: u32, meta: &LIRDocumentMeta) -> Self {
        Self {
            id: 0,
            geometry: LIRGeometry::new(Fp266::ZERO, Fp266::ZERO, meta.page_width, meta.page_height),
            source_node_id: None,
            style_id: None,
            page_width: meta.page_width,
            page_height: meta.page_height,
            margin_top: meta.margin_top,
            margin_bottom: meta.margin_bottom,
            margin_left: meta.margin_left,
            margin_right: meta.margin_right,
            page_number,
            children: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// LIRFlow (DEF-LIR-FLOW)
// ---------------------------------------------------------------------------

/// A vertical stack of block-level boxes within a page or column (DEF-LIR-FLOW).
#[derive(Debug, Clone, PartialEq)]
pub struct LIRFlow {
    /// Unique node ID.
    pub id: u32,
    /// Resolved geometry.
    pub geometry: LIRGeometry,
    /// S-IR source node ID.
    pub source_node_id: Option<u32>,
    /// Resolved style ID.
    pub style_id: Option<u32>,
    /// Stacking direction.
    pub direction: FlowDirection,
    /// Block-level children in flow order.
    pub children: Vec<LIRNode>,
}

impl LIRFlow {
    /// Create a new flow container.
    pub fn new(direction: FlowDirection) -> Self {
        Self {
            id: 0,
            geometry: LIRGeometry::ZERO,
            source_node_id: None,
            style_id: None,
            direction,
            children: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// LIRParagraph
// ---------------------------------------------------------------------------

/// A line-broken paragraph of text.
#[derive(Debug, Clone, PartialEq)]
pub struct LIRParagraph {
    /// Unique node ID.
    pub id: u32,
    /// Resolved geometry.
    pub geometry: LIRGeometry,
    /// S-IR source node ID.
    pub source_node_id: Option<u32>,
    /// Resolved style ID.
    pub style_id: Option<u32>,
    /// Text alignment.
    pub text_align: TextAlign,
    /// First line indent.
    pub first_line_indent: Fp266,
    /// Space before this paragraph.
    pub paragraph_spacing_before: Fp266,
    /// Space after this paragraph.
    pub paragraph_spacing_after: Fp266,
    /// Lines of text.
    pub children: Vec<LIRNode>,
}

impl LIRParagraph {
    /// Create a new empty paragraph (left-aligned, no indent).
    pub fn new() -> Self {
        Self {
            id: 0,
            geometry: LIRGeometry::ZERO,
            source_node_id: None,
            style_id: None,
            text_align: TextAlign::Left,
            first_line_indent: Fp266::ZERO,
            paragraph_spacing_before: Fp266::ZERO,
            paragraph_spacing_after: Fp266::ZERO,
            children: Vec::new(),
        }
    }
}

impl Default for LIRParagraph {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// LIRLine
// ---------------------------------------------------------------------------

/// A single horizontal line within a paragraph or heading.
#[derive(Debug, Clone, PartialEq)]
pub struct LIRLine {
    /// Unique node ID.
    pub id: u32,
    /// Resolved geometry.
    pub geometry: LIRGeometry,
    /// S-IR source node ID.
    pub source_node_id: Option<u32>,
    /// Resolved style ID.
    pub style_id: Option<u32>,
    /// 0-based line number within the paragraph.
    pub line_number: u32,
    /// Knuth-Plass adjustment ratio (0 = optimal, negative = tight, positive = loose).
    pub adjustment_ratio: f32,
    /// Glyphs and spaces within this line.
    pub children: Vec<LIRNode>,
}

impl LIRLine {
    /// Create a new empty line.
    pub fn new(line_number: u32) -> Self {
        Self {
            id: 0,
            geometry: LIRGeometry::ZERO,
            source_node_id: None,
            style_id: None,
            line_number,
            adjustment_ratio: 0.0,
            children: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// LIRGlyph (leaf)
// ---------------------------------------------------------------------------

/// A single positioned glyph — leaf node in the LIR tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LIRGlyph {
    /// Unique node ID.
    pub id: u32,
    /// Resolved geometry.
    pub geometry: LIRGeometry,
    /// S-IR source node ID.
    pub source_node_id: Option<u32>,
    /// Resolved style ID.
    pub style_id: Option<u32>,
    /// Glyph identifier (font-specific).
    pub glyph_id: u32,
    /// Font resource identifier.
    pub font_id: u32,
    /// Horizontal advance width.
    pub advance_x: Fp266,
}

impl LIRGlyph {
    /// Create a new glyph.
    pub fn new(glyph_id: u32, font_id: u32, advance_x: Fp266) -> Self {
        Self {
            id: 0,
            geometry: LIRGeometry::ZERO,
            source_node_id: None,
            style_id: None,
            glyph_id,
            font_id,
            advance_x,
        }
    }
}

// ---------------------------------------------------------------------------
// LIRSpace (leaf)
// ---------------------------------------------------------------------------

/// Inter-word or inter-glyph spacing — leaf node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LIRSpace {
    /// Unique node ID.
    pub id: u32,
    /// Resolved geometry.
    pub geometry: LIRGeometry,
    /// S-IR source node ID.
    pub source_node_id: Option<u32>,
    /// Resolved style ID.
    pub style_id: Option<u32>,
    /// Natural width of the space.
    pub natural_width: Fp266,
    /// Stretchability (how much it can grow).
    pub stretch: Fp266,
    /// Shrinkability (how much it can shrink).
    pub shrink: Fp266,
}

impl LIRSpace {
    /// Create a space with natural width only (no stretch/shrink).
    pub fn new(natural_width: Fp266) -> Self {
        Self {
            id: 0,
            geometry: LIRGeometry::new(natural_width, Fp266::ZERO, natural_width, Fp266::ZERO),
            source_node_id: None,
            style_id: None,
            natural_width,
            stretch: Fp266::ZERO,
            shrink: Fp266::ZERO,
        }
    }

    /// Create a glue space with stretch and shrink (Knuth-Plass model).
    pub fn with_glue(natural_width: Fp266, stretch: Fp266, shrink: Fp266) -> Self {
        Self {
            id: 0,
            geometry: LIRGeometry::new(natural_width, Fp266::ZERO, natural_width, Fp266::ZERO),
            source_node_id: None,
            style_id: None,
            natural_width,
            stretch,
            shrink,
        }
    }
}

// ---------------------------------------------------------------------------
// LIRHeading
// ---------------------------------------------------------------------------

/// A numbered section heading.
#[derive(Debug, Clone, PartialEq)]
pub struct LIRHeading {
    /// Unique node ID.
    pub id: u32,
    /// Resolved geometry.
    pub geometry: LIRGeometry,
    /// S-IR source node ID.
    pub source_node_id: Option<u32>,
    /// Resolved style ID.
    pub style_id: Option<u32>,
    /// Heading level (1 = chapter, 2 = section, etc.).
    pub level: u8,
    /// Section number (e.g. "1.2.3").
    pub number: String,
    /// Heading label text.
    pub label: String,
    /// Lines of text within the heading.
    pub children: Vec<LIRNode>,
}

impl LIRHeading {
    /// Create a new heading at the given level.
    pub fn new(level: u8) -> Self {
        Self {
            id: 0,
            geometry: LIRGeometry::ZERO,
            source_node_id: None,
            style_id: None,
            level,
            number: String::new(),
            label: String::new(),
            children: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// LIRList
// ---------------------------------------------------------------------------

/// An ordered or unordered list.
#[derive(Debug, Clone, PartialEq)]
pub struct LIRList {
    /// Unique node ID.
    pub id: u32,
    /// Resolved geometry.
    pub geometry: LIRGeometry,
    /// S-IR source node ID.
    pub source_node_id: Option<u32>,
    /// Resolved style ID.
    pub style_id: Option<u32>,
    /// List marker type.
    pub list_type: ListType,
    /// Starting number (for ordered lists).
    pub start: u32,
    /// Content indentation from the left margin.
    pub indent: Fp266,
    /// Distance from list edge to marker.
    pub marker_indent: Fp266,
    /// List items.
    pub children: Vec<LIRNode>,
}

impl LIRList {
    /// Create a new list.
    pub fn new(list_type: ListType) -> Self {
        Self {
            id: 0,
            geometry: LIRGeometry::ZERO,
            source_node_id: None,
            style_id: None,
            list_type,
            start: 1,
            indent: Fp266::from_int(36),
            marker_indent: Fp266::from_int(18),
            children: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// LIRListItem
// ---------------------------------------------------------------------------

/// A single list item with optional marker.
#[derive(Debug, Clone, PartialEq)]
pub struct LIRListItem {
    /// Unique node ID.
    pub id: u32,
    /// Resolved geometry.
    pub geometry: LIRGeometry,
    /// S-IR source node ID.
    pub source_node_id: Option<u32>,
    /// Resolved style ID.
    pub style_id: Option<u32>,
    /// Marker text (e.g. "1.", "a.", "*").
    pub marker: Option<String>,
    /// Item content.
    pub children: Vec<LIRNode>,
}

impl LIRListItem {
    /// Create a new list item.
    pub fn new() -> Self {
        Self {
            id: 0,
            geometry: LIRGeometry::ZERO,
            source_node_id: None,
            style_id: None,
            marker: None,
            children: Vec::new(),
        }
    }
}

impl Default for LIRListItem {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// LIRTable
// ---------------------------------------------------------------------------

/// A grid layout table.
#[derive(Debug, Clone, PartialEq)]
pub struct LIRTable {
    /// Unique node ID.
    pub id: u32,
    /// Resolved geometry.
    pub geometry: LIRGeometry,
    /// S-IR source node ID.
    pub source_node_id: Option<u32>,
    /// Resolved style ID.
    pub style_id: Option<u32>,
    /// Number of columns.
    pub num_cols: u16,
    /// Column widths in scaled points.
    pub col_widths: Vec<Fp266>,
    /// Whether the table has a border.
    pub border: bool,
    /// Table rows.
    pub children: Vec<LIRNode>,
}

impl LIRTable {
    /// Create a new table with the given number of columns.
    pub fn new(num_cols: u16) -> Self {
        Self {
            id: 0,
            geometry: LIRGeometry::ZERO,
            source_node_id: None,
            style_id: None,
            num_cols,
            col_widths: vec![Fp266::ZERO; num_cols as usize],
            border: false,
            children: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// LIRTableRow
// ---------------------------------------------------------------------------

/// A single row in a table.
#[derive(Debug, Clone, PartialEq)]
pub struct LIRTableRow {
    /// Unique node ID.
    pub id: u32,
    /// Resolved geometry.
    pub geometry: LIRGeometry,
    /// S-IR source node ID.
    pub source_node_id: Option<u32>,
    /// Resolved style ID.
    pub style_id: Option<u32>,
    /// Whether this is a header row.
    pub is_header: bool,
    /// Table cells in this row.
    pub children: Vec<LIRNode>,
}

impl LIRTableRow {
    /// Create a new table row.
    pub fn new(is_header: bool) -> Self {
        Self {
            id: 0,
            geometry: LIRGeometry::ZERO,
            source_node_id: None,
            style_id: None,
            is_header,
            children: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// LIRTableCell
// ---------------------------------------------------------------------------

/// A single cell in a table row.
#[derive(Debug, Clone, PartialEq)]
pub struct LIRTableCell {
    /// Unique node ID.
    pub id: u32,
    /// Resolved geometry.
    pub geometry: LIRGeometry,
    /// S-IR source node ID.
    pub source_node_id: Option<u32>,
    /// Resolved style ID.
    pub style_id: Option<u32>,
    /// Column index (0-based).
    pub col: u16,
    /// Number of columns spanned.
    pub colspan: u16,
    /// Number of rows spanned.
    pub rowspan: u16,
    /// Cell padding.
    pub padding: Padding,
    /// Cell content (typically a single LIRFlow).
    pub children: Vec<LIRNode>,
}

impl LIRTableCell {
    /// Create a new table cell at the given column.
    pub fn new(col: u16) -> Self {
        Self {
            id: 0,
            geometry: LIRGeometry::ZERO,
            source_node_id: None,
            style_id: None,
            col,
            colspan: 1,
            rowspan: 1,
            padding: Padding::ZERO,
            children: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// LIRFigure
// ---------------------------------------------------------------------------

/// A figure (image) with optional caption.
#[derive(Debug, Clone, PartialEq)]
pub struct LIRFigure {
    /// Unique node ID.
    pub id: u32,
    /// Resolved geometry.
    pub geometry: LIRGeometry,
    /// S-IR source node ID.
    pub source_node_id: Option<u32>,
    /// Resolved style ID.
    pub style_id: Option<u32>,
    /// Figure placement strategy.
    pub placement: Placement,
    /// Index into the document's image table.
    pub image_index: Option<u32>,
    /// Optional caption below the figure.
    pub caption: Option<Box<LIRCaption>>,
}

impl LIRFigure {
    /// Create a new empty figure.
    pub fn new() -> Self {
        Self {
            id: 0,
            geometry: LIRGeometry::ZERO,
            source_node_id: None,
            style_id: None,
            placement: Placement::Here,
            image_index: None,
            caption: None,
        }
    }
}

impl Default for LIRFigure {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// LIRCaption
// ---------------------------------------------------------------------------

/// A figure or table caption.
#[derive(Debug, Clone, PartialEq)]
pub struct LIRCaption {
    /// Unique node ID.
    pub id: u32,
    /// Resolved geometry.
    pub geometry: LIRGeometry,
    /// S-IR source node ID.
    pub source_node_id: Option<u32>,
    /// Resolved style ID.
    pub style_id: Option<u32>,
    /// Caption category (e.g. "Figure", "Table").
    pub category: String,
    /// Caption number within the category.
    pub number: u32,
    /// Caption text lines.
    pub children: Vec<LIRNode>,
}

impl LIRCaption {
    /// Create a new caption.
    pub fn new(category: impl Into<String>, number: u32) -> Self {
        Self {
            id: 0,
            geometry: LIRGeometry::ZERO,
            source_node_id: None,
            style_id: None,
            category: category.into(),
            number,
            children: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// LIRFootnote (leaf, in-flow marker)
// ---------------------------------------------------------------------------

/// An in-flow footnote superscript marker — leaf node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LIRFootnote {
    /// Unique node ID.
    pub id: u32,
    /// Resolved geometry.
    pub geometry: LIRGeometry,
    /// S-IR source node ID.
    pub source_node_id: Option<u32>,
    /// Resolved style ID.
    pub style_id: Option<u32>,
    /// Footnote identifier (matches a footnote block).
    pub footnote_id: u32,
    /// Superscript marker character (e.g. b'*', b'1').
    pub marker: u8,
}

impl LIRFootnote {
    /// Create a new footnote marker.
    pub fn new(footnote_id: u32, marker: u8) -> Self {
        Self {
            id: 0,
            geometry: LIRGeometry::ZERO,
            source_node_id: None,
            style_id: None,
            footnote_id,
            marker,
        }
    }
}

// ---------------------------------------------------------------------------
// LIRFootnoteBlock
// ---------------------------------------------------------------------------

/// A block of collected footnote paragraphs (typically at page bottom).
#[derive(Debug, Clone, PartialEq)]
pub struct LIRFootnoteBlock {
    /// Unique node ID.
    pub id: u32,
    /// Resolved geometry.
    pub geometry: LIRGeometry,
    /// S-IR source node ID.
    pub source_node_id: Option<u32>,
    /// Resolved style ID.
    pub style_id: Option<u32>,
    /// Footnote IDs contained in this block.
    pub footnote_ids: Vec<u32>,
    /// Footnote paragraphs.
    pub children: Vec<LIRNode>,
}

impl LIRFootnoteBlock {
    /// Create a new empty footnote block.
    pub fn new() -> Self {
        Self {
            id: 0,
            geometry: LIRGeometry::ZERO,
            source_node_id: None,
            style_id: None,
            footnote_ids: Vec::new(),
            children: Vec::new(),
        }
    }
}

impl Default for LIRFootnoteBlock {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// LIRBlockQuote
// ---------------------------------------------------------------------------

/// A block quotation with indentation and optional left rule.
#[derive(Debug, Clone, PartialEq)]
pub struct LIRBlockQuote {
    /// Unique node ID.
    pub id: u32,
    /// Resolved geometry.
    pub geometry: LIRGeometry,
    /// S-IR source node ID.
    pub source_node_id: Option<u32>,
    /// Resolved style ID.
    pub style_id: Option<u32>,
    /// Left indentation.
    pub indent: Fp266,
    /// Left rule thickness.
    pub rule_width: Fp266,
    /// Left rule color (packed RGBA).
    pub rule_color: u32,
    /// Quoted content.
    pub children: Vec<LIRNode>,
}

impl LIRBlockQuote {
    /// Create a new block quote with default styling.
    pub fn new() -> Self {
        Self {
            id: 0,
            geometry: LIRGeometry::ZERO,
            source_node_id: None,
            style_id: None,
            indent: Fp266::from_int(36),
            rule_width: Fp266::from_int(2),
            rule_color: 0x000000FF,
            children: Vec::new(),
        }
    }
}

impl Default for LIRBlockQuote {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// LIRCodeBlock
// ---------------------------------------------------------------------------

/// A monospace code block with pre-broken lines.
#[derive(Debug, Clone, PartialEq)]
pub struct LIRCodeBlock {
    /// Unique node ID.
    pub id: u32,
    /// Resolved geometry.
    pub geometry: LIRGeometry,
    /// S-IR source node ID.
    pub source_node_id: Option<u32>,
    /// Resolved style ID.
    pub style_id: Option<u32>,
    /// Programming language identifier.
    pub language: String,
    /// Background color (packed RGBA).
    pub background_color: u32,
    /// Pre-broken code lines.
    pub children: Vec<LIRNode>,
}

impl LIRCodeBlock {
    /// Create a new code block for the given language.
    pub fn new(language: impl Into<String>) -> Self {
        Self {
            id: 0,
            geometry: LIRGeometry::ZERO,
            source_node_id: None,
            style_id: None,
            language: language.into(),
            background_color: 0xF5F5F5FF,
            children: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// LIRMathBlock
// ---------------------------------------------------------------------------

/// A mathematical equation (display or inline).
#[derive(Debug, Clone, PartialEq)]
pub struct LIRMathBlock {
    /// Unique node ID.
    pub id: u32,
    /// Resolved geometry.
    pub geometry: LIRGeometry,
    /// S-IR source node ID.
    pub source_node_id: Option<u32>,
    /// Resolved style ID.
    pub style_id: Option<u32>,
    /// Math display type.
    pub math_type: MathType,
    /// Equation number (for numbered equations).
    pub number: Option<u32>,
    /// Math symbols and glyphs.
    pub children: Vec<LIRNode>,
}

impl LIRMathBlock {
    /// Create a new math block.
    pub fn new(math_type: MathType) -> Self {
        Self {
            id: 0,
            geometry: LIRGeometry::ZERO,
            source_node_id: None,
            style_id: None,
            math_type,
            number: None,
            children: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// LIRThematicBreak (leaf)
// ---------------------------------------------------------------------------

/// A horizontal rule — leaf node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LIRThematicBreak {
    /// Unique node ID.
    pub id: u32,
    /// Resolved geometry.
    pub geometry: LIRGeometry,
    /// S-IR source node ID.
    pub source_node_id: Option<u32>,
    /// Resolved style ID.
    pub style_id: Option<u32>,
    /// Rule thickness.
    pub thickness: Fp266,
    /// Rule color (packed RGBA).
    pub color: u32,
}

impl LIRThematicBreak {
    /// Create a new thematic break with default styling (1pt, black).
    pub fn new() -> Self {
        Self {
            id: 0,
            geometry: LIRGeometry::ZERO,
            source_node_id: None,
            style_id: None,
            thickness: Fp266::from_int(1),
            color: 0x000000FF,
        }
    }
}

impl Default for LIRThematicBreak {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// LIRTableOfContents
// ---------------------------------------------------------------------------

/// A single entry in the table of contents.
#[derive(Debug, Clone, PartialEq)]
pub struct TOCEntry {
    /// Heading level.
    pub level: u8,
    /// Section number string (e.g. "1.2.3").
    pub number: String,
    /// Heading label text.
    pub label: String,
    /// Page number where this heading appears.
    pub page_number: u32,
}

/// Auto-generated table of contents.
#[derive(Debug, Clone, PartialEq)]
pub struct LIRTableOfContents {
    /// Unique node ID.
    pub id: u32,
    /// Resolved geometry.
    pub geometry: LIRGeometry,
    /// S-IR source node ID.
    pub source_node_id: Option<u32>,
    /// Resolved style ID.
    pub style_id: Option<u32>,
    /// Maximum heading depth to include.
    pub max_depth: u8,
    /// TOC entries.
    pub entries: Vec<TOCEntry>,
}

impl LIRTableOfContents {
    /// Create a new empty table of contents.
    pub fn new(max_depth: u8) -> Self {
        Self {
            id: 0,
            geometry: LIRGeometry::ZERO,
            source_node_id: None,
            style_id: None,
            max_depth,
            entries: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// LIRPageBreak (leaf, zero-height sentinel)
// ---------------------------------------------------------------------------

/// An explicit page break — zero-height sentinel leaf node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LIRPageBreak {
    /// Unique node ID.
    pub id: u32,
    /// Resolved geometry (zero-height).
    pub geometry: LIRGeometry,
    /// S-IR source node ID.
    pub source_node_id: Option<u32>,
    /// Resolved style ID.
    pub style_id: Option<u32>,
}

impl LIRPageBreak {
    /// Create a new page break sentinel.
    pub fn new() -> Self {
        Self {
            id: 0,
            geometry: LIRGeometry::ZERO,
            source_node_id: None,
            style_id: None,
        }
    }
}

impl Default for LIRPageBreak {
    fn default() -> Self {
        Self::new()
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geometry_zero() {
        let g = LIRGeometry::ZERO;
        assert!(g.x.is_zero());
        assert!(g.y.is_zero());
        assert!(g.width.is_zero());
        assert!(g.height.is_zero());
        assert!(g.baseline.is_zero());
    }

    #[test]
    fn test_geometry_from_int() {
        let g = LIRGeometry::from_int(10, 20, 300, 400);
        assert_eq!(g.x, Fp266::from_int(10));
        assert_eq!(g.y, Fp266::from_int(20));
        assert_eq!(g.width, Fp266::from_int(300));
        assert_eq!(g.height, Fp266::from_int(400));
        assert!(g.baseline.is_zero());
    }

    #[test]
    fn test_geometry_with_baseline() {
        let g = LIRGeometry::with_baseline(
            Fp266::ZERO,
            Fp266::ZERO,
            Fp266::from_int(300),
            Fp266::from_int(12),
            Fp266::from_int(10),
        );
        assert_eq!(g.baseline, Fp266::from_int(10));
    }

    #[test]
    fn test_geometry_to_rect() {
        let g = LIRGeometry::from_int(10, 20, 100, 200);
        let r = g.to_rect();
        assert_eq!(r.x, Fp266::from_int(10));
        assert_eq!(r.y, Fp266::from_int(20));
        assert_eq!(r.width, Fp266::from_int(100));
        assert_eq!(r.height, Fp266::from_int(200));
    }

    #[test]
    fn test_document_meta_us_letter() {
        let meta = LIRDocumentMeta::us_letter();
        assert_eq!(meta.page_width, Fp266::from_int(612));
        assert_eq!(meta.page_height, Fp266::from_int(792));
        assert_eq!(meta.margin_top, Fp266::from_int(72));
        assert_eq!(meta.content_width(), Fp266::from_int(468));
        assert_eq!(meta.content_height(), Fp266::from_int(648));
    }

    #[test]
    fn test_document_construction() {
        let doc = LIRDocument::default();
        assert_eq!(doc.id, 0);
        assert!(doc.pages.is_empty());
        assert!(doc.footnotes.is_empty());
        assert!(doc.toc.is_none());
        assert_eq!(doc.metadata.language, "en");
    }

    #[test]
    fn test_page_construction() {
        let meta = LIRDocumentMeta::us_letter();
        let page = LIRPage::new(1, &meta);
        assert_eq!(page.page_number, 1);
        assert_eq!(page.page_width, Fp266::from_int(612));
        assert_eq!(page.geometry.width, Fp266::from_int(612));
        assert!(page.children.is_empty());
    }

    #[test]
    fn test_glyph_construction() {
        let g = LIRGlyph::new(42, 0, Fp266::from_int(10));
        assert_eq!(g.glyph_id, 42);
        assert_eq!(g.font_id, 0);
        assert_eq!(g.advance_x, Fp266::from_int(10));
    }

    #[test]
    fn test_space_construction() {
        let s = LIRSpace::new(Fp266::from_int(5));
        assert_eq!(s.natural_width, Fp266::from_int(5));
        assert!(s.stretch.is_zero());
    }

    #[test]
    fn test_space_with_glue() {
        let s = LIRSpace::with_glue(Fp266::from_int(5), Fp266::from_int(2), Fp266::from_int(1));
        assert_eq!(s.natural_width, Fp266::from_int(5));
        assert_eq!(s.stretch, Fp266::from_int(2));
        assert_eq!(s.shrink, Fp266::from_int(1));
    }

    #[test]
    fn test_heading_construction() {
        let h = LIRHeading::new(2);
        assert_eq!(h.level, 2);
        assert!(h.children.is_empty());
    }

    #[test]
    fn test_list_construction() {
        let l = LIRList::new(ListType::Ordered);
        assert_eq!(l.list_type, ListType::Ordered);
        assert_eq!(l.start, 1);
    }

    #[test]
    fn test_table_construction() {
        let t = LIRTable::new(3);
        assert_eq!(t.num_cols, 3);
        assert_eq!(t.col_widths.len(), 3);
    }

    #[test]
    fn test_table_cell_construction() {
        let c = LIRTableCell::new(0);
        assert_eq!(c.col, 0);
        assert_eq!(c.colspan, 1);
        assert_eq!(c.rowspan, 1);
    }

    #[test]
    fn test_figure_with_caption() {
        let mut fig = LIRFigure::new();
        fig.image_index = Some(0);
        fig.caption = Some(Box::new(LIRCaption::new("Figure", 1)));
        assert!(fig.image_index.is_some());
        assert!(fig.caption.is_some());
    }

    #[test]
    fn test_footnote_construction() {
        let f = LIRFootnote::new(1, b'*');
        assert_eq!(f.footnote_id, 1);
        assert_eq!(f.marker, b'*');
    }

    #[test]
    fn test_code_block_construction() {
        let cb = LIRCodeBlock::new("rust");
        assert_eq!(cb.language, "rust");
    }

    #[test]
    fn test_math_block_construction() {
        let mb = LIRMathBlock::new(MathType::Display);
        assert_eq!(mb.math_type, MathType::Display);
    }

    #[test]
    fn test_thematic_break_construction() {
        let hr = LIRThematicBreak::new();
        assert_eq!(hr.thickness, Fp266::from_int(1));
    }

    #[test]
    fn test_toc_entry_construction() {
        let entry = TOCEntry {
            level: 1,
            number: "1".to_string(),
            label: "Introduction".to_string(),
            page_number: 1,
        };
        assert_eq!(entry.level, 1);
        assert_eq!(entry.page_number, 1);
    }

    #[test]
    fn test_toc_construction() {
        let toc = LIRTableOfContents::new(3);
        assert_eq!(toc.max_depth, 3);
        assert!(toc.entries.is_empty());
    }

    #[test]
    fn test_node_enum_id_access() {
        let node = LIRNode::Glyph(LIRGlyph::new(42, 0, Fp266::from_int(10)));
        assert_eq!(node.type_name(), "Glyph");
    }

    #[test]
    fn test_node_enum_display() {
        let node = LIRNode::Page(LIRPage::new(1, &LIRDocumentMeta::us_letter()));
        let s = format!("{node}");
        assert!(s.contains("Page"));
    }

    #[test]
    fn test_tree_building() {
        let mut doc = LIRDocument::default();
        let meta = LIRDocumentMeta::us_letter();

        let mut page = LIRPage::new(1, &meta);
        page.id = 1;

        let mut flow = LIRFlow::new(FlowDirection::TopToBottom);
        flow.id = 2;
        flow.geometry = LIRGeometry::from_int(72, 72, 468, 648);

        let mut para = LIRParagraph::new();
        para.id = 3;
        para.geometry = LIRGeometry::from_int(72, 72, 468, 48);

        let mut line = LIRLine::new(0);
        line.id = 4;
        line.geometry = LIRGeometry::with_baseline(
            Fp266::from_int(72),
            Fp266::from_int(72),
            Fp266::from_int(468),
            Fp266::from_int(12),
            Fp266::from_int(10),
        );

        let mut glyph = LIRGlyph::new(72, 0, Fp266::from_int(10));
        glyph.id = 5;
        glyph.geometry = LIRGeometry::new(
            Fp266::from_int(72),
            Fp266::from_int(72),
            Fp266::from_int(10),
            Fp266::from_int(12),
        );

        line.children.push(LIRNode::Glyph(glyph));
        para.children.push(LIRNode::Line(line));
        flow.children.push(LIRNode::Paragraph(para));
        page.children.push(LIRNode::Flow(flow));
        doc.pages.push(page);

        assert_eq!(doc.pages.len(), 1);
        assert_eq!(doc.pages[0].children.len(), 1);

        let page_ref = &doc.pages[0];
        if let LIRNode::Flow(flow_ref) = &page_ref.children[0] {
            if let LIRNode::Paragraph(para_ref) = &flow_ref.children[0] {
                assert_eq!(para_ref.children.len(), 1);
                if let LIRNode::Line(line_ref) = &para_ref.children[0] {
                    assert_eq!(line_ref.children.len(), 1);
                    if let LIRNode::Glyph(g) = &line_ref.children[0] {
                        assert_eq!(g.glyph_id, 72);
                    } else {
                        panic!("expected glyph");
                    }
                } else {
                    panic!("expected line");
                }
            } else {
                panic!("expected paragraph");
            }
        } else {
            panic!("expected flow");
        }
    }

    #[test]
    fn test_nested_table_tree() {
        let mut table = LIRTable::new(2);
        table.id = 10;
        table.col_widths = vec![Fp266::from_int(200), Fp266::from_int(268)];

        let mut row = LIRTableRow::new(false);
        row.id = 11;

        let mut cell = LIRTableCell::new(0);
        cell.id = 12;
        let mut cell_flow = LIRFlow::new(FlowDirection::TopToBottom);
        cell_flow.id = 13;
        cell.children.push(LIRNode::Flow(cell_flow));

        row.children.push(LIRNode::TableCell(cell));
        table.children.push(LIRNode::TableRow(row));

        if let LIRNode::Table(t) = LIRNode::Table(table) {
            assert_eq!(t.num_cols, 2);
            if let LIRNode::TableRow(r) = &t.children[0] {
                if let LIRNode::TableCell(c) = &r.children[0] {
                    assert_eq!(c.col, 0);
                    assert_eq!(c.children.len(), 1);
                } else {
                    panic!("expected cell");
                }
            } else {
                panic!("expected row");
            }
        }
    }

    #[test]
    fn test_block_quote_with_content() {
        let mut bq = LIRBlockQuote::new();
        bq.id = 20;
        bq.indent = Fp266::from_int(48);

        let mut inner_para = LIRParagraph::new();
        inner_para.id = 21;
        bq.children.push(LIRNode::Paragraph(inner_para));

        assert_eq!(bq.indent, Fp266::from_int(48));
        assert_eq!(bq.children.len(), 1);
    }

    #[test]
    fn test_all_node_types_constructible() {
        let _ = LIRNode::Document(LIRDocument::default());
        let _ = LIRNode::Page(LIRPage::new(1, &LIRDocumentMeta::us_letter()));
        let _ = LIRNode::Flow(LIRFlow::new(FlowDirection::TopToBottom));
        let _ = LIRNode::Paragraph(LIRParagraph::new());
        let _ = LIRNode::Line(LIRLine::new(0));
        let _ = LIRNode::Glyph(LIRGlyph::new(0, 0, Fp266::ZERO));
        let _ = LIRNode::Space(LIRSpace::new(Fp266::ZERO));
        let _ = LIRNode::Heading(LIRHeading::new(1));
        let _ = LIRNode::List(LIRList::new(ListType::Unordered));
        let _ = LIRNode::ListItem(LIRListItem::new());
        let _ = LIRNode::Table(LIRTable::new(1));
        let _ = LIRNode::TableRow(LIRTableRow::new(false));
        let _ = LIRNode::TableCell(LIRTableCell::new(0));
        let _ = LIRNode::Figure(LIRFigure::new());
        let _ = LIRNode::Caption(LIRCaption::new("Fig", 1));
        let _ = LIRNode::Footnote(LIRFootnote::new(0, 1));
        let _ = LIRNode::FootnoteBlock(LIRFootnoteBlock::new());
        let _ = LIRNode::BlockQuote(LIRBlockQuote::new());
        let _ = LIRNode::CodeBlock(LIRCodeBlock::new("text"));
        let _ = LIRNode::MathBlock(LIRMathBlock::new(MathType::Inline));
        let _ = LIRNode::ThematicBreak(LIRThematicBreak::new());
        let _ = LIRNode::TableOfContents(LIRTableOfContents::new(3));
        let _ = LIRNode::PageBreak(LIRPageBreak::new());
    }

    #[test]
    fn test_leaf_nodes_are_copy() {
        let g1 = LIRGlyph::new(1, 0, Fp266::from_int(10));
        let g2 = g1;
        assert_eq!(g1.glyph_id, g2.glyph_id);

        let s1 = LIRSpace::new(Fp266::from_int(5));
        let s2 = s1;
        assert_eq!(s1.natural_width, s2.natural_width);

        let f1 = LIRFootnote::new(1, 1);
        let f2 = f1;
        assert_eq!(f1.footnote_id, f2.footnote_id);

        let h1 = LIRThematicBreak::new();
        let h2 = h1;
        assert_eq!(h1.thickness, h2.thickness);

        let p1 = LIRPageBreak::new();
        let p2 = p1;
        assert_eq!(p1.id, p2.id);
    }

    #[test]
    fn test_node_geometry_access() {
        let mut glyph = LIRGlyph::new(42, 0, Fp266::from_int(10));
        glyph.geometry = LIRGeometry::from_int(100, 200, 10, 12);
        let node = LIRNode::Glyph(glyph);
        let geo = node.geometry();
        assert_eq!(geo.x, Fp266::from_int(100));
        assert_eq!(geo.y, Fp266::from_int(200));
    }

    #[test]
    fn test_node_source_node_id() {
        let mut para = LIRParagraph::new();
        para.source_node_id = Some(99);
        let node = LIRNode::Paragraph(para);
        assert_eq!(node.source_node_id(), Some(99));
    }
}
