//! G-IR page type (DEF-002).
//!
//! A `GIRPage` is an ordered sequence of G-IR commands representing one
//! rendered page. Per DEF-005, a well-formed page must have a balanced
//! coordinate stack.
//!
//! Matches Lean 4 `abbrev GIRPage := List GIRCommand` in
//! `ProofIRWellformedness.lean` Section 2.
//!
//! # Stack Balance (DEF-005 cond. 3)
//!
//! The coordinate stack must be balanced:
//! - Every `PushStack` must have a matching `PopStack`.
//! - Stack depth must never go negative.
//!
//! Matches Lean 4 `pageStackBalanced` in Section 4:
//! ```lean
//! def pageStackBalancedGo (cmds : List GIRCommand) (depth : Int) : Bool :=
//!   match cmds with
//!   | [] => depth = 0
//!   | cmd :: rest =>
//!     if depth + stackDelta cmd.opcode < 0 then false
//!     else pageStackBalancedGo rest (depth + stackDelta cmd.opcode)
//! ```

use crate::gir::command::GIRCommand;

/// An ordered sequence of G-IR rendering commands representing one page.
///
/// Per REQ-3.2.1, G-IR compiles into a flat command buffer per page.
///
/// # Well-Formedness (DEF-005)
///
/// Per DEF-005 cond. 3, the coordinate stack must be balanced:
/// `PushStack` and `PopStack` must be properly nested, and stack depth
/// must never go negative.
///
/// # Examples
///
/// ```
/// use ldir_ir::gir::{GIRPage, GIRCommand};
///
/// let mut page = GIRPage::new();
/// page.push(GIRCommand::new_push_stack());
/// page.push(GIRCommand::new_set_font(0));
/// page.push(GIRCommand::new_pop_stack());
///
/// assert!(page.is_stack_balanced());
/// ```
/// A hyperlink annotation on a page.
///
/// Stores the clickable rectangle and target URL for link annotations
/// in the PDF output.
#[derive(Debug, Clone, PartialEq)]
pub struct GIRLink {
    /// X coordinate of the link rectangle's left edge (in points).
    pub x: f64,
    /// Y coordinate of the link rectangle's bottom edge (in points).
    pub y: f64,
    /// Width of the link rectangle (in points).
    pub width: f64,
    /// Height of the link rectangle (in points).
    pub height: f64,
    /// The URL the link points to.
    pub url: String,
    /// If set, this is an internal destination link (page index).
    /// When set, `url` is ignored and the link jumps to this page.
    pub destination_page: Option<usize>,
}

/// An ordered sequence of G-IR rendering commands representing one page.
///
/// Per REQ-3.2.1, G-IR compiles into a flat command buffer per page.
///
/// # Well-Formedness (DEF-005)
///
/// Per DEF-005 cond. 3, the coordinate stack must be balanced:
/// `PushStack` and `PopStack` must be properly nested, and stack depth
/// must never go negative.
#[derive(Debug, Clone, PartialEq)]
pub struct GIRPage {
    /// Ordered sequence of rendering commands.
    commands: Vec<GIRCommand>,

    /// Page width in 26.6 scaled points.
    ///
    /// Per DEF-005 cond. 4: coordinates must be within page bounds.
    pub width: i32,

    /// Page height in 26.6 scaled points.
    ///
    /// Per DEF-005 cond. 4: coordinates must be within page bounds.
    pub height: i32,

    /// Hyperlink annotations on this page.
    pub links: Vec<GIRLink>,
}

