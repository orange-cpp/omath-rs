mod common;

use common::*;
use omath::{Mat, Vector4};

#[test]
fn subtraction() {

    let a = Vector4::<f32>::new(5.0f32, 6.0f32, 7.0f32, 8.0f32);
    let b = Vector4::<f32>::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);

    let r = a - b;
    expect_float_eq(r.x, 4.0f32);
    expect_float_eq(r.y, 4.0f32);
    expect_float_eq(r.z, 4.0f32);
    expect_float_eq(r.w, 4.0f32);

}

#[test]
fn forced_throw_lines() {
    // from_rows checks its shape at compile time; from_slice is the runtime path that can still get a bad shape.
    assert!(Mat::<2, 2, f32>::from_slice(&[1.0f32, 2.0f32]).is_none());
    assert!(Mat::<2, 2, f32>::from_slice(&[1.0f32, 2.0f32, 1.0f32]).is_none());
}

#[test]
fn copy_and_move_self_assign() {
    let mut m = Mat::<2, 2, f32>::from_rows([[1.0f32, 2.0f32], [3.0f32, 4.0f32]]);
    // self copy-assignment. rustc warns on a literal `m = m`, so the value goes through a copy.
    let copy = m;
    m = copy;
    expect_float_eq(m.at(0, 0), 1.0f32);

    // self move-assignment: a moved Copy value stays usable
    let moved = m;
    m = moved;
    expect_float_eq(m.at(0, 0), 1.0f32);
}
