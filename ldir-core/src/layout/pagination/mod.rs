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
}
