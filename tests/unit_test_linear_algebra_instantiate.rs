mod common;

use common::*;
use omath::{Mat, Vector3, Vector4};

#[test]
fn vector3_and_vector4_and_mat_coverage() {
    // Vector3 usage
    let a = Vector3::<f32>::new(1.0f32, 2.0f32, 3.0f32);
    let b = Vector3::<f32>::new(4.0f32, 5.0f32, 6.0f32);

    // call various methods
    let _ = a.distance_to_sqr(b);
    let _ = a.dot(b);
    let _ = a.cross(b);
    let tup = a.as_tuple();
    let _ = a.point_to_same_direction(b);

    // non-inlined helpers
    let _ = a.length();
    let _ = a.angle_between(b);
    let _ = a.is_perpendicular(b, 0.1f32);
    let _ = a.normalized();

    // formatter and hash instantiations (char only)
    let _ = format!("{}", a);
    let _ = hash_of(&a);

    // Vector4 usage
    let mut v4 = Vector4::<f32>::new(1.0f32, -2.0f32, 3.0f32, -4.0f32);
    let _ = v4.length();
    let _ = v4.sum();
    v4 = v4.clamped(-2.0f32, 2.0f32);
    let _ = format!("{}", v4);
    let _ = hash_of(&v4);

    // Mat usage: instantiate several sizes and store orders
    let m1 = Mat::<1, 1>::from_rows([[42.0f32]]);
    let _ = m1.determinant();

    let m2 = Mat::<2, 2>::from_rows([[1.0f32, 2.0f32], [3.0f32, 4.0f32]]);
    let det2 = m2.determinant();
    let _ = m2.transposed();
    let _ = m2.minor(0, 0);
    let _ = m2.alg_complement(0, 1);
    let _ = m2.as_slice();
    let inv2 = m2.inverted();

    let m3 = Mat::<3, 3>::from_rows([
        [1.0f32, 2.0f32, 3.0f32],
        [4.0f32, 5.0f32, 6.0f32],
        [7.0f32, 8.0f32, 9.0f32],
    ]);
    let _ = m3.determinant();
    let _: Mat<2, 2> = m3.strip(0, 0);
    let _ = m3.minor(2, 2);

    // to_string and to_screen_mat
    let _ = m2.to_string();
    let _ = Mat::<4, 4>::to_screen(800.0f32, 600.0f32);

    // simple sanity checks (not strict, only to use values)
    assert_eq!(tup.0, 1.0f32);
    assert!(det2 != 0.0f32 || inv2.is_none());
}
