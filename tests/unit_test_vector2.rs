mod common;

use common::*;
use omath::Vector2;
use std::cmp::Ordering;

#[test]
fn constructor_default() {

    let v = Vector2::<f32>::default();
    expect_float_eq(v.x, 0.0f32);
    expect_float_eq(v.y, 0.0f32);

}

#[test]
fn constructor_values() {

    let v = Vector2::new(1.0f32, 2.0f32);
    expect_float_eq(v.x, 1.0f32);
    expect_float_eq(v.y, 2.0f32);

}

#[test]
fn equality_operator() {
    let v1 = Vector2::new(1.0f32, 2.0f32);
    let v2 = Vector2::new(4.0f32, 5.0f32);

    let v3 = Vector2::new(1.0f32, 2.0f32);
    assert!(v1 == v3);
    assert!(!(v1 == v2));

}

#[test]
fn inequality_operator() {
    let v1 = Vector2::new(1.0f32, 2.0f32);
    let v2 = Vector2::new(4.0f32, 5.0f32);

    let v3 = Vector2::new(1.0f32, 2.0f32);
    assert!(!(v1 != v3));
    assert!(v1 != v2);

}

#[test]
fn addition_operator() {

    let v3 = Vector2::new(1.0f32, 2.0f32) + Vector2::new(4.0f32, 5.0f32);
    expect_float_eq(v3.x, 5.0f32);
    expect_float_eq(v3.y, 7.0f32);

}

#[test]
fn subtraction_operator() {

    let v3 = Vector2::new(4.0f32, 5.0f32) - Vector2::new(1.0f32, 2.0f32);
    expect_float_eq(v3.x, 3.0f32);
    expect_float_eq(v3.y, 3.0f32);

}

#[test]
fn multiplication_operator() {

    let v3 = Vector2::new(1.0f32, 2.0f32) * 2.0f32;
    expect_float_eq(v3.x, 2.0f32);
    expect_float_eq(v3.y, 4.0f32);

}

#[test]
fn division_operator() {

    let v3 = Vector2::new(4.0f32, 5.0f32) / 2.0f32;
    expect_float_eq(v3.x, 2.0f32);
    expect_float_eq(v3.y, 2.5f32);

}

#[test]
fn negation_operator() {

    let v3 = -Vector2::new(1.0f32, 2.0f32);
    expect_float_eq(v3.x, -1.0f32);
    expect_float_eq(v3.y, -2.0f32);

}

#[test]
fn addition_assignment_operator() {
    let mut v1 = Vector2::new(1.0f32, 2.0f32);
    let v2 = Vector2::new(4.0f32, 5.0f32);

    v1 += v2;
    expect_float_eq(v1.x, 5.0f32);
    expect_float_eq(v1.y, 7.0f32);

}

#[test]
fn subtraction_assignment_operator() {
    let mut v1 = Vector2::new(1.0f32, 2.0f32);
    let v2 = Vector2::new(4.0f32, 5.0f32);

    v1 -= v2;
    expect_float_eq(v1.x, -3.0f32);
    expect_float_eq(v1.y, -3.0f32);

}

#[test]
fn multiplication_assignment_operator() {
    let mut v1 = Vector2::new(1.0f32, 2.0f32);

    v1 *= 2.0f32;
    expect_float_eq(v1.x, 2.0f32);
    expect_float_eq(v1.y, 4.0f32);

}

#[test]
fn division_assignment_operator() {
    let mut v1 = Vector2::new(1.0f32, 2.0f32);

    v1 /= 2.0f32;
    expect_float_eq(v1.x, 0.5f32);
    expect_float_eq(v1.y, 1.0f32);

}

#[test]
fn multiplication_assignment_operator_vector() {
    let mut v1 = Vector2::new(1.0f32, 2.0f32);
    let v2 = Vector2::new(4.0f32, 5.0f32);

    v1 *= v2;
    expect_float_eq(v1.x, 1.0f32 * 4.0f32);
    expect_float_eq(v1.y, 2.0f32 * 5.0f32);

}

#[test]
fn division_assignment_operator_vector() {
    let mut v1 = Vector2::new(1.0f32, 2.0f32);
    let v2 = Vector2::new(4.0f32, 5.0f32);

    v1 /= v2;
    expect_float_eq(v1.x, 1.0f32 / 4.0f32);
    expect_float_eq(v1.y, 2.0f32 / 5.0f32);

}

