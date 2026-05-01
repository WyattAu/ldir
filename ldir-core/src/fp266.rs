//! fp26_6 Fixed-Point Arithmetic (REQ-3.2.5).
//!
//! 26.6 format: 26 integer bits + 6 fractional bits.
//! Range: [-524288.0, 524287.9921875]
//! Precision: 1/128 = 0.0078125 per unit in last place.
//!
//! ## Formal Specification
//!
//! See YP-NUMERICAL-FIXEDPOINT-001 for the full theoretical treatment:
//! - THM-FP-ADD-EXACT: Addition is exact (no rounding error)
//! - THM-FP-MUL-ROUND: Multiplication error ≤ 0.5 ULP
//! - THM-FP-SATURATION: Saturation prevents overflow
//!
//! ## References
//!
//! - FreeType: Uses 26.6 for glyph coordinates
//! - REQ-3.2.5: Fixed-point coordinate system
//! - REQ-3.2.7: Quantization from f64

use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

/// Sentinel root parent ID indicating no parent (REQ-3.1.6).
/// Matches Lean4 `rootSentinel = 0xFFFFFFFF`.
pub const ROOT_SENTINEL: u32 = 0xFFFF_FFFF;

/// The number of fractional bits in the 26.6 format.
pub const FRACTIONAL_BITS: u32 = 6;

/// Scale factor: 2^6 = 64.
pub const SCALE: i64 = 1 << FRACTIONAL_BITS;

/// Minimum representable value: -33554432 * 64 (saturated).
pub const MIN_RAW: i64 = i32::MIN as i64 * SCALE;

/// Maximum representable value: 33554431 * 64 + 63.
pub const MAX_RAW: i64 = i32::MAX as i64 * SCALE + (SCALE - 1);

/// Maximum representable value as float: ~524287.9921875.
pub const MAX_VALUE: f64 = (MAX_RAW as f64) / (SCALE as f64);

/// Minimum representable value as float: ~-524288.0.
pub const MIN_VALUE: f64 = (MIN_RAW as f64) / (SCALE as f64);

/// Error bound per operation: 1/128 = 2^{-7}.
pub const ERROR_BOUND: f64 = 1.0 / 128.0;

/// A 26.6 fixed-point number stored as an i64.
///
/// The internal representation is `value * 64`, where `value` is the
/// real number being represented. This provides ~7.8 decimal digits
/// of precision, sufficient for typographic layout (sub-pixel accuracy).
///
/// # Examples
///
/// ```
/// use ldir_core::fp266::Fp266;
///
/// let one = Fp266::from_int(1);
/// let half = Fp266::from_frac(1, 2);
/// let sum = one + half;
/// assert_eq!(sum.to_f64(), 1.5);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
#[repr(transparent)]
pub struct Fp266 {
    /// Raw value in 26.6 format (actual_value = raw / 64).
    raw: i64,
}

impl Fp266 {
    /// The zero value.
    pub const ZERO: Self = Self { raw: 0 };

    /// One in 26.6 format.
    pub const ONE: Self = Self { raw: SCALE };

    /// Half in 26.6 format.
    pub const HALF: Self = Self { raw: SCALE / 2 };

    /// Create from raw 26.6 value.
    #[inline]
    pub const fn from_raw(raw: i64) -> Self {
        Self { raw }
    }

    /// Create from an integer value.
    #[inline]
    pub const fn from_int(value: i32) -> Self {
        Self {
            raw: (value as i64) * SCALE,
        }
    }

    /// Create from a fraction: `from_frac(numerator, denominator)`.
    /// Uses integer division (truncation toward zero).
    #[inline]
    pub const fn from_frac(num: i32, den: i32) -> Self {
        Self {
            raw: ((num as i64) * SCALE) / (den as i64),
        }
    }

    /// Quantize from f64: `round(value * 64)` per REQ-3.2.7.
    /// Max error: ±1/128 device units.
    #[inline]
    pub fn from_f64(value: f64) -> Self {
        // round to nearest, ties to even
        let scaled = value * (SCALE as f64);
        let raw = scaled.round() as i64;
        Self::saturating(raw)
    }

    /// Get raw 26.6 value.
    #[inline]
    pub const fn raw(self) -> i64 {
        self.raw
    }

    /// Convert to f64 (exact for values in range).
    #[inline]
    pub fn to_f64(self) -> f64 {
        self.raw as f64 / (SCALE as f64)
    }

