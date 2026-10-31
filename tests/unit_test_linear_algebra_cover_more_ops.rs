mod common;

use common::*;
use omath::{Mat, Vector3, Vector4};

#[test]
fn in_place_scalar_operators() {
    let mut v = Vector3::<f32>::new(1.0f32, 2.0f32, 3.0f32);

    v += 1.0f32;
    expect_float_eq(v.x, 2.0f32);
    expect_float_eq(v.y, 3.0f32);
    expect_float_eq(v.z, 4.0f32);

    v /= 2.0f32;
    expect_float_eq(v.x, 1.0f32);
    expect_float_eq(v.y, 1.5f32);
    expect_float_eq(v.z, 2.0f32);

    v -= 0.5f32;
    expect_float_eq(v.x, 0.5f32);
    expect_float_eq(v.y, 1.0f32);
    expect_float_eq(v.z, 1.5f32);
}

#[test]
fn element_wise_mul_div() {
    let a = Vector4::<f32>::new(2.0f32, 4.0f32, 6.0f32, 8.0f32);
    let b = Vector4::<f32>::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);

    let m = a * b;
    expect_float_eq(m.x, 2.0f32);
    expect_float_eq(m.y, 8.0f32);
    expect_float_eq(m.z, 18.0f32);
    expect_float_eq(m.w, 32.0f32);

    let d = a / b;
    expect_float_eq(d.x, 2.0f32);
    expect_float_eq(d.y, 2.0f32);
    expect_float_eq(d.z, 2.0f32);
    expect_float_eq(d.w, 2.0f32);
}

#[test]
fn invalid_initializer_lists() {
    // from_rows checks its shape at compile time; from_slice is the runtime path that can still get a bad shape.
    // Wrong number of rows
    assert!(Mat::<2, 2, f32>::from_slice(&[1.0f32, 2.0f32]).is_none());

    // Row with wrong number of columns
    assert!(Mat::<2, 2, f32>::from_slice(&[1.0f32, 2.0f32, 1.0f32]).is_none());
}
