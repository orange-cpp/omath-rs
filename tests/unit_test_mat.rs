mod common;

use common::*;
use omath::scalar::Scalar;
use omath::trigonometry::angles;
use omath::{
    Angle, ColumnMajor, Mat, MatStoreType, NdcDepthRange, RowMajor, StoreOrder, Vector3,
    angle_range,
};

type M4 = Mat<4, 4>;

angle_range!(PitchRange, -90.0, 90.0, Clamped);
angle_range!(AnyRange, -180.0, 180.0, Normalized);
type Pitch = Angle<f32, PitchRange>;
type AnyAngle = Angle<f32, AnyRange>;

const K_AFFINE: [[f32; 4]; 4] = [
    [0.936f32, -0.289f32, 0.198f32, 12.5f32],
    [0.312f32, 0.884f32, -0.348f32, -4.25f32],
    [-0.160f32, 0.369f32, 0.916f32, 3.75f32],
    [0.0f32, 0.0f32, 0.0f32, 1.0f32],
];

fn col(vector: Vector3<f32>) -> Mat<4, 1> {
    Mat::column_from_vector(vector)
}

fn close_to(actual: f32, expected: f32, epsilon: f32) -> bool {
    (actual - expected).abs() <= epsilon
}

fn expect_multiplication_matches_scalar_reference<
    const R: usize,
    const C: usize,
    const O: usize,
    T: Scalar,
    S: StoreOrder,
>(
    left: Mat<R, C, T, S>,
    right: Mat<C, O, T, S>,
) {
    let result = left * right;
    for row in 0..R {
        for column in 0..O {
            let mut expected = T::default();
            for shared_index in 0..C {
                expected += left.at(row, shared_index) * right.at(shared_index, column);
            }
            assert_eq!(result.at(row, column), expected);
        }
    }
}

fn expect_identity<const N: usize, S: StoreOrder>(mat: Mat<N, N, f32, S>, tolerance: f32) {
    for row in 0..N {
        for column in 0..N {
            expect_near(
                mat.at(row, column),
                if row == column { 1.0f32 } else { 0.0f32 },
                tolerance,
            );
        }
    }
}

fn expect_mat3_near(actual: M4, expected: M4) {
    for row in 0..4 {
        for col in 0..4 {
            expect_near(actual.at(row, col), expected.at(row, col), 1e-5f32);
        }
    }
}

/// `std::remainder(a - b, 360.f)`.
fn angle_difference(a: f32, b: f32) -> f32 {
    let difference = (a - b) as f64;
    (difference - 360.0 * (difference / 360.0).round_ties_even()) as f32
}

fn any(degrees: f32) -> AnyAngle {
    AnyAngle::from_degrees(degrees)
}

#[test]
fn constructor_default() {
    let m = Mat::<3, 3>::default();
    assert_eq!(row_count(&m), 3);
    assert_eq!(columns_count(&m), 3);
    for i in 0..3 {
        for j in 0..3 {
            expect_float_eq(m.at(i, j), 0.0f32);
        }
    }
}

#[test]
fn constructor_initializer_list() {
    let m = Mat::<2, 2>::from_rows([[1.0f32, 2.0f32], [3.0f32, 4.0f32]]);
    assert_eq!(row_count(&m), 2);
    assert_eq!(columns_count(&m), 2);
    expect_float_eq(m.at(0, 0), 1.0f32);
    expect_float_eq(m.at(0, 1), 2.0f32);
    expect_float_eq(m.at(1, 0), 3.0f32);
    expect_float_eq(m.at(1, 1), 4.0f32);
}

#[test]
fn operator_square_brackets() {
    let m2 = Mat::<2, 2>::from_rows([[1.0f32, 2.0f32], [3.0f32, 4.0f32]]);
    assert_eq!(m2[(0, 0)], 1.0f32);
    assert_eq!(m2[(0, 1)], 2.0f32);
    assert_eq!(m2[(1, 0)], 3.0f32);
    assert_eq!(m2[(1, 1)], 4.0f32);
}

#[test]
fn constructor_copy() {
    let m2 = Mat::<2, 2>::from_rows([[1.0f32, 2.0f32], [3.0f32, 4.0f32]]);

    let m3 = m2;
    assert_eq!(row_count(&m3), row_count(&m2));
    assert_eq!(columns_count(&m3), columns_count(&m2));
    expect_float_eq(m3.at(0, 0), m2.at(0, 0));
    expect_float_eq(m3.at(1, 1), m2.at(1, 1));
}

#[test]
fn constructor_move() {
    // Rust moves are the Copy of this trivially copyable type.
    let m2 = Mat::<2, 2>::from_rows([[1.0f32, 2.0f32], [3.0f32, 4.0f32]]);
    let m3 = m2;
    assert_eq!(row_count(&m3), 2);
    assert_eq!(columns_count(&m3), 2);
    expect_float_eq(m3.at(0, 0), 1.0f32);
    expect_float_eq(m3.at(1, 1), 4.0f32);
}

#[test]
fn operator_multiplication_matrix() {
    let m2 = Mat::<2, 2>::from_rows([[1.0f32, 2.0f32], [3.0f32, 4.0f32]]);

    let m3 = m2 * m2;
    assert_eq!(row_count(&m3), 2);
    assert_eq!(columns_count(&m3), 2);
    expect_float_eq(m3.at(0, 0), 7.0f32);
    expect_float_eq(m3.at(0, 1), 10.0f32);
    expect_float_eq(m3.at(1, 0), 15.0f32);
    expect_float_eq(m3.at(1, 1), 22.0f32);
}

