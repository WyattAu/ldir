//! G-IR document type (DEF-003).
//!
//! A `GIRDocument` is an ordered sequence of pages representing the
//! complete rendered output of a compiled document.
//!
//! Matches Lean 4 `abbrev GIRDocument := List GIRPage` in
//! `ProofIRWellformedness.lean` Section 2.
//!
//! # Well-Formedness (DEF-005)
//!
//! Per DEF-005, every page must satisfy `pageStackBalanced`.
//! Per DEF-003, a document must have at least 1 page.
//!
//! Matches Lean 4 `wellFormedGIR` in Section 4:
//! ```lean
//! def wellFormedGIR (doc : GIRDocument) : Bool :=
//!   doc.all pageStackBalanced
//! ```

use crate::gir::page::GIRPage;

/// Image format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// PNG image format.
    Png,
    /// JPEG image format.
    Jpeg,
}

/// Image data for embedding in PDF.
#[derive(Debug, Clone, PartialEq)]
pub struct GIRImage {
    /// Raw image bytes (PNG or JPEG).
    pub data: Vec<u8>,
    /// Display width in fp26_6.
    pub width: i32,
    /// Display height in fp26_6.
    pub height: i32,
    /// Image format.
    pub format: ImageFormat,
}

/// Ordered sequence of G-IR pages representing a compiled document.
///
/// Per DEF-003: a document must have at least 1 page.
/// Per DEF-005: every page must have a balanced coordinate stack.
///
/// # Examples
///
/// ```
/// use ldir_ir::gir::{GIRDocument, GIRPage, GIRCommand};
///
/// let mut doc = GIRDocument::new();
/// let mut page = GIRPage::new();
/// page.push(GIRCommand::new_push_stack());
/// page.push(GIRCommand::new_set_font(0));
/// page.push(GIRCommand::new_pop_stack());
/// doc.push_page(page);
///
/// assert_eq!(doc.page_count(), 1);
/// assert!(doc.is_well_formed());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct GIRDocument {
    pages: Vec<GIRPage>,
    images: Vec<GIRImage>,
}

impl GIRDocument {
    /// Create a new empty G-IR document.
    #[inline]
    pub fn new() -> Self {
        Self {
            pages: Vec::new(),
            images: Vec::new(),
        }
    }

    /// Create a new G-IR document with pre-allocated capacity for pages.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            pages: Vec::with_capacity(capacity),
            images: Vec::new(),
        }
    }

    /// Number of pages in the document.
    ///
    /// Per DEF-003: must be >= 1 for a valid document.
    #[inline]
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// Check if the document has no pages.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }

    /// Get a page by index.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&GIRPage> {
        self.pages.get(index)
    }

    /// Get a mutable reference to a page by index.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut GIRPage> {
        self.pages.get_mut(index)
    }

    /// Add a page to the document.
    #[inline]
    pub fn push_page(&mut self, page: GIRPage) {
        self.pages.push(page);
    }

    /// Add an image to the document's image table and return its index.
    pub fn push_image(&mut self, image: GIRImage) -> usize {
        let idx = self.images.len();
        self.images.push(image);
        idx
    }

    /// Get the image table.
    #[inline]
    pub fn images(&self) -> &[GIRImage] {
        &self.images
    }

    /// Add a new empty page with default dimensions and return a mutable
    /// reference to it.
    pub fn new_page(&mut self) -> &mut GIRPage {
        let idx = self.pages.len();
        self.pages.push(GIRPage::new());
        &mut self.pages[idx]
    }

    /// Iterate over pages in order.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &GIRPage> {
        self.pages.iter()
    }

    /// Iterate over pages mutably.
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut GIRPage> {
        self.pages.iter_mut()
    }

    /// Check if all pages have balanced coordinate stacks (DEF-005).
    ///
    /// Implements `wellFormedGIR` from `ProofIRWellformedness.lean` Section 4:
    /// ```lean
    /// def wellFormedGIR (doc : GIRDocument) : Bool :=
    ///   doc.all pageStackBalanced
    /// ```
    ///
    /// Note: This checks DEF-005 cond. 3 (stack balance) only.
    /// A complete well-formedness check also verifies coordinate ranges
    /// and font precedence (DEF-005 conds. 1, 2, 4).
    pub fn is_well_formed(&self) -> bool {
        self.pages.iter().all(|p| p.is_stack_balanced())
    }

    /// Total number of commands across all pages.
    pub fn total_commands(&self) -> usize {
        self.pages.iter().map(|p| p.len()).sum()
    }

    /// Get the raw page slice.
    #[inline]
    pub fn as_slice(&self) -> &[GIRPage] {
        &self.pages
    }

    /// Clear all pages.
    pub fn clear(&mut self) {
        self.pages.clear();
        self.images.clear();
    }

    /// Reserve capacity for additional pages.
    pub fn reserve(&mut self, additional: usize) {
        self.pages.reserve(additional);
    }
}

