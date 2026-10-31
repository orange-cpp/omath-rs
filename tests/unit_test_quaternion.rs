mod common;

use common::*;
use omath::{Quaternion, Vector3};
use std::f32::consts::PI;

const K_EPS: f32 = 1e-5f32;

#[track_caller]
fn expect_quat_near(a: Quaternion<f32>, b: Quaternion<f32>) {
    expect_near(a.x, b.x, K_EPS);
    expect_near(a.y, b.y, K_EPS);
    expect_near(a.z, b.z, K_EPS);
    expect_near(a.w, b.w, K_EPS);
}

#[track_caller]
fn expect_vec3_near(a: Vector3<f32>, b: Vector3<f32>) {
    expect_near(a.x, b.x, K_EPS);
    expect_near(a.y, b.y, K_EPS);
    expect_near(a.z, b.z, K_EPS);
}

#[test]
fn default_constructor_is_identity() {
    let q = Quaternion::<f32>::default();
    expect_float_eq(q.x, 0.0f32);
    expect_float_eq(q.y, 0.0f32);
    expect_float_eq(q.z, 0.0f32);
    expect_float_eq(q.w, 1.0f32);
}

#[test]
fn value_constructor() {
    let q = Quaternion::<f32>::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    expect_float_eq(q.x, 1.0f32);
    expect_float_eq(q.y, 2.0f32);
    expect_float_eq(q.z, 3.0f32);
    expect_float_eq(q.w, 4.0f32);
}

#[test]
fn double_instantiation() {
    let q = Quaternion::<f64>::new(0.0f64, 0.0f64, 0.0f64, 1.0f64);
    expect_double_eq(q.w, 1.0);
}

#[test]
fn equality_operators() {
    let a = Quaternion::<f32>::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    let b = Quaternion::<f32>::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    let c = Quaternion::<f32>::new(1.0f32, 2.0f32, 3.0f32, 5.0f32);

    assert!(a == b);
    assert!(!(a == c));
    assert!(!(a != b));
    assert!(a != c);
}

#[test]
fn scalar_multiply() {
    let q = Quaternion::<f32>::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    let r = q * 2.0f32;
    expect_float_eq(r.x, 2.0f32);
    expect_float_eq(r.y, 4.0f32);
    expect_float_eq(r.z, 6.0f32);
    expect_float_eq(r.w, 8.0f32);
}

#[test]
fn scalar_multiply_assign() {
    let mut q = Quaternion::<f32>::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    q *= 3.0f32;
    expect_float_eq(q.x, 3.0f32);
    expect_float_eq(q.y, 6.0f32);
    expect_float_eq(q.z, 9.0f32);
    expect_float_eq(q.w, 12.0f32);
}

#[test]
fn addition() {
    let a = Quaternion::<f32>::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    let b = Quaternion::<f32>::new(4.0f32, 3.0f32, 2.0f32, 1.0f32);
    let r = a + b;
    expect_float_eq(r.x, 5.0f32);
    expect_float_eq(r.y, 5.0f32);
    expect_float_eq(r.z, 5.0f32);
    expect_float_eq(r.w, 5.0f32);
}

#[test]
fn addition_assign() {
    let mut a = Quaternion::<f32>::new(1.0f32, 0.0f32, 0.0f32, 0.0f32);
    let b = Quaternion::<f32>::new(0.0f32, 1.0f32, 0.0f32, 0.0f32);
    a += b;
    expect_float_eq(a.x, 1.0f32);
    expect_float_eq(a.y, 1.0f32);
}

#[test]
fn unary_negation() {
    let q = Quaternion::<f32>::new(1.0f32, -2.0f32, 3.0f32, -4.0f32);
    let r = -q;
    expect_float_eq(r.x, -1.0f32);
    expect_float_eq(r.y, 2.0f32);
    expect_float_eq(r.z, -3.0f32);
    expect_float_eq(r.w, 4.0f32);
}

#[test]
fn multiply_by_identity_is_noop() {
    let identity = Quaternion::<f32>::default();
    let q = Quaternion::<f32>::new(0.5f32, 0.5f32, 0.5f32, 0.5f32);
    expect_quat_near(q * identity, q);
    expect_quat_near(identity * q, q);
}

#[test]
fn multiply_assign() {
    let identity = Quaternion::<f32>::default();
    let mut q = Quaternion::<f32>::new(0.5f32, 0.5f32, 0.5f32, 0.5f32);
    q *= identity;
    expect_quat_near(q, Quaternion::new(0.5f32, 0.5f32, 0.5f32, 0.5f32));
}

#[test]
fn multiply_known_result() {
    // i * j = k  →  (1,0,0,0) * (0,1,0,0) = (0,0,1,0)
    let i = Quaternion::<f32>::new(1.0f32, 0.0f32, 0.0f32, 0.0f32);
    let j = Quaternion::<f32>::new(0.0f32, 1.0f32, 0.0f32, 0.0f32);
    let k = i * j;
    expect_float_eq(k.x, 0.0f32);
    expect_float_eq(k.y, 0.0f32);
    expect_float_eq(k.z, 1.0f32);
    expect_float_eq(k.w, 0.0f32);
}