#[test]
fn operator_multiplication_scalar() {
    let m2 = Mat::<2, 2>::from_rows([[1.0f32, 2.0f32], [3.0f32, 4.0f32]]);

    let m3 = m2 * 2.0f32;
    expect_float_eq(m3.at(0, 0), 2.0f32);
    expect_float_eq(m3.at(1, 1), 8.0f32);
}

#[test]
fn operator_division_scalar() {
    let m2 = Mat::<2, 2>::from_rows([[1.0f32, 2.0f32], [3.0f32, 4.0f32]]);

    let m3 = m2 / 2.0f32;
    expect_float_eq(m3.at(0, 0), 0.5f32);
    expect_float_eq(m3.at(1, 1), 2.0f32);
}

#[test]
fn transpose() {
    let m2 = Mat::<2, 2>::from_rows([[1.0f32, 2.0f32], [3.0f32, 4.0f32]]);

    let m3 = m2.transposed();
    expect_float_eq(m3.at(0, 0), m2.at(0, 0));
    expect_float_eq(m3.at(0, 1), m2.at(1, 0));
    expect_float_eq(m3.at(1, 0), m2.at(0, 1));
    expect_float_eq(m3.at(1, 1), m2.at(1, 1));
}

#[test]
fn determinant() {
    let m2 = Mat::<2, 2>::from_rows([[1.0f32, 2.0f32], [3.0f32, 4.0f32]]);

    let det: f32 = m2.determinant();
    expect_float_eq(det, -2.0f32);
}

#[test]
fn sum() {
    let m2 = Mat::<2, 2>::from_rows([[1.0f32, 2.0f32], [3.0f32, 4.0f32]]);

    let sum: f32 = m2.sum();
    expect_float_eq(sum, 10.0f32);
}

#[test]
fn clear() {
    let mut m2 = Mat::<2, 2>::from_rows([[1.0f32, 2.0f32], [3.0f32, 4.0f32]]);
    m2.clear();
    for i in 0..row_count(&m2) {
        for j in 0..columns_count(&m2) {
            expect_float_eq(m2.at(i, j), 0.0f32);
        }
    }
}

#[test]
fn to_string() {
    let m2 = Mat::<2, 2>::from_rows([[1.0f32, 2.0f32], [3.0f32, 4.0f32]]);
    let str = format!("{}", m2);
    assert!(!str.is_empty());
    assert_eq!(str, "[[    1.000,     2.000]\n [    3.000,     4.000]]");
}

#[test]
fn assignment_operator_copy() {
    let m2 = Mat::<2, 2>::from_rows([[1.0f32, 2.0f32], [3.0f32, 4.0f32]]);
    let mut m3 = Mat::<2, 2>::default();
    assert_eq!(m3.at(0, 0), 0.0f32);
    m3 = m2;
    assert_eq!(row_count(&m3), row_count(&m2));
    assert_eq!(columns_count(&m3), columns_count(&m2));
    expect_float_eq(m3.at(0, 0), m2.at(0, 0));
}

#[test]
fn assignment_operator_move() {
    let m2 = Mat::<2, 2>::from_rows([[1.0f32, 2.0f32], [3.0f32, 4.0f32]]);
    let mut m3 = Mat::<2, 2>::default();
    assert_eq!(m3.at(0, 0), 0.0f32);
    m3 = m2;
    assert_eq!(row_count(&m3), 2);
    assert_eq!(columns_count(&m3), 2);
    expect_float_eq(m3.at(0, 0), 1.0f32);
    expect_float_eq(m3.at(1, 1), 4.0f32);
}

#[test]
fn static_method_to_screen_mat() {
    let screen_mat = M4::to_screen(800.0f32, 600.0f32);
    expect_float_eq(screen_mat.at(0, 0), 400.0f32);
    expect_float_eq(screen_mat.at(1, 1), -300.0f32);
    expect_float_eq(screen_mat.at(3, 0), 400.0f32);
    expect_float_eq(screen_mat.at(3, 1), 300.0f32);
    expect_float_eq(screen_mat.at(3, 3), 1.0f32);
}

#[test]
#[should_panic]
fn method_at_out_of_range_row() {
    let m2 = Mat::<2, 2>::from_rows([[1.0f32, 2.0f32], [3.0f32, 4.0f32]]);
    let _ = m2.at(2, 0);
}

#[test]
#[should_panic]
fn method_at_out_of_range_column() {
    let m2 = Mat::<2, 2>::from_rows([[1.0f32, 2.0f32], [3.0f32, 4.0f32]]);
    let _ = m2.at(0, 2);
}

#[test]
fn operator_multiplication_row_major_simd_and_tail() {
    let left = Mat::<3, 5, f32, RowMajor>::from_fn(|row, column| (row * 3 + column + 1) as f32);
    let right =
        Mat::<5, 33, f32, RowMajor>::from_fn(|row, column| ((row + 1) * (column % 5 + 1)) as f32);

    expect_multiplication_matches_scalar_reference(left, right);
}

