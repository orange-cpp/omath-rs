mod common;

use common::*;
use omath::Vector3;
use std::cmp::Ordering;

#[test]
fn constructors_and_equality() {

    let a = Vector3::<f32>::default();
    assert_eq!(a.x, 0.0f32);
    assert_eq!(a.y, 0.0f32);
    assert_eq!(a.z, 0.0f32);

    let b = Vector3::<f32>::new(1.0f32, 2.0f32, 3.0f32);
    assert_eq!(b.x, 1.0f32);
    assert_eq!(b.y, 2.0f32);
    assert_eq!(b.z, 3.0f32);

    let c = b;
    assert_eq!(c, b);

}

#[test]
fn arithmetic_and_dot_cross() {

    let a = Vector3::<f32>::new(1.0f32, 0.0f32, 0.0f32);
    let b = Vector3::<f32>::new(0.0f32, 1.0f32, 0.0f32);
    let c = a + b;
    let expect_c = Vector3::<f32>::new(1.0f32, 1.0f32, 0.0f32);
    assert_eq!(c, expect_c);

    let d = a - b;
    let expect_d = Vector3::<f32>::new(1.0f32, -1.0f32, 0.0f32);
    assert_eq!(d, expect_d);

    let e = a * 2.0f32;
    let expect_e = Vector3::<f32>::new(2.0f32, 0.0f32, 0.0f32);
    assert_eq!(e, expect_e);

    expect_float_eq(a.dot(b), 0.0f32);
    // manual cross product check
    let cr = Vector3::<f32>::new(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x);
    let expect_cr = Vector3::<f32>::new(0.0f32, 0.0f32, 1.0f32);
    assert_eq!(cr, expect_cr);

}

#[test]
fn normalization_edge_cases() {

    let z = Vector3::<f64>::new(0.0f64, 0.0f64, 0.0f64);
    let zn = z.normalized();
    expect_double_eq(zn.x, 0.0);
    expect_double_eq(zn.y, 0.0);
    expect_double_eq(zn.z, 0.0);

    let v = Vector3::<f64>::new(3.0f64, 4.0f64, 0.0f64);
    let vn = v.normalized();
    expect_near(vn.x, 0.6, 1e-12);
    expect_near(vn.y, 0.8, 1e-12);

}

#[test]
fn constructor_default() {

    let v = Vector3::<f32>::default();
    expect_float_eq(v.x, 0.0f32);
    expect_float_eq(v.y, 0.0f32);
    expect_float_eq(v.z, 0.0f32);

}

#[test]
fn constructor_values() {

    let v = Vector3::<f32>::new(1.0f32, 2.0f32, 3.0f32);
    expect_float_eq(v.x, 1.0f32);
    expect_float_eq(v.y, 2.0f32);
    expect_float_eq(v.z, 3.0f32);

}

#[test]
fn equality_operator() {
    let v1 = Vector3::new(1.0f32, 2.0f32, 3.0f32);
    let v2 = Vector3::new(4.0f32, 5.0f32, 6.0f32);

    let v3 = Vector3::new(1.0f32, 2.0f32, 3.0f32);
    assert!(v1 == v3);
    assert!(!(v1 == v2));

}

#[test]
fn inequality_operator() {
    let v1 = Vector3::new(1.0f32, 2.0f32, 3.0f32);
    let v2 = Vector3::new(4.0f32, 5.0f32, 6.0f32);

    let v3 = Vector3::new(1.0f32, 2.0f32, 3.0f32);
    assert!(!(v1 != v3));
    assert!(v1 != v2);

}

#[test]
fn addition_operator() {

    let v3 = Vector3::new(1.0f32, 2.0f32, 3.0f32) + Vector3::new(4.0f32, 5.0f32, 6.0f32);
    expect_float_eq(v3.x, 5.0f32);
    expect_float_eq(v3.y, 7.0f32);
    expect_float_eq(v3.z, 9.0f32);

}

#[test]
fn subtraction_operator() {

    let v3 = Vector3::new(4.0f32, 5.0f32, 6.0f32) - Vector3::new(1.0f32, 2.0f32, 3.0f32);
    expect_float_eq(v3.x, 3.0f32);
    expect_float_eq(v3.y, 3.0f32);
    expect_float_eq(v3.z, 3.0f32);

}

