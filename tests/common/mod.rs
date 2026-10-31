//! gtest-equivalent assertions shared by the ported C++ tests.
#![allow(dead_code)]

use omath::{Mat, StoreOrder};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Distance in units in the last place, the way gtest's `FloatingPoint::AlmostEquals` measures it.
fn ulps_f32(a: f32, b: f32) -> u32 {
    let biased = |bits: u32| {
        if bits & 0x8000_0000 != 0 {
            (!bits).wrapping_add(1)
        } else {
            bits | 0x8000_0000
        }
    };
    biased(a.to_bits()).abs_diff(biased(b.to_bits()))
}

fn ulps_f64(a: f64, b: f64) -> u64 {
    let biased = |bits: u64| {
        if bits & (1 << 63) != 0 {
            (!bits).wrapping_add(1)
        } else {
            bits | (1 << 63)
        }
    };
    biased(a.to_bits()).abs_diff(biased(b.to_bits()))
}

/// `EXPECT_FLOAT_EQ`: within 4 ULPs.
#[track_caller]
pub fn expect_float_eq(actual: f32, expected: f32) {
    assert!(
        !actual.is_nan() && !expected.is_nan() && ulps_f32(actual, expected) <= 4,
        "{actual} is not within 4 ULPs of {expected}"
    );
}

/// `EXPECT_DOUBLE_EQ`: within 4 ULPs.
#[track_caller]
pub fn expect_double_eq(actual: f64, expected: f64) {
    assert!(
        !actual.is_nan() && !expected.is_nan() && ulps_f64(actual, expected) <= 4,
        "{actual} is not within 4 ULPs of {expected}"
    );
}

/// `EXPECT_NEAR`: gtest widens its arguments to `double`, so the difference is taken in `f64` here too.
#[track_caller]
pub fn expect_near(actual: impl Into<f64>, expected: impl Into<f64>, tolerance: impl Into<f64>) {
    let (actual, expected, tolerance) = (actual.into(), expected.into(), tolerance.into());
    assert!(
        (actual - expected).abs() <= tolerance,
        "|{actual} - {expected}| > {tolerance}"
    );
}

pub fn hash_of<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

pub fn row_count<const R: usize, const C: usize, T: Copy, S: StoreOrder>(
    _: &Mat<R, C, T, S>,
) -> usize {
    R
}

pub fn columns_count<const R: usize, const C: usize, T: Copy, S: StoreOrder>(
    _: &Mat<R, C, T, S>,
) -> usize {
    C
}
