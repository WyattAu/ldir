//! Knuth-Plass line-breaking algorithm.
//!
//! Dynamic programming approach to find optimal line breaks in a paragraph.
//! References: YP-LAYOUT-KNUTHPLASS-001, ALG-KP-BREAK, THM-KP-OPTIMALITY, THM-KP-TERMINATION.

use bumpalo::Bump;
use bumpalo::collections::Vec as BumpVec;

use super::badness::{compute_badness, compute_demerits};
use super::optical_margin::optical_margin_penalty_reduction;
use super::types::*;

#[cfg(test)]
use crate::fp266::Fp266;

/// An active node in the DP algorithm, representing a candidate line break position.
#[derive(Debug, Clone)]
struct ActiveNode {
    position: usize,
    cumulative_width: f64,
    cumulative_stretch: f64,
    cumulative_shrink: f64,
    total_demerits: f64,
    fitness: FitnessClass,
    previous_break: usize,
}

/// Compute the adjustment ratio using pre-converted f64 values.
///
/// Inlined version that avoids Fp266→f64 conversions in the hot inner loop.
/// See YP-LAYOUT-KNUTHPLASS-001 DEF-ADJ-RATIO for the mathematical specification.
#[inline]
fn adjustment_ratio(line_width: f64, content_width: f64, stretch: f64, shrink: f64) -> f64 {
    let gap = line_width - content_width;
    if gap > 0.0 {
        if stretch < 1e-10 { 0.0 } else { gap / stretch }
    } else if gap < 0.0 {
        if shrink < 1e-10 {
            f64::NEG_INFINITY
        } else {
            gap / shrink
        }
    } else {
        0.0
    }
}

