mod common;

use common::*;
use omath::{Mat, Vector3, Vector4};
use std::cmp::Ordering;

#[test]
fn vector3_non_inlined_helpers() {
    let v = Vector3::<f32>::new(3.0f32, 4.0f32, 0.0f32);
    expect_float_eq(v.length(), 5.0f32);

    let vn = v.normalized();
    expect_near(vn.length(), 1.0f32, 1e-6f32);

    let zero = Vector3::<f32>::new(0.0f32, 0.0f32, 0.0f32);
    let ang = v.angle_between(zero);
    assert!(!(ang.is_ok()));

    let a = Vector3::<f32>::new(1.0f32, 0.0f32, 0.0f32);
    let b = Vector3::<f32>::new(0.0f32, 1.0f32, 0.0f32);
    assert!(a.is_perpendicular(b, 0.0001f32));
    assert!(!(a.is_perpendicular(a, 0.0001f32)));

    let tup = v.as_tuple();
    assert_eq!(tup.0, 3.0f32);
    assert_eq!(tup.1, 4.0f32);
    assert_eq!(tup.2, 0.0f32);

    assert!(a.point_to_same_direction(Vector3::<f32>::new(2.0f32, 0.0f32, 0.0f32)));

    // exercise hash specialization for Vector3<float>
    let _ = hash_of(&v);
}

#[test]
fn vector4_non_inlined_helpers() {
    let mut v = Vector4::<f32>::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    expect_float_eq(v.length(), v.length());
    expect_float_eq(v.sum(), v.sum());

    // clamp noinline should modify the vector
    v = v.clamped(0.0f32, 2.5f32);
    assert!(v.x >= 0.0f32);
    assert!(v.z <= 2.5f32);

    let shorter = Vector4::<f32>::new(0.1f32, 0.1f32, 0.1f32, 0.1f32);
    assert!(shorter.cmp_length(v).is_some_and(Ordering::is_lt));
    assert!(!(v.cmp_length(shorter).is_some_and(Ordering::is_lt)));
}

#[test]
fn mat_non_inlined_and_string_helpers() {
    let m = Mat::<2, 2, f32>::from_rows([[4.0f32, 7.0f32], [2.0f32, 6.0f32]]);
    expect_float_eq(m.determinant(), 10.0f32);

    let maybe_inv = m.inverted();
    assert!(maybe_inv.is_some());
    let inv = maybe_inv.unwrap();

    // m * inv should be identity (approximately)
    let prod = m * inv;
    expect_near(prod.at(0, 0), 1.0f32, 1e-5f32);
    expect_near(prod.at(1, 1), 1.0f32, 1e-5f32);
    expect_near(prod.at(0, 1), 0.0f32, 1e-5f32);

    // transposed and to_string variants
    let t = m.transposed();
    assert_eq!(t.at(0, 1), m.at(1, 0));

    let raw = m.as_slice();
    assert_eq!(raw.len(), 4usize);

    let s = m.to_string();
    assert_ne!(s.len(), 0);

    // to_screen_mat static helper
    let screen = Mat::<4, 4, f32>::to_screen(800.0f32, 600.0f32);
    expect_near(screen.at(0, 0), 800.0f32 / 2.0f32, 1e-6f32);
}