impl GIRPage {
    /// Create a new empty page with default dimensions.
    ///
    /// Default dimensions: US Letter (612 x 792 points in 26.6 = 39168 x 50688).
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            width: 612 * 64,
            height: 792 * 64,
            links: Vec::new(),
        }
    }

    /// Create a new empty page with specified dimensions.
    ///
    /// # Arguments
    ///
    /// * `width` - Page width in 26.6 scaled points.
    /// * `height` - Page height in 26.6 scaled points.
    pub fn with_dimensions(width: i32, height: i32) -> Self {
        Self {
            commands: Vec::new(),
            width,
            height,
            links: Vec::new(),
        }
    }

    /// Number of commands in the page.
    #[inline]
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Check if the page has no commands.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Get a command by index.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&GIRCommand> {
        self.commands.get(index)
    }

    /// Get a mutable reference to a command by index.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut GIRCommand> {
        self.commands.get_mut(index)
    }

    /// Push a command onto the page.
    #[inline]
    pub fn push(&mut self, command: GIRCommand) {
        self.commands.push(command);
    }

    /// Push a `PushStack` command onto the page.
    #[inline]
    pub fn push_stack(&mut self) {
        self.commands.push(GIRCommand::new_push_stack());
    }

    /// Push a `PopStack` command onto the page.
    #[inline]
    pub fn pop_stack(&mut self) {
        self.commands.push(GIRCommand::new_pop_stack());
    }

    /// Iterate over commands in order.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &GIRCommand> {
        self.commands.iter()
    }

    /// Iterate over commands mutably.
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut GIRCommand> {
        self.commands.iter_mut()
    }

    /// Check if the coordinate stack is balanced (DEF-005 cond. 3).
    ///
    /// Implements `pageStackBalanced` from `ProofIRWellformedness.lean` Section 4:
    /// - Every `PushStack` (+1) must have a matching `PopStack` (-1).
    /// - Stack depth must never go negative at any point.
    /// - Final stack depth must be zero.
    ///
    /// Returns `true` for empty pages (trivially balanced).
    pub fn is_stack_balanced(&self) -> bool {
        let mut depth: i32 = 0;
        for cmd in &self.commands {
            let delta = cmd.opcode().stack_delta();
            depth += delta;
            if depth < 0 {
                return false;
            }
        }
        depth == 0
    }

    /// Get the current stack depth after processing all commands.
    ///
    /// Returns 0 for a balanced page. Useful for diagnostics.
    pub fn stack_depth(&self) -> i32 {
        self.commands
            .iter()
            .map(|cmd| cmd.opcode().stack_delta())
            .sum()
    }

    /// Get the raw command slice.
    #[inline]
    pub fn as_slice(&self) -> &[GIRCommand] {
        &self.commands
    }

    /// Clear all commands.
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// Reserve capacity for additional commands.
    pub fn reserve(&mut self, additional: usize) {
        self.commands.reserve(additional);
    }
}

impl Default for GIRPage {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Deref for GIRPage {
    type Target = [GIRCommand];

    fn deref(&self) -> &Self::Target {
        &self.commands
    }
}

impl std::ops::DerefMut for GIRPage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gir::opcode::GIROpcode;

    #[test]
    fn test_new_empty() {
        let page = GIRPage::new();
        assert!(page.is_empty());
        assert!(page.is_stack_balanced());
        assert_eq!(page.stack_depth(), 0);
    }

    #[test]
    fn test_push_pop_balanced() {
        let mut page = GIRPage::new();
        page.push(GIRCommand::new_push_stack());
        page.push(GIRCommand::new_set_font(0));
        page.push(GIRCommand::new_pop_stack());
        assert_eq!(page.len(), 3);
        assert!(page.is_stack_balanced());
    }

    #[test]
    fn test_unbalanced_pop_first() {
        let mut page = GIRPage::new();
        page.push(GIRCommand::new_pop_stack());
        assert!(!page.is_stack_balanced());
    }

    #[test]
    fn test_unbalanced_extra_push() {
        let mut page = GIRPage::new();
        page.push(GIRCommand::new_push_stack());
        assert!(!page.is_stack_balanced());
        assert_eq!(page.stack_depth(), 1);
    }

    #[test]
    fn test_nested_push_pop() {
        let mut page = GIRPage::new();
        page.push(GIRCommand::new_push_stack());
        page.push(GIRCommand::new_push_stack());
        page.push(GIRCommand::new_move_xy(100, 200));
        page.push(GIRCommand::new_pop_stack());
        page.push(GIRCommand::new_pop_stack());
        assert!(page.is_stack_balanced());
    }

    #[test]
    fn test_push_pop_helpers() {
        let mut page = GIRPage::new();
        page.push_stack();
        page.push_stack();
        page.pop_stack();
        page.pop_stack();
        assert!(page.is_stack_balanced());
    }

    #[test]
    fn test_with_dimensions() {
        let page = GIRPage::with_dimensions(500 * 64, 800 * 64);
        assert_eq!(page.width, 500 * 64);
        assert_eq!(page.height, 800 * 64);
    }

    #[test]
    fn test_default_dimensions() {
        let page = GIRPage::default();
        assert_eq!(page.width, 612 * 64);
        assert_eq!(page.height, 792 * 64);
    }

    #[test]
    fn test_iter() {
        let mut page = GIRPage::new();
        page.push(GIRCommand::new_set_font(1));
        page.push(GIRCommand::new_set_font(2));
        let opcodes: Vec<_> = page.iter().map(|c| c.opcode()).collect();
        assert_eq!(opcodes, vec![GIROpcode::SetFont, GIROpcode::SetFont]);
    }

    #[test]
    fn test_stack_depth() {
        let mut page = GIRPage::new();
        page.push_stack();
        page.push_stack();
        page.pop_stack();
        assert_eq!(page.stack_depth(), 1);
    }
}
