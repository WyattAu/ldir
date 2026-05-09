//! Table of contents and document outline generation.

/// A single entry in a table of contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TocEntry {
    /// Heading level (1-6).
    pub level: u8,
    /// Display text of the heading.
    pub title: String,
    /// Section number if numbered (e.g., "2.3.1").
    pub number: Option<String>,
    /// Page number (1-indexed, 0 if unknown).
    pub page: u32,
    /// Nesting depth in the TOC (0-indexed).
    pub depth: usize,
    /// Unique anchor/label for cross-references.
    pub anchor: Option<String>,
}

/// A complete table of contents for a document.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TableOfContents {
    entries: Vec<TocEntry>,
}

impl TableOfContents {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an entry to the TOC.
    pub fn push(&mut self, entry: TocEntry) {
        self.entries.push(entry);
    }

    /// Get all entries.
    pub fn entries(&self) -> &[TocEntry] {
        &self.entries
    }

    /// Return entries up to a given depth (inclusive).
    pub fn up_to_depth(&self, max_depth: u8) -> Vec<&TocEntry> {
        self.entries
            .iter()
            .filter(|e| e.level <= max_depth)
            .collect()
    }

    /// Return entries at a specific level.
    pub fn at_level(&self, level: u8) -> Vec<&TocEntry> {
        self.entries.iter().filter(|e| e.level == level).collect()
    }

    /// Compute nesting depth for each entry based on heading levels.
    pub fn compute_depths(&mut self) {
        let mut stack: Vec<u8> = Vec::new();
        for entry in &mut self.entries {
            // Pop levels >= current
            while stack.last().is_some_and(|&l| l >= entry.level) {
                stack.pop();
            }
            entry.depth = stack.len();
            stack.push(entry.level);
        }
    }