#[test]
fn operator_multiplication_column_major_simd_and_tail() {
    let left = Mat::<17, 5, f64, ColumnMajor>::from_fn(|row, column| (row * 3 + column + 1) as f64);
    let right =
        Mat::<5, 3, f64, ColumnMajor>::from_fn(|row, column| ((row + 1) * (column + 1)) as f64);

    expect_multiplication_matches_scalar_reference(left, right);
}

#[test]
fn operator_multiplication_integer_falls_back_from_avx() {
    let left = Mat::<2, 3, i32>::from_rows([[1, 2, 3], [4, 5, 6]]);
    let right = Mat::<3, 2, i32>::from_rows([[7, 8], [9, 10], [11, 12]]);
    let result = left * right;
    assert_eq!(result.at(0, 0), 58);
    assert_eq!(result.at(1, 1), 154);

    expect_multiplication_matches_scalar_reference(left, right);
}

#[test]
fn determinant_3x3() {
    let det = Mat::<3, 3>::from_rows([
        [6.0f32, 1.0f32, 1.0f32],
        [4.0f32, -2.0f32, 5.0f32],
        [2.0f32, 8.0f32, 7.0f32],
    ])
    .determinant();
    expect_float_eq(det, -306.0f32);
}

#[test]
fn strip_3x3() {
    let m = Mat::<3, 3>::from_rows([
        [3.0f32, 0.0f32, 2.0f32],
        [2.0f32, 0.0f32, -2.0f32],
        [0.0f32, 1.0f32, 1.0f32],
    ]);
    let minor: Mat<2, 2> = m.strip(0, 0);
    assert_eq!(row_count(&minor), 2);
    assert_eq!(columns_count(&minor), 2);
    expect_float_eq(minor.at(0, 0), 0.0f32);
    expect_float_eq(minor.at(0, 1), -2.0f32);
    expect_float_eq(minor.at(1, 0), 1.0f32);
    expect_float_eq(minor.at(1, 1), 1.0f32);
}

#[test]
fn transpose_non_square() {
    let m = Mat::<2, 3>::from_rows([[1.0f32, 2.0f32, 3.0f32], [4.0f32, 5.0f32, 6.0f32]]);
    let transposed = m.transposed();
    assert_eq!(row_count(&transposed), 3);
    assert_eq!(columns_count(&transposed), 2);
    expect_float_eq(transposed.at(0, 0), 1.0f32);
    expect_float_eq(transposed.at(1, 0), 2.0f32);
    expect_float_eq(transposed.at(2, 0), 3.0f32);
    expect_float_eq(transposed.at(0, 1), 4.0f32);
    expect_float_eq(transposed.at(1, 1), 5.0f32);
    expect_float_eq(transposed.at(2, 1), 6.0f32);
}

#[test]
fn enverse() {
    let m = Mat::<2, 2>::from_rows([[1.0f32, 3.0f32], [2.0f32, 5.0f32]]);
    let mv = Mat::<2, 2>::from_rows([[-5.0f32, 3.0f32], [2.0f32, -1.0f32]]);

    assert_eq!(Some(mv), m.inverted());
}

#[test]
fn inverted4x4_round_trip() {
    let k_affine = M4::from_rows(K_AFFINE);
    let inverse = k_affine.inverted();
    assert!(inverse.is_some());

    expect_identity(k_affine * inverse.unwrap(), 1e-5f32);
    expect_identity(inverse.unwrap() * k_affine, 1e-5f32);
}

#[test]
fn inverted4x4_column_major() {
    let mat = Mat::<4, 4, f32, ColumnMajor>::from_rows(K_AFFINE);

    let inverse = mat.inverted();
    assert!(inverse.is_some());

    expect_identity(mat * inverse.unwrap(), 1e-5f32);
}

#[test]
fn inverted4x4_view_projection() {
    let projection = M4::perspective_left_handed_vertical_fov(
        73.7f32,
        16.0f32 / 9.0f32,
        1.0f32,
        1000.0f32,
        NdcDepthRange::ZeroToOne,
    );
    let view = M4::camera_view(
        Vector3::new(0.936f32, 0.312f32, -0.160f32),
        Vector3::new(-0.289f32, 0.884f32, 0.369f32),
        Vector3::new(0.198f32, -0.348f32, 0.916f32),
        Vector3::new(128.0f32, -512.0f32, 64.0f32),
    );
    let view_projection = projection * view;

    let inverse = view_projection.inverted();
    assert!(inverse.is_some());

    // Looser than the affine case: a near/far spread of three orders of magnitude costs precision in float
    expect_identity(view_projection * inverse.unwrap(), 1e-3f32);
}

#[test]
fn inverted4x4_singular() {
    // Row 3 is row 1 doubled, so the determinant is zero
    let singular = Mat::<4, 4>::from_rows([
        [1.0f32, 2.0f32, 3.0f32, 4.0f32],
        [5.0f32, 6.0f32, 7.0f32, 8.0f32],
        [2.0f32, 4.0f32, 6.0f32, 8.0f32],
        [9.0f32, 1.0f32, 2.0f32, 3.0f32],
    ]);

    assert!(!(singular.inverted().is_some()));
}

#[test]
fn inverted3x3_round_trip() {
    let mat = Mat::<3, 3>::from_rows([
        [2.0f32, 0.0f32, 1.0f32],
        [1.0f32, 3.0f32, 2.0f32],
        [1.0f32, 1.0f32, 2.0f32],
    ]);

    let inverse = mat.inverted();
    assert!(inverse.is_some());

    expect_identity(mat * inverse.unwrap(), 1e-5f32);
}

