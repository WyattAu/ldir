//! Global pagination algorithm (TASK-018).
//!
//! Greedy page-breaking that accumulates paragraph blocks until page
//! height is exceeded, then applies widow/orphan avoidance rules.
//!
//! ## Algorithm
//!
//! 1. Accumulate paragraphs on the current page by total height
//! 2. When a paragraph doesn't fit, check for orphan (first line alone
//!    at bottom of page) and widow (last line alone at top of new page)
//! 3. If detected, move the paragraph to the next page entirely
//! 4. Track demerits for suboptimal breaks
//!
//! ## References
//!
//! - REQ-3.2.5: Fixed-point coordinate system
//! - YP-PAGINATION-001: Pagination specification

#![allow(dead_code)]

pub mod page_break;

use crate::fp266::Fp266;
pub use page_break::{LineBlock, ParagraphBlock};

/// Configuration for the pagination algorithm.
#[derive(Clone, Debug, PartialEq)]
pub struct PaginationOptions {
    /// Total page height including margins.
    pub page_height: Fp266,
    /// Total page width including margins.
    pub page_width: Fp266,
    /// Top margin (subtract from page_height to get usable area).
    pub margin_top: Fp266,
    /// Bottom margin (subtract from page_height to get usable area).
    pub margin_bottom: Fp266,
    /// Demerit penalty for a widow (last paragraph line alone on new page).
    pub widow_penalty: f64,
    /// Demerit penalty for an orphan (first paragraph line alone at page bottom).
    pub orphan_penalty: f64,
}

impl PaginationOptions {
    /// Create options with the given dimensions and default penalties.
    pub fn new(
        page_height: Fp266,
        page_width: Fp266,
        margin_top: Fp266,
        margin_bottom: Fp266,
    ) -> Self {
        Self {
            page_height,
            page_width,
            margin_top,
            margin_bottom,
            widow_penalty: 50.0,
            orphan_penalty: 50.0,
        }
    }

    /// Usable vertical space on a page (page height minus margins).
    pub fn usable_height(&self) -> Fp266 {
        self.page_height - self.margin_top - self.margin_bottom
    }

    /// Demerit for empty space on a page (tightness penalty).
    /// Returns 100 * (remaining / usable) — less empty space = better.
    pub fn tightness_demerits(&self, used: Fp266) -> f64 {
        let usable = self.usable_height();
        if usable <= Fp266::ZERO {
            return 0.0;
        }
        let remaining = (usable - used).to_f64();
        let usable_f = usable.to_f64();
        100.0 * (remaining / usable_f).abs()
    }
}

/// A page break record: the range of paragraph indices assigned to a page.
#[derive(Clone, Debug, PartialEq)]
pub struct PageBreak {
    /// Start index (inclusive) into the paragraph block list.
    pub start_index: usize,
    /// End index (inclusive) into the paragraph block list.
    pub end_index: usize,
    /// Demerits accumulated for suboptimal breaks on this page.
    pub demerits: f64,
}

/// Result of paginating a document: ordered page breaks with total demerits.
#[derive(Clone, Debug, PartialEq)]
pub struct PaginatedDocument {
    /// Ordered page breaks. Each covers paragraphs[start_index..=end_index].
    pub pages: Vec<PageBreak>,
    /// Sum of all demerits across pages.
    pub total_demerits: f64,
}

impl PaginatedDocument {
    /// An empty paginated document (no pages).
    pub fn empty() -> Self {
        Self {
            pages: Vec::new(),
            total_demerits: 0.0,
        }
    }

    /// Number of pages.
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// Check if the document has no pages.
    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }
}

/// Count how many lines of a paragraph fit within the given remaining height.
fn count_fitting_lines(lines: &[LineBlock], remaining: Fp266) -> usize {
    let mut used = Fp266::ZERO;
    let mut count = 0;
    for line in lines {
        if used + line.height <= remaining {
            used += line.height;
            count += 1;
        } else {
            break;
        }
    }
    count
}

