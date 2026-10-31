mod common;

use common::*;
use omath::Vector4;
use std::cmp::Ordering;

#[test]
fn constructors_and_clamp() {
    let a = Vector4::<f32>::default();
    assert_eq!(a.x, 0.0f32);
    assert_eq!(a.y, 0.0f32);
    assert_eq!(a.z, 0.0f32);
    assert_eq!(a.w, 0.0f32);

    let mut b = Vector4::<f32>::new(1.0f32, -2.0f32, 3.5f32, 4.0f32);
    b = b.clamped(0.0f32, 3.0f32);
    assert!(b.x >= 0.0f32);
    assert!(b.y >= 0.0f32);
    assert!(b.z <= 3.0f32);
}

#[test]
fn comparisons_and_hash_formatter() {
    let a = Vector4::<i32>::new(1, 2, 3, 4);
    let b = Vector4::<i32>::new(1, 2, 3, 5);
    assert_ne!(a, b);

    // exercise to_string via formatting if available by converting via std::format
    // call length and comparison to exercise more branches
    assert!(a.length_sqr() < b.length_sqr());
}

#[test]
fn constructor_default() {
    let v = Vector4::<f32>::default();
    expect_float_eq(v.x, 0.0f32);
    expect_float_eq(v.y, 0.0f32);
    expect_float_eq(v.z, 0.0f32);
    expect_float_eq(v.w, 0.0f32);
}

#[test]
fn constructor_values() {
    let v = Vector4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    expect_float_eq(v.x, 1.0f32);
    expect_float_eq(v.y, 2.0f32);
    expect_float_eq(v.z, 3.0f32);
    expect_float_eq(v.w, 4.0f32);
}

#[test]
fn equality_operator() {
    let v1 = Vector4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    let v2 = Vector4::new(4.0f32, 5.0f32, 6.0f32, 7.0f32);

    let v3 = Vector4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    assert!(v1 == v3);
    assert!(!(v1 == v2));
}

#[test]
fn inequality_operator() {
    let v1 = Vector4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    let v2 = Vector4::new(4.0f32, 5.0f32, 6.0f32, 7.0f32);

    let v3 = Vector4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    assert!(!(v1 != v3));
    assert!(v1 != v2);
}

#[test]
fn addition_operator() {
    let v3 =
        Vector4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32) + Vector4::new(4.0f32, 5.0f32, 6.0f32, 7.0f32);
    expect_float_eq(v3.x, 5.0f32);
    expect_float_eq(v3.y, 7.0f32);
    expect_float_eq(v3.z, 9.0f32);
    expect_float_eq(v3.w, 11.0f32);
}

#[test]
fn subtraction_operator() {
    let v3 =
        Vector4::new(4.0f32, 5.0f32, 6.0f32, 7.0f32) - Vector4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    expect_float_eq(v3.x, 3.0f32);
    expect_float_eq(v3.y, 3.0f32);
    expect_float_eq(v3.z, 3.0f32);
    expect_float_eq(v3.w, 3.0f32);
}

#[test]
fn multiplication_operator_scalar() {
    let v3 = Vector4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32) * 2.0f32;
    expect_float_eq(v3.x, 2.0f32);
    expect_float_eq(v3.y, 4.0f32);
    expect_float_eq(v3.z, 6.0f32);
    expect_float_eq(v3.w, 8.0f32);
}

#[test]
fn multiplication_operator_vector() {
    let v3 =
        Vector4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32) * Vector4::new(4.0f32, 5.0f32, 6.0f32, 7.0f32);
    expect_float_eq(v3.x, 4.0f32);
    expect_float_eq(v3.y, 10.0f32);
    expect_float_eq(v3.z, 18.0f32);
    expect_float_eq(v3.w, 28.0f32);
}

#[test]
fn division_operator_scalar() {
    let v3 = Vector4::new(4.0f32, 5.0f32, 6.0f32, 7.0f32) / 2.0f32;
    expect_float_eq(v3.x, 2.0f32);
    expect_float_eq(v3.y, 2.5f32);
    expect_float_eq(v3.z, 3.0f32);
    expect_float_eq(v3.w, 3.5f32);
}

#[test]
fn division_operator_vector() {
    let v3 =
        Vector4::new(4.0f32, 5.0f32, 6.0f32, 7.0f32) / Vector4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    expect_float_eq(v3.x, 4.0f32);
    expect_float_eq(v3.y, 2.5f32);
    expect_float_eq(v3.z, 2.0f32);
    expect_float_eq(v3.w, 1.75f32);
}

