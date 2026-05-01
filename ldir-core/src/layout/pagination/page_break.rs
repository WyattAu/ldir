//! Paragraph and line block types for pagination (TASK-018).
//!
//! These types represent the output of line-breaking (Knuth-Plass or
//! simplified) and serve as input to the pagination algorithm.
//! All dimensions use [`Fp266`] (26.6 fixed-point) per REQ-3.2.5.

use crate::fp266::Fp266;

/// A single typeset line ready for pagination.
///
/// Carries vertical metrics needed for page-height accumulation
/// and widow/orphan detection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineBlock {
    /// Total vertical extent of this line (ascent + descent + leading).
    pub height: Fp266,
    /// Distance from the top of the line to the baseline.
    pub baseline: Fp266,
    /// How much this line can shrink vertically to help fill a page.
    pub shrinkability: Fp266,
}

impl LineBlock {
    /// Create a line block with the given height and baseline.
    pub fn new(height: Fp266, baseline: Fp266) -> Self {
        Self {
            height,
            baseline,
            shrinkability: Fp266::ZERO,
        }
    }

    /// Create a line block with explicit shrinkability.
    pub fn with_shrinkability(height: Fp266, baseline: Fp266, shrinkability: Fp266) -> Self {
        Self {
            height,
            baseline,
            shrinkability,
        }
    }
}

/// A paragraph composed of one or more lines, ready for pagination.
///
/// The `height` field is the sum of all line heights and must equal
/// `lines.iter().map(|l| l.height).sum()`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParagraphBlock {
    /// Individual lines produced by line-breaking.
    pub lines: Vec<LineBlock>,
    /// Total height: sum of all line heights.
    pub height: Fp266,
}

impl ParagraphBlock {
    /// Create a paragraph block, computing height from lines.
    pub fn new(lines: Vec<LineBlock>) -> Self {
        let height = lines.iter().fold(Fp266::ZERO, |acc, l| acc + l.height);
        Self { lines, height }
    }

    /// Create a single-line paragraph block.
    pub fn single_line(height: Fp266, baseline: Fp266) -> Self {
        Self {
            lines: vec![LineBlock::new(height, baseline)],
            height,
        }
    }

    /// Number of lines in this paragraph.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_block_new() {
        let lb = LineBlock::new(Fp266::from_int(12), Fp266::from_int(10));
        assert_eq!(lb.height, Fp266::from_int(12));
        assert_eq!(lb.baseline, Fp266::from_int(10));
        assert!(lb.shrinkability.is_zero());
    }

    #[test]
    fn line_block_with_shrinkability() {
        let lb = LineBlock::with_shrinkability(
            Fp266::from_int(12),
            Fp266::from_int(10),
            Fp266::from_int(2),
        );
        assert_eq!(lb.shrinkability, Fp266::from_int(2));
    }

    #[test]
    fn paragraph_block_single_line() {
        let pb = ParagraphBlock::single_line(Fp266::from_int(14), Fp266::from_int(12));
        assert_eq!(pb.line_count(), 1);
        assert_eq!(pb.height, Fp266::from_int(14));
    }

    #[test]
    fn paragraph_block_new_computes_height() {
        let lines = vec![
            LineBlock::new(Fp266::from_int(12), Fp266::from_int(10)),
            LineBlock::new(Fp266::from_int(12), Fp266::from_int(10)),
            LineBlock::new(Fp266::from_int(12), Fp266::from_int(10)),
        ];
        let pb = ParagraphBlock::new(lines);
        assert_eq!(pb.line_count(), 3);
        assert_eq!(pb.height, Fp266::from_int(36));
    }

    #[test]
    fn paragraph_block_empty_lines() {
        let pb = ParagraphBlock::new(vec![]);
        assert_eq!(pb.line_count(), 0);
        assert!(pb.height.is_zero());
    }
}
