mod common;

use common::*;
use omath::{Angle, Clamped0To180, Normalized0To360, angle_range};
use std::f32::consts::PI;

angle_range!(OffsetRange, 10.0, 20.0, Clamped);
angle_range!(PitchRange, -90.0, 90.0, Clamped);
angle_range!(TurnRange, -180.0, 180.0, Normalized);

type Deg = Angle<f32, Normalized0To360>;
type Fov = Angle<f32, Clamped0To180>;
type Offset = Angle<f32, OffsetRange>;
type Pitch = Angle<f32, PitchRange>;
type Turn = Angle<f32, TurnRange>;

const K_EPS: f32 = 1e-5f32;

fn close_to(actual: f32, expected: f32, epsilon: f32) -> bool {
    (actual - expected).abs() <= epsilon
}

#[test]
fn default_constructor_is_zero_degrees() {
    let a = Deg::default(); // default ctor
    expect_float_eq(a.as_degrees(), 0.0f32);
    expect_float_eq(a.as_degrees(), 0.0f32);
}

#[test]
fn default_constructor_applies_range_policy() {
    let a = Offset::default();
    expect_float_eq(a.as_degrees(), 10.0f32);
}

#[test]
fn from_degrees_normalized_wraps_above_max() {
    let a = Deg::from_degrees(370.0f32);
    expect_float_eq(a.as_degrees(), 10.0f32);
}

#[test]
fn from_degrees_normalized_wraps_below_min() {
    let a = Deg::from_degrees(-10.0f32);
    expect_float_eq(a.as_degrees(), 350.0f32);
}

#[test]
fn from_degrees_clamped_clamps_to_range() {
    let hi = Pitch::from_degrees(100.0f32);
    let lo = Pitch::from_degrees(-120.0f32);

    expect_float_eq(hi.as_degrees(), 90.0f32);
    expect_float_eq(lo.as_degrees(), -90.0f32);
}

#[test]
fn from_radians_and_as_radians() {
    let a = Deg::from_radians(PI);
    expect_float_eq(a.as_degrees(), 180.0f32);

    let b = Deg::from_degrees(180.0f32);
    expect_near(b.as_radians(), PI, 1e-6f32);
}

#[test]
fn from_inverse_trigonometric_functions() {
    expect_near(Pitch::from_asin(0.5f32).as_degrees(), 30.0f32, K_EPS);
    expect_near(Pitch::from_acos(0.5f32).as_degrees(), 60.0f32, K_EPS);
    expect_near(Pitch::from_atan(1.0f32).as_degrees(), 45.0f32, K_EPS);
    expect_near(
        Turn::from_atan2(-1.0f32, -1.0f32).as_degrees(),
        -135.0f32,
        K_EPS,
    );
}

#[test]
fn unary_minus_normalized() {
    let a = Deg::from_degrees(30.0f32);
    let b = -a;
    expect_float_eq(b.as_degrees(), 330.0f32);
}

#[test]
fn dereference_returns_degrees() {
    // operator* has no Rust counterpart; as_degrees is the same accessor.
    let a = Deg::from_degrees(42.0f32);
    expect_float_eq(a.as_degrees(), 42.0f32);
}

#[test]
fn sin_cos_tan_cot_basic_cases() {
    let a0 = Deg::from_degrees(0.0f32);
    expect_near(a0.sin(), 0.0f32, K_EPS);
    expect_near(a0.cos(), 1.0f32, K_EPS);
    // cot(0) -> cos/sin -> div by 0: allow inf or nan
    let cot0: f32 = a0.cot();
    assert!((cot0).is_infinite() || (cot0).is_nan());

    let a45 = Deg::from_degrees(45.0f32);
    expect_near(a45.tan(), 1.0f32, 1e-4f32);
    expect_near(a45.cot(), 1.0f32, 1e-4f32);

    let a90 = Deg::from_degrees(90.0f32);
    expect_near(a90.sin(), 1.0f32, 1e-4f32);
    expect_near(a90.cos(), 0.0f32, 1e-4f32);
}