#[test]
fn multiplication_operator_scalar() {

    let v3 = Vector3::new(1.0f32, 2.0f32, 3.0f32) * 2.0f32;
    expect_float_eq(v3.x, 2.0f32);
    expect_float_eq(v3.y, 4.0f32);
    expect_float_eq(v3.z, 6.0f32);

}

#[test]
fn multiplication_operator_vector() {

    let v3 = Vector3::new(1.0f32, 2.0f32, 3.0f32) * Vector3::new(4.0f32, 5.0f32, 6.0f32);
    expect_float_eq(v3.x, 4.0f32);
    expect_float_eq(v3.y, 10.0f32);
    expect_float_eq(v3.z, 18.0f32);

}

#[test]
fn division_operator_scalar() {

    let v3 = Vector3::new(4.0f32, 5.0f32, 6.0f32) / 2.0f32;
    expect_float_eq(v3.x, 2.0f32);
    expect_float_eq(v3.y, 2.5f32);
    expect_float_eq(v3.z, 3.0f32);

}

#[test]
fn division_operator_vector() {

    let v3 = Vector3::new(4.0f32, 5.0f32, 6.0f32) / Vector3::new(1.0f32, 2.0f32, 3.0f32);
    expect_float_eq(v3.x, 4.0f32);
    expect_float_eq(v3.y, 2.5f32);
    expect_float_eq(v3.z, 2.0f32);

}

#[test]
fn addition_assignment_operator() {
    let mut v1 = Vector3::new(1.0f32, 2.0f32, 3.0f32);
    let v2 = Vector3::new(4.0f32, 5.0f32, 6.0f32);

    v1 += v2;
    expect_float_eq(v1.x, 5.0f32);
    expect_float_eq(v1.y, 7.0f32);
    expect_float_eq(v1.z, 9.0f32);

}

#[test]
fn subtraction_assignment_operator() {
    let mut v1 = Vector3::new(1.0f32, 2.0f32, 3.0f32);
    let v2 = Vector3::new(4.0f32, 5.0f32, 6.0f32);

    v1 -= v2;
    expect_float_eq(v1.x, -3.0f32);
    expect_float_eq(v1.y, -3.0f32);
    expect_float_eq(v1.z, -3.0f32);

}

#[test]
fn multiplication_assignment_operator_scalar() {
    let mut v1 = Vector3::new(1.0f32, 2.0f32, 3.0f32);

    v1 *= 2.0f32;
    expect_float_eq(v1.x, 2.0f32);
    expect_float_eq(v1.y, 4.0f32);
    expect_float_eq(v1.z, 6.0f32);

}

#[test]
fn multiplication_assignment_operator_vector() {
    let mut v1 = Vector3::new(1.0f32, 2.0f32, 3.0f32);
    let v2 = Vector3::new(4.0f32, 5.0f32, 6.0f32);

    v1 *= v2;
    expect_float_eq(v1.x, 4.0f32);
    expect_float_eq(v1.y, 10.0f32);
    expect_float_eq(v1.z, 18.0f32);

}

#[test]
fn division_assignment_operator_scalar() {
    let mut v1 = Vector3::new(1.0f32, 2.0f32, 3.0f32);

    v1 /= 2.0f32;
    expect_float_eq(v1.x, 0.5f32);
    expect_float_eq(v1.y, 1.0f32);
    expect_float_eq(v1.z, 1.5f32);

}

#[test]
fn division_assignment_operator_vector() {
    let mut v1 = Vector3::new(1.0f32, 2.0f32, 3.0f32);
    let v2 = Vector3::new(4.0f32, 5.0f32, 6.0f32);

    v1 /= v2;
    expect_float_eq(v1.x, 0.25f32);
    expect_float_eq(v1.y, 0.4f32);
    expect_float_eq(v1.z, 0.5f32);

}

#[test]
fn negation_operator() {

    let v3 = -Vector3::new(1.0f32, 2.0f32, 3.0f32);
    expect_float_eq(v3.x, -1.0f32);
    expect_float_eq(v3.y, -2.0f32);
    expect_float_eq(v3.z, -3.0f32);

}

