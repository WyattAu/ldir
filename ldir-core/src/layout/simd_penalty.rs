//! SIMD-accelerated penalty evaluation for Knuth-Plass line breaking.
//! Uses portable SIMD via raw intrinsics (x86_64 AVX2 when available).
//! Falls back to scalar code when SIMD is not available.

#![allow(unsafe_code)]

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// SIMD configuration for penalty evaluation
pub struct SimdConfig {
    /// Whether to use the AVX2 path when available.
    pub use_simd: bool,
}

#[allow(clippy::derivable_impls)] // cfg(target_arch) makes derive impossible
impl Default for SimdConfig {
    fn default() -> Self {
        Self {
            #[cfg(target_arch = "x86_64")]
            use_simd: is_x86_avx2_available(),
            #[cfg(not(target_arch = "x86_64"))]
            use_simd: false,
        }
    }
}

#[cfg(target_arch = "x86_64")]
fn is_x86_avx2_available() -> bool {
    is_x86_feature_detected!("avx2")
}

/// Evaluate demerits for a batch of break points using SIMD.
///
/// For each candidate break point, demerit = line_penalty + (badness * badness * fitness_penalty).
/// Values < 0 are clamped to infinity (flag for suboptimal breaks).
///
/// This is the inner loop of Knuth-Plass and benefits heavily from SIMD.
pub fn evaluate_demerits_batch(
    breaks: &[BreakPoint],
    line_penalty: f32,
    fitness_penalty: f32,
    config: &SimdConfig,
) -> Vec<f32> {
    if !config.use_simd || breaks.is_empty() {
        return evaluate_demerits_scalar(breaks, line_penalty, fitness_penalty);
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            // SAFETY: AVX2 availability is checked above via is_x86_feature_detected!.
            return unsafe { evaluate_demerits_avx2(breaks, line_penalty, fitness_penalty) };
        }
    }

    evaluate_demerits_scalar(breaks, line_penalty, fitness_penalty)
}

/// Scalar fallback for demerits evaluation.
fn evaluate_demerits_scalar(
    breaks: &[BreakPoint],
    line_penalty: f32,
    fitness_penalty: f32,
) -> Vec<f32> {
    breaks
        .iter()
        .map(|bp| {
            let badness = bp.badness;
            let demerit = line_penalty + badness * badness * fitness_penalty;
            if demerit < 0.0 {
                f32::INFINITY
            } else {
                demerit
            }
        })
        .collect()
}

/// A candidate break point for line breaking.
#[derive(Debug, Clone, Copy)]
pub struct BreakPoint {
    /// Index of the potential break in the item list.
    pub position: usize,
    /// Fitness class of the line ending at this break.
    pub fitness: f32,
    /// Badness of the line ending at this break.
    pub badness: f32,
    /// Accumulated width of content up to this break.
    pub total_width: f32,
    /// Width of the shortest line considered so far.
    pub shortest_line: f32,
}

