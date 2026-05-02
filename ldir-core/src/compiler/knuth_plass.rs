/// Knuth-Plass line breaking: find optimal line breaks for a paragraph.
///
/// Implements the classic algorithm from "Breaking Paragraphs into Lines"
/// (Knuth & Plass, 1981). Given a sequence of boxes, glue, penalties, and
/// forced breaks, finds the set of breakpoints that minimises total demerits.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KPBox {
    Content {
        width: i32,
    },
    Glue {
        width: i32,
        stretch: i32,
        shrink: i32,
    },
    Penalty {
        penalty: i32,
        width: i32,
        flagged: bool,
    },
    ForcedBreak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Fitness {
    Tight,
    Normal,
    Loose,
    VeryLoose,
}

const LINE_PENALTY: i32 = 10;
const HYPHEN_PENALTY: i32 = 50;
const FITNESS_DEMERIT: i32 = 100;
const FLAGGED_DEMERIT: i32 = 100;
const DOUBLE_HYPHEN_DEMERIT: i32 = 10000;
const TOLERANCE: f64 = 2.0;
const INFINITY_PENALTY: i32 = 10000;

#[derive(Debug, Clone)]
struct ActiveNode {
    position: usize,
    fitness: Fitness,
    ratio: f64,
    demerits: f64,
    previous: usize,
    total_width: i32,
    total_stretch: i32,
    total_shrink: i32,
}

fn classify_fitness(ratio: f64) -> Fitness {
    if ratio < -0.5 {
        Fitness::Tight
    } else if ratio <= 0.5 {
        Fitness::Normal
    } else if ratio <= 1.0 {
        Fitness::Loose
    } else {
        Fitness::VeryLoose
    }
}

fn compute_ratio(
    line_width: i32,
    total_width: i32,
    total_stretch: i32,
    total_shrink: i32,
) -> Option<f64> {
    let diff = total_width - line_width;
    if diff == 0 {
        return Some(0.0);
    }
    if diff < 0 {
        if total_stretch > 0 {
            let r = (-diff) as f64 / total_stretch as f64;
            if r <= TOLERANCE { Some(r) } else { None }
        } else {
            Some(0.0)
        }
    } else if total_shrink > 0 {
        let r = -(diff as f64 / total_shrink as f64);
        if r >= -TOLERANCE { Some(r) } else { None }
    } else {
        None
    }
}