#[test]
fn equanity() {
    let left_handed = Vector3::<f32>::new(0.0f32, 2.0f32, 10.0f32);
    let right_handed = Vector3::<f32>::new(0.0f32, 2.0f32, -10.0f32);

    let proj_left_handed = M4::perspective_left_handed_vertical_fov(
        90.0f32,
        16.0f32 / 9.0f32,
        0.1f32,
        1000.0f32,
        NdcDepthRange::NegativeOneToOne,
    );
    let proj_right_handed = M4::perspective_right_handed_vertical_fov(
        90.0f32,
        16.0f32 / 9.0f32,
        0.1f32,
        1000.0f32,
        NdcDepthRange::NegativeOneToOne,
    );

    let mut ndc_left_handed = proj_left_handed * col(left_handed);
    let mut ndc_right_handed = proj_right_handed * col(right_handed);

    ndc_left_handed /= ndc_left_handed.at(3, 0);
    ndc_right_handed /= ndc_right_handed.at(3, 0);

    assert_eq!(ndc_left_handed, ndc_right_handed);
}

#[test]
fn mat_perspective_left_handed() {
    let perspective_proj = M4::perspective_left_handed_vertical_fov(
        90.0f32,
        16.0f32 / 9.0f32,
        0.1f32,
        1000.0f32,
        NdcDepthRange::NegativeOneToOne,
    );
    let mut projected = perspective_proj * col(Vector3::new(0.0f32, 0.0f32, 0.1001f32));

    projected /= projected.at(3, 0);

    assert!(projected.at(2, 0) > -1.0f32 && projected.at(2, 0) < 0.0f32);
}

#[test]
fn mat_perspective_left_handed_zero_to_one() {
    let proj = M4::perspective_left_handed_vertical_fov(
        90.0f32,
        16.0f32 / 9.0f32,
        0.1f32,
        1000.0f32,
        NdcDepthRange::ZeroToOne,
    );

    // Near plane point should map to z ~ 0
    let mut near_pt = proj * col(Vector3::new(0.0f32, 0.0f32, 0.1f32));
    near_pt /= near_pt.at(3, 0);
    expect_near(near_pt.at(2, 0), 0.0f32, 1e-4f32);

    // Far plane point should map to z ~ 1
    let mut far_pt = proj * col(Vector3::new(0.0f32, 0.0f32, 1000.0f32));
    far_pt /= far_pt.at(3, 0);
    expect_near(far_pt.at(2, 0), 1.0f32, 1e-4f32);

    // Mid-range point should be in [0, 1]
    let mut mid_pt = proj * col(Vector3::new(0.0f32, 0.0f32, 500.0f32));
    mid_pt /= mid_pt.at(3, 0);
    assert!(mid_pt.at(2, 0) > 0.0f32);
    assert!(mid_pt.at(2, 0) < 1.0f32);
}

#[test]
fn mat_perspective_right_handed_zero_to_one() {
    let proj = M4::perspective_right_handed_vertical_fov(
        90.0f32,
        16.0f32 / 9.0f32,
        0.1f32,
        1000.0f32,
        NdcDepthRange::ZeroToOne,
    );

    // Near plane point (negative z for right-handed) should map to z ~ 0
    let mut near_pt = proj * col(Vector3::new(0.0f32, 0.0f32, -0.1f32));
    near_pt /= near_pt.at(3, 0);
    expect_near(near_pt.at(2, 0), 0.0f32, 1e-4f32);

    // Far plane point should map to z ~ 1
    let mut far_pt = proj * col(Vector3::new(0.0f32, 0.0f32, -1000.0f32));
    far_pt /= far_pt.at(3, 0);
    expect_near(far_pt.at(2, 0), 1.0f32, 1e-4f32);

    // Mid-range point should be in [0, 1]
    let mut mid_pt = proj * col(Vector3::new(0.0f32, 0.0f32, -500.0f32));
    mid_pt /= mid_pt.at(3, 0);
    assert!(mid_pt.at(2, 0) > 0.0f32);
    assert!(mid_pt.at(2, 0) < 1.0f32);
}

#[test]
fn mat_perspective_negative_one_to_one_range() {
    // Verify existing [-1, 1] behavior with explicit template arg matches default
    let proj_default = M4::perspective_left_handed_vertical_fov(
        90.0f32,
        16.0f32 / 9.0f32,
        0.1f32,
        1000.0f32,
        NdcDepthRange::NegativeOneToOne,
    );
    let proj_explicit = M4::perspective_left_handed_vertical_fov(
        90.0f32,
        16.0f32 / 9.0f32,
        0.1f32,
        1000.0f32,
        NdcDepthRange::NegativeOneToOne,
    );

    assert_eq!(proj_default, proj_explicit);

    // Near plane should map to z ~ -1
    let mut near_pt = proj_default * col(Vector3::new(0.0f32, 0.0f32, 0.1f32));
    near_pt /= near_pt.at(3, 0);
    expect_near(near_pt.at(2, 0), -1.0f32, 1e-3f32);

    // Far plane should map to z ~ 1
    let mut far_pt = proj_default * col(Vector3::new(0.0f32, 0.0f32, 1000.0f32));
    far_pt /= far_pt.at(3, 0);
    expect_near(far_pt.at(2, 0), 1.0f32, 1e-3f32);
}

