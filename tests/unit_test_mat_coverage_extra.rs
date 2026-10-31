mod common;

use common::*;
use omath::Mat;

#[test]
fn init_list_rows_mismatch_throws() {
    // from_rows checks its shape at compile time; from_slice is the runtime path that can still get a bad shape.
    // Rows mismatch: provide 3 rows for a 2x2 Mat
    assert!(Mat::<2, 2>::from_slice(&[1.0f32, 2.0f32, 3.0f32, 4.0f32, 5.0f32, 6.0f32]).is_none());
}

#[test]
fn init_list_columns_mismatch_throws() {
    // from_rows checks its shape at compile time; from_slice is the runtime path that can still get a bad shape.
    // Columns mismatch: second row has wrong number of columns
    assert!(Mat::<2, 2>::from_slice(&[1.0f32, 2.0f32, 3.0f32]).is_none());
}

#[test]
fn determinant_fallback_is_callable() {
    // Call determinant for 1x1 and 2x2 matrices to cover determinant paths
    let m1 = Mat::<1, 1>::from_rows([[3.14f32]]);
    expect_float_eq(m1.determinant(), 3.14f32);

    let m2 = Mat::<2, 2>::from_rows([[1.0f32, 2.0f32], [3.0f32, 4.0f32]]);
    expect_float_eq(m2.determinant(), -2.0f32);
}