#[test]
fn plus_equals_normalized_wraps() {
    let mut a = Deg::from_degrees(350.0f32);
    a += Deg::from_degrees(20.0f32);
    expect_float_eq(a.as_degrees(), 10.0f32);
}

#[test]
fn minus_equals_normalized_wraps() {
    let mut a = Deg::from_degrees(10.0f32);
    a -= Deg::from_degrees(30.0f32);
    expect_float_eq(a.as_degrees(), 340.0f32);
}

#[test]
fn plus_equals_clamped_clamps() {
    let mut p = Pitch::from_degrees(80.0f32);
    p += Pitch::from_degrees(30.0f32);
    expect_float_eq(p.as_degrees(), 90.0f32);
}

#[test]
fn minus_equals_clamped_clamps() {
    let mut p = Pitch::from_degrees(-70.0f32);
    p -= Pitch::from_degrees(40.0f32);
    expect_float_eq(p.as_degrees(), -90.0f32);
}

#[test]
fn subtraction_clamped_non_symmetric_range() {
    let mut compound = Fov::from_degrees(90.0f32);
    compound -= Fov::from_degrees(10.0f32);
    expect_float_eq(compound.as_degrees(), 80.0f32);

    let binary = Fov::from_degrees(90.0f32) - Fov::from_degrees(10.0f32);
    expect_float_eq(binary.as_degrees(), 80.0f32);
}

#[test]
fn normalized_range_neg180_to180() {
    let a = Turn::from_degrees(190.0f32);
    let b = Turn::from_degrees(-190.0f32);

    expect_float_eq(a.as_degrees(), -170.0f32);
    expect_float_eq(b.as_degrees(), 170.0f32);
}

#[test]
fn comparisons_work_with_partial_ordering() {
    let a = Deg::from_degrees(10.0f32);
    let b = Deg::from_degrees(20.0f32);
    let c = Deg::from_degrees(10.0f32);

    assert!(a < b);
    assert!(b > a);
    assert!(a <= c);
    assert!(c >= a);
}

#[test]
fn formatter_prints_degrees_with_suffix() {
    let a = Deg::from_degrees(15.0f32);
    assert_eq!(format!("{}", a), "15deg");

    let b = Deg::from_degrees(10.5f32);
    assert_eq!(format!("{}", b), "10.5deg");

    let t = Turn::from_degrees(-170.0f32);
    assert_eq!(format!("{}", t), "-170deg");
}

#[test]
fn binary_plus_returns_wrapped_sum() {
    let a = Deg::from_degrees(350.0f32);
    let b = Deg::from_degrees(20.0f32);
    let c = a + b;
    expect_float_eq(c.as_degrees(), 10.0f32);
}

#[test]
fn binary_minus_returns_wrapped_diff() {
    let a = Deg::from_degrees(10.0f32);
    let b = Deg::from_degrees(30.0f32);
    let c = a - b;
    expect_float_eq(c.as_degrees(), 340.0f32);
}

// The file-scope static_asserts of the C++ test, as runtime checks. `Angle<int>` being rejected is enforced by the
// `Real` bound at compile time and has no runtime form.
#[test]
fn static_assert_trigonometry() {
    assert!(close_to(Pitch::from_degrees(0.0f32).sin(), 0.0f32, K_EPS));
    assert!(close_to(Pitch::from_degrees(0.0f32).cos(), 1.0f32, K_EPS));
    assert!(close_to(
        Pitch::from_degrees(45.0f32).tan(),
        1.0f32,
        1e-4f32
    ));
    assert!(close_to(
        Pitch::from_degrees(45.0f32).cot(),
        1.0f32,
        1e-4f32
    ));
    assert!(close_to(
        Pitch::from_asin(0.5f32).as_degrees(),
        30.0f32,
        K_EPS
    ));
    assert!(close_to(
        Pitch::from_acos(0.5f32).as_degrees(),
        60.0f32,
        K_EPS
    ));
    assert!(close_to(
        Pitch::from_atan(1.0f32).as_degrees(),
        45.0f32,
        K_EPS
    ));
    assert!(close_to(
        Turn::from_atan2(-1.0f32, -1.0f32).as_degrees(),
        -135.0f32,
        K_EPS
    ));
}