impl Default for GIRDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Deref for GIRDocument {
    type Target = [GIRPage];

    fn deref(&self) -> &Self::Target {
        &self.pages
    }
}

impl std::ops::DerefMut for GIRDocument {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.pages
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gir::command::GIRCommand;

    fn make_balanced_page() -> GIRPage {
        let mut page = GIRPage::new();
        page.push(GIRCommand::new_push_stack());
        page.push(GIRCommand::new_set_font(0));
        page.push(GIRCommand::new_put_glyph(65, 640));
        page.push(GIRCommand::new_pop_stack());
        page
    }

    #[test]
    fn test_new_empty() {
        let doc = GIRDocument::new();
        assert!(doc.is_empty());
        assert_eq!(doc.page_count(), 0);
    }

    #[test]
    fn test_single_page() {
        let mut doc = GIRDocument::new();
        doc.push_page(make_balanced_page());
        assert_eq!(doc.page_count(), 1);
        assert!(doc.is_well_formed());
    }

    #[test]
    fn test_multiple_pages() {
        let mut doc = GIRDocument::new();
        doc.push_page(make_balanced_page());
        doc.push_page(make_balanced_page());
        assert_eq!(doc.page_count(), 2);
        assert!(doc.is_well_formed());
    }

    #[test]
    fn test_unbalanced_page() {
        let mut doc = GIRDocument::new();
        let mut page = GIRPage::new();
        page.push(GIRCommand::new_push_stack());
        doc.push_page(page);
        assert!(!doc.is_well_formed());
    }

    #[test]
    fn test_new_page() {
        let mut doc = GIRDocument::new();
        let page = doc.new_page();
        page.push(GIRCommand::new_push_stack());
        page.push(GIRCommand::new_pop_stack());
        assert_eq!(doc.page_count(), 1);
        assert!(doc.is_well_formed());
    }

    #[test]
    fn test_total_commands() {
        let mut doc = GIRDocument::new();
        doc.push_page(make_balanced_page());
        doc.push_page(make_balanced_page());
        assert_eq!(doc.total_commands(), 8);
    }

    #[test]
    fn test_empty_is_well_formed() {
        let doc = GIRDocument::new();
        assert!(doc.is_well_formed());
    }

    #[test]
    fn test_default() {
        let doc = GIRDocument::default();
        assert!(doc.is_empty());
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use crate::gir::command::GIRCommand;
    use crate::gir::opcode::GIROpcode;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn balanced_stack_is_well_formed(ref n in 0usize..20usize) {
            let mut page = GIRPage::new();
            // Push n times, then pop n times — guaranteed balanced
            for _ in 0..*n {
                page.push(GIRCommand::new_push_stack());
            }
            for _ in 0..*n {
                page.push(GIRCommand::new_pop_stack());
            }
            // Mix in non-stack ops (SetFont, MoveXY) which don't affect balance
            for _ in 0..*n {
                page.push(GIRCommand::new_set_font(0));
                page.push(GIRCommand::new_move_xy(100, 200));
            }
            assert!(page.is_stack_balanced());
            assert!(page.len() == 4 * *n);
        }
    }
}