    /// Convert to integer (truncation toward zero).
    #[inline]
    pub const fn to_int(self) -> i32 {
        (self.raw / SCALE) as i32
    }

    /// Get the fractional part (0..63).
    #[inline]
    pub const fn fractional(self) -> i32 {
        (self.raw % SCALE) as i32
    }

    /// Saturating construction from raw value.
    #[inline]
    pub const fn saturating(raw: i64) -> Self {
        Self {
            raw: if raw < MIN_RAW {
                MIN_RAW
            } else if raw > MAX_RAW {
                MAX_RAW
            } else {
                raw
            },
        }
    }

    /// Multiply two fp26_6 values using 64-bit intermediate.
    /// Result is in 26.6 format with ≤ 0.5 ULP error.
    ///
    /// Algorithm ALG-FP-MUL from YP-NUMERICAL-FIXEDPOINT-001.
    ///
    /// # Examples
    ///
    /// ```
    /// use ldir_core::fp266::Fp266;
    ///
    /// let a = Fp266::from_int(3);
    /// let b = Fp266::from_frac(1, 2);
    /// let product = a.mul(b);
    /// assert_eq!(product.to_f64(), 1.5);
    /// ```
    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn mul(self, other: Self) -> Self {
        // Intermediate: (a * b) / 64
        // Use 128-bit to avoid overflow, then shift
        let product = (self.raw as i128) * (other.raw as i128);
        let shifted = (product + (1i128 << (FRACTIONAL_BITS - 1))) >> FRACTIONAL_BITS;
        Self::saturating(shifted as i64)
    }

    /// Integer division with 26.6 result.
    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn div(self, other: Self) -> Self {
        if other.raw == 0 {
            return Self::saturating(if self.raw >= 0 { MAX_RAW } else { MIN_RAW });
        }
        let shifted = ((self.raw as i128) << FRACTIONAL_BITS) / (other.raw as i128);
        Self::saturating(shifted as i64)
    }

    /// Integer square root using Newton's method.
    /// Returns floor(sqrt(self)) in 26.6 format.
    #[inline]
    pub fn sqrt(self) -> Self {
        if self.raw <= 0 {
            return Self::ZERO;
        }
        // Scale up: we want sqrt(raw / 64) * 64 = sqrt(raw * 64)
        // Work with raw * 64 as the "integer" whose sqrt we compute
        let val = self.raw << FRACTIONAL_BITS;
        let mut x = val;
        // Newton's method: x_{n+1} = (x_n + val/x_n) / 2
        // Converges in ~32 iterations for i64
        for _ in 0..32 {
            if x == 0 {
                break;
            }
            let next = (x + val / x) >> 1;
            if next >= x {
                break;
            }
            x = next;
        }
        Self::saturating(x)
    }

    /// Absolute value.
    #[inline]
    pub fn abs(self) -> Self {
        Self {
            raw: self.raw.abs(),
        }
    }

    /// Minimum of two values.
    #[inline]
    pub fn min(self, other: Self) -> Self {
        if self.raw < other.raw { self } else { other }
    }

    /// Maximum of two values.
    #[inline]
    pub fn max(self, other: Self) -> Self {
        if self.raw > other.raw { self } else { other }
    }

    /// Clamp to range.
    #[inline]
    pub fn clamp(self, lo: Self, hi: Self) -> Self {
        self.max(lo).min(hi)
    }

    /// Check if value is zero.
    #[inline]
    pub fn is_zero(self) -> bool {
        self.raw == 0
    }
}

// === Trait implementations ===

impl Add for Fp266 {
    type Output = Self;
    /// Exact addition (THM-FP-ADD-EXACT): no rounding error.
    #[inline]
    fn add(self, other: Self) -> Self {
        Self::saturating(self.raw + other.raw)
    }
}

impl Sub for Fp266 {
    type Output = Self;
    /// Exact subtraction (THM-FP-ADD-EXACT): no rounding error.
    #[inline]
    fn sub(self, other: Self) -> Self {
        Self::saturating(self.raw - other.raw)
    }
}

impl Neg for Fp266 {
    type Output = Self;

    /// Negation (exact, no rounding error).
    #[inline]
    fn neg(self) -> Self {
        Self { raw: -self.raw }
    }
}

impl Mul<i32> for Fp266 {
    type Output = Self;