#[test]
fn addition_assignment_operator_float() {
    let mut v1 = Vector2::new(1.0f32, 2.0f32);

    v1 += 3.0f32;
    expect_float_eq(v1.x, 4.0f32);
    expect_float_eq(v1.y, 5.0f32);

}

#[test]
fn subtraction_assignment_operator_float() {
    let mut v1 = Vector2::new(1.0f32, 2.0f32);

    v1 -= 1.0f32;
    expect_float_eq(v1.x, 0.0f32);
    expect_float_eq(v1.y, 1.0f32);

}

#[test]
fn dist_to() {
    let v1 = Vector2::new(1.0f32, 2.0f32);
    let v2 = Vector2::new(4.0f32, 5.0f32);

    let dist: f32 = v1.distance_to(v2);
    expect_float_eq(dist, (18.0f32).sqrt());

}

#[test]
fn dist_to_same_point() {
    let v1 = Vector2::new(1.0f32, 2.0f32);

    let dist: f32 = v1.distance_to(v1);
    expect_float_eq(dist, 0.0f32);

}

#[test]
fn dist_to_sqr() {

    let dist_sqr: f32 = Vector2::new(1.0f32, 2.0f32).distance_to_sqr(Vector2::new(4.0f32, 5.0f32));
    expect_float_eq(dist_sqr, 18.0f32);

}

#[test]
fn dist_to_sqr_same_point() {

    let dist_sqr: f32 = Vector2::new(1.0f32, 2.0f32).distance_to_sqr(Vector2::new(1.0f32, 2.0f32));
    expect_float_eq(dist_sqr, 0.0f32);

}

#[test]
fn dot_product() {

    let dot: f32 = Vector2::new(1.0f32, 2.0f32).dot(Vector2::new(4.0f32, 5.0f32));
    expect_float_eq(dot, 14.0f32);

}

#[test]
fn dot_product_perpendicular_vectors() {

    let dot: f32 = Vector2::new(1.0f32, 0.0f32).dot(Vector2::new(0.0f32, 1.0f32));
    expect_float_eq(dot, 0.0f32);

}

#[test]
fn dot_product_parallel_vectors() {

    let dot: f32 = Vector2::new(1.0f32, 1.0f32).dot(Vector2::new(2.0f32, 2.0f32));
    expect_float_eq(dot, 4.0f32);

}

#[test]
fn length() {
    let v1 = Vector2::new(1.0f32, 2.0f32);

    let length: f32 = v1.length();
    expect_float_eq(length, (5.0f32).sqrt());

}

#[test]
fn length_zero_vector() {

    let v_zero = Vector2::new(0.0f32, 0.0f32);
    let length: f32 = v_zero.length();
    expect_float_eq(length, 0.0f32);

}

#[test]
fn length_large_values() {

    let v_large = Vector2::new(f32::MAX, f32::MAX);
    let length: f32 = v_large.length();
    assert!((length).is_infinite());

}

#[test]
fn length_sqr() {

    let length_sqr: f32 = Vector2::new(1.0f32, 2.0f32).length_sqr();
    expect_float_eq(length_sqr, 5.0f32);

}

#[test]
fn abs() {

    let mut v3 = Vector2::new(-1.0f32, -2.0f32);
    v3 = v3.abs();
    expect_float_eq(v3.x, 1.0f32);
    expect_float_eq(v3.y, 2.0f32);

}

#[test]
fn abs_positive_values() {

    let mut v3 = Vector2::new(1.0f32, 2.0f32);
    v3 = v3.abs();
    expect_float_eq(v3.x, 1.0f32);
    expect_float_eq(v3.y, 2.0f32);

}

#[test]
fn abs_zero_values() {

    let mut v3 = Vector2::new(0.0f32, 0.0f32);
    v3 = v3.abs();
    expect_float_eq(v3.x, 0.0f32);
    expect_float_eq(v3.y, 0.0f32);

}

#[test]
fn abs_const_returns_absolute_copy() {

    let v3 = Vector2::new(-1.0f32, -2.0f32);
    let result = v3.abs();
    expect_float_eq(result.x, 1.0f32);
    expect_float_eq(result.y, 2.0f32);

}