#[test]
fn addition_assignment_operator() {
    let mut v1 = Vector4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    let v2 = Vector4::new(4.0f32, 5.0f32, 6.0f32, 7.0f32);

    v1 += v2;
    expect_float_eq(v1.x, 5.0f32);
    expect_float_eq(v1.y, 7.0f32);
    expect_float_eq(v1.z, 9.0f32);
    expect_float_eq(v1.w, 11.0f32);
}

#[test]
fn subtraction_assignment_operator() {
    let mut v1 = Vector4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    let v2 = Vector4::new(4.0f32, 5.0f32, 6.0f32, 7.0f32);

    v1 -= v2;
    expect_float_eq(v1.x, -3.0f32);
    expect_float_eq(v1.y, -3.0f32);
    expect_float_eq(v1.z, -3.0f32);
    expect_float_eq(v1.w, -3.0f32);
}

#[test]
fn multiplication_assignment_operator_scalar() {
    let mut v1 = Vector4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);

    v1 *= 2.0f32;
    expect_float_eq(v1.x, 2.0f32);
    expect_float_eq(v1.y, 4.0f32);
    expect_float_eq(v1.z, 6.0f32);
    expect_float_eq(v1.w, 8.0f32);
}

#[test]
fn multiplication_assignment_operator_vector() {
    let mut v1 = Vector4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    let v2 = Vector4::new(4.0f32, 5.0f32, 6.0f32, 7.0f32);

    v1 *= v2;
    expect_float_eq(v1.x, 4.0f32);
    expect_float_eq(v1.y, 10.0f32);
    expect_float_eq(v1.z, 18.0f32);
    expect_float_eq(v1.w, 28.0f32);
}

#[test]
fn division_assignment_operator_scalar() {
    let mut v1 = Vector4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);

    v1 /= 2.0f32;
    expect_float_eq(v1.x, 0.5f32);
    expect_float_eq(v1.y, 1.0f32);
    expect_float_eq(v1.z, 1.5f32);
    expect_float_eq(v1.w, 2.0f32);
}

#[test]
fn division_assignment_operator_vector() {
    let mut v1 = Vector4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    let v2 = Vector4::new(4.0f32, 5.0f32, 6.0f32, 7.0f32);

    v1 /= v2;
    expect_float_eq(v1.x, 0.25f32);
    expect_float_eq(v1.y, 0.4f32);
    expect_float_eq(v1.z, 0.5f32);
    expect_float_eq(v1.w, 4.0f32 / 7.0f32);
}

#[test]
fn negation_operator() {
    let v3 = -Vector4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    expect_float_eq(v3.x, -1.0f32);
    expect_float_eq(v3.y, -2.0f32);
    expect_float_eq(v3.z, -3.0f32);
    expect_float_eq(v3.w, -4.0f32);
}

#[test]
fn length_sqr() {
    let length_sqr: f32 = Vector4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32).length_sqr();
    expect_float_eq(length_sqr, 30.0f32);
}

#[test]
fn dot_product() {
    let dot: f32 = Vector4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32)
        .dot(Vector4::new(4.0f32, 5.0f32, 6.0f32, 7.0f32));
    expect_float_eq(dot, 60.0f32);
}

#[test]
fn abs() {
    let mut v3 = Vector4::new(-1.0f32, -2.0f32, -3.0f32, -4.0f32);
    v3 = v3.abs();
    expect_float_eq(v3.x, 1.0f32);
    expect_float_eq(v3.y, 2.0f32);
    expect_float_eq(v3.z, 3.0f32);
    expect_float_eq(v3.w, 4.0f32);
}

#[test]
fn abs_const_returns_absolute_copy() {
    let v3 = Vector4::new(-1.0f32, -2.0f32, -3.0f32, -4.0f32);
    let result = v3.abs();
    expect_float_eq(result.x, 1.0f32);
    expect_float_eq(result.y, 2.0f32);
    expect_float_eq(result.z, 3.0f32);
    expect_float_eq(result.w, 4.0f32);
}

#[test]
fn abs_const_does_not_mutate_source() {
    let v3 = Vector4::new(-1.0f32, -2.0f32, -3.0f32, -4.0f32);
    let _ = v3.abs();
    expect_float_eq(v3.x, -1.0f32);
    expect_float_eq(v3.y, -2.0f32);
    expect_float_eq(v3.z, -3.0f32);
    expect_float_eq(v3.w, -4.0f32);
}

#[test]
fn abs_const_positive_values() {
    let v3 = Vector4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    let result = v3.abs();
    expect_float_eq(result.x, 1.0f32);
    expect_float_eq(result.y, 2.0f32);
    expect_float_eq(result.z, 3.0f32);
    expect_float_eq(result.w, 4.0f32);
}