#[test]
fn mat_perspective_right_handed_neg_one_to_one() {
    let proj = M4::perspective_right_handed_vertical_fov(
        90.0f32,
        16.0f32 / 9.0f32,
        0.1f32,
        1000.0f32,
        NdcDepthRange::NegativeOneToOne,
    );

    // Near plane (negative z for RH) should map to z ~ -1
    let mut near_pt = proj * col(Vector3::new(0.0f32, 0.0f32, -0.1f32));
    near_pt /= near_pt.at(3, 0);
    expect_near(near_pt.at(2, 0), -1.0f32, 1e-3f32);

    // Far plane should map to z ~ 1
    let mut far_pt = proj * col(Vector3::new(0.0f32, 0.0f32, -1000.0f32));
    far_pt /= far_pt.at(3, 0);
    expect_near(far_pt.at(2, 0), 1.0f32, 1e-3f32);

    // Mid-range point should be in (-1, 1)
    let mut mid_pt = proj * col(Vector3::new(0.0f32, 0.0f32, -500.0f32));
    mid_pt /= mid_pt.at(3, 0);
    assert!(mid_pt.at(2, 0) > -1.0f32);
    assert!(mid_pt.at(2, 0) < 1.0f32);
}

#[test]
fn mat_perspective_left_handed_horizontal_fov_zero_to_one() {
    // hfov=90 deg, aspect=16/9 => tan(hfov/2)=1, so x_axis=1 and y_axis=aspect
    let proj = M4::perspective_left_handed_horizontal_fov(
        90.0f32,
        16.0f32 / 9.0f32,
        0.1f32,
        1000.0f32,
        NdcDepthRange::ZeroToOne,
    );

    // Near plane should map to z ~ 0
    let mut near_pt = proj * col(Vector3::new(0.0f32, 0.0f32, 0.1f32));
    near_pt /= near_pt.at(3, 0);
    expect_near(near_pt.at(2, 0), 0.0f32, 1e-4f32);

    // Far plane should map to z ~ 1
    let mut far_pt = proj * col(Vector3::new(0.0f32, 0.0f32, 1000.0f32));
    far_pt /= far_pt.at(3, 0);
    expect_near(far_pt.at(2, 0), 1.0f32, 1e-4f32);

    // Right edge of horizontal frustum at near plane (view_x = tan(hfov/2)*near = 0.1)
    let mut right_edge = proj * col(Vector3::new(0.1f32, 0.0f32, 0.1f32));
    right_edge /= right_edge.at(3, 0);
    expect_near(right_edge.at(0, 0), 1.0f32, 1e-4f32);
}

#[test]
fn mat_perspective_left_handed_horizontal_fov_neg_one_to_one() {
    let proj = M4::perspective_left_handed_horizontal_fov(
        90.0f32,
        16.0f32 / 9.0f32,
        0.1f32,
        1000.0f32,
        NdcDepthRange::NegativeOneToOne,
    );

    let mut near_pt = proj * col(Vector3::new(0.0f32, 0.0f32, 0.1f32));
    near_pt /= near_pt.at(3, 0);
    expect_near(near_pt.at(2, 0), -1.0f32, 1e-3f32);

    let mut far_pt = proj * col(Vector3::new(0.0f32, 0.0f32, 1000.0f32));
    far_pt /= far_pt.at(3, 0);
    expect_near(far_pt.at(2, 0), 1.0f32, 1e-3f32);

    let mut right_edge = proj * col(Vector3::new(0.1f32, 0.0f32, 0.1f32));
    right_edge /= right_edge.at(3, 0);
    expect_near(right_edge.at(0, 0), 1.0f32, 1e-4f32);
}

#[test]
fn mat_perspective_right_handed_horizontal_fov_zero_to_one() {
    let proj = M4::perspective_right_handed_horizontal_fov(
        90.0f32,
        16.0f32 / 9.0f32,
        0.1f32,
        1000.0f32,
        NdcDepthRange::ZeroToOne,
    );

    let mut near_pt = proj * col(Vector3::new(0.0f32, 0.0f32, -0.1f32));
    near_pt /= near_pt.at(3, 0);
    expect_near(near_pt.at(2, 0), 0.0f32, 1e-4f32);

    let mut far_pt = proj * col(Vector3::new(0.0f32, 0.0f32, -1000.0f32));
    far_pt /= far_pt.at(3, 0);
    expect_near(far_pt.at(2, 0), 1.0f32, 1e-4f32);

    let mut right_edge = proj * col(Vector3::new(0.1f32, 0.0f32, -0.1f32));
    right_edge /= right_edge.at(3, 0);
    expect_near(right_edge.at(0, 0), 1.0f32, 1e-4f32);
}