#[test]
fn dist_to_sqr() {

    let dist_sqr = Vector3::new(1.0f32, 2.0f32, 3.0f32).distance_to_sqr(Vector3::new(4.0f32, 5.0f32, 6.0f32));
    expect_float_eq(dist_sqr, 27.0f32);

}

#[test]
fn dot_product() {

    let dot = Vector3::new(1.0f32, 2.0f32, 3.0f32).dot(Vector3::new(4.0f32, 5.0f32, 6.0f32));
    expect_float_eq(dot, 32.0f32);

}

#[test]
fn length_sqr() {

    let length_sqr = Vector3::new(1.0f32, 2.0f32, 3.0f32).length_sqr();
    expect_float_eq(length_sqr, 14.0f32);

}

#[test]
fn abs() {

    let mut v3 = Vector3::new(-1.0f32, -2.0f32, -3.0f32);
    v3 = v3.abs();
    expect_float_eq(v3.x, 1.0f32);
    expect_float_eq(v3.y, 2.0f32);
    expect_float_eq(v3.z, 3.0f32);

}

#[test]
fn abs_const_returns_absolute_copy() {

    let v3 = Vector3::new(-1.0f32, -2.0f32, -3.0f32);
    let result = v3.abs();
    expect_float_eq(result.x, 1.0f32);
    expect_float_eq(result.y, 2.0f32);
    expect_float_eq(result.z, 3.0f32);

}

#[test]
fn abs_const_does_not_mutate_source() {

    let v3 = Vector3::new(-1.0f32, -2.0f32, -3.0f32);
    let _ = v3.abs();
    expect_float_eq(v3.x, -1.0f32);
    expect_float_eq(v3.y, -2.0f32);
    expect_float_eq(v3.z, -3.0f32);

}

#[test]
fn abs_const_positive_values() {

    let v3 = Vector3::new(1.0f32, 2.0f32, 3.0f32);
    let result = v3.abs();
    expect_float_eq(result.x, 1.0f32);
    expect_float_eq(result.y, 2.0f32);
    expect_float_eq(result.z, 3.0f32);

}

#[test]
fn abs_const_mixed_signs() {

    let v3 = Vector3::new(-1.5f32, 2.5f32, -3.5f32);
    let result = v3.abs();
    expect_float_eq(result.x, 1.5f32);
    expect_float_eq(result.y, 2.5f32);
    expect_float_eq(result.z, 3.5f32);

}

#[test]
fn sum() {

    let sum = Vector3::new(1.0f32, 2.0f32, 3.0f32).sum();
    expect_float_eq(sum, 6.0f32);

}

#[test]
fn sum_2d() {

    let sum_2d = Vector3::new(1.0f32, 2.0f32, 3.0f32).sum_2d();
    expect_float_eq(sum_2d, 3.0f32);

}

#[test]
fn cross_product() {

    let v3 = Vector3::new(1.0f32, 2.0f32, 3.0f32).cross(Vector3::new(4.0f32, 5.0f32, 6.0f32));
    expect_float_eq(v3.x, -3.0f32);
    expect_float_eq(v3.y, 6.0f32);
    expect_float_eq(v3.z, -3.0f32);

}

#[test]
fn addition_with_zero_vector() {
    let v1 = Vector3::new(1.0f32, 2.0f32, 3.0f32);

    let v_zero = Vector3::new(0.0f32, 0.0f32, 0.0f32);
    let result = v1 + v_zero;
    expect_float_eq(result.x, v1.x);
    expect_float_eq(result.y, v1.y);
    expect_float_eq(result.z, v1.z);

}

#[test]
fn subtraction_with_zero_vector() {
    let v1 = Vector3::new(1.0f32, 2.0f32, 3.0f32);

    let v_zero = Vector3::new(0.0f32, 0.0f32, 0.0f32);
    let result = v1 - v_zero;
    expect_float_eq(result.x, v1.x);
    expect_float_eq(result.y, v1.y);
    expect_float_eq(result.z, v1.z);

}