#[test]
fn abs_const_mixed_signs() {
    let v3 = Vector4::new(-1.5f32, 2.5f32, -3.5f32, 4.5f32);
    let result = v3.abs();
    expect_float_eq(result.x, 1.5f32);
    expect_float_eq(result.y, 2.5f32);
    expect_float_eq(result.z, 3.5f32);
    expect_float_eq(result.w, 4.5f32);
}

#[test]
fn sum() {
    let sum: f32 = Vector4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32).sum();
    expect_float_eq(sum, 10.0f32);
}

#[test]
fn clamp() {
    let mut v3 = Vector4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    v3 = v3.clamped(1.5f32, 2.5f32);
    expect_float_eq(v3.x, 1.5f32);
    expect_float_eq(v3.y, 2.0f32);
    expect_float_eq(v3.z, 2.5f32);
    expect_float_eq(v3.w, 2.5f32);
}

#[test]
fn less_operator() {
    let v1 = Vector4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    let v2 = Vector4::new(4.0f32, 5.0f32, 6.0f32, 7.0f32);

    assert!(v1.cmp_length(v2).is_some_and(Ordering::is_lt));
}

#[test]
fn greater_operator() {
    let v1 = Vector4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    let v2 = Vector4::new(4.0f32, 5.0f32, 6.0f32, 7.0f32);

    assert!(v2.cmp_length(v1).is_some_and(Ordering::is_gt));
}

#[test]
fn less_equal_operator() {
    assert!(
        Vector4::<f32>::default()
            .cmp_length(Vector4::<f32>::default())
            .is_some_and(Ordering::is_le)
    );
    assert!(
        Vector4::<f32>::default()
            .cmp_length(Vector4::new(1.0f32, 1.0f32, 1.0f32, 1.0f32))
            .is_some_and(Ordering::is_le)
    );
}

#[test]
fn greater_equal_operator() {
    assert!(
        Vector4::<f32>::default()
            .cmp_length(Vector4::<f32>::default())
            .is_some_and(Ordering::is_ge)
    );
    assert!(
        Vector4::new(1.0f32, 1.0f32, 1.0f32, 1.0f32)
            .cmp_length(Vector4::<f32>::default())
            .is_some_and(Ordering::is_ge)
    );
}

#[test]
fn float_to_double() {
    let v = Vector4::<f32>::new(1.5f32, -2.5f32, 3.0f32, 4.25f32);
    let result = v.cast::<f64>();
    expect_double_eq(result.x, 1.5);
    expect_double_eq(result.y, -2.5);
    expect_double_eq(result.z, 3.0);
    expect_double_eq(result.w, 4.25);
}

#[test]
fn double_to_float() {
    let v = Vector4::<f64>::new(1.25f64, -3.75f64, 0.5f64, -0.125f64);
    let result = v.cast::<f32>();
    expect_float_eq(result.x, 1.25f32);
    expect_float_eq(result.y, -3.75f32);
    expect_float_eq(result.z, 0.5f32);
    expect_float_eq(result.w, -0.125f32);
}

#[test]
fn float_to_int_truncates() {
    let v = Vector4::<f32>::new(3.9f32, -2.1f32, 7.7f32, -0.9f32);
    let result = v.cast::<i32>();
    assert_eq!(result.x, 3);
    assert_eq!(result.y, -2);
    assert_eq!(result.z, 7);
    assert_eq!(result.w, 0);
}

#[test]
fn int_to_float_exact() {
    let v = Vector4::<i32>::new(7, -4, 0, 1);
    let result = v.cast::<f32>();
    expect_float_eq(result.x, 7.0f32);
    expect_float_eq(result.y, -4.0f32);
    expect_float_eq(result.z, 0.0f32);
    expect_float_eq(result.w, 1.0f32);
}

#[test]
fn zero_preserved() {
    let v = Vector4::<f32>::new(0.0f32, 0.0f32, 0.0f32, 0.0f32);
    let result = v.cast::<f64>();
    expect_double_eq(result.x, 0.0);
    expect_double_eq(result.y, 0.0);
    expect_double_eq(result.z, 0.0);
    expect_double_eq(result.w, 0.0);
}

#[test]
fn negative_values() {
    let v = Vector4::<f64>::new(-100.0f64, -200.0f64, -300.0f64, -400.0f64);
    let result = v.cast::<f32>();
    expect_float_eq(result.x, -100.0f32);
    expect_float_eq(result.y, -200.0f32);
    expect_float_eq(result.z, -300.0f32);
    expect_float_eq(result.w, -400.0f32);
}

#[test]
fn same_type_roundtrip() {
    let v = Vector4::<f32>::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    let result = v.cast::<f32>();
    expect_float_eq(result.x, v.x);
    expect_float_eq(result.y, v.y);
    expect_float_eq(result.z, v.z);
    expect_float_eq(result.w, v.w);
}