#[test]
fn mat_perspective_right_handed_horizontal_fov_neg_one_to_one() {
    let proj = M4::perspective_right_handed_horizontal_fov(
        90.0f32,
        16.0f32 / 9.0f32,
        0.1f32,
        1000.0f32,
        NdcDepthRange::NegativeOneToOne,
    );

    let mut near_pt = proj * col(Vector3::new(0.0f32, 0.0f32, -0.1f32));
    near_pt /= near_pt.at(3, 0);
    expect_near(near_pt.at(2, 0), -1.0f32, 1e-3f32);

    let mut far_pt = proj * col(Vector3::new(0.0f32, 0.0f32, -1000.0f32));
    far_pt /= far_pt.at(3, 0);
    expect_near(far_pt.at(2, 0), 1.0f32, 1e-3f32);

    let mut right_edge = proj * col(Vector3::new(0.1f32, 0.0f32, -0.1f32));
    right_edge /= right_edge.at(3, 0);
    expect_near(right_edge.at(0, 0), 1.0f32, 1e-4f32);
}

#[test]
fn mat_perspective_horizontal_vs_vertical_fov_equivalence() {
    let hfov_deg = 90.0f32;
    let aspect = 16.0f32 / 9.0f32;
    let vfov_deg = angles::horizontal_fov_to_vertical(hfov_deg, aspect);

    let proj_h = M4::perspective_left_handed_horizontal_fov(
        hfov_deg,
        aspect,
        0.1f32,
        1000.0f32,
        NdcDepthRange::NegativeOneToOne,
    );
    let proj_v = M4::perspective_left_handed_vertical_fov(
        vfov_deg,
        aspect,
        0.1f32,
        1000.0f32,
        NdcDepthRange::NegativeOneToOne,
    );

    for i in 0..4 {
        for j in 0..4 {
            expect_near(proj_h.at(i, j), proj_v.at(i, j), 1e-4f32);
        }
    }
}

#[test]
fn mat_perspective_left_handed_vertical_fov_handedness() {
    let proj = M4::perspective_left_handed_vertical_fov(
        90.0f32,
        16.0f32 / 9.0f32,
        0.1f32,
        1000.0f32,
        NdcDepthRange::NegativeOneToOne,
    );

    let in_front = proj * col(Vector3::new(0.0f32, 0.0f32, 1.0f32));
    let behind = proj * col(Vector3::new(0.0f32, 0.0f32, -1.0f32));

    assert!(in_front.at(3, 0) > 0.0f32);
    assert!(behind.at(3, 0) < 0.0f32);
}

#[test]
fn mat_perspective_right_handed_vertical_fov_handedness() {
    let proj = M4::perspective_right_handed_vertical_fov(
        90.0f32,
        16.0f32 / 9.0f32,
        0.1f32,
        1000.0f32,
        NdcDepthRange::NegativeOneToOne,
    );

    let in_front = proj * col(Vector3::new(0.0f32, 0.0f32, -1.0f32));
    let behind = proj * col(Vector3::new(0.0f32, 0.0f32, 1.0f32));

    assert!(in_front.at(3, 0) > 0.0f32);
    assert!(behind.at(3, 0) < 0.0f32);
}

#[test]
fn mat_perspective_left_handed_horizontal_fov_handedness() {
    let proj = M4::perspective_left_handed_horizontal_fov(
        90.0f32,
        16.0f32 / 9.0f32,
        0.1f32,
        1000.0f32,
        NdcDepthRange::NegativeOneToOne,
    );

    let in_front = proj * col(Vector3::new(0.0f32, 0.0f32, 1.0f32));
    let behind = proj * col(Vector3::new(0.0f32, 0.0f32, -1.0f32));

    assert!(in_front.at(3, 0) > 0.0f32);
    assert!(behind.at(3, 0) < 0.0f32);
}

#[test]
fn mat_perspective_right_handed_horizontal_fov_handedness() {
    let proj = M4::perspective_right_handed_horizontal_fov(
        90.0f32,
        16.0f32 / 9.0f32,
        0.1f32,
        1000.0f32,
        NdcDepthRange::NegativeOneToOne,
    );

    let in_front = proj * col(Vector3::new(0.0f32, 0.0f32, -1.0f32));
    let behind = proj * col(Vector3::new(0.0f32, 0.0f32, 1.0f32));

    assert!(in_front.at(3, 0) > 0.0f32);
    assert!(behind.at(3, 0) < 0.0f32);
}

#[test]
fn mat_perspective_zero_to_one_equanity() {
    // LH and RH should produce same NDC for mirrored z
    let left_handed = Vector3::<f32>::new(0.0f32, 2.0f32, 10.0f32);
    let right_handed = Vector3::<f32>::new(0.0f32, 2.0f32, -10.0f32);

    let proj_lh = M4::perspective_left_handed_vertical_fov(
        90.0f32,
        16.0f32 / 9.0f32,
        0.1f32,
        1000.0f32,
        NdcDepthRange::ZeroToOne,
    );
    let proj_rh = M4::perspective_right_handed_vertical_fov(
        90.0f32,
        16.0f32 / 9.0f32,
        0.1f32,
        1000.0f32,
        NdcDepthRange::ZeroToOne,
    );

    let mut ndc_lh = proj_lh * col(left_handed);
    let mut ndc_rh = proj_rh * col(right_handed);

    ndc_lh /= ndc_lh.at(3, 0);
    ndc_rh /= ndc_rh.at(3, 0);

    assert_eq!(ndc_lh, ndc_rh);
}

