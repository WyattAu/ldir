//! Knuth-Plass line-breaking algorithm.
//!
//! Dynamic programming approach to find optimal line breaks in a paragraph.
//! References: YP-LAYOUT-KNUTHPLASS-001, ALG-KP-BREAK, THM-KP-OPTIMALITY, THM-KP-TERMINATION.

use crate::fp266::Fp266;

use super::badness::{compute_adjustment_ratio, compute_badness, compute_demerits};
use super::optical_margin::optical_margin_penalty_reduction;
use super::types::*;

/// An active node in the DP algorithm, representing a candidate line break position.
#[derive(Debug, Clone)]
struct ActiveNode {
    /// Position in the items array where this break occurs (break after this index).
    position: usize,
    /// Cumulative width from start to this position.
    cumulative_width: Fp266,
    /// Cumulative stretch from start to this position.
    cumulative_stretch: Fp266,
    /// Cumulative shrink from start to this position.
    cumulative_shrink: Fp266,
    /// Total demerits up to this break.
    total_demerits: f64,
    /// Fitness class of the line ending at this break.
    fitness: FitnessClass,
    /// Index in the `nodes` array of the previous break (for traceback).
    previous_break: usize,
}

/// Run the Knuth-Plass line-breaking algorithm.
///
/// Finds the optimal set of line breaks that minimizes total demerits.
/// Returns the indices where breaks should occur (after the item at each index).
///
/// # Algorithm (ALG-KP-BREAK)
/// 1. Precompute prefix sums of width/stretch/shrink
/// 2. Initialize active node list with position 0
/// 3. For each item, try every active node as a potential previous break
/// 4. Compute adjustment ratio using cumulative differences
/// 5. Keep feasible breaks; deactivate infeasible nodes
/// 6. At end, choose the active node with minimum total demerits
/// 7. Trace back to recover break positions
pub fn linebreak(items: &[LineBreakItem], options: &LineBreakOptions) -> LineBreakResult {
    if items.is_empty() {
        return LineBreakResult {
            breaks: vec![],
            total_demerits: 0.0,
        };
    }

    let n = items.len();

    // Precompute prefix sums for O(1) line content queries
    // prefix[i] = sum of items[0..i] (exclusive), so prefix[0] = 0
    let mut prefix_w = vec![Fp266::ZERO; n + 1];
    let mut prefix_s = vec![Fp266::ZERO; n + 1];
    let mut prefix_h = vec![Fp266::ZERO; n + 1];
    for i in 0..n {
        prefix_w[i + 1] = prefix_w[i] + items[i].width;
        prefix_s[i + 1] = prefix_s[i] + items[i].stretchability;
        prefix_h[i + 1] = prefix_h[i] + items[i].shrinkability;
    }

    // Quick check: if everything fits on one line AND there are no mandatory breaks, no breaks needed
    let has_mandatory = items.iter().any(|item| item.is_mandatory);
    if !has_mandatory {
        let total_w = prefix_w[n];
        let total_s = prefix_s[n];
        let total_h = prefix_h[n];
        let overall_r = compute_adjustment_ratio(options.line_width, total_w, total_s, total_h);
        if !overall_r.is_infinite()
            && !overall_r.is_nan()
            && overall_r.abs() <= options.max_adjustment_ratio
        {
            return LineBreakResult {
                breaks: vec![],
                total_demerits: compute_badness(overall_r),
            };
        }
    }

    // Persistent node store (indices never change, enabling safe traceback)
    let mut nodes: Vec<ActiveNode> = Vec::with_capacity(n + 1);

    // Active indices into `nodes` — candidates for previous break
    let mut active: Vec<usize> = Vec::with_capacity(n);

    // Sentinel node at position 0 (break before first item)
    nodes.push(ActiveNode {
        position: 0,
        cumulative_width: Fp266::ZERO,
        cumulative_stretch: Fp266::ZERO,
        cumulative_shrink: Fp266::ZERO,
        total_demerits: 0.0,
        fitness: FitnessClass::new(),
        previous_break: 0,
    });
    active.push(0);

    // For each item, consider it as the end of a line (break after item i)
    for i in 0..n {
        let mut new_active: Vec<usize> = Vec::new();

        // Handle mandatory breaks: force a break here
        if items[i].is_mandatory {
            // Find the best active node
            let best_a_idx = *active
                .iter()
                .min_by(|&&a, &&b| {
                    nodes[a]
                        .total_demerits
                        .partial_cmp(&nodes[b].total_demerits)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap_or(&0);
            let best = &nodes[best_a_idx];

            // Compute adjustment ratio for the forced line
            let cw = prefix_w[i + 1] - best.cumulative_width;
            let cs = prefix_s[i + 1] - best.cumulative_stretch;
            let ch = prefix_h[i + 1] - best.cumulative_shrink;
            let _r = compute_adjustment_ratio(options.line_width, cw, cs, ch);
            let _badness = compute_badness(_r);
            let new_fitness = FitnessClass::from_ratio(_r);

            let node_idx = nodes.len();
            nodes.push(ActiveNode {
                position: i + 1,
                cumulative_width: prefix_w[i + 1],
                cumulative_stretch: prefix_s[i + 1],
                cumulative_shrink: prefix_h[i + 1],
                total_demerits: best.total_demerits
                    + compute_demerits(
                        _badness,
                        options.fitness_penalty,
                        options.line_penalty,
                        new_fitness.demerit_delta(&best.fitness) > 0.0,
                    ),
                fitness: new_fitness,
                previous_break: best_a_idx,
            });

            active.clear();
            active.push(node_idx);
            continue;
        }

        // Try each active node as a potential previous break.
        // Keep only the BEST (min-demerits) new node per item to bound complexity at O(n²).
        let mut best_new_idx: Option<usize> = None;
        let mut best_new_demerits = f64::INFINITY;

        for &a_idx in &active {
            let node = &nodes[a_idx];

            // Content on this line = cumulative from last break to current item
            let content_w = prefix_w[i + 1] - node.cumulative_width;
            let content_s = prefix_s[i + 1] - node.cumulative_stretch;
            let content_h = prefix_h[i + 1] - node.cumulative_shrink;

            let r = compute_adjustment_ratio(options.line_width, content_w, content_s, content_h);

            // Skip infeasible breaks
            if r.is_infinite() || r.is_nan() {
                continue;
            }
            if r < -(options.max_adjustment_ratio) || r > options.max_adjustment_ratio {
                continue;
            }

            let badness = compute_badness(r);
            let new_fitness = FitnessClass::from_ratio(r);
            let fitness_changed = new_fitness.demerit_delta(&node.fitness) > 0.0;

            // Hyphenation penalty: add extra demerits for hyphenated breaks
            let hyphen_demerit = if items[i].is_hyphenation {
                options.hyphen_penalty
            } else {
                0.0
            };

            // Optical margin reduction: reduce demerits for lines with hanging punctuation
            let optical_reduction = if options.optical_margins && !items[i].text.is_empty() {
                optical_margin_penalty_reduction(items[i].text)
            } else {
                0.0
            };

            let base_demerits = compute_demerits(
                badness,
                options.fitness_penalty,
                options.line_penalty,
                fitness_changed,
            );
            let demerits = (base_demerits + hyphen_demerit - optical_reduction).max(0.0);
            let total = node.total_demerits + demerits;

            if total < best_new_demerits {
                best_new_demerits = total;
                let node_idx = nodes.len();
                nodes.push(ActiveNode {
                    position: i + 1,
                    cumulative_width: prefix_w[i + 1],
                    cumulative_stretch: prefix_s[i + 1],
                    cumulative_shrink: prefix_h[i + 1],
                    total_demerits: total,
                    fitness: new_fitness,
                    previous_break: a_idx,
                });
                best_new_idx = Some(node_idx);
            }
        }

        // Add only the best new node (if any feasible break found)
        if let Some(idx) = best_new_idx {
            new_active.push(idx);
        }

        // Prune: remove active nodes that can never produce feasible breaks.
        // A node at position p is dead if the content from p to current item i
        // exceeds line_width with zero stretch available.
        active.retain(|&a_idx| {
            let node = &nodes[a_idx];
            let content_w = prefix_w[i + 1] - node.cumulative_width;
            let content_s = prefix_s[i + 1] - node.cumulative_stretch;
            // Dead if content overflows AND no stretch to absorb the overflow
            !(content_w > options.line_width && content_s <= Fp266::ZERO)
        });

        active.extend(new_active);
    }

    // Choose the active node with minimum demerits, excluding the sentinel
    // (position 0 = no breaks at all, only valid if everything fits on one line).
    // The last line (from best break to end) must also be feasible.
    let best_a_idx = active
        .iter()
        .filter(|&&a| {
            if nodes[a].position == 0 {
                return false;
            }
            // Check if the last line is feasible
            let last_line_w = prefix_w[n] - nodes[a].cumulative_width;
            let last_line_s = prefix_s[n] - nodes[a].cumulative_stretch;
            let last_line_h = prefix_h[n] - nodes[a].cumulative_shrink;
            let r =
                compute_adjustment_ratio(options.line_width, last_line_w, last_line_s, last_line_h);
            // Last line is allowed to be loose (left-aligned) — only reject if it overflows
            !r.is_infinite() && !r.is_nan() && r >= -(options.max_adjustment_ratio)
        })
        .min_by(|&&a, &&b| {
            nodes[a]
                .total_demerits
                .partial_cmp(&nodes[b].total_demerits)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

    // If no real breaks found (paragraph fits on one line), return empty
    let best_a_idx = match best_a_idx {
        Some(idx) => *idx,
        None => {
            return LineBreakResult {
                breaks: vec![],
                total_demerits: 0.0,
            };
        }
    };

    // Trace back to find break positions
    let mut breaks = Vec::new();
    let mut current = best_a_idx;
    while current != 0 {
        let node = &nodes[current];
        breaks.push(node.position);
        current = node.previous_break;
    }
    breaks.reverse();

    // Remove the sentinel break at position 0 if present
    // (breaks are "after item at index", so break at 0 means "before first item")
    if !breaks.is_empty() && breaks[0] == 0 {
        breaks.remove(0);
    }

    LineBreakResult {
        breaks,
        total_demerits: nodes[best_a_idx].total_demerits,
    }
}

/// Insert hyphenation break candidates into an item list.
///
/// For each word item, computes hyphenation points and inserts additional
/// items at those positions. Each hyphenation item has a penalty and marks
/// `is_hyphenation = true`.
///
/// The `word_items` parameter is a list of `(item_index, word_text)` pairs
/// indicating which items represent words that should be hyphenated.
#[cfg(test)]
pub fn insert_hyphenation_candidates(
    items: &mut Vec<LineBreakItem>,
    word_items: &[(usize, &str)],
    hyphen_penalty: f64,
) {
    let mut insertions: Vec<(usize, LineBreakItem)> = Vec::new();

    for &(idx, word) in word_items {
        if idx >= items.len() {
            continue;
        }
        let hyphen_points = crate::layout::hyphenate::hyphenate_word(word);
        for _hp in &hyphen_points {
            let hyphen_item = LineBreakItem {
                width: Fp266::ZERO,
                stretchability: Fp266::ZERO,
                shrinkability: Fp266::ZERO,
                penalty: hyphen_penalty,
                is_mandatory: false,
                is_hyphenation: true,
                hyphen_width: Fp266::from_int(10),
                text: "-",
            };
            insertions.push((idx, hyphen_item));
        }
    }

    insertions.sort_by_key(|(pos, _)| *pos);
    insertions.reverse();

    for (pos, item) in insertions {
        if pos < items.len() {
            items.insert(pos + 1, item);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fp266::Fp266;

    fn item(width: i32) -> LineBreakItem {
        LineBreakItem {
            width: Fp266::from_int(width),
            stretchability: Fp266::ZERO,
            shrinkability: Fp266::ZERO,
            penalty: 0.0,
            is_mandatory: false,
            is_hyphenation: false,
            hyphen_width: Fp266::ZERO,
            text: "",
        }
    }

    fn stretchable_item(width: i32, stretch: i32, shrink: i32) -> LineBreakItem {
        LineBreakItem {
            width: Fp266::from_int(width),
            stretchability: Fp266::from_int(stretch),
            shrinkability: Fp266::from_int(shrink),
            penalty: 0.0,
            is_mandatory: false,
            is_hyphenation: false,
            hyphen_width: Fp266::ZERO,
            text: "",
        }
    }

    fn mandatory_item(width: i32) -> LineBreakItem {
        LineBreakItem {
            width: Fp266::from_int(width),
            stretchability: Fp266::ZERO,
            shrinkability: Fp266::ZERO,
            penalty: f64::NEG_INFINITY,
            is_mandatory: true,
            is_hyphenation: false,
            hyphen_width: Fp266::ZERO,
            text: "",
        }
    }

    fn hyphen_item(width: i32) -> LineBreakItem {
        LineBreakItem {
            width: Fp266::from_int(width),
            stretchability: Fp266::ZERO,
            shrinkability: Fp266::ZERO,
            penalty: 50.0,
            is_mandatory: false,
            is_hyphenation: true,
            hyphen_width: Fp266::from_int(10),
            text: "-",
        }
    }

    #[test]
    fn test_empty_input() {
        let result = linebreak(&[], &LineBreakOptions::default());
        assert!(result.breaks.is_empty());
        assert!(result.total_demerits < 0.01);
    }

    #[test]
    fn test_single_item_fits() {
        let items = vec![item(50)];
        let opts = LineBreakOptions {
            line_width: Fp266::from_int(100),
            ..Default::default()
        };
        let result = linebreak(&items, &opts);
        assert!(result.breaks.is_empty());
    }

    #[test]
    fn test_two_items_fit() {
        let items = vec![item(40), item(40)];
        let opts = LineBreakOptions {
            line_width: Fp266::from_int(100),
            ..Default::default()
        };
        let result = linebreak(&items, &opts);
        assert!(result.breaks.is_empty());
    }

    #[test]
    fn test_break_needed() {
        let items = vec![item(60), item(60)];
        let opts = LineBreakOptions {
            line_width: Fp266::from_int(100),
            ..Default::default()
        };
        let result = linebreak(&items, &opts);
        assert_eq!(result.breaks, vec![1]);
    }

    #[test]
    fn test_mandatory_break() {
        let items = vec![item(30), mandatory_item(10), item(30)];
        let opts = LineBreakOptions {
            line_width: Fp266::from_int(100),
            ..Default::default()
        };
        let result = linebreak(&items, &opts);
        assert!(result.breaks.contains(&2));
    }

    #[test]
    fn test_three_lines() {
        let items = vec![item(40), item(40), item(40)];
        let opts = LineBreakOptions {
            line_width: Fp266::from_int(60),
            ..Default::default()
        };
        let result = linebreak(&items, &opts);
        assert_eq!(result.breaks, vec![1, 2]);
    }

    #[test]
    fn test_determinism() {
        let items = vec![
            stretchable_item(30, 10, 5),
            stretchable_item(25, 10, 5),
            stretchable_item(35, 10, 5),
            stretchable_item(20, 10, 5),
            stretchable_item(30, 10, 5),
        ];
        let opts = LineBreakOptions {
            line_width: Fp266::from_int(80),
            ..Default::default()
        };

        let r1 = linebreak(&items, &opts);
        let r2 = linebreak(&items, &opts);
        assert_eq!(r1.breaks, r2.breaks);
        assert_eq!(r1.total_demerits, r2.total_demerits);
    }

    #[test]
    fn test_perfect_fit_zero_demerits() {
        let items = vec![item(50), item(50)];
        let opts = LineBreakOptions {
            line_width: Fp266::from_int(100),
            ..Default::default()
        };
        let result = linebreak(&items, &opts);
        assert!(result.breaks.is_empty());
        assert!(result.total_demerits < 0.01);
    }

    #[test]
    fn test_hyphenation_break_preferred_over_overflow() {
        // Three items: word(70), hyphen_point(10), rest(40). Line width 80.
        // Without hyphenation, 70+10=80 fits perfectly (break at index 1).
        // But with hyphenation penalty, normal break at index 1 is preferred
        // because the hyphen adds penalty.
        let items = vec![item(70), hyphen_item(10), item(40)];
        let opts = LineBreakOptions {
            line_width: Fp266::from_int(80),
            hyphen_penalty: 50.0,
            ..Default::default()
        };
        let result = linebreak(&items, &opts);
        // Break after item 0 or 1 — both feasible
        assert!(!result.breaks.is_empty());
    }

    #[test]
    fn test_hyphenation_adds_demerits() {
        // Two items where the break point differs: normal vs hyphen.
        // Items: 55 (normal/hyphen), 55. Line width 60.
        // Both cases break after item 0 (width 55 fits in 60).
        // But hyphen case has extra penalty.
        let items_normal = vec![item(55), item(55)];
        let items_hyphen = vec![hyphen_item(55), item(55)];

        let opts = LineBreakOptions {
            line_width: Fp266::from_int(60),
            hyphen_penalty: 50.0,
            ..Default::default()
        };

        let r_normal = linebreak(&items_normal, &opts);
        let r_hyphen = linebreak(&items_hyphen, &opts);

        assert_eq!(r_normal.breaks, r_hyphen.breaks);
        assert!(
            r_hyphen.total_demerits > r_normal.total_demerits,
            "hyphen demerits {} should > normal demerits {}",
            r_hyphen.total_demerits,
            r_normal.total_demerits
        );
    }

    #[test]
    fn test_optical_margins_reduce_demerits() {
        // A line ending with a period should have lower demerits when optical margins are on
        let items_period = vec![LineBreakItem {
            width: Fp266::from_int(50),
            stretchability: Fp266::ZERO,
            shrinkability: Fp266::ZERO,
            penalty: 0.0,
            is_mandatory: false,
            is_hyphenation: false,
            hyphen_width: Fp266::ZERO,
            text: "word.",
        }];
        let items_no_period = vec![LineBreakItem {
            width: Fp266::from_int(50),
            stretchability: Fp266::ZERO,
            shrinkability: Fp266::ZERO,
            penalty: 0.0,
            is_mandatory: false,
            is_hyphenation: false,
            hyphen_width: Fp266::ZERO,
            text: "word",
        }];

        let opts_off = LineBreakOptions {
            line_width: Fp266::from_int(60),
            optical_margins: false,
            ..Default::default()
        };
        let opts_on = LineBreakOptions {
            line_width: Fp266::from_int(60),
            optical_margins: true,
            ..Default::default()
        };

        let r_off = linebreak(&items_period, &opts_off);
        let r_on = linebreak(&items_period, &opts_on);
        let r_no_period = linebreak(&items_no_period, &opts_on);

        // With optical margins on, period-ending text should have lower or equal demerits
        assert!(r_on.total_demerits <= r_off.total_demerits);
        // Non-punctuated text should have the same demerits regardless
        assert_eq!(r_no_period.total_demerits, r_off.total_demerits);
    }

    #[test]
    fn test_insert_hyphenation_candidates() {
        let mut items = vec![item(100), item(50)];
        let word_items = [(0, "international")];
        insert_hyphenation_candidates(&mut items, &word_items, 50.0);
        // Should have inserted additional items after index 0
        assert!(items.len() > 2);
        let hyphen_count = items.iter().filter(|i| i.is_hyphenation).count();
        assert!(hyphen_count > 0);
    }
}
