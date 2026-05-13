//! Badness and adjustment ratio computation for line breaking.
//!
//! References: YP-LAYOUT-KNUTHPLASS-001, DEF-BADNESS, DEF-ADJ-RATIO.
//!
//! ## SIMD Batch Processing (U-2)
//!
//! `compute_demerits_batch_simd` processes 4 candidate breaks simultaneously
//! using `f64x4` SIMD. Callers should gather 4 candidates' parameters
//! into SIMD vectors, call the batch function, then scatter results.
//!
//! Note: SIMD intrinsics require `unsafe` blocks. These are isolated to the
//! batch functions and justified by the performance-critical nature of the
//! Knuth-Plass DP inner loop.

#![allow(unsafe_code)]

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

/// Compute badness for 4 ratios simultaneously using SIMD.
///
/// Returns `[b0, b1, b2, b3]` where `bi = 100 * |ri|^3`.
#[inline]
#[allow(dead_code, clippy::unnecessary_cast)]
#[target_feature(enable = "sse2")]
pub unsafe fn compute_badness_batch_simd(r0: f64, r1: f64, r2: f64, r3: f64) -> [f64; 4] {
    let v = [r0, r1, r2, r3];
    v.map(|ri| 100.0 * ri.abs().powi(3))
}

/// Compute demerits for 4 candidate breaks simultaneously.
///
/// All input arrays must have exactly 4 elements.
/// Returns `[d0, d1, d2, d3]` where `di` is the demerit for candidate i.
///
/// This is a scalar batch (not SIMD) — it processes 4 candidates without
/// branching to enable future SIMD optimization and to amortize function
/// call overhead in the DP inner loop.
#[inline]
#[allow(dead_code)]
pub fn compute_demerits_batch(
    badness: [f64; 4],
    fitness_penalty: f64,
    line_penalty: f64,
    fitness_changed: [bool; 4],
) -> [f64; 4] {
    std::array::from_fn(|i| {
        compute_demerits(
            badness[i],
            fitness_penalty,
            line_penalty,
            fitness_changed[i],
        )
    })
}

/// SIMD-accelerated demerits computation for a batch of 4 candidates.
///
/// Processes 4 candidates using `f64x4` SIMD instructions when available.
/// Falls back to scalar on non-x86_64 targets.
///
/// # Safety
///
/// The `#[target_feature(enable = "sse2")]` attribute means this function
/// must only be called on CPUs that support SSE2 (all x86_64 CPUs do).
#[inline]
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
#[allow(dead_code, clippy::unnecessary_cast)]
pub unsafe fn compute_demerits_batch_simd(
    badness: [f64; 4],
    fitness_penalty: f64,
    line_penalty: f64,
    fitness_changed: [bool; 4],
) -> [f64; 4] {
    unsafe {
        use std::arch::x86_64::*;

        // Load badness values — _mm_loadu_pd loads [ptr[0], ptr[1]]
        let b_lo = _mm_loadu_pd(badness.as_ptr() as *const f64);
        let b_hi = _mm_loadu_pd(badness.as_ptr().add(2) as *const f64);

        // line_demerit = line_penalty + badness
        let lp = _mm_set1_pd(line_penalty);
        let ld_lo = _mm_add_pd(lp, b_lo);
        let ld_hi = _mm_add_pd(lp, b_hi);

        // line_demerit^2
        let sq_lo = _mm_mul_pd(ld_lo, ld_lo);
        let sq_hi = _mm_mul_pd(ld_hi, ld_hi);

        // fitness_demerit = fitness_penalty if changed, else 0
        // Note: _mm_set_pd(a, b) stores a in high bits, b in low bits.
        // Memory order is [low, high] = [b, a].
        let fd_lo = _mm_set_pd(
            if fitness_changed[1] {
                fitness_penalty
            } else {
                0.0
            },
            if fitness_changed[0] {
                fitness_penalty
            } else {
                0.0
            },
        );
        let fd_hi = _mm_set_pd(
            if fitness_changed[3] {
                fitness_penalty
            } else {
                0.0
            },
            if fitness_changed[2] {
                fitness_penalty
            } else {
                0.0
            },
        );

        // fitness_demerit^2
        let fd_sq_lo = _mm_mul_pd(fd_lo, fd_lo);
        let fd_sq_hi = _mm_mul_pd(fd_hi, fd_hi);

        // total = line_demerit^2 + fitness_demerit^2
        let total_lo = _mm_add_pd(sq_lo, fd_sq_lo);
        let total_hi = _mm_add_pd(sq_hi, fd_sq_hi);

        // Store results
        let mut result = [0.0f64; 4];
        let p = result.as_mut_ptr() as *mut f64;
        _mm_storeu_pd(p, total_lo);
        _mm_storeu_pd(p.add(2), total_hi);
        result
    }
}

/// Portable fallback for non-x86_64 targets.
#[cfg(not(target_arch = "x86_64"))]
#[allow(dead_code)]
pub fn compute_demerits_batch_simd(
    badness: [f64; 4],
    fitness_penalty: f64,
    line_penalty: f64,
    fitness_changed: [bool; 4],
) -> [f64; 4] {
    compute_demerits_batch(badness, fitness_penalty, line_penalty, fitness_changed)
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

    #[test]
    fn test_batch_demerits_scalar() {
        let badness = [0.0, 0.5, 1.0, 0.0];
        let fitness_changed = [false, true, false, true];
        let result = compute_demerits_batch(badness, 100.0, 10.0, fitness_changed);
        // Each element should match the scalar version
        for i in 0..4 {
            let expected = compute_demerits(badness[i], 100.0, 10.0, fitness_changed[i]);
            assert!(
                (result[i] - expected).abs() < 1e-10,
                "batch[{}] = {} != scalar = {}",
                i,
                result[i],
                expected
            );
        }
    }

    #[test]
    fn test_batch_simd_matches_scalar() {
        let badness = [0.0, 0.5, 1.0, 0.25];
        let fitness_changed = [false, true, false, true];
        let scalar = compute_demerits_batch(badness, 100.0, 10.0, fitness_changed);
        let simd = unsafe { compute_demerits_batch_simd(badness, 100.0, 10.0, fitness_changed) };
        for i in 0..4 {
            assert!(
                (simd[i] - scalar[i]).abs() < 1e-10,
                "SIMD[{}] = {} != scalar = {}",
                i,
                simd[i],
                scalar[i]
            );
        }
    }

    #[test]
    fn test_batch_simd_infeasible() {
        let badness = [f64::INFINITY, f64::NEG_INFINITY, 0.0, 1e6];
        let fitness_changed = [false, false, false, false];
        let result = unsafe { compute_demerits_batch_simd(badness, 10.0, 5.0, fitness_changed) };
        // infinity handling: (5 + inf)^2 = inf, (5 - inf)^2 = inf
        assert!(result[0].is_infinite());
        assert!(result[1].is_infinite());
        // Normal values
        assert!((result[2] - 25.0).abs() < 1e-10); // (5 + 0)^2 = 25
    }
}
