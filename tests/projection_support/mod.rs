//! Helpers shared by the camera and engine tests. Each test crate uses a subset.
#![allow(dead_code, unused_macros)]

use omath::Real;

/// `EXPECT_NEAR`: compares in `f64`, whatever the operand types.
macro_rules! expect_near {
    ($actual:expr, $expected:expr, $tolerance:expr) => {{
        let actual = f64::from($actual);
        let expected = f64::from($expected);
        let tolerance = f64::from($tolerance);
        assert!(
            (actual - expected).abs() <= tolerance,
            "{actual} is not within {tolerance} of {expected}: {} vs {}",
            stringify!($actual),
            stringify!($expected),
        );
    }};
}

/// `EXPECT_FLOAT_EQ`: within 4 ULPs.
macro_rules! expect_float_eq {
    ($actual:expr, $expected:expr) => {{
        let (actual, expected): (f32, f32) = ($actual, $expected);
        assert!(
            $crate::projection_support::ulps_f32(actual, expected) <= 4,
            "{actual} is not within 4 ULPs of {expected}",
        );
    }};
}

/// `EXPECT_DOUBLE_EQ`: within 4 ULPs.
macro_rules! expect_double_eq {
    ($actual:expr, $expected:expr) => {{
        let (actual, expected): (f64, f64) = ($actual, $expected);
        assert!(
            $crate::projection_support::ulps_f64(actual, expected) <= 4,
            "{actual} is not within 4 ULPs of {expected}",
        );
    }};
}

fn biased_f32(value: f32) -> i64 {
    let bits = i64::from(value.abs().to_bits());
    if value.is_sign_negative() { -bits } else { bits }
}

fn biased_f64(value: f64) -> i128 {
    let bits = i128::from(value.abs().to_bits());
    if value.is_sign_negative() { -bits } else { bits }
}

pub fn ulps_f32(a: f32, b: f32) -> u64 {
    if a.is_nan() || b.is_nan() { u64::MAX } else { biased_f32(a).abs_diff(biased_f32(b)) }
}

pub fn ulps_f64(a: f64, b: f64) -> u128 {
    if a.is_nan() || b.is_nan() { u128::MAX } else { biased_f64(a).abs_diff(biased_f64(b)) }
}

/// `std::remainder`: the result lies in `[-divisor / 2, divisor / 2]`.
pub fn remainder(value: f64, divisor: f64) -> f64 {
    value - divisor * (value / divisor).round_ties_even()
}

/// A small deterministic generator, standing in for `std::mt19937`. The C++ tests only depend on the range of the values.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    /// A value in `[range.0, range.1)`.
    pub fn uniform<T: Real>(&mut self, range: (T, T)) -> T {
        let unit = (self.next() >> 11) as f64 / (1_u64 << 53) as f64;
        T::from_f64(range.0.to_f64() + unit * (range.1.to_f64() - range.0.to_f64()))
    }
}