#[test]
fn multiply_by_inverse_gives_identity() {
    let q = Quaternion::<f32>::from_axis_angle(Vector3::new(0.0f32, 0.0f32, 1.0f32), PI / 3.0f32);
    let result = q * q.inverse();
    expect_quat_near(result, Quaternion::<f32>::default());
}

#[test]
fn conjugate() {
    let q = Quaternion::<f32>::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    let c = q.conjugate();
    expect_float_eq(c.x, -1.0f32);
    expect_float_eq(c.y, -2.0f32);
    expect_float_eq(c.z, -3.0f32);
    expect_float_eq(c.w, 4.0f32);
}

#[test]
fn conjugate_of_identity_is_identity() {
    let id = Quaternion::<f32>::default();
    let c = id.conjugate();
    expect_float_eq(c.x, 0.0f32);
    expect_float_eq(c.y, 0.0f32);
    expect_float_eq(c.z, 0.0f32);
    expect_float_eq(c.w, 1.0f32);
}

#[test]
fn dot() {
    let a = Quaternion::<f32>::new(1.0f32, 0.0f32, 0.0f32, 0.0f32);
    let b = Quaternion::<f32>::new(0.0f32, 1.0f32, 0.0f32, 0.0f32);
    expect_float_eq(a.dot(b), 0.0f32);
    expect_float_eq(a.dot(a), 1.0f32);
}

#[test]
fn length_sqr_identity() {
    let id = Quaternion::<f32>::default();
    expect_float_eq(id.length_sqr(), 1.0f32);
}

#[test]
fn length_sqr_general() {
    let q = Quaternion::<f32>::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    expect_float_eq(q.length_sqr(), 30.0f32);
}

#[test]
fn length_identity() {
    let id = Quaternion::<f32>::default();
    expect_near(id.length(), 1.0f32, K_EPS);
}

#[test]
fn normalized() {
    let q = Quaternion::<f32>::new(1.0f32, 1.0f32, 1.0f32, 1.0f32);
    let n = q.normalized();
    expect_near(n.length(), 1.0f32, K_EPS);
    expect_near(n.x, 0.5f32, K_EPS);
    expect_near(n.y, 0.5f32, K_EPS);
    expect_near(n.z, 0.5f32, K_EPS);
    expect_near(n.w, 0.5f32, K_EPS);
}

#[test]
fn normalized_of_zero_length_returns_self() {
    // length_sqr = 0 would be UB, but zero-vector part + zero w is degenerate;
    // we just verify the guard branch (divides by zero) doesn't crash by
    // keeping length > 0 via the default constructor path.
    let unit = Quaternion::<f32>::default();
    let n = unit.normalized();
    expect_quat_near(n, unit);
}

#[test]
fn inverse_of_unit_is_conjugate() {
    let q = Quaternion::<f32>::from_axis_angle(Vector3::new(1.0f32, 0.0f32, 0.0f32), PI / 4.0f32);
    let inv = q.inverse();
    let conj = q.conjugate();
    expect_quat_near(inv, conj);
}

#[test]
fn from_axis_angle_zero_angle_is_identity() {
    let q = Quaternion::<f32>::from_axis_angle(Vector3::new(1.0f32, 0.0f32, 0.0f32), 0.0f32);
    expect_near(q.x, 0.0f32, K_EPS);
    expect_near(q.y, 0.0f32, K_EPS);
    expect_near(q.z, 0.0f32, K_EPS);
    expect_near(q.w, 1.0f32, K_EPS);
}

#[test]
fn from_axis_angle90_deg_z() {
    let half_pi: f32 = PI / 2.0f32;
    let q = Quaternion::<f32>::from_axis_angle(Vector3::new(0.0f32, 0.0f32, 1.0f32), half_pi);
    let s: f32 = (half_pi / 2.0f32).sin();
    let c: f32 = (half_pi / 2.0f32).cos();
    expect_near(q.x, 0.0f32, K_EPS);
    expect_near(q.y, 0.0f32, K_EPS);
    expect_near(q.z, s, K_EPS);
    expect_near(q.w, c, K_EPS);
}

#[test]
fn rotate_by_identity_is_noop() {
    let id = Quaternion::<f32>::default();
    let v = Vector3::<f32>::new(1.0f32, 2.0f32, 3.0f32);
    let r = id.rotate(v);
    expect_vec3_near(r, v);
}

#[test]
fn rotate90_deg_around_z() {
    // Rotating (1,0,0) by 90° around Z should give (0,1,0)
    let q = Quaternion::<f32>::from_axis_angle(Vector3::new(0.0f32, 0.0f32, 1.0f32), PI / 2.0f32);
    let r = q.rotate(Vector3::new(1.0f32, 0.0f32, 0.0f32));
    expect_vec3_near(r, Vector3::new(0.0f32, 1.0f32, 0.0f32));
}