#[test]
fn multiplication_with_zero_vector() {
    let v1 = Vector3::new(1.0f32, 2.0f32, 3.0f32);

    let v_zero = Vector3::new(0.0f32, 0.0f32, 0.0f32);
    let result = v1 * v_zero;
    expect_float_eq(result.x, 0.0f32);
    expect_float_eq(result.y, 0.0f32);
    expect_float_eq(result.z, 0.0f32);

}

#[test]
fn division_by_zero_vector() {
    let v1 = Vector3::new(1.0f32, 2.0f32, 3.0f32);

    let v_zero = Vector3::new(0.0f32, 0.0f32, 0.0f32);
    let result = v1 / v_zero;
    assert!((result.x).is_infinite() || (result.x).is_nan());
    assert!((result.y).is_infinite() || (result.y).is_nan());
    assert!((result.z).is_infinite() || (result.z).is_nan());

}

#[test]
fn division_by_zero_scalar() {
    let v1 = Vector3::new(1.0f32, 2.0f32, 3.0f32);

    let zero: f32 = 0.0f32;
    let result = v1 / zero;
    assert!((result.x).is_infinite() || (result.x).is_nan());
    assert!((result.y).is_infinite() || (result.y).is_nan());
    assert!((result.z).is_infinite() || (result.z).is_nan());

}

#[test]
fn addition_with_infinity() {
    let v1 = Vector3::new(1.0f32, 2.0f32, 3.0f32);

    let v_inf = Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
    let result = v1 + v_inf;
    assert!((result.x).is_infinite());
    assert!((result.y).is_infinite());
    assert!((result.z).is_infinite());

}

#[test]
fn subtraction_with_infinity() {
    let v1 = Vector3::new(1.0f32, 2.0f32, 3.0f32);

    let v_inf = Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
    let result = v1 - v_inf;
    assert!((result.x).is_infinite());
    assert!((result.y).is_infinite());
    assert!((result.z).is_infinite());

}

#[test]
fn multiplication_with_na_n() {
    let v1 = Vector3::new(1.0f32, 2.0f32, 3.0f32);

    let v_nan = Vector3::new(f32::NAN, f32::NAN, f32::NAN);
    let result = v1 * v_nan;
    assert!((result.x).is_nan());
    assert!((result.y).is_nan());
    assert!((result.z).is_nan());

}

#[test]
fn division_with_na_n() {
    let v1 = Vector3::new(1.0f32, 2.0f32, 3.0f32);

    let v_nan = Vector3::new(f32::NAN, f32::NAN, f32::NAN);
    let result = v1 / v_nan;
    assert!((result.x).is_nan());
    assert!((result.y).is_nan());
    assert!((result.z).is_nan());

}

#[test]
fn length() {
    let v1 = Vector3::new(1.0f32, 2.0f32, 3.0f32);

    let length: f32 = v1.length();
    expect_float_eq(length, (14.0f32).sqrt());

}

#[test]
fn length_zero_vector() {

    let v_zero = Vector3::new(0.0f32, 0.0f32, 0.0f32);
    let length: f32 = v_zero.length();
    expect_float_eq(length, 0.0f32);

}

#[test]
fn length_large_values() {

    let v_large = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
    let length: f32 = v_large.length();
    assert!((length).is_infinite());

}

#[test]
fn length_2d() {
    let v1 = Vector3::new(1.0f32, 2.0f32, 3.0f32);

    let length_2d: f32 = v1.length_2d();
    expect_float_eq(length_2d, (5.0f32).sqrt());

}

#[test]
fn normalized() {
    let v1 = Vector3::new(1.0f32, 2.0f32, 3.0f32);

    let v_norm = v1.normalized();
    let length: f32 = v_norm.length();
    expect_near(length, 1.0f32, 0.0001f32);

}

#[test]
fn normalized_zero_vector() {

    let v_zero = Vector3::new(0.0f32, 0.0f32, 0.0f32);
    let v_norm = v_zero.normalized();
    expect_float_eq(v_norm.x, 0.0f32);
    expect_float_eq(v_norm.y, 0.0f32);
    expect_float_eq(v_norm.z, 0.0f32);

}