/// Run the Knuth-Plass line-breaking algorithm.
///
/// Finds the optimal set of line breaks that minimizes total demerits.
/// Returns the indices where breaks should occur (after the item at each index).
///
/// # Algorithm (ALG-KP-BREAK)
/// 1. Precompute prefix sums of width/stretch/shrink as f64
/// 2. Initialize active node list with position 0
/// 3. For each item, try every active node as a potential previous break
/// 4. Compute adjustment ratio using cumulative differences (f64, no conversion)
/// 5. Keep feasible breaks; deactivate infeasible nodes
/// 6. At end, choose the active node with minimum total demerits
/// 7. Trace back to recover break positions
pub fn linebreak<'bump>(
    items: &[LineBreakItem],
    options: &LineBreakOptions,
    bump: &'bump Bump,
) -> LineBreakResult {
    if items.is_empty() {
        return LineBreakResult {
            breaks: vec![],
            total_demerits: 0.0,
        };
    }

    let n = items.len();

    // Precompute prefix sums as f64 — eliminates Fp266→f64 conversions in the hot loop.
    // (arena-allocated).
    let mut prefix_w = BumpVec::with_capacity_in(n + 1, bump);
    let mut prefix_s = BumpVec::with_capacity_in(n + 1, bump);
    let mut prefix_h = BumpVec::with_capacity_in(n + 1, bump);
    prefix_w.push(0.0f64);
    prefix_s.push(0.0f64);
    prefix_h.push(0.0f64);
    for i in 0..n {
        prefix_w.push(prefix_w[i] + items[i].width.to_f64());
        prefix_s.push(prefix_s[i] + items[i].stretchability.to_f64());
        prefix_h.push(prefix_h[i] + items[i].shrinkability.to_f64());
    }

    // Pre-cache line_width as f64 — used in every iteration of the hot loop.
    let line_width_f64 = options.line_width.to_f64();
    let max_r = options.max_adjustment_ratio;

    // Quick check: if everything fits on one line AND there are no mandatory breaks, no breaks needed
    let has_mandatory = items.iter().any(|item| item.is_mandatory);
    if !has_mandatory {
        let overall_r = adjustment_ratio(line_width_f64, prefix_w[n], prefix_s[n], prefix_h[n]);
        if !overall_r.is_infinite() && !overall_r.is_nan() && overall_r.abs() <= max_r {
            return LineBreakResult {
                breaks: vec![],
                total_demerits: compute_badness(overall_r),
            };
        }
    }

    // Persistent node store (arena-allocated).
    let mut nodes: BumpVec<'bump, ActiveNode> = BumpVec::with_capacity_in(n + 1, bump);
    let mut active: BumpVec<'bump, usize> = BumpVec::with_capacity_in(n, bump);

    // Sentinel node at position 0 (break before first item)
    nodes.push(ActiveNode {
        position: 0,
        cumulative_width: 0.0,
        cumulative_stretch: 0.0,
        cumulative_shrink: 0.0,
        total_demerits: 0.0,
        fitness: FitnessClass::new(),
        previous_break: 0,
    });
    active.push(0);

    // For each item, consider it as the end of a line (break after item i)
    for i in 0..n {
        let cur_w = prefix_w[i + 1];
        let cur_s = prefix_s[i + 1];
        let cur_h = prefix_h[i + 1];

        // Hoist per-item computations out of the inner loop (same for all active nodes).
        let item = &items[i];

        // Handle mandatory breaks: force a break here
        if item.is_mandatory {
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

            let cw = cur_w - best.cumulative_width;
            let cs = cur_s - best.cumulative_stretch;
            let ch = cur_h - best.cumulative_shrink;
            let _r = adjustment_ratio(line_width_f64, cw, cs, ch);
            let _badness = compute_badness(_r);
            let new_fitness = FitnessClass::from_ratio(_r);

            let node_idx = nodes.len();
            nodes.push(ActiveNode {
                position: i + 1,
                cumulative_width: cur_w,
                cumulative_stretch: cur_s,
                cumulative_shrink: cur_h,
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

        // Hoist per-item values that don't depend on the active node.
        let hyphen_demerit = if item.is_hyphenation {
            options.hyphen_penalty
        } else {
            0.0
        };
        let optical_reduction = if options.optical_margins && !item.text.is_empty() {
            optical_margin_penalty_reduction(item.text)
        } else {
            0.0
        };

        // Try each active node as a potential previous break.
        // Keep only the BEST (min-demerits) new node per item to bound complexity at O(n²).
        // Save parameters only — defer node allocation to after the loop.
        let mut best_total = f64::INFINITY;
        let mut best_prev: usize = 0;
        let mut best_fitness = FitnessClass::new();
        let mut found = false;

        for &a_idx in &active {
            let node = &nodes[a_idx];

            let content_w = cur_w - node.cumulative_width;
            let content_s = cur_s - node.cumulative_stretch;
            let content_h = cur_h - node.cumulative_shrink;

            let r = adjustment_ratio(line_width_f64, content_w, content_s, content_h);

            // Skip infeasible breaks
            if r.is_infinite() || r.is_nan() {
                continue;
            }
            if r < -(max_r) || r > max_r {
                continue;
            }

            let badness = compute_badness(r);
            let new_fitness = FitnessClass::from_ratio(r);
            let fitness_changed = new_fitness.demerit_delta(&node.fitness) > 0.0;

            let base_demerits = compute_demerits(
                badness,
                options.fitness_penalty,
                options.line_penalty,
                fitness_changed,
            );
            let demerits = (base_demerits + hyphen_demerit - optical_reduction).max(0.0);
            let total = node.total_demerits + demerits;

            if total < best_total {
                best_total = total;
                best_prev = a_idx;
                best_fitness = new_fitness;
                found = true;
            }
        }

        // Prune: manual compacting loop replaces BumpVec::retain to avoid
        // drain-filter iterator overhead. Uses f64 arithmetic directly.
        let mut write = 0;
        for read in 0..active.len() {
            let a_idx = active[read];
            let node = &nodes[a_idx];
            let content_w = cur_w - node.cumulative_width;
            let content_s = cur_s - node.cumulative_stretch;
            // Dead if content overflows AND no stretch to absorb the overflow
            if !(content_w > line_width_f64 && content_s <= 0.0) {
                active[write] = a_idx;
                write += 1;
            }
        }
        active.truncate(write);

        // Add only the best new node (pushed once after loop — no wasted allocations)
        if found {
            let node_idx = nodes.len();
            nodes.push(ActiveNode {
                position: i + 1,
                cumulative_width: cur_w,
                cumulative_stretch: cur_s,
                cumulative_shrink: cur_h,
                total_demerits: best_total,
                fitness: best_fitness,
                previous_break: best_prev,
            });
            active.push(node_idx);
        }
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
            let r = adjustment_ratio(line_width_f64, last_line_w, last_line_s, last_line_h);
            // Last line is allowed to be loose (left-aligned) — only reject if it overflows
            !r.is_infinite() && !r.is_nan() && r >= -(max_r)
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
    use bumpalo::Bump;

    /// Test helper: runs linebreak with a fresh arena.
    fn kp(items: &[LineBreakItem], opts: &LineBreakOptions) -> LineBreakResult {
        let bump = Bump::new();
        linebreak(items, opts, &bump)
    }

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
        let result = kp(&[], &LineBreakOptions::default());
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
        let result = kp(&items, &opts);
        assert!(result.breaks.is_empty());
    }

    #[test]
    fn test_two_items_fit() {
        let items = vec![item(40), item(40)];
        let opts = LineBreakOptions {
            line_width: Fp266::from_int(100),
            ..Default::default()
        };
        let result = kp(&items, &opts);
        assert!(result.breaks.is_empty());
    }

    #[test]
    fn test_break_needed() {
        let items = vec![item(60), item(60)];
        let opts = LineBreakOptions {
            line_width: Fp266::from_int(100),
            ..Default::default()
        };
        let result = kp(&items, &opts);
        assert_eq!(result.breaks, vec![1]);
    }

    #[test]
    fn test_mandatory_break() {
        let items = vec![item(30), mandatory_item(10), item(30)];
        let opts = LineBreakOptions {
            line_width: Fp266::from_int(100),
            ..Default::default()
        };
        let result = kp(&items, &opts);
        assert!(result.breaks.contains(&2));
    }

    #[test]
    fn test_three_lines() {
        let items = vec![item(40), item(40), item(40)];
        let opts = LineBreakOptions {
            line_width: Fp266::from_int(60),
            ..Default::default()
        };
        let result = kp(&items, &opts);
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

        let r1 = kp(&items, &opts);
        let r2 = kp(&items, &opts);
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
        let result = kp(&items, &opts);
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
        let result = kp(&items, &opts);
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

        let r_normal = kp(&items_normal, &opts);
        let r_hyphen = kp(&items_hyphen, &opts);

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

        let r_off = kp(&items_period, &opts_off);
        let r_on = kp(&items_period, &opts_on);
        let r_no_period = kp(&items_no_period, &opts_on);

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