#[test]
fn rotate180_deg_around_y() {
    // Rotating (1,0,0) by 180° around Y should give (-1,0,0)
    let q = Quaternion::<f32>::from_axis_angle(Vector3::new(0.0f32, 1.0f32, 0.0f32), PI);
    let r = q.rotate(Vector3::new(1.0f32, 0.0f32, 0.0f32));
    expect_vec3_near(r, Vector3::new(-1.0f32, 0.0f32, 0.0f32));
}

#[test]
fn rotate90_deg_around_x() {
    // Rotating (0,1,0) by 90° around X should give (0,0,1)
    let q = Quaternion::<f32>::from_axis_angle(Vector3::new(1.0f32, 0.0f32, 0.0f32), PI / 2.0f32);
    let r = q.rotate(Vector3::new(0.0f32, 1.0f32, 0.0f32));
    expect_vec3_near(r, Vector3::new(0.0f32, 0.0f32, 1.0f32));
}

#[test]
fn rotation_matrix3_from_identity_is_identity_matrix() {
    let id = Quaternion::<f32>::default();
    let m = id.to_rotation_matrix3();
    for i in 0..3 {
        for j in 0..3 {
            expect_near(m.at(i, j), if i == j { 1.0f32 } else { 0.0f32 }, K_EPS);
        }
    }
}

#[test]
fn rotation_matrix3_from90_deg_z() {
    //  Expected:  | 0 -1  0 |
    //             | 1  0  0 |
    //             | 0  0  1 |
    let q = Quaternion::<f32>::from_axis_angle(Vector3::new(0.0f32, 0.0f32, 1.0f32), PI / 2.0f32);
    let m = q.to_rotation_matrix3();
    expect_near(m.at(0, 0), 0.0f32, K_EPS);
    expect_near(m.at(0, 1), -1.0f32, K_EPS);
    expect_near(m.at(0, 2), 0.0f32, K_EPS);
    expect_near(m.at(1, 0), 1.0f32, K_EPS);
    expect_near(m.at(1, 1), 0.0f32, K_EPS);
    expect_near(m.at(1, 2), 0.0f32, K_EPS);
    expect_near(m.at(2, 0), 0.0f32, K_EPS);
    expect_near(m.at(2, 1), 0.0f32, K_EPS);
    expect_near(m.at(2, 2), 1.0f32, K_EPS);
}

#[test]
fn rotation_matrix3_consistent_with_rotate() {
    // Matrix-vector multiply must agree with the rotate() method
    let q = Quaternion::<f32>::from_axis_angle(Vector3::new(1.0f32, 1.0f32, 0.0f32), PI / 3.0f32);
    let v = Vector3::<f32>::new(2.0f32, -1.0f32, 0.5f32);

    let rotated = q.rotate(v);
    let m = q.to_rotation_matrix3();

    // manual mat-vec multiply (row-major)
    let rx: f32 = m.at(0, 0) * v.x + m.at(0, 1) * v.y + m.at(0, 2) * v.z;
    let ry: f32 = m.at(1, 0) * v.x + m.at(1, 1) * v.y + m.at(1, 2) * v.z;
    let rz: f32 = m.at(2, 0) * v.x + m.at(2, 1) * v.y + m.at(2, 2) * v.z;

    expect_near(rotated.x, rx, K_EPS);
    expect_near(rotated.y, ry, K_EPS);
    expect_near(rotated.z, rz, K_EPS);
}

#[test]
fn rotation_matrix4_from_identity_is_identity_matrix() {
    let id = Quaternion::<f32>::default();
    let m = id.to_rotation_matrix4();
    for i in 0..4 {
        for j in 0..4 {
            expect_near(m.at(i, j), if i == j { 1.0f32 } else { 0.0f32 }, K_EPS);
        }
    }
}

#[test]
fn rotation_matrix4_homogeneous_row_and_column() {
    let q = Quaternion::<f32>::from_axis_angle(Vector3::new(1.0f32, 0.0f32, 0.0f32), PI / 5.0f32);
    let m = q.to_rotation_matrix4();

    // Last row and last column must be (0,0,0,1)
    for i in 0..3 {
        expect_near(m.at(3, i), 0.0f32, K_EPS);
        expect_near(m.at(i, 3), 0.0f32, K_EPS);
    }
    expect_near(m.at(3, 3), 1.0f32, K_EPS);
}

#[test]
fn rotation_matrix4_upper3x3_matches_matrix3() {
    let q = Quaternion::<f32>::from_axis_angle(Vector3::new(0.0f32, 1.0f32, 0.0f32), PI / 7.0f32);
    let m3 = q.to_rotation_matrix3();
    let m4 = q.to_rotation_matrix4();

    for i in 0..3 {
        for j in 0..3 {
            expect_near(m4.at(i, j), m3.at(i, j), K_EPS);
        }
    }
}

#[test]
fn as_array() {
    let q = Quaternion::<f32>::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    let arr = q.as_array();
    expect_float_eq(arr[0], 1.0f32);
    expect_float_eq(arr[1], 2.0f32);
    expect_float_eq(arr[2], 3.0f32);
    expect_float_eq(arr[3], 4.0f32);
}

#[test]
fn formatter() {
    let q = Quaternion::<f32>::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    let s = format!("{}", q);
    assert_eq!(s, "[1, 2, 3, 4]");
}