/// Paginate a sequence of paragraph blocks using a greedy algorithm.
///
/// # Algorithm
///
/// 1. Walk paragraphs in order, accumulating height on the current page
/// 2. When a paragraph doesn't fit:
///    a. If it's the first item on the page, force it (overflow)
///    b. Otherwise, check orphan (only first line fits at page bottom)
///    c. Check widow (only last line would remain for next page)
///    d. If orphan or widow, push paragraph to next page with demerits
///    e. Finalize current page, start new page
/// 3. Finalize last page after all paragraphs are placed
///
/// # Panics
///
/// Does not panic. Returns empty `PaginatedDocument` for empty input
/// or non-positive usable height.
pub fn paginate(items: &[ParagraphBlock], options: &PaginationOptions) -> PaginatedDocument {
    if items.is_empty() {
        return PaginatedDocument::empty();
    }

    let usable = options.usable_height();
    if usable <= Fp266::ZERO {
        return PaginatedDocument::empty();
    }

    let mut pages: Vec<PageBreak> = Vec::new();
    let mut total_demerits = 0.0;

    let mut page_start: usize = 0;
    let mut current_height = Fp266::ZERO;
    let mut i = 0;

    while i < items.len() {
        let para = &items[i];
        let new_height = current_height + para.height;

        if new_height <= usable {
            current_height = new_height;
            i += 1;
            continue;
        }

        let has_content = i > page_start;

        if !has_content {
            if !para.lines.is_empty() {
                pages.push(PageBreak {
                    start_index: i,
                    end_index: i,
                    demerits: 0.0,
                });
            }
            i += 1;
            page_start = i;
            current_height = Fp266::ZERO;
            continue;
        }

        let space_remaining = usable - current_height;
        let lines_that_fit = count_fitting_lines(&para.lines, space_remaining);
        let remaining = para.lines.len().saturating_sub(lines_that_fit);

        let is_orphan = lines_that_fit == 1 && para.lines.len() > 1;
        let is_widow = lines_that_fit > 0 && remaining == 1 && para.lines.len() > 1;

        if is_orphan {
            total_demerits += options.orphan_penalty;
        } else if is_widow {
            total_demerits += options.widow_penalty;
        }

        let page_end = i - 1;
        pages.push(PageBreak {
            start_index: page_start,
            end_index: page_end,
            demerits: 0.0,
        });

        page_start = i;
        current_height = Fp266::ZERO;
    }

    if page_start < items.len() {
        pages.push(PageBreak {
            start_index: page_start,
            end_index: items.len() - 1,
            demerits: 0.0,
        });
    }

    PaginatedDocument {
        pages,
        total_demerits,
    }
}