#[test]
fn mat_ortho_left_handed_zero_to_one() {
    let ortho = M4::ortho_left_handed(
        -1.0f32,
        1.0f32,
        -1.0f32,
        1.0f32,
        0.1f32,
        100.0f32,
        NdcDepthRange::ZeroToOne,
    );

    // Near plane should map to z ~ 0
    let near_pt = ortho * col(Vector3::new(0.0f32, 0.0f32, 0.1f32));
    expect_near(near_pt.at(2, 0), 0.0f32, 1e-4f32);

    // Far plane should map to z ~ 1
    let far_pt = ortho * col(Vector3::new(0.0f32, 0.0f32, 100.0f32));
    expect_near(far_pt.at(2, 0), 1.0f32, 1e-4f32);
}

#[test]
fn mat_ortho_right_handed_zero_to_one() {
    let ortho = M4::ortho_right_handed(
        -1.0f32,
        1.0f32,
        -1.0f32,
        1.0f32,
        0.1f32,
        100.0f32,
        NdcDepthRange::ZeroToOne,
    );

    // Near plane (negative z for RH) should map to z ~ 0
    let near_pt = ortho * col(Vector3::new(0.0f32, 0.0f32, -0.1f32));
    expect_near(near_pt.at(2, 0), 0.0f32, 1e-4f32);

    // Far plane should map to z ~ 1
    let far_pt = ortho * col(Vector3::new(0.0f32, 0.0f32, -100.0f32));
    expect_near(far_pt.at(2, 0), 1.0f32, 1e-4f32);
}

#[test]
fn mat_ortho_negative_one_to_one_default() {
    // Verify explicit [-1, 1] matches default
    let ortho_default = M4::ortho_left_handed(
        -1.0f32,
        1.0f32,
        -1.0f32,
        1.0f32,
        0.1f32,
        100.0f32,
        NdcDepthRange::NegativeOneToOne,
    );
    let ortho_explicit = M4::ortho_left_handed(
        -1.0f32,
        1.0f32,
        -1.0f32,
        1.0f32,
        0.1f32,
        100.0f32,
        NdcDepthRange::NegativeOneToOne,
    );

    assert_eq!(ortho_default, ortho_explicit);
}

#[test]
fn to_screen_mat_keeps_scalar_type() {
    let screen: Mat<4, 4, f64> = Mat::to_screen(800.0, 600.0);
    expect_double_eq(screen.at(0, 0), 400.0);
    expect_double_eq(screen.at(1, 1), -300.0);
    expect_double_eq(screen.at(3, 0), 400.0);
    expect_double_eq(screen.at(3, 1), 300.0);

    let column_major = Mat::<4, 4, f32, ColumnMajor>::to_screen(800.0f32, 600.0f32);
    assert_eq!(
        Mat::<4, 4, f32, ColumnMajor>::store_ordering(),
        MatStoreType::ColumnMajor
    );
    expect_float_eq(column_major.at(3, 1), 300.0f32);
}

#[test]
fn is_trivially_copyable() {
    // MatTemplateEqual is a C++ concept; Rust's type system compares the parameters on its own.
    fn assert_copy<T: Copy>() {}
    assert_copy::<Mat<4, 4>>();
    assert_copy::<Mat<3, 2, f64, ColumnMajor>>();
    assert_eq!(Mat::<2, 3>::row_count(), 2);
    assert_eq!(Mat::<2, 3>::columns_count(), 3);
}

#[test]
fn in_place_multiply_returns_self() {
    // Rust's `*=` returns (), so only the value half of the C++ test exists here.
    let mut a = Mat::<2, 2>::from_rows([[1.0f32, 2.0f32], [3.0f32, 4.0f32]]);
    let b = Mat::<2, 2>::from_rows([[0.0f32, 1.0f32], [1.0f32, 0.0f32]]);

    a *= b;
    assert_eq!(
        a,
        Mat::<2, 2>::from_rows([[2.0f32, 1.0f32], [4.0f32, 3.0f32]])
    );
}

#[test]
fn equality_is_constexpr() {
    let a = Mat::<2, 2>::from_rows([[1.0f32, 2.0f32], [3.0f32, 4.0f32]]);
    let b = Mat::<2, 2>::from_rows([[1.0f32, 2.0f32], [3.0f32, 4.0f32]]);
    let c = Mat::<2, 2>::from_rows([[1.0f32, 2.0f32], [3.0f32, 5.0f32]]);
    assert!(a == b);
    assert!(a != c);
}

#[test]
fn to_string_formatting() {
    let a = Mat::<2, 2>::from_rows([[1.0f32, 2.0f32], [3.0f32, 4.0f32]]);
    assert_eq!(
        a.to_string(),
        "[[    1.000,     2.000]\n [    3.000,     4.000]]"
    );
    assert_eq!(format!("{}", a), a.to_string());
}

#[test]
fn rotation_yxz_and_zxy_match_the_axis_products() {
    for a in [-150.0f32, -40.0f32, 0.0f32, 25.0f32, 100.0f32] {
        for b in [-80.0f32, -15.0f32, 0.0f32, 60.0f32] {
            for c in [-120.0f32, 0.0f32, 35.0f32, 170.0f32] {
                let first = any(a);
                let pitch = any(b);
                let last = any(c);

                expect_mat3_near(
                    M4::rotation_yxz(first, pitch, last),
                    M4::rotation_axis_y(first)
                        * M4::rotation_axis_x(pitch)
                        * M4::rotation_axis_z(last),
                );
                expect_mat3_near(
                    M4::rotation_zxy(first, pitch, last),
                    M4::rotation_axis_z(first)
                        * M4::rotation_axis_x(pitch)
                        * M4::rotation_axis_y(last),
                );
            }
        }
    }
}