/// AVX2 implementation: process 8 f32 values simultaneously.
///
/// # Safety
///
/// Requires AVX2 support. Caller must verify `is_x86_feature_detected!("avx2")`
/// before calling. The `#[target_feature(enable = "avx2")]` attribute ensures the
/// compiler emits AVX2 instructions. All pointer accesses are within bounds of
/// stack-allocated fixed-size arrays or the input slice.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn evaluate_demerits_avx2(
    breaks: &[BreakPoint],
    line_penalty: f32,
    fitness_penalty: f32,
) -> Vec<f32> {
    // SAFETY: AVX2 availability verified by caller via is_x86_feature_detected!("avx2").
    // All pointer accesses are within bounds of stack-allocated fixed-size arrays
    // or within the input slice. _mm256_{loadu,storeu}_ps handle unaligned access.
    unsafe {
        let n = breaks.len();
        let mut results = Vec::with_capacity(n);

        let lp = _mm256_set1_ps(line_penalty);
        let fp = _mm256_set1_ps(fitness_penalty);
        let inf = _mm256_set1_ps(f32::INFINITY);
        let zero = _mm256_setzero_ps();

        let chunks = n / 8;
        let mut badness_buf = [0.0f32; 8];

        for chunk in 0..chunks {
            let base = chunk * 8;

            for i in 0..8 {
                badness_buf[i] = breaks[base + i].badness;
            }
            let badness = _mm256_loadu_ps(badness_buf.as_ptr());

            let badness_sq = _mm256_mul_ps(badness, badness);
            let demerit = _mm256_add_ps(lp, _mm256_mul_ps(badness_sq, fp));

            let mask = _mm256_cmp_ps(demerit, zero, _CMP_LT_OQ);
            let clamped = _mm256_blendv_ps(demerit, inf, mask);

            let mut result_buf = [0.0f32; 8];
            _mm256_storeu_ps(result_buf.as_mut_ptr(), clamped);
            results.extend_from_slice(&result_buf);
        }

        for bp in &breaks[(chunks * 8)..] {
            let badness = bp.badness;
            let demerit = line_penalty + badness * badness * fitness_penalty;
            results.push(if demerit < 0.0 {
                f32::INFINITY
            } else {
                demerit
            });
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_break(badness: f32) -> BreakPoint {
        BreakPoint {
            position: 0,
            fitness: 1.0,
            badness,
            total_width: 100.0,
            shortest_line: 100.0,
        }
    }

    #[test]
    fn test_scalar_demerits() {
        let config = SimdConfig { use_simd: false };
        let breaks = vec![make_break(10.0), make_break(0.0), make_break(5.0)];
        let results = evaluate_demerits_batch(&breaks, 1.0, 2.0, &config);
        assert_eq!(results.len(), 3);
        // demerit = 1.0 + badness^2 * 2.0
        assert!((results[0] - 201.0).abs() < 0.01); // 1 + 100*2 = 201
        assert!((results[1] - 1.0).abs() < 0.01); // 1 + 0*2 = 1
        assert!((results[2] - 51.0).abs() < 0.01); // 1 + 25*2 = 51
    }

    #[test]
    fn test_batch_basic() {
        let config = SimdConfig::default();
        let breaks: Vec<BreakPoint> = (0..16).map(|i| make_break(i as f32)).collect();
        let results = evaluate_demerits_batch(&breaks, 1.0, 1.0, &config);
        assert_eq!(results.len(), 16);
        for (i, &r) in results.iter().enumerate() {
            let expected = 1.0 + (i as f32) * (i as f32) * 1.0;
            assert!(
                (r - expected).abs() < 1.0,
                "break {i}: got {r}, expected {expected}"
            );
        }
    }

    #[test]
    fn test_negative_demerit_becomes_inf() {
        let config = SimdConfig { use_simd: false };
        // Use large badness with negative penalty to force negative demerit
        let breaks = vec![make_break(0.0)];
        let results = evaluate_demerits_batch(&breaks, -1.0, 0.0, &config);
        // demerit = -1.0 + 0*0*0.0 = -1.0 < 0 => INFINITY
        assert!(results[0].is_infinite());
    }

    #[test]
    fn test_empty_breaks() {
        let config = SimdConfig::default();
        let results = evaluate_demerits_batch(&[], 1.0, 1.0, &config);
        assert!(results.is_empty());
    }

    #[test]
    fn test_config_default() {
        let config = SimdConfig::default();
        // On x86_64 with AVX2 this should be true, otherwise false
        #[cfg(target_arch = "x86_64")]
        assert_eq!(config.use_simd, is_x86_feature_detected!("avx2"));
        #[cfg(not(target_arch = "x86_64"))]
        assert!(!config.use_simd);
    }

    #[test]
    fn test_manifest_parse_validate() {
        use crate::wasm_plugins::manifest::{ManifestError, parse_manifest, validate_manifest};

        let json = r#"{
            "name": "simd-test",
            "version": "1.0.0",
            "description": "test",
            "author": "test",
            "license": "MIT",
            "capabilities": [{"kind": "ir_transformer", "description": "t", "file_extensions": [], "mime_types": []}],
            "resource_limits": {"max_fuel": 1, "max_memory_mb": 1, "max_time_ms": 1, "max_output_kb": 1}
        }"#;

        let m = parse_manifest(json).expect("parse ok");
        validate_manifest(&m).expect("valid");

        let bad = parse_manifest("{}").expect_err("empty should fail parse");
        assert!(matches!(bad, ManifestError::ParseError(_)));
    }
}