#[test]
fn abs_const_does_not_mutate_source() {

    let v3 = Vector2::new(-1.0f32, -2.0f32);
    let _ = v3.abs();
    expect_float_eq(v3.x, -1.0f32);
    expect_float_eq(v3.y, -2.0f32);

}

#[test]
fn abs_const_positive_values() {

    let v3 = Vector2::new(1.0f32, 2.0f32);
    let result = v3.abs();
    expect_float_eq(result.x, 1.0f32);
    expect_float_eq(result.y, 2.0f32);

}

#[test]
fn abs_const_mixed_signs() {

    let v3 = Vector2::new(-3.5f32, 4.5f32);
    let result = v3.abs();
    expect_float_eq(result.x, 3.5f32);
    expect_float_eq(result.y, 4.5f32);

}

#[test]
fn sum() {

    let sum: f32 = Vector2::new(1.0f32, 2.0f32).sum();
    expect_float_eq(sum, 3.0f32);

}

#[test]
fn sum_negative_values() {

    let sum: f32 = Vector2::new(-1.0f32, -2.0f32).sum();
    expect_float_eq(sum, -3.0f32);

}

#[test]
fn normalized() {
    let v1 = Vector2::new(1.0f32, 2.0f32);

    let v3 = v1.normalized();
    expect_near(v3.x, 0.44721f32, 0.0001f32);
    expect_near(v3.y, 0.89443f32, 0.0001f32);

}

#[test]
fn normalized_zero_vector() {

    let v_zero = Vector2::new(0.0f32, 0.0f32);
    let v_norm = v_zero.normalized();
    expect_float_eq(v_norm.x, 0.0f32);
    expect_float_eq(v_norm.y, 0.0f32);

}

#[test]
fn as_tuple() {
    let v1 = Vector2::new(1.0f32, 2.0f32);

    let tuple = v1.as_tuple();
    expect_float_eq(tuple.0, v1.x);
    expect_float_eq(tuple.1, v1.y);

}

#[test]
fn division_operator_divide_by_zero() {

    let v = Vector2::new(1.0f32, 2.0f32);
    let zero: f32 = 0.0f32;
    let result = v / zero;
    assert!((result.x).is_infinite() || (result.x).is_nan());
    assert!((result.y).is_infinite() || (result.y).is_nan());

}

#[test]
fn division_assignment_operator_divide_by_zero() {

    let mut v = Vector2::new(1.0f32, 2.0f32);
    let zero: f32 = 0.0f32;
    v /= zero;
    assert!((v.x).is_infinite() || (v.x).is_nan());
    assert!((v.y).is_infinite() || (v.y).is_nan());

}

#[test]
fn division_assignment_operator_vector_with_zero() {

    let mut v = Vector2::new(1.0f32, 2.0f32);
    let v_zero = Vector2::new(0.0f32, 1.0f32);
    v /= v_zero;
    assert!((v.x).is_infinite() || (v.x).is_nan());
    expect_float_eq(v.y, 2.0f32 / 1.0f32);

}

#[test]
fn operator_with_infinity() {
    let v1 = Vector2::new(1.0f32, 2.0f32);

    let v_inf = Vector2::new(f32::INFINITY, f32::INFINITY);
    let result = v1 + v_inf;
    assert!((result.x).is_infinite());
    assert!((result.y).is_infinite());

}

#[test]
fn operator_with_na_n() {
    let v1 = Vector2::new(1.0f32, 2.0f32);

    let v_nan = Vector2::new(f32::NAN, f32::NAN);
    let result = v1 + v_nan;
    assert!((result.x).is_nan());
    assert!((result.y).is_nan());

}

#[test]
fn addition_operator_negative_values() {
    let v1 = Vector2::new(1.0f32, 2.0f32);

    let v_neg = Vector2::new(-1.0f32, -2.0f32);
    let result = v1 + v_neg;
    expect_float_eq(result.x, 0.0f32);
    expect_float_eq(result.y, 0.0f32);

}

#[test]
fn subtraction_operator_negative_values() {
    let v1 = Vector2::new(1.0f32, 2.0f32);

    let v_neg = Vector2::new(-1.0f32, -2.0f32);
    let result = v1 - v_neg;
    expect_float_eq(result.x, 2.0f32);
    expect_float_eq(result.y, 4.0f32);

}