/// Global pagination via dynamic programming (branch-and-bound).
///
/// Finds the optimal page breaks that minimize total demerits across the
/// entire document. Demerits include:
/// - Tightness penalty: 100 * (empty_space / usable_height)
/// - Widow/orphan penalties for single-line paragraphs at page boundaries
/// - Page break penalty (base cost per page)
///
/// # Algorithm
///
/// 1. Precompute prefix sums of paragraph heights for O(1) range queries.
/// 2. For each candidate page (i..=j), compute feasibility and demerits.
/// 3. DP: dp[j+1] = min_{i ≤ j, para[i..=j] fit} dp[i] + cost(i, j)
/// 4. Trace back to recover break positions.
///
/// Time: O(n²) where n is paragraph count. Memory: O(n).
pub fn paginate_global(items: &[ParagraphBlock], options: &PaginationOptions) -> PaginatedDocument {
    if items.is_empty() {
        return PaginatedDocument::empty();
    }

    let usable = options.usable_height();
    if usable <= Fp266::ZERO {
        return PaginatedDocument::empty();
    }

    let n = items.len();

    // Prefix sums of paragraph heights (prefix[i] = sum of heights[0..i))
    let mut prefix = vec![Fp266::ZERO; n + 1];
    for i in 0..n {
        prefix[i + 1] = prefix[i] + items[i].height;
    }

    // DP: dp[i] = minimum demerits to cover paragraphs[0..i)
    let mut dp: Vec<f64> = vec![f64::INFINITY; n + 1];
    let mut prev: Vec<usize> = vec![0; n + 1];
    dp[0] = 0.0;

    // Page break base cost
    let break_cost: f64 = 1.0;

    for j in 1..=n {
        // Branch-and-bound: only consider i where paragraphs[i..j) fit
        for i in 0..j {
            let segment_height = prefix[j] - prefix[i];
            if segment_height > usable {
                // Overflows — infeasible
                continue;
            }

            // Compute demerits for this page
            let tightness = options.tightness_demerits(segment_height);

            // Widow/orphan check: paragraphs[i] spans a page boundary
            // (simplified: check first/last paragraph line counts)
            let mut widow_pen = 0.0;
            let mut orphan_pen = 0.0;
            if j > i + 1 {
                // More than one paragraph on page — check last paragraph on previous page
                // (this is a simplification; full widow/orphan needs line-level analysis)
                if items[j - 1].lines.len() == 1 && j > i + 1 {
                    orphan_pen = options.orphan_penalty;
                }
                if i + 1 < j && items[i].lines.len() == 1 && i > 0 {
                    widow_pen = options.widow_penalty;
                }
            }

            let cost = tightness + widow_pen + orphan_pen + break_cost;

            if dp[i] + cost < dp[j] {
                dp[j] = dp[i] + cost;
                prev[j] = i;
            }
        }
    }

    // Trace back
    let total_demerits = dp[n];
    if total_demerits.is_infinite() {
        // Fallback: one page per paragraph (shouldn't happen with usable > 0)
        return paginate(items, options);
    }

    let mut breaks: Vec<usize> = Vec::new();
    let mut cur = n;
    while cur > 0 {
        breaks.push(cur - 1);
        cur = prev[cur];
    }
    breaks.reverse();

    let mut pages = Vec::new();
    let mut start = 0;
    for &end in &breaks {
        let mut demerits = 0.0;
        let seg_h = prefix[end + 1] - prefix[start];
        demerits += options.tightness_demerits(seg_h);
        demerits += break_cost;
        if end > start && items[end].lines.len() == 1 {
            demerits += options.orphan_penalty;
        }
        pages.push(PageBreak {
            start_index: start,
            end_index: end,
            demerits,
        });
        start = end + 1;
    }

    PaginatedDocument {
        pages,
        total_demerits,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_options(page_height: i32) -> PaginationOptions {
        PaginationOptions::new(
            Fp266::from_int(page_height),
            Fp266::from_int(612),
            Fp266::from_int(72),
            Fp266::from_int(72),
        )
    }

    fn make_paragraph(line_count: usize, line_height: i32) -> ParagraphBlock {
        let lines: Vec<LineBlock> = (0..line_count)
            .map(|_| {
                LineBlock::new(
                    Fp266::from_int(line_height),
                    Fp266::from_int(line_height - 2),
                )
            })
            .collect();
        ParagraphBlock::new(lines)
    }

    #[test]
    fn single_page_fits_all() {
        let items = vec![make_paragraph(3, 12), make_paragraph(2, 12)];
        let opts = default_options(792);
        let result = paginate(&items, &opts);
        assert_eq!(result.page_count(), 1);
        assert_eq!(result.pages[0].start_index, 0);
        assert_eq!(result.pages[0].end_index, 1);
        assert!(result.total_demerits < f64::EPSILON);
    }

    #[test]
    fn multi_page_break() {
        let opts = default_options(792);
        let usable = opts.usable_height().to_int();
        let lines_per_page = usable / 12;

        let items = vec![
            make_paragraph(lines_per_page as usize, 12),
            make_paragraph(3, 12),
        ];
        let result = paginate(&items, &opts);
        assert_eq!(result.page_count(), 2);
        assert_eq!(result.pages[0].start_index, 0);
        assert_eq!(result.pages[0].end_index, 0);
        assert_eq!(result.pages[1].start_index, 1);
        assert_eq!(result.pages[1].end_index, 1);
    }

    #[test]
    fn widow_avoidance() {
        let mut opts = default_options(792);
        opts.widow_penalty = 100.0;
        opts.orphan_penalty = 100.0;

        let usable = opts.usable_height().to_int();
        let full_para_lines = (usable / 12 - 1) as usize;

        let items = vec![make_paragraph(full_para_lines, 12), make_paragraph(2, 12)];
        let result = paginate(&items, &opts);
        assert_eq!(result.page_count(), 2);
        assert!(result.total_demerits > 0.0);
    }

    #[test]
    fn orphan_avoidance() {
        let mut opts = default_options(792);
        opts.widow_penalty = 100.0;
        opts.orphan_penalty = 100.0;

        let usable = opts.usable_height().to_int();
        let almost_full_lines = (usable / 12 - 1) as usize;

        let items = vec![make_paragraph(almost_full_lines, 12), make_paragraph(3, 12)];
        let result = paginate(&items, &opts);
        assert_eq!(result.page_count(), 2);
        assert!(result.total_demerits > 0.0);
    }

    #[test]
    fn empty_input() {
        let items: Vec<ParagraphBlock> = vec![];
        let opts = default_options(792);
        let result = paginate(&items, &opts);
        assert!(result.is_empty());
        assert_eq!(result.page_count(), 0);
        assert!(result.total_demerits < f64::EPSILON);
    }

    #[test]
    fn single_oversized_paragraph() {
        let items = vec![make_paragraph(1000, 12)];
        let opts = default_options(792);
        let result = paginate(&items, &opts);
        assert_eq!(result.page_count(), 1);
        assert_eq!(result.pages[0].start_index, 0);
        assert_eq!(result.pages[0].end_index, 0);
    }

    #[test]
    fn determinism_same_input_same_output() {
        let items = vec![
            make_paragraph(5, 12),
            make_paragraph(3, 14),
            make_paragraph(7, 10),
            make_paragraph(2, 12),
        ];
        let opts = default_options(792);
        let r1 = paginate(&items, &opts);
        let r2 = paginate(&items, &opts);
        assert_eq!(r1, r2);
    }

    #[test]
    fn zero_usable_height() {
        let items = vec![make_paragraph(1, 12)];
        let opts = PaginationOptions::new(
            Fp266::from_int(72),
            Fp266::from_int(612),
            Fp266::from_int(36),
            Fp266::from_int(36),
        );
        let result = paginate(&items, &opts);
        assert!(result.is_empty());
    }

    #[test]
    fn empty_paragraph_skipped() {
        let items = vec![ParagraphBlock::new(vec![]), make_paragraph(2, 12)];
        let opts = default_options(792);
        let result = paginate(&items, &opts);
        assert_eq!(result.page_count(), 1);
        assert_eq!(result.pages[0].start_index, 0);
        assert_eq!(result.pages[0].end_index, 1);
    }

    #[test]
    fn pagination_options_usable_height() {
        let opts = PaginationOptions::new(
            Fp266::from_int(792),
            Fp266::from_int(612),
            Fp266::from_int(72),
            Fp266::from_int(72),
        );
        assert_eq!(opts.usable_height(), Fp266::from_int(648));
    }

    #[test]
    fn global_paginate_single_page() {
        let items = vec![make_paragraph(3, 12), make_paragraph(2, 12)];
        let opts = default_options(792);
        let result = paginate_global(&items, &opts);
        assert_eq!(result.page_count(), 1);
        assert_eq!(result.pages[0].start_index, 0);
        assert_eq!(result.pages[0].end_index, 1);
    }

    #[test]
    fn global_paginate_multi_page() {
        let items = vec![
            make_paragraph(30, 12), // 360pt
            make_paragraph(30, 12), // 360pt → overflow
            make_paragraph(20, 12), // 240pt
        ];
        let opts = default_options(792);
        let result = paginate_global(&items, &opts);
        assert!(result.page_count() >= 2);
        // Total demerits should be finite
        assert!(result.total_demerits.is_finite());
    }

    #[test]
    fn global_paginate_empty() {
        let items: Vec<ParagraphBlock> = vec![];
        let opts = default_options(792);
        let result = paginate_global(&items, &opts);
        assert!(result.is_empty());
    }

    #[test]
    fn global_paginate_deterministic() {
        let items = vec![make_paragraph(5, 12), make_paragraph(5, 12), make_paragraph(5, 12)];
        let opts = default_options(300);
        let r1 = paginate_global(&items, &opts);
        let r2 = paginate_global(&items, &opts);
        assert_eq!(r1.page_count(), r2.page_count());
        assert_eq!(r1.total_demerits, r2.total_demerits);
    }
}
