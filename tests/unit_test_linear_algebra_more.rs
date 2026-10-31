mod common;

use common::*;
use omath::{Triangle, Vector3, Vector4};
use std::cmp::Ordering;

#[test]
fn vector3_edge_cases() {
    let zero = Vector3::<f32>::new(0.0f32, 0.0f32, 0.0f32);
    let v = Vector3::<f32>::new(1.0f32, 0.0f32, 0.0f32);

    // angle_between should be unexpected when one vector has zero length
    let angle = zero.angle_between(v);
    assert!(angle.is_err());

    // normalized of zero should return zero
    let nz = zero.normalized();
    assert_eq!(nz.x, 0.0f32);
    assert_eq!(nz.y, 0.0f32);
    assert_eq!(nz.z, 0.0f32);

    // perpendicular case: x-axis and y-axis
    let x = Vector3::<f32>::new(1.0f32, 0.0f32, 0.0f32);
    let y = Vector3::<f32>::new(0.0f32, 1.0f32, 0.0f32);
    assert!(x.is_perpendicular(y, 0.0001f32));
}

#[test]
fn triangle_rectangular_and_degenerate() {
    let v1 = Vector3::<f32>::new(0.0f32, 0.0f32, 0.0f32);
    let v2 = Vector3::<f32>::new(3.0f32, 0.0f32, 0.0f32);
    let v3 = Vector3::<f32>::new(3.0f32, 4.0f32, 0.0f32); // 3-4-5 triangle, rectangular at v2

    let t = Triangle::new(v1, v2, v3);

    expect_near(t.side_a_length(), 3.0f32, 1e-6f32);
    expect_near(t.side_b_length(), 4.0f32, 1e-6f32);
    expect_near(t.hypot(), 5.0f32, 1e-6f32);
    assert!(t.is_rectangular());

    // Degenerate: all points same
    let d = Triangle::new(v1, v1, v1);
    expect_near(d.side_a_length(), 0.0f32, 1e-6f32);
    expect_near(d.side_b_length(), 0.0f32, 1e-6f32);
    expect_near(d.hypot(), 0.0f32, 1e-6f32);
}

#[test]
fn vector4_clamp_and_comparisons() {
    let mut v = Vector4::<f32>::new(10.0f32, -20.0f32, 30.0f32, -40.0f32);
    let s = v.sum();
    expect_near(s, -20.0f32, 1e-6f32);

    v = v.clamped(-10.0f32, 10.0f32);
    assert!(v.x <= 10.0f32);
    assert!(v.x >= -10.0f32);
    assert!(v.y <= 10.0f32);
    assert!(v.y >= -10.0f32);

    let a = Vector4::<f32>::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    let b = Vector4::<f32>::new(2.0f32, 2.0f32, 2.0f32, 2.0f32);
    assert!(
        a.cmp_length(b).is_some_and(Ordering::is_lt)
            || a.cmp_length(b).is_some_and(Ordering::is_gt)
            || a == b
    ); // just exercise comparisons
}