#[test]
fn cross_product_parallel_vectors() {

    let v_a = Vector3::new(1.0f32, 2.0f32, 3.0f32);
    let v_b = v_a * 2.0f32;
    let cross = v_a.cross(v_b);
    expect_float_eq(cross.x, 0.0f32);
    expect_float_eq(cross.y, 0.0f32);
    expect_float_eq(cross.z, 0.0f32);

}

#[test]
fn cross_product_orthogonal_vectors() {

    let v_a = Vector3::new(1.0f32, 0.0f32, 0.0f32);
    let v_b = Vector3::new(0.0f32, 1.0f32, 0.0f32);
    let cross = v_a.cross(v_b);
    expect_float_eq(cross.x, 0.0f32);
    expect_float_eq(cross.y, 0.0f32);
    expect_float_eq(cross.z, 1.0f32);

}

#[test]
fn addition_negative_values() {
    let v1 = Vector3::new(1.0f32, 2.0f32, 3.0f32);

    let v_neg = Vector3::new(-1.0f32, -2.0f32, -3.0f32);
    let result = v1 + v_neg;
    expect_float_eq(result.x, 0.0f32);
    expect_float_eq(result.y, 0.0f32);
    expect_float_eq(result.z, 0.0f32);

}

#[test]
fn subtraction_negative_values() {
    let v1 = Vector3::new(1.0f32, 2.0f32, 3.0f32);

    let v_neg = Vector3::new(-1.0f32, -2.0f32, -3.0f32);
    let result = v1 - v_neg;
    expect_float_eq(result.x, 2.0f32);
    expect_float_eq(result.y, 4.0f32);
    expect_float_eq(result.z, 6.0f32);

}

#[test]
fn as_tuple() {
    let v1 = Vector3::new(1.0f32, 2.0f32, 3.0f32);

    let tuple = v1.as_tuple();
    expect_float_eq(tuple.0, v1.x);
    expect_float_eq(tuple.1, v1.y);
    expect_float_eq(tuple.2, v1.z);

}

#[test]
fn angle_beatween() {
    expect_near(Vector3::new(0.0f32, 0.0f32, 1.0f32).angle_between(Vector3::new(1.0, 0.0, 0.0)).unwrap().as_degrees(), 90.0f32, 0.001f32);
    expect_near(Vector3::new(0.0f32, 0.0f32, 1.0f32).angle_between(Vector3::new(0.0f32, 0.0f32, 1.0f32)).unwrap().as_degrees(), 0.0f32, 0.001f32);
    assert!(!(Vector3::new(0.0f32, 0.0f32, 0.0f32).angle_between(Vector3::new(0.0f32, 0.0f32, 1.0f32)).is_ok()));
}

#[test]
fn is_perpendicular() {
    // The C++ default epsilon is 0.0001; Rust has no default arguments.
    assert_eq!(Vector3::new(0.0f32, 0.0f32, 1.0f32).is_perpendicular(Vector3::new(1.0, 0.0, 0.0), 0.0001f32), true);
    assert_eq!(Vector3::new(0.0f32, 0.0f32, 1.0f32).is_perpendicular(Vector3::new(0.0f32, 0.0f32, 1.0f32), 0.0001f32), false);
    assert!(!(Vector3::new(0.0f32, 0.0f32, 0.0f32).is_perpendicular(Vector3::new(0.0f32, 0.0f32, 1.0f32), 0.0001f32)));
}

#[test]
fn less_operator() {
    let v1 = Vector3::new(1.0f32, 2.0f32, 3.0f32);
    let v2 = Vector3::new(4.0f32, 5.0f32, 6.0f32);

    assert!(v1.cmp_length(v2).is_some_and(Ordering::is_lt));

}

#[test]
fn greater_operator() {
    let v1 = Vector3::new(1.0f32, 2.0f32, 3.0f32);
    let v2 = Vector3::new(4.0f32, 5.0f32, 6.0f32);

    assert!(v2.cmp_length(v1).is_some_and(Ordering::is_gt));

}

#[test]
fn less_equal_operator() {

    assert!(Vector3::<f32>::default().cmp_length(Vector3::<f32>::default()).is_some_and(Ordering::is_le));
    assert!(Vector3::<f32>::default().cmp_length(Vector3::new(1.0f32, 1.0f32, 1.0f32)).is_some_and(Ordering::is_le));

}