    /// Return count of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Generate section numbers for a flat list of heading levels.
/// Returns a Vec of (level, number_string) pairs.
///
/// # Examples
/// ```
/// use ldir_core::toc::generate_section_numbers;
/// let levels = vec![1u8, 2, 2, 1, 3, 2];
/// let nums = generate_section_numbers(&levels);
/// assert_eq!(nums[0], (1, "1".to_string()));
/// assert_eq!(nums[1], (2, "1.1".to_string()));
/// assert_eq!(nums[2], (2, "1.2".to_string()));
/// assert_eq!(nums[3], (1, "2".to_string()));
/// assert_eq!(nums[4], (3, "2.1.1".to_string()));
/// assert_eq!(nums[5], (2, "2.2".to_string()));
/// ```
pub fn generate_section_numbers(levels: &[u8]) -> Vec<(u8, String)> {
    let mut counters: Vec<usize> = Vec::new();
    let mut result = Vec::with_capacity(levels.len());
    for &level in levels {
        let lvl = level as usize;
        if lvl == 0 {
            continue;
        }
        counters.truncate(lvl);
        let was_present = counters.len() >= lvl;
        while counters.len() < lvl {
            counters.push(1);
        }
        if was_present && let Some(c) = counters.last_mut() {
            *c += 1;
        }
        let num_str = counters
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(".");
        result.push((level, num_str));
    }
    result
}

/// Compute nesting depth for a sequence of heading levels.
pub fn compute_depths(levels: &[u8]) -> Vec<usize> {
    let mut stack: Vec<u8> = Vec::new();
    levels
        .iter()
        .map(|&level| {
            while stack.last().is_some_and(|&l| l >= level) {
                stack.pop();
            }
            let depth = stack.len();
            stack.push(level);
            depth
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toc_push_and_len() {
        let mut toc = TableOfContents::new();
        assert!(toc.is_empty());
        toc.push(TocEntry {
            level: 1,
            title: "Intro".into(),
            number: Some("1".into()),
            page: 1,
            depth: 0,
            anchor: Some("intro".into()),
        });
        toc.push(TocEntry {
            level: 2,
            title: "Background".into(),
            number: Some("1.1".into()),
            page: 2,
            depth: 0,
            anchor: None,
        });
        assert_eq!(toc.len(), 2);
    }

    #[test]
    fn test_toc_up_to_depth() {
        let mut toc = TableOfContents::new();
        toc.push(TocEntry {
            level: 1,
            title: "A".into(),
            number: None,
            page: 1,
            depth: 0,
            anchor: None,
        });
        toc.push(TocEntry {
            level: 2,
            title: "B".into(),
            number: None,
            page: 2,
            depth: 0,
            anchor: None,
        });
        toc.push(TocEntry {
            level: 3,
            title: "C".into(),
            number: None,
            page: 3,
            depth: 0,
            anchor: None,
        });
        assert_eq!(toc.up_to_depth(2).len(), 2);
        assert_eq!(toc.up_to_depth(1).len(), 1);
        assert_eq!(toc.up_to_depth(6).len(), 3);
    }

    #[test]
    fn test_toc_at_level() {
        let mut toc = TableOfContents::new();
        toc.push(TocEntry {
            level: 1,
            title: "A".into(),
            number: None,
            page: 1,
            depth: 0,
            anchor: None,
        });
        toc.push(TocEntry {
            level: 2,
            title: "B".into(),
            number: None,
            page: 2,
            depth: 0,
            anchor: None,
        });
        toc.push(TocEntry {
            level: 1,
            title: "C".into(),
            number: None,
            page: 3,
            depth: 0,
            anchor: None,
        });
        assert_eq!(toc.at_level(1).len(), 2);
        assert_eq!(toc.at_level(2).len(), 1);
        assert_eq!(toc.at_level(3).len(), 0);
    }

    #[test]
    fn test_compute_depths_simple() {
        let depths = compute_depths(&[1, 2, 2, 3, 2, 1, 2]);
        assert_eq!(depths, vec![0, 1, 1, 2, 1, 0, 1]);
    }

    #[test]
    fn test_compute_depths_single_level() {
        let depths = compute_depths(&[1, 1, 1]);
        assert_eq!(depths, vec![0, 0, 0]);
    }

    #[test]
    fn test_compute_depths_empty() {
        let depths = compute_depths(&[]);
        assert!(depths.is_empty());
    }

    #[test]
    fn test_compute_depths_skipping_levels() {
        let depths = compute_depths(&[1, 3, 2, 4]);
        assert_eq!(depths, vec![0, 1, 1, 2]);
    }

    #[test]
    fn test_toc_compute_depths() {
        let mut toc = TableOfContents::new();
        toc.push(TocEntry {
            level: 1,
            title: "A".into(),
            number: None,
            page: 1,
            depth: 0,
            anchor: None,
        });
        toc.push(TocEntry {
            level: 2,
            title: "B".into(),
            number: None,
            page: 2,
            depth: 0,
            anchor: None,
        });
        toc.push(TocEntry {
            level: 2,
            title: "C".into(),
            number: None,
            page: 3,
            depth: 0,
            anchor: None,
        });
        toc.push(TocEntry {
            level: 1,
            title: "D".into(),
            number: None,
            page: 4,
            depth: 0,
            anchor: None,
        });
        toc.compute_depths();
        let depths: Vec<usize> = toc.entries().iter().map(|e| e.depth).collect();
        assert_eq!(depths, vec![0, 1, 1, 0]);
    }

    #[test]
    fn test_section_numbers_simple() {
        let levels = vec![1, 2, 2, 1, 3, 2];
        let nums = generate_section_numbers(&levels);
        assert_eq!(nums[0], (1, "1".to_string()));
        assert_eq!(nums[1], (2, "1.1".to_string()));
        assert_eq!(nums[2], (2, "1.2".to_string()));
        assert_eq!(nums[3], (1, "2".to_string()));
        assert_eq!(nums[4], (3, "2.1.1".to_string()));
        assert_eq!(nums[5], (2, "2.2".to_string()));
    }

    #[test]
    fn test_section_numbers_deep() {
        let levels = vec![1, 2, 3, 4, 4, 3, 2, 1];
        let nums = generate_section_numbers(&levels);
        assert_eq!(nums[0].1, "1");
        assert_eq!(nums[1].1, "1.1");
        assert_eq!(nums[2].1, "1.1.1");
        assert_eq!(nums[3].1, "1.1.1.1");
        assert_eq!(nums[4].1, "1.1.1.2");
        assert_eq!(nums[5].1, "1.1.2");
        assert_eq!(nums[6].1, "1.2");
        assert_eq!(nums[7].1, "2");
    }

    #[test]
    fn test_section_numbers_empty() {
        let nums = generate_section_numbers(&[]);
        assert!(nums.is_empty());
    }

    #[test]
    fn test_section_numbers_single() {
        let nums = generate_section_numbers(&[1]);
        assert_eq!(nums[0], (1, "1".to_string()));
    }

    #[test]
    fn test_toc_entries_accessor() {
        let mut toc = TableOfContents::new();
        toc.push(TocEntry {
            level: 1,
            title: "X".into(),
            number: None,
            page: 5,
            depth: 0,
            anchor: Some("x".into()),
        });
        assert_eq!(toc.entries()[0].title, "X");
        assert_eq!(toc.entries()[0].page, 5);
    }
}
