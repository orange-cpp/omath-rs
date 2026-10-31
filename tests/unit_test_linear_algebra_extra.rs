mod common;

use common::*;
use omath::{Mat, Vector2, Vector3, Vector4};

#[test]
fn formatter_and_hash_vector2() {
    let v = Vector2::<f32>::new(1.0f32, 2.0f32);
    let s = format!("{}", v);
    assert_eq!(s, "[1, 2]");

    let h1 = hash_of(&v);
    let h2 = hash_of(&Vector2::<f32>::new(1.0f32, 2.0f32));
    let h3 = hash_of(&Vector2::<f32>::new(2.0f32, 3.0f32));

    assert_eq!(h1, h2);
    assert_ne!(h1, h3);
}

#[test]
fn formatter_and_hash_vector3() {
    let v = Vector3::<f32>::new(1.0f32, 2.0f32, 3.0f32);
    let s = format!("{}", v);
    assert_eq!(s, "[1, 2, 3]");

    let h1 = hash_of(&v);
    let h2 = hash_of(&Vector3::<f32>::new(1.0f32, 2.0f32, 3.0f32));
    assert_eq!(h1, h2);

    // point_to_same_direction
    assert!(
        Vector3::<f32>::new(1.0, 0.0, 0.0).point_to_same_direction(Vector3::new(2.0, 0.0, 0.0))
    );
    assert!(
        !Vector3::<f32>::new(1.0, 0.0, 0.0).point_to_same_direction(Vector3::new(-1.0, 0.0, 0.0))
    );
}

#[test]
fn formatter_and_hash_vector4() {
    let v = Vector4::<f32>::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    let s = format!("{}", v);
    assert_eq!(s, "[1, 2, 3, 4]");

    let h1 = hash_of(&v);
    let h2 = hash_of(&Vector4::<f32>::new(1.0f32, 2.0f32, 3.0f32, 4.0f32));
    assert_eq!(h1, h2);
}

#[test]
fn mat_raw_array_and_operators() {
    let m = Mat::<2, 2>::from_rows([[1.0f32, 2.0f32], [3.0f32, 4.0f32]]);
    let raw = m.as_slice();
    assert_eq!(raw.len(), 4);
    expect_float_eq(raw[0], 1.0f32);
    expect_float_eq(raw[3], 4.0f32);

    // operator[] index access
    expect_float_eq(m.at(0, 0), 1.0f32);
    expect_float_eq(m.at(1, 1), 4.0f32);
}
