//! Badness and adjustment ratio computation for line breaking.
//!
//! References: YP-LAYOUT-KNUTHPLASS-001, DEF-BADNESS, DEF-ADJ-RATIO.

use crate::fp266::Fp266;

/// Compute the adjustment ratio for a line.
///
/// - If gap > 0 (line needs stretching): r = gap / stretch
/// - If gap < 0 (line needs shrinking):  r = gap / shrink
/// - If gap = 0: r = 0 (perfect fit)
///
/// When stretch or shrink is zero and the corresponding gap direction applies:
/// - gap > 0 with no stretch: feasible (r = 0, extra space goes to margin)
/// - gap < 0 with no shrink: infeasible (r = -∞, content overflows)
///
/// See YP-LAYOUT-KNUTHPLASS-001 DEF-ADJ-RATIO.
#[inline]
pub fn compute_adjustment_ratio(
    line_width: Fp266,
    content_width: Fp266,
    total_stretch: Fp266,
    total_shrink: Fp266,
) -> f64 {
    let gap = (line_width - content_width).to_f64();

    if gap.abs() < 1e-10 {
        return 0.0;
    }

    if gap > 0.0 {
        // Line needs stretching
        let s = total_stretch.to_f64();
        if s < 1e-10 {
            // No stretch available — extra space goes to margin, feasible
            0.0
        } else {
            gap / s
        }
    } else {
        // Line needs shrinking (gap < 0)
        let s = total_shrink.to_f64();
        if s < 1e-10 {
            // No shrink available — content overflows, infeasible
            f64::NEG_INFINITY
        } else {
            gap / s
        }
    }
}

/// Compute the badness of a line given its adjustment ratio.
///
/// b = 100 * |r|^3
///
/// A badness of 0 means perfect fit. Higher badness means worse fit.
/// See YP-LAYOUT-KNUTHPLASS-001 DEF-BADNESS.
#[inline]
pub fn compute_badness(r: f64) -> f64 {
    100.0 * r.abs().powi(3)
}

/// Compute the demerits for a single line break.
///
/// d = (line_penalty + badness)^2 + fitness_penalty * (1 if fitness class changes)
///
/// See YP-LAYOUT-KNUTHPLASS-001 DEF-DEMERITS.
#[inline]
pub fn compute_demerits(
    badness: f64,
    fitness_penalty: f64,
    line_penalty: f64,
    fitness_changed: bool,
) -> f64 {
    let line_demerit = line_penalty + badness;
    let fitness_demerit = if fitness_changed {
        fitness_penalty
    } else {
        0.0
    };
    line_demerit * line_demerit + fitness_demerit * fitness_demerit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_fit() {
        let r = compute_adjustment_ratio(
            Fp266::from_int(100),
            Fp266::from_int(100),
            Fp266::from_int(10),
            Fp266::from_int(10),
        );
        assert!(r.abs() < 1e-10);
    }

    #[test]
    fn test_need_stretch() {
        let r = compute_adjustment_ratio(
            Fp266::from_int(100),
            Fp266::from_int(80),
            Fp266::from_int(40),
            Fp266::from_int(0),
        );
        assert!(r > 0.0);
        assert!(r < 1.0); // feasible
    }

    #[test]
    fn test_need_shrink() {
        let r = compute_adjustment_ratio(
            Fp266::from_int(100),
            Fp266::from_int(120),
            Fp266::from_int(0),
            Fp266::from_int(40),
        );
        assert!(r < 0.0);
        assert!(r > -1.0); // feasible
    }

    #[test]
    fn test_infeasible() {
        let r = compute_adjustment_ratio(
            Fp266::from_int(100),
            Fp266::from_int(200),
            Fp266::from_int(10),
            Fp266::from_int(0),
        );
        assert!(r.abs() > 1.0); // infeasible
    }

    #[test]
    fn test_badness_perfect() {
        assert!(compute_badness(0.0) < 1e-10);
    }

    #[test]
    fn test_badness_half_stretch() {
        let b = compute_badness(0.5);
        assert!((b - 12.5).abs() < 0.01); // 100 * 0.125
    }

    #[test]
    fn test_demerits_perfect() {
        let d = compute_demerits(0.0, 100.0, 10.0, false);
        assert!((d - 100.0).abs() < 0.01); // (10 + 0)^2 = 100
    }

    #[test]
    fn test_demerits_with_fitness_change() {
        let d1 = compute_demerits(0.0, 100.0, 10.0, false);
        let d2 = compute_demerits(0.0, 100.0, 10.0, true);
        assert!(d2 > d1); // fitness change adds demerits
    }
}
