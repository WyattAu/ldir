//! L-IR (Layout Intermediate Representation) module.
//!
//! Positioned box tree capturing all layout decisions as explicit geometry.
//! Serves as the single source of truth for rendering to PDF/HTML/EPUB.
//!
//! ## Module Structure
//!
//! - `types`: All 23 LIR node types and the `LIRNode` enum
//! - `position`: Point, Size, Rect using Fp266
//! - `style`: Text styles, alignment, flow direction, padding
//!
//! ## References
//!
//! - YP-LAYOUT-LIR-001: Full specification
//! - AX-LIR-001: All geometry is 26.6 fixed-point
//! - AX-LIR-002: Tree well-formedness

pub mod position;
pub mod style;
pub mod types;

pub use position::{Point, Rect, Size};
pub use style::{
    FlowDirection, LIRStyleTable, LIRTextStyle, ListType, MathType, Padding, Placement, TextAlign,
};
pub use types::{
    LIRBlockQuote, LIRCaption, LIRCodeBlock, LIRDocument, LIRDocumentMeta, LIRFigure, LIRFlow,
    LIRFootnote, LIRFootnoteBlock, LIRGeometry, LIRGlyph, LIRHeading, LIRLine, LIRList,
    LIRListItem, LIRMathBlock, LIRNode, LIRPage, LIRPageBreak, LIRParagraph, LIRSpace, LIRTable,
    LIRTableCell, LIRTableOfContents, LIRTableRow, LIRThematicBreak, TOCEntry,
};