#[test]
fn extract_rotation_yxz_and_zxy_round_trip() {
    // x (the pitch) stays within [-90, 90]; the outer angles cover the full turn
    for x in [-80.0f32, -30.0f32, 0.0f32, 45.0f32, 85.0f32] {
        for outer in [-170.0f32, -60.0f32, 0.0f32, 100.0f32, 179.0f32] {
            for inner in [-150.0f32, -20.0f32, 0.0f32, 70.0f32, 160.0f32] {
                let yxz = M4::rotation_yxz(any(outer), any(x), any(inner)).extract_rotation_yxz();
                expect_near(yxz.x, x, 1e-3f32);
                expect_near(angle_difference(yxz.y, outer), 0.0f32, 1e-3f32);
                expect_near(angle_difference(yxz.z, inner), 0.0f32, 1e-3f32);

                let zxy = M4::rotation_zxy(any(outer), any(x), any(inner)).extract_rotation_zxy();
                expect_near(zxy.x, x, 1e-3f32);
                expect_near(angle_difference(zxy.z, outer), 0.0f32, 1e-3f32);
                expect_near(angle_difference(zxy.y, inner), 0.0f32, 1e-3f32);
            }
        }
    }
}

#[test]
fn extract_rotation_yxz_and_zxy_at_the_singularity_keep_the_rotation() {
    for x in [-90.0f32, 90.0f32] {
        let yxz_rotation = M4::rotation_yxz(any(40.0f32), any(x), any(25.0f32));
        let yxz = yxz_rotation.extract_rotation_yxz();
        expect_float_eq(yxz.z, 0.0f32);
        expect_mat3_near(
            M4::rotation_yxz(any(yxz.y), any(yxz.x), any(yxz.z)),
            yxz_rotation,
        );

        let zxy_rotation = M4::rotation_zxy(any(40.0f32), any(x), any(25.0f32));
        let zxy = zxy_rotation.extract_rotation_zxy();
        expect_float_eq(zxy.y, 0.0f32);
        expect_mat3_near(
            M4::rotation_zxy(any(zxy.z), any(zxy.x), any(zxy.y)),
            zxy_rotation,
        );
    }
}

// The file-scope static_asserts of the C++ test, as runtime checks.
#[test]
fn static_assert_inverse_round_trip() {
    let k_affine = M4::from_rows(K_AFFINE);
    let inverse = k_affine.inverted().unwrap();
    let product = k_affine * inverse;
    for row in 0..4 {
        for column in 0..4 {
            assert!(close_to(
                product.at(row, column),
                if row == column { 1.0f32 } else { 0.0f32 },
                1e-5f32
            ));
        }
    }
}

#[test]
fn static_assert_extract_scale() {
    let scale = M4::scale(Vector3::new(2.0f32, 3.0f32, 4.0f32)).extract_scale();
    assert!(
        close_to(scale.x, 2.0f32, 1e-5f32)
            && close_to(scale.y, 3.0f32, 1e-5f32)
            && close_to(scale.z, 4.0f32, 1e-5f32)
    );
}

#[test]
fn static_assert_rotation_axis_z() {
    let rotation = M4::rotation_axis_z(Pitch::from_degrees(90.0f32));
    assert!(
        close_to(rotation.at(0, 0), 0.0f32, 1e-5f32)
            && close_to(rotation.at(0, 1), -1.0f32, 1e-5f32)
    );
    assert!(
        close_to(rotation.at(1, 0), 1.0f32, 1e-5f32)
            && close_to(rotation.at(1, 1), 0.0f32, 1e-5f32)
    );
    assert!(
        close_to(rotation.at(2, 2), 1.0f32, 1e-5f32)
            && close_to(rotation.at(3, 3), 1.0f32, 1e-5f32)
    );
}

#[test]
fn static_assert_vertical_fov_perspective() {
    let projection = M4::perspective_left_handed_vertical_fov(
        90.0f32,
        1.0f32,
        1.0f32,
        11.0f32,
        NdcDepthRange::ZeroToOne,
    );
    assert!(
        close_to(projection.at(0, 0), 1.0f32, 1e-5f32)
            && close_to(projection.at(1, 1), 1.0f32, 1e-5f32)
    );
    assert!(
        close_to(projection.at(2, 2), 1.1f32, 1e-5f32)
            && close_to(projection.at(2, 3), -1.1f32, 1e-5f32)
    );
    assert!(close_to(projection.at(3, 2), 1.0f32, 1e-5f32));
}

#[test]
fn static_assert_horizontal_fov_perspective() {
    let projection = M4::perspective_right_handed_horizontal_fov(
        90.0f32,
        2.0f32,
        1.0f32,
        11.0f32,
        NdcDepthRange::ZeroToOne,
    );
    assert!(
        close_to(projection.at(0, 0), 1.0f32, 1e-5f32)
            && close_to(projection.at(1, 1), 2.0f32, 1e-5f32)
    );
    assert!(
        close_to(projection.at(2, 2), -1.1f32, 1e-5f32)
            && close_to(projection.at(2, 3), -1.1f32, 1e-5f32)
    );
    assert!(close_to(projection.at(3, 2), -1.0f32, 1e-5f32));
}