#[test]
fn greater_equal_operator() {

    assert!(Vector3::<f32>::default().cmp_length(Vector3::<f32>::default()).is_some_and(Ordering::is_ge));
    assert!(Vector3::new(1.0f32, 1.0f32, 1.0f32).cmp_length(Vector3::<f32>::default()).is_some_and(Ordering::is_ge));

}

#[test]
fn float_to_double() {

    let v = Vector3::<f32>::new(1.5f32, -2.5f32, 3.0f32);
    let result = v.cast::<f64>();
    expect_double_eq(result.x, 1.5);
    expect_double_eq(result.y, -2.5);
    expect_double_eq(result.z, 3.0);

}

#[test]
fn double_to_float() {

    let v = Vector3::<f64>::new(1.25f64, -3.75f64, 0.5f64);
    let result = v.cast::<f32>();
    expect_float_eq(result.x, 1.25f32);
    expect_float_eq(result.y, -3.75f32);
    expect_float_eq(result.z, 0.5f32);

}

#[test]
fn float_to_int_truncates() {

    let v = Vector3::<f32>::new(3.9f32, -2.1f32, 7.7f32);
    let result = v.cast::<i32>();
    assert_eq!(result.x, 3);
    assert_eq!(result.y, -2);
    assert_eq!(result.z, 7);

}

#[test]
fn int_to_float_exact() {

    let v = Vector3::<i32>::new(7, -4, 0);
    let result = v.cast::<f32>();
    expect_float_eq(result.x, 7.0f32);
    expect_float_eq(result.y, -4.0f32);
    expect_float_eq(result.z, 0.0f32);

}

#[test]
fn zero_preserved() {

    let v = Vector3::<f32>::new(0.0f32, 0.0f32, 0.0f32);
    let result = v.cast::<f64>();
    expect_double_eq(result.x, 0.0);
    expect_double_eq(result.y, 0.0);
    expect_double_eq(result.z, 0.0);

}

#[test]
fn negative_values() {

    let v = Vector3::<f64>::new(-100.0f64, -200.0f64, -300.0f64);
    let result = v.cast::<f32>();
    expect_float_eq(result.x, -100.0f32);
    expect_float_eq(result.y, -200.0f32);
    expect_float_eq(result.z, -300.0f32);

}

#[test]
fn same_type_roundtrip() {

    let v = Vector3::<f32>::new(1.0f32, 2.0f32, 3.0f32);
    let result = v.cast::<f32>();
    expect_float_eq(result.x, v.x);
    expect_float_eq(result.y, v.y);
    expect_float_eq(result.z, v.z);

}

// The file-scope static_asserts of the C++ test, as runtime checks.
#[test]
fn static_asserts() {
    assert!(Vector3::new(1.0f32, 2.0f32, 3.0f32).length_sqr() == 14.0f32);
    assert!(Vector3::new(1.0f32, 2.0f32, 3.0f32).dot(Vector3::new(4.0f32, 5.0f32, 6.0f32)) == 32.0f32);
    assert!(Vector3::new(4.0f32, 5.0f32, 6.0f32).distance_to_sqr(Vector3::new(1.0f32, 2.0f32, 3.0f32)) == 27.0f32);
    assert!(Vector3::new(-1.0f32, -2.0f32, -3.0f32).abs() == Vector3::new(1.0f32, 2.0f32, 3.0f32));
    assert!(Vector3::new(1.0f32, 2.0f32, 2.0f32).length() == 3.0f32);
    assert!(Vector3::new(0.0f32, 0.0f32, 0.0f32).distance_to(Vector3::new(1.0f32, 2.0f32, 2.0f32)) == 3.0f32);
    assert!(Vector3::new(1.0f32, 1.0f32, 1.0f32).cmp_length(Vector3::new(3.0f32, 4.0f32, 5.0f32)).is_some_and(Ordering::is_lt));

    let angle = Vector3::new(1.0f32, 0.0f32, 0.0f32).angle_between(Vector3::new(0.0f32, 1.0f32, 0.0f32));
    assert!(angle.is_ok_and(|angle| angle.as_degrees() > 89.999f32 && angle.as_degrees() < 90.001f32));
}