#[test]
fn negation_operator_zero_vector() {

    let v_zero = Vector2::new(0.0f32, 0.0f32);
    let result = -v_zero;
    expect_float_eq(result.x, -0.0f32);
    expect_float_eq(result.y, -0.0f32);

}

#[test]
fn less_operator() {
    let v1 = Vector2::new(1.0f32, 2.0f32);
    let v2 = Vector2::new(4.0f32, 5.0f32);

    assert!(v1.cmp_length(v2).is_some_and(Ordering::is_lt));

}

#[test]
fn greater_operator() {
    let v1 = Vector2::new(1.0f32, 2.0f32);
    let v2 = Vector2::new(4.0f32, 5.0f32);

    assert!(v2.cmp_length(v1).is_some_and(Ordering::is_gt));

}

#[test]
fn less_equal_operator() {

    assert!(Vector2::<f32>::default().cmp_length(Vector2::<f32>::default()).is_some_and(Ordering::is_le));
    assert!(Vector2::<f32>::default().cmp_length(Vector2::new(1.0f32, 1.0f32)).is_some_and(Ordering::is_le));

}

#[test]
fn greater_equal_operator() {

    assert!(Vector2::<f32>::default().cmp_length(Vector2::<f32>::default()).is_some_and(Ordering::is_ge));
    assert!(Vector2::new(1.0f32, 1.0f32).cmp_length(Vector2::<f32>::default()).is_some_and(Ordering::is_ge));

}

#[test]
fn float_to_double() {

    let v = Vector2::<f32>::new(1.5f32, -2.5f32);
    let result = v.cast::<f64>();
    expect_double_eq(result.x, 1.5);
    expect_double_eq(result.y, -2.5);

}

#[test]
fn double_to_float() {

    let v = Vector2::<f64>::new(1.25f64, -3.75f64);
    let result = v.cast::<f32>();
    expect_float_eq(result.x, 1.25f32);
    expect_float_eq(result.y, -3.75f32);

}

#[test]
fn float_to_int_truncates() {

    let v = Vector2::<f32>::new(3.9f32, -2.1f32);
    let result = v.cast::<i32>();
    assert_eq!(result.x, 3);
    assert_eq!(result.y, -2);

}

#[test]
fn int_to_float_exact() {

    let v = Vector2::<i32>::new(7, -4);
    let result = v.cast::<f32>();
    expect_float_eq(result.x, 7.0f32);
    expect_float_eq(result.y, -4.0f32);

}

#[test]
fn zero_preserved() {

    let v = Vector2::<f32>::new(0.0f32, 0.0f32);
    let result = v.cast::<f64>();
    expect_double_eq(result.x, 0.0);
    expect_double_eq(result.y, 0.0);

}

#[test]
fn negative_values() {

    let v = Vector2::<f64>::new(-100.0f64, -200.0f64);
    let result = v.cast::<f32>();
    expect_float_eq(result.x, -100.0f32);
    expect_float_eq(result.y, -200.0f32);

}

#[test]
fn same_type_roundtrip() {

    let v = Vector2::<f32>::new(1.0f32, 2.0f32);
    let result = v.cast::<f32>();
    expect_float_eq(result.x, v.x);
    expect_float_eq(result.y, v.y);

}

// The file-scope static_asserts of the C++ test, as runtime checks.
#[test]
fn static_asserts() {
    assert!(Vector2::new(1.0f32, 2.0f32).length_sqr() == 5.0f32);
    assert!(Vector2::new(1.0f32, 2.0f32).dot(Vector2::new(4.0f32, 5.0f32)) == 14.0f32);
    assert!(Vector2::new(4.0f32, 5.0f32).distance_to_sqr(Vector2::new(1.0f32, 2.0f32)) == 18.0f32);
    assert!(Vector2::new(-1.0f32, -2.0f32).abs() == Vector2::new(1.0f32, 2.0f32));
    assert!(Vector2::new(3.0f32, 4.0f32).length() == 5.0f32);
    assert!(Vector2::new(0.0f32, 0.0f32).distance_to(Vector2::new(3.0f32, 4.0f32)) == 5.0f32);
    assert!(Vector2::new(1.0f32, 1.0f32).cmp_length(Vector2::new(3.0f32, 4.0f32)).is_some_and(Ordering::is_lt));
}
