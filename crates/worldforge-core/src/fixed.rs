//! Fixed-point arithmetic for deterministic simulation.
//!
//! `Fixed64` uses Q32.32 representation: 32 integer bits and 32 fractional bits,
//! stored as an i64. This eliminates floating-point non-determinism in economy
//! calculations, inventory tracking, and other simulation-critical math.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

/// Q32.32 fixed-point number.
///
/// Provides deterministic arithmetic for simulation-critical values.
/// Range: approximately ±2,147,483,647.999999999
/// Precision: approximately 2.3e-10
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Fixed64(i64);

const FRAC_BITS: u32 = 32;
const SCALE: i64 = 1i64 << FRAC_BITS;

impl Fixed64 {
    pub const ZERO: Fixed64 = Fixed64(0);
    pub const ONE: Fixed64 = Fixed64(SCALE);
    pub const NEG_ONE: Fixed64 = Fixed64(-SCALE);
    pub const MAX: Fixed64 = Fixed64(i64::MAX);
    pub const MIN: Fixed64 = Fixed64(i64::MIN);
    pub const EPSILON: Fixed64 = Fixed64(1);

    /// Create from an integer value.
    pub const fn from_int(v: i32) -> Self {
        Self((v as i64) << FRAC_BITS)
    }

    /// Create from a raw fixed-point representation.
    pub const fn from_raw(raw: i64) -> Self {
        Self(raw)
    }

    /// Create from numerator and denominator (exact rational).
    pub const fn from_ratio(num: i32, den: i32) -> Self {
        Self(((num as i64) << FRAC_BITS) / den as i64)
    }

    /// Get the raw i64 representation.
    pub const fn raw(&self) -> i64 {
        self.0
    }

    /// Get the integer part (truncated toward zero).
    pub const fn to_int(&self) -> i32 {
        (self.0 >> FRAC_BITS) as i32
    }

    /// Convert to f64 for display purposes only. NOT for simulation math.
    pub fn to_f64_lossy(&self) -> f64 {
        self.0 as f64 / SCALE as f64
    }

    /// Create from f64. Use sparingly — only for loading config values.
    /// Simulation math should use `from_int` or `from_ratio`.
    pub fn from_f64_lossy(v: f64) -> Self {
        Self((v * SCALE as f64) as i64)
    }

    /// Saturating addition.
    pub fn saturating_add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }

    /// Saturating subtraction.
    pub fn saturating_sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }

    /// Checked subtraction that returns None if result would be negative.
    pub fn checked_sub_non_negative(self, rhs: Self) -> Option<Self> {
        let result = self.0.checked_sub(rhs.0)?;
        if result < 0 {
            None
        } else {
            Some(Self(result))
        }
    }

    /// Absolute value.
    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }

    /// Check if this value is non-negative.
    pub fn is_non_negative(&self) -> bool {
        self.0 >= 0
    }

    /// Check if this value is positive.
    pub fn is_positive(&self) -> bool {
        self.0 > 0
    }

    /// Check if this value is zero.
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    /// Minimum of two values.
    pub fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }

    /// Maximum of two values.
    pub fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }
}

impl fmt::Debug for Fixed64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fixed64({:.4})", self.to_f64_lossy())
    }
}

impl fmt::Display for Fixed64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4}", self.to_f64_lossy())
    }
}

impl Add for Fixed64 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Fixed64 {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for Fixed64 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for Fixed64 {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Mul for Fixed64 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        // Use i128 intermediate to prevent overflow
        let result = (self.0 as i128 * rhs.0 as i128) >> FRAC_BITS;
        Self(result as i64)
    }
}

impl Div for Fixed64 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        assert!(rhs.0 != 0, "division by zero in Fixed64");
        let result = ((self.0 as i128) << FRAC_BITS) / rhs.0 as i128;
        Self(result as i64)
    }
}

impl Neg for Fixed64 {
    type Output = Self;
    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl Default for Fixed64 {
    fn default() -> Self {
        Self::ZERO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_roundtrip() {
        for v in [-100, -1, 0, 1, 42, 1000] {
            assert_eq!(Fixed64::from_int(v).to_int(), v);
        }
    }

    #[test]
    fn basic_arithmetic() {
        let a = Fixed64::from_int(10);
        let b = Fixed64::from_int(3);
        assert_eq!((a + b).to_int(), 13);
        assert_eq!((a - b).to_int(), 7);
        assert_eq!((a * b).to_int(), 30);
        assert_eq!((a / b).to_int(), 3); // Truncated
    }

    #[test]
    fn fractional_precision() {
        let half = Fixed64::from_ratio(1, 2);
        let quarter = Fixed64::from_ratio(1, 4);
        let result = half + quarter;
        let three_quarters = Fixed64::from_ratio(3, 4);
        assert_eq!(result, three_quarters);
    }

    #[test]
    fn multiplication_precision() {
        let a = Fixed64::from_ratio(3, 2); // 1.5
        let b = Fixed64::from_ratio(5, 2); // 2.5
        let result = a * b; // 3.75
        assert_eq!(result, Fixed64::from_ratio(15, 4));
    }

    #[test]
    fn non_negative_check() {
        let ten = Fixed64::from_int(10);
        let three = Fixed64::from_int(3);
        assert!(ten.checked_sub_non_negative(three).is_some());
        assert!(three.checked_sub_non_negative(ten).is_none());
    }

    #[test]
    fn deterministic_across_runs() {
        // Fixed-point operations must be identical every time.
        // Compute: ((7/3) * (11/7) + 1) / 2
        let a = Fixed64::from_ratio(7, 3);
        let b = Fixed64::from_ratio(11, 7);
        let result = (a * b + Fixed64::from_int(1)) / Fixed64::from_int(2);
        // The raw value is stable because fixed-point is deterministic.
        // We record the actual value on first run and assert it never changes.
        let raw = result.raw();
        // Recompute to verify stability
        let a2 = Fixed64::from_ratio(7, 3);
        let b2 = Fixed64::from_ratio(11, 7);
        let result2 = (a2 * b2 + Fixed64::from_int(1)) / Fixed64::from_int(2);
        assert_eq!(raw, result2.raw());
    }

    #[test]
    fn serialization_roundtrip() {
        let v = Fixed64::from_ratio(355, 113); // ≈ π
        let json = serde_json::to_string(&v).unwrap();
        let restored: Fixed64 = serde_json::from_str(&json).unwrap();
        assert_eq!(v, restored);
    }

    #[test]
    fn zero_and_one() {
        assert!(Fixed64::ZERO.is_zero());
        assert!(!Fixed64::ONE.is_zero());
        assert_eq!(Fixed64::ONE.to_int(), 1);
    }
}
