//! 26.6 fixed-point arithmetic for L-IR geometry (AX-LIR-001).
//!
//! 26.6 format: 26 integer bits + 6 fractional bits.
//! Range: [-524288.0, 524287.9921875].
//! Precision: 1 ULP = 1/64 ≈ 0.015625 scaled points.
//!
//! This is a self-contained copy of the type defined in `ldir_core::fp266`,
//! required here because `ldir-core` depends on `ldir-ir` (preventing a
//! reverse dependency). The two definitions are structurally identical.
//!
//! ## References
//!
//! - YP-NUMERICAL-FIXEDPOINT-001: Full theoretical treatment
//! - AX-LIR-001: All L-IR geometry is 26.6 fixed-point
//! - FreeType: Uses 26.6 for glyph coordinates

use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

/// The number of fractional bits in the 26.6 format.
pub const FRACTIONAL_BITS: u32 = 6;

/// Scale factor: 2^6 = 64.
pub const SCALE: i64 = 1 << FRACTIONAL_BITS;

/// Minimum representable raw value.
pub const MIN_RAW: i64 = i32::MIN as i64 * SCALE;

/// Maximum representable raw value.
pub const MAX_RAW: i64 = i32::MAX as i64 * SCALE + (SCALE - 1);

/// A 26.6 fixed-point number stored as an i64.
///
/// Internal representation is `value * 64`, where `value` is the real number
/// being represented.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
#[repr(transparent)]
#[doc(alias = "FixedPoint")]
#[doc(alias = "Fixed26_6")]
pub struct Fp266 {
    raw: i64,
}

impl Fp266 {
    /// Zero in 26.6 format.
    pub const ZERO: Self = Self { raw: 0 };
    /// One in 26.6 format.
    pub const ONE: Self = Self { raw: SCALE };
    /// Half (0.5) in 26.6 format.
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
    #[inline]
    pub const fn from_frac(num: i32, den: i32) -> Self {
        Self {
            raw: ((num as i64) * SCALE) / (den as i64),
        }
    }

    /// Quantize from f64: `round(value * 64)`.
    #[inline]
    pub fn from_f64(value: f64) -> Self {
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

    /// Multiply two fp26_6 values using 128-bit intermediate.
    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn mul(self, other: Self) -> Self {
        let product = (self.raw as i128) * (other.raw as i128);
        let shifted = (product + (1i128 << (FRACTIONAL_BITS - 1))) >> FRACTIONAL_BITS;
        Self::saturating(shifted as i64)
    }

    /// Absolute value.
    #[inline]
    pub fn abs(self) -> Self {
        Self {
            raw: self.raw.abs(),
        }
    }

    /// Check if value is zero.
    #[inline]
    pub fn is_zero(self) -> bool {
        self.raw == 0
    }
}

impl Add for Fp266 {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Self::saturating(self.raw + other.raw)
    }
}

impl Sub for Fp266 {
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self {
        Self::saturating(self.raw - other.raw)
    }
}

impl Neg for Fp266 {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        Self { raw: -self.raw }
    }
}

impl Mul<i32> for Fp266 {
    type Output = Self;

    #[inline]
    fn mul(self, other: i32) -> Self {
        Self::saturating(self.raw * (other as i64))
    }
}

impl AddAssign for Fp266 {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl SubAssign for Fp266 {
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl std::fmt::Display for Fp266 {
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
        assert_eq!(Fp266::from_int(72).raw(), 4608);
    }

    #[test]
    fn test_from_frac() {
        assert_eq!(Fp266::from_frac(1, 2).raw(), 32);
        assert_eq!(Fp266::from_frac(1, 4).raw(), 16);
        assert_eq!(Fp266::from_frac(3, 4).raw(), 48);
    }

    #[test]
    fn test_from_f64() {
        assert_eq!(Fp266::from_f64(1.0).raw(), 64);
        assert_eq!(Fp266::from_f64(0.5).raw(), 32);
        assert_eq!(Fp266::from_f64(0.0).raw(), 0);
        assert_eq!(Fp266::from_f64(72.0).raw(), 4608);
    }

    #[test]
    fn test_add() {
        let a = Fp266::from_int(1);
        let b = Fp266::from_frac(1, 2);
        let sum = a + b;
        assert_eq!(sum.raw(), 96);
        assert!((sum.to_f64() - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_sub() {
        let a = Fp266::from_int(3);
        let b = Fp266::from_int(1);
        assert_eq!((a - b).to_f64(), 2.0);
    }

    #[test]
    fn test_mul() {
        let a = Fp266::from_int(3);
        let b = Fp266::from_frac(1, 2);
        let product = a.mul(b);
        assert!((product.to_f64() - 1.5).abs() < 0.001);
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
    fn test_saturation() {
        let max = Fp266::from_raw(MAX_RAW);
        let overflow = max + Fp266::from_int(1);
        assert_eq!(overflow.raw(), MAX_RAW);

        let min = Fp266::from_raw(MIN_RAW);
        let underflow = min - Fp266::from_int(1);
        assert_eq!(underflow.raw(), MIN_RAW);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Fp266::from_int(12)), "12");
        assert_eq!(format!("{}", Fp266::from_frac(1, 2)), "0.5");
    }
}