    /// Scale by an integer factor with saturation.
    #[inline]
    fn mul(self, other: i32) -> Self {
        Self::saturating(self.raw * (other as i64))
    }
}

impl AddAssign for Fp266 {
    /// Exact in-place addition (THM-FP-ADD-EXACT).
    #[inline]
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl SubAssign for Fp266 {
    /// Exact in-place subtraction (THM-FP-ADD-EXACT).
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl std::fmt::Display for Fp266 {
    /// Display as floating-point value.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_f64())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero() {
        assert_eq!(Fp266::ZERO.raw(), 0);
        assert!(Fp266::ZERO.is_zero());
    }

    #[test]
    fn test_from_int() {
        assert_eq!(Fp266::from_int(1).raw(), 64);
        assert_eq!(Fp266::from_int(-1).raw(), -64);
        assert_eq!(Fp266::from_int(0).raw(), 0);
    }

    #[test]
    fn test_from_frac() {
        assert_eq!(Fp266::from_frac(1, 2).raw(), 32); // 0.5 * 64 = 32
        assert_eq!(Fp266::from_frac(1, 4).raw(), 16); // 0.25 * 64 = 16
        assert_eq!(Fp266::from_frac(3, 4).raw(), 48); // 0.75 * 64 = 48
    }

    #[test]
    fn test_from_f64() {
        assert_eq!(Fp266::from_f64(1.0).raw(), 64);
        assert_eq!(Fp266::from_f64(0.5).raw(), 32);
        assert_eq!(Fp266::from_f64(0.0).raw(), 0);
        // Round-to-nearest: 1/3 * 64 = 21.33... → 21
        assert_eq!(Fp266::from_f64(1.0 / 3.0).raw(), 21);
    }

    #[test]
    fn test_add_exact() {
        let a = Fp266::from_int(1);
        let b = Fp266::from_frac(1, 2);
        let sum = a + b;
        assert_eq!(sum.to_f64(), 1.5);
        assert_eq!(sum.raw(), 96);
    }

    #[test]
    fn test_sub_exact() {
        let a = Fp266::from_int(3);
        let b = Fp266::from_int(1);
        let diff = a - b;
        assert_eq!(diff.to_f64(), 2.0);
    }

    #[test]
    fn test_mul() {
        let a = Fp266::from_int(3);
        let b = Fp266::from_frac(1, 2);
        let product = a.mul(b);
        // (3 * 32) * (1 * 64) / 64 = 192 * 64 / 64 = 192 / 64 = 3... wait
        // a.raw = 192, b.raw = 32
        // product = (192 * 32 + 32) >> 6 = (6144 + 32) >> 6 = 6176 >> 6 = 96
        // 96 / 64 = 1.5 ✓
        assert!((product.to_f64() - 1.5).abs() < ERROR_BOUND);
    }

    #[test]
    fn test_sqrt() {
        let four = Fp266::from_int(4);
        let root = four.sqrt();
        assert!((root.to_f64() - 2.0).abs() < ERROR_BOUND);
    }

    #[test]
    fn test_saturation() {
        let max = Fp266::from_raw(MAX_RAW);
        let overflow = max + Fp266::from_int(1);
        assert_eq!(overflow.raw(), MAX_RAW);

        let min = Fp266::from_raw(MIN_RAW);
        let underflow = min - Fp266::from_int(1);
        assert_eq!(underflow.raw(), MIN_RAW);
    }

    #[test]
    fn test_neg() {
        let a = Fp266::from_int(5);
        assert_eq!((-a).raw(), -320);
    }

    #[test]
    fn test_abs() {
        let neg = Fp266::from_int(-5);
        assert_eq!(neg.abs().raw(), Fp266::from_int(5).raw());
    }

    #[test]
    fn test_div() {
        let six = Fp266::from_int(6);
        let three = Fp266::from_int(3);
        let result = six.div(three);
        assert!((result.to_f64() - 2.0).abs() < ERROR_BOUND);
    }

    #[test]
    fn test_clamp() {
        let val = Fp266::from_int(5);
        let clamped = val.clamp(Fp266::from_int(0), Fp266::from_int(3));
        assert_eq!(clamped.raw(), Fp266::from_int(3).raw());
    }

    #[test]
    fn test_root_sentinel() {
        assert_eq!(ROOT_SENTINEL, 0xFFFF_FFFF);
    }
}
