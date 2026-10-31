mod common;

use common::*;
use omath::Mat;

#[test]
fn init_list_and_multiply() {

    let m = Mat::<3, 3, f32>::from_rows([[1.0f32, 2.0f32, 3.0f32], [0.0f32, 1.0f32, 4.0f32], [5.0f32, 6.0f32, 0.0f32]]);
    // multiply by scalar and check element
    let r = m * 1.0f32;
    assert_eq!(r.at(0,0), m.at(0,0));
    assert_eq!(r.at(1,2), m.at(1,2));

}

#[test]
fn determinant() {

    let m = Mat::<2, 2, f64>::from_rows([[1.0f64, 2.0f64], [2.0f64, 4.0f64]]);
    let det: f64 = m.determinant();
    expect_double_eq(det, 0.0);

}