pub fn knuth_plass_break(boxes: &[KPBox], line_width: i32) -> Vec<usize> {
    if boxes.is_empty() {
        return Vec::new();
    }

    let mut active_nodes: Vec<ActiveNode> = vec![ActiveNode {
        position: 0,
        fitness: Fitness::Normal,
        ratio: 0.0,
        demerits: 0.0,
        previous: 0,
        total_width: 0,
        total_stretch: 0,
        total_shrink: 0,
    }];

    let mut width_acc: i32 = 0;
    let mut stretch_acc: i32 = 0;
    let mut shrink_acc: i32 = 0;

    for (idx, box_item) in boxes.iter().enumerate() {
        match *box_item {
            KPBox::Content { width } => {
                width_acc += width;
            }
            KPBox::Glue {
                width,
                stretch,
                shrink,
            } => {
                width_acc += width;
                stretch_acc += stretch;
                shrink_acc += shrink;

                let pos = idx + 1;
                let mut new_nodes: Vec<ActiveNode> = Vec::new();
                for (ai, an) in active_nodes.iter().enumerate() {
                    let lw = width_acc - an.total_width;
                    let ls = stretch_acc - an.total_stretch;
                    let lsh = shrink_acc - an.total_shrink;
                    let Some(ratio) = compute_ratio(line_width, lw, ls, lsh) else {
                        continue;
                    };
                    let fitness = classify_fitness(ratio);
                    let r_abs = ratio.abs();
                    let mut demerits = (LINE_PENALTY as f64 + 1.0) * r_abs * r_abs * r_abs;
                    if ai > 0 {
                        let prev = &active_nodes[an.previous];
                        if fitness == prev.fitness {
                            demerits += FITNESS_DEMERIT as f64;
                        }
                    }
                    new_nodes.push(ActiveNode {
                        position: pos,
                        fitness,
                        ratio,
                        demerits: an.demerits + demerits,
                        previous: ai,
                        total_width: width_acc,
                        total_stretch: stretch_acc,
                        total_shrink: shrink_acc,
                    });
                }
                active_nodes.extend(new_nodes);
            }
            KPBox::Penalty {
                penalty,
                width,
                flagged,
            } => {
                width_acc += width;
                if penalty == INFINITY_PENALTY {
                    continue;
                }

                let pos = idx + 1;
                let mut new_nodes: Vec<ActiveNode> = Vec::new();
                for (ai, an) in active_nodes.iter().enumerate() {
                    let lw = width_acc - an.total_width;
                    let ls = stretch_acc - an.total_stretch;
                    let lsh = shrink_acc - an.total_shrink;
                    let Some(ratio) = compute_ratio(line_width, lw, ls, lsh) else {
                        continue;
                    };
                    let fitness = classify_fitness(ratio);
                    let r_abs = ratio.abs();
                    let mut demerits = (LINE_PENALTY as f64 + 1.0) * r_abs * r_abs * r_abs;
                    if penalty > 0 {
                        demerits += penalty as f64;
                    } else if penalty < 0 {
                        demerits -= penalty as f64;
                    }
                    if ai > 0 {
                        let prev = &active_nodes[an.previous];
                        if flagged && prev.ratio.abs() > 0.5 {
                            demerits += FLAGGED_DEMERIT as f64;
                        }
                        let is_hyphen = flagged || penalty == HYPHEN_PENALTY;
                        if is_hyphen && prev.fitness == Fitness::Tight {
                            demerits += DOUBLE_HYPHEN_DEMERIT as f64;
                        }
                        if fitness == prev.fitness {
                            demerits += FITNESS_DEMERIT as f64;
                        }
                    }
                    new_nodes.push(ActiveNode {
                        position: pos,
                        fitness,
                        ratio,
                        demerits: an.demerits + demerits,
                        previous: ai,
                        total_width: width_acc,
                        total_stretch: stretch_acc,
                        total_shrink: shrink_acc,
                    });
                }
                active_nodes.extend(new_nodes);
            }
            KPBox::ForcedBreak => {
                let pos = idx + 1;
                let mut new_nodes: Vec<ActiveNode> = Vec::new();
                for (ai, an) in active_nodes.iter().enumerate() {
                    let lw = width_acc - an.total_width;
                    let ls = stretch_acc - an.total_stretch;
                    let lsh = shrink_acc - an.total_shrink;
                    let diff = lw - line_width;
                    let ratio = if diff == 0 {
                        0.0
                    } else if diff < 0 {
                        if ls > 0 {
                            (-diff) as f64 / ls as f64
                        } else {
                            0.0
                        }
                    } else if lsh > 0 {
                        -(diff as f64 / lsh as f64)
                    } else {
                        -TOLERANCE - 1.0
                    };
                    let fitness = classify_fitness(ratio);
                    let r_abs = ratio.abs();
                    let mut demerits = (LINE_PENALTY as f64 + 1.0) * r_abs * r_abs * r_abs;
                    if ai > 0 {
                        let prev = &active_nodes[an.previous];
                        if fitness == prev.fitness {
                            demerits += FITNESS_DEMERIT as f64;
                        }
                    }
                    new_nodes.push(ActiveNode {
                        position: pos,
                        fitness,
                        ratio,
                        demerits: an.demerits + demerits,
                        previous: ai,
                        total_width: width_acc,
                        total_stretch: stretch_acc,
                        total_shrink: shrink_acc,
                    });
                }
                active_nodes.extend(new_nodes);
            }
        }
    }

    if active_nodes.len() <= 1 {
        return Vec::new();
    }

    let last_pos = active_nodes[1..]
        .iter()
        .map(|n| n.position)
        .max()
        .unwrap_or(0);

    let mut best_idx = 1;
    let mut found = false;
    for i in 1..active_nodes.len() {
        if active_nodes[i].position == last_pos
            && (!found || active_nodes[i].demerits < active_nodes[best_idx].demerits)
        {
            best_idx = i;
            found = true;
        }
    }

    let mut breaks = Vec::new();
    let mut current = best_idx;
    while current > 0 {
        breaks.push(active_nodes[current].position);
        current = active_nodes[current].previous;
    }
    breaks.reverse();
    breaks
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word_box(width: i32) -> KPBox {
        KPBox::Content { width }
    }

    fn glue_box(width: i32, stretch: i32, shrink: i32) -> KPBox {
        KPBox::Glue {
            width,
            stretch,
            shrink,
        }
    }

    fn penalty_box(penalty: i32, width: i32, flagged: bool) -> KPBox {
        KPBox::Penalty {
            penalty,
            width,
            flagged,
        }
    }

    fn forced_break() -> KPBox {
        KPBox::ForcedBreak
    }

    #[test]
    fn test_fitness_classify() {
        assert_eq!(classify_fitness(-0.6), Fitness::Tight);
        assert_eq!(classify_fitness(-0.5), Fitness::Normal);
        assert_eq!(classify_fitness(0.0), Fitness::Normal);
        assert_eq!(classify_fitness(0.5), Fitness::Normal);
        assert_eq!(classify_fitness(0.6), Fitness::Loose);
        assert_eq!(classify_fitness(1.0), Fitness::Loose);
        assert_eq!(classify_fitness(1.1), Fitness::VeryLoose);
    }

    #[test]
    fn test_compute_ratio_zero_diff() {
        assert_eq!(compute_ratio(100, 100, 10, 10), Some(0.0));
    }

    #[test]
    fn test_compute_ratio_stretch() {
        let r = compute_ratio(100, 90, 10, 5).unwrap();
        assert!((r - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_compute_ratio_shrink() {
        let r = compute_ratio(100, 110, 5, 10).unwrap();
        assert!((r - (-1.0)).abs() < 1e-9);
    }

    #[test]
    fn test_compute_ratio_infeasible_stretch() {
        assert_eq!(compute_ratio(100, 70, 10, 5), None);
    }

    #[test]
    fn test_compute_ratio_infeasible_shrink() {
        assert_eq!(compute_ratio(100, 130, 5, 10), None);
    }

    #[test]
    fn test_compute_ratio_no_stretch() {
        assert_eq!(compute_ratio(100, 90, 0, 5), Some(0.0));
    }

    #[test]
    fn test_compute_ratio_no_shrink() {
        assert_eq!(compute_ratio(100, 110, 5, 0), None);
    }

    #[test]
    fn test_empty_input() {
        let boxes: Vec<KPBox> = vec![];
        let breaks = knuth_plass_break(&boxes, 100);
        assert!(breaks.is_empty());
    }

    #[test]
    fn test_single_word() {
        let boxes = vec![word_box(30), forced_break()];
        let breaks = knuth_plass_break(&boxes, 100);
        assert_eq!(breaks, vec![2]);
    }

    #[test]
    fn test_forced_break() {
        let boxes = vec![word_box(30), forced_break()];
        let breaks = knuth_plass_break(&boxes, 100);
        assert_eq!(breaks, vec![2]);
    }

    #[test]
    fn test_exact_fit() {
        let boxes = vec![
            word_box(40),
            glue_box(10, 5, 5),
            word_box(40),
            forced_break(),
        ];
        let breaks = knuth_plass_break(&boxes, 90);
        assert_eq!(breaks, vec![4]);
    }

    #[test]
    fn test_single_line_fits() {
        let boxes = vec![
            word_box(30),
            glue_box(10, 5, 5),
            word_box(30),
            glue_box(10, 5, 5),
            word_box(30),
            forced_break(),
        ];
        let breaks = knuth_plass_break(&boxes, 100);
        assert_eq!(breaks, vec![6]);
    }

    #[test]
    fn test_only_glue() {
        let boxes = vec![glue_box(10, 5, 5), glue_box(10, 5, 5), forced_break()];
        let breaks = knuth_plass_break(&boxes, 100);
        assert_eq!(breaks, vec![3]);
    }

    #[test]
    fn test_two_lines() {
        let boxes = vec![
            word_box(30),
            glue_box(10, 10, 5),
            word_box(30),
            glue_box(10, 10, 5),
            word_box(30),
            glue_box(10, 10, 5),
            word_box(30),
            forced_break(),
        ];
        let breaks = knuth_plass_break(&boxes, 80);
        assert_eq!(breaks.len(), 2);
        assert_eq!(breaks[0], 4);
    }

    #[test]
    fn test_three_lines() {
        let boxes = vec![
            word_box(20),
            glue_box(10, 10, 5),
            word_box(20),
            glue_box(10, 10, 5),
            word_box(20),
            glue_box(10, 10, 5),
            word_box(20),
            glue_box(10, 10, 5),
            word_box(20),
            glue_box(10, 10, 5),
            word_box(20),
            forced_break(),
        ];
        let breaks = knuth_plass_break(&boxes, 60);
        assert_eq!(breaks.len(), 3);
    }

    #[test]
    fn test_loose_line() {
        let boxes = vec![
            word_box(20),
            glue_box(10, 20, 2),
            word_box(20),
            forced_break(),
        ];
        let breaks = knuth_plass_break(&boxes, 60);
        assert_eq!(breaks, vec![4]);
    }

    #[test]
    fn test_tight_line() {
        let boxes = vec![
            word_box(40),
            glue_box(10, 2, 10),
            word_box(40),
            forced_break(),
        ];
        let breaks = knuth_plass_break(&boxes, 80);
        assert!(!breaks.is_empty());
        assert_eq!(breaks.last().unwrap(), &4);
    }

    #[test]
    fn test_very_loose_line() {
        let boxes = vec![
            word_box(10),
            glue_box(10, 20, 2),
            word_box(10),
            forced_break(),
        ];
        let breaks = knuth_plass_break(&boxes, 60);
        assert!(!breaks.is_empty());
    }

    #[test]
    fn test_infeasible_lines() {
        let boxes = vec![
            word_box(50),
            glue_box(10, 5, 5),
            word_box(50),
            glue_box(10, 5, 5),
            word_box(50),
            forced_break(),
        ];
        let breaks = knuth_plass_break(&boxes, 60);
        assert!(breaks.len() >= 2);
    }

    #[test]
    fn test_prefer_fewer_breaks() {
        let boxes = vec![
            word_box(25),
            glue_box(10, 10, 5),
            word_box(25),
            glue_box(10, 10, 5),
            word_box(25),
            forced_break(),
        ];
        let breaks = knuth_plass_break(&boxes, 80);
        assert_eq!(breaks.len(), 1);
    }

    #[test]
    fn test_penalty_break() {
        let boxes = vec![
            word_box(30),
            glue_box(10, 5, 5),
            word_box(30),
            penalty_box(50, 0, false),
            word_box(30),
            forced_break(),
        ];
        let breaks = knuth_plass_break(&boxes, 70);
        assert!(!breaks.is_empty());
    }

    #[test]
    fn test_adjacency_penalty() {
        let boxes = vec![
            word_box(20),
            glue_box(10, 10, 10),
            word_box(20),
            glue_box(10, 10, 10),
            word_box(20),
            glue_box(10, 10, 10),
            word_box(20),
            forced_break(),
        ];
        let breaks = knuth_plass_break(&boxes, 55);
        assert!(!breaks.is_empty());
    }

    #[test]
    fn test_different_fitness_no_adj_penalty() {
        let boxes = vec![
            word_box(30),
            glue_box(10, 20, 2),
            word_box(10),
            glue_box(10, 2, 10),
            word_box(30),
            forced_break(),
        ];
        let breaks = knuth_plass_break(&boxes, 50);
        assert!(!breaks.is_empty());
    }

    #[test]
    fn test_flagged_penalty_extra_demerit() {
        let boxes = vec![
            word_box(30),
            glue_box(10, 5, 5),
            word_box(20),
            penalty_box(HYPHEN_PENALTY, 5, true),
            word_box(20),
            glue_box(10, 5, 5),
            word_box(30),
            forced_break(),
        ];
        let breaks = knuth_plass_break(&boxes, 100);
        assert!(!breaks.is_empty());
    }

    #[test]
    fn test_infinity_penalty_skipped() {
        let boxes = vec![
            word_box(30),
            penalty_box(INFINITY_PENALTY, 0, false),
            word_box(30),
            forced_break(),
        ];
        let breaks = knuth_plass_break(&boxes, 100);
        assert_eq!(breaks, vec![4]);
    }

    #[test]
    fn test_multiple_forced_breaks() {
        let boxes = vec![word_box(30), forced_break(), word_box(30), forced_break()];
        let breaks = knuth_plass_break(&boxes, 100);
        assert!(!breaks.is_empty());
    }

    #[test]
    fn test_multiple_penalties() {
        let boxes = vec![
            word_box(20),
            glue_box(10, 5, 5),
            word_box(20),
            penalty_box(10, 0, false),
            word_box(20),
            glue_box(10, 5, 5),
            word_box(20),
            penalty_box(10, 0, false),
            word_box(20),
            forced_break(),
        ];
        let breaks = knuth_plass_break(&boxes, 55);
        assert!(!breaks.is_empty());
        assert!(breaks.len() >= 2);
    }

    #[test]
    fn test_widow_penalty() {
        let boxes = vec![
            word_box(20),
            glue_box(10, 5, 5),
            word_box(20),
            glue_box(10, 5, 5),
            word_box(20),
            glue_box(10, 5, 5),
            word_box(20),
            glue_box(10, 5, 5),
            word_box(20),
            forced_break(),
        ];
        let breaks = knuth_plass_break(&boxes, 50);
        assert!(!breaks.is_empty());
    }

    #[test]
    fn test_knuth_plass_example() {
        let boxes = vec![
            word_box(30),
            glue_box(10, 5, 5),
            word_box(30),
            glue_box(10, 5, 5),
            word_box(30),
            forced_break(),
        ];
        let breaks = knuth_plass_break(&boxes, 70);
        assert_eq!(breaks.len(), 2);
    }

    #[test]
    fn test_long_paragraph() {
        let mut boxes: Vec<KPBox> = Vec::new();
        for i in 0..30 {
            boxes.push(word_box(20));
            if i < 29 {
                boxes.push(glue_box(10, 5, 5));
            }
        }
        boxes.push(forced_break());
        let breaks = knuth_plass_break(&boxes, 80);
        assert!(!breaks.is_empty());
        assert!(breaks.len() >= 3);
    }

    #[test]
    fn test_no_stretch_only_shrink() {
        let boxes = vec![
            word_box(40),
            glue_box(10, 0, 5),
            word_box(40),
            forced_break(),
        ];
        let breaks = knuth_plass_break(&boxes, 85);
        assert_eq!(breaks, vec![4]);
    }
}
