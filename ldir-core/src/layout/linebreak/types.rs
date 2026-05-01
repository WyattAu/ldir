//! Line-breaking types for Knuth-Plass algorithm.

use crate::fp266::Fp266;

/// A single item to be placed by the line-breaking algorithm.
#[derive(Debug, Clone, Copy)]
pub struct LineBreakItem {
    /// Width of this item in 26.6 fixed-point.
    pub width: Fp266,
    /// Stretchability (how much this item can expand).
    pub stretchability: Fp266,
    /// Shrinkability (how much this item can contract).
    pub shrinkability: Fp266,
    /// Penalty for breaking after this item (negative = mandatory break).
    pub penalty: f64,
    /// Whether this is a forced (mandatory) break.
    pub is_mandatory: bool,
}

/// Options controlling the line-breaking algorithm.
#[derive(Debug, Clone, Copy)]
pub struct LineBreakOptions {
    /// Target line width in 26.6 fixed-point.
    pub line_width: Fp266,
    /// Maximum adjustment ratio (1.0 = allow up to 100% stretch/shrink).
    pub max_adjustment_ratio: f64,
    /// Base penalty for each line break.
    pub line_penalty: f64,
    /// Penalty delta for each fitness class change.
    pub fitness_penalty: f64,
}

impl Default for LineBreakOptions {
    fn default() -> Self {
        Self {
            line_width: Fp266::from_int(500), // ~500pt default
            max_adjustment_ratio: 1.0,
            line_penalty: 10.0,
            fitness_penalty: 100.0,
        }
    }
}

/// Result of the line-breaking algorithm.
#[derive(Debug, Clone)]
pub struct LineBreakResult {
    /// Indices into the items array where line breaks occur.
    pub breaks: Vec<usize>,
    /// Total demerits of the optimal solution.
    pub total_demerits: f64,
}

/// Fitness class of a line (Knuth-Plass classification).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FitnessClass {
    /// Count of tight lines.
    pub tight: i32,
    /// Count of loose lines.
    pub loose: i32,
    /// Count of very loose lines.
    pub very_loose: i32,
}

impl FitnessClass {
    /// Creates a new zero-initialized fitness class.
    pub fn new() -> Self {
        Self::default()
    }

    /// Compute fitness class from adjustment ratio.
    /// - Tight: -0.5 < r < 0
    /// - Normal: r == 0
    /// - Loose: 0 < r < 0.5
    /// - Very loose: r >= 0.5
    pub fn from_ratio(r: f64) -> Self {
        Self {
            tight: if -0.5 < r && r < 0.0 { 1 } else { 0 },
            loose: if 0.0 < r && r < 0.5 { 1 } else { 0 },
            very_loose: if r >= 0.5 { 1 } else { 0 },
        }
    }

    /// Compute fitness class difference (demerit penalty).
    pub fn demerit_delta(&self, other: &Self) -> f64 {
        let dt = (self.tight - other.tight).unsigned_abs()
            + (self.loose - other.loose).unsigned_abs()
            + (self.very_loose - other.very_loose).unsigned_abs();
        if dt > 0 { 1.0 } else { 0.0 }
    }
}
