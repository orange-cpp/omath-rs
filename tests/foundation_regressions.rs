//! Not ported from C++: a test for each foundation bug found while comparing `src/` against the headers, plus a dense
//! 4x4 inverse. Every C++ inverse fixture is affine or perspective, which leaves `bot_02` at zero.

use omath::{Angle, Mat, Triangle, Vector, Vector2, Vector3, Vector4, angle_range};

angle_range!(TurnRange, -180.0, 180.0, Normalized);
type Turn = Angle<f32, TurnRange>;

#[test]
fn vector2_normalized_keeps_a_vector_of_nan_length() {
    // `len > 0` in vector2.hpp, unlike the `len != 0` of Vector3 and Vector4.
    let normalized = Vector2::new(f32::NAN, 1.0f32).normalized();
    assert!(normalized.x.is_nan());
    assert_eq!(normalized.y, 1.0f32);

    assert!(Vector3::new(f32::NAN, 1.0f32, 0.0f32).normalized().y.is_nan());
    assert!(Vector4::new(f32::NAN, 1.0f32, 0.0f32, 0.0f32).normalized().y.is_nan());
}

#[test]
fn vector2_distance_to_is_the_square_root_of_distance_to_sqr() {
    // vector2.hpp uses sqrt(distance_to_sqr()) here, which overflows where length() does not.
    let far = Vector2::new(3e19f32, 0.0f32);
    assert!(far.distance_to(Vector2::default()).is_infinite());
    assert_eq!(far.length(), 3e19f32);
}

#[test]
fn trait_length_and_distance_match_the_inherent_methods() {
    fn generic<V: Vector<Scalar = f32>>(a: V, b: V) -> (f32, f32) {
        (a.length(), a.distance_to(b))
    }

    let far = Vector3::new(3e19f32, 0.0f32, 0.0f32);
    assert_eq!(generic(far, Vector3::default()), (far.length(), far.distance_to(Vector3::default())));

    let triangle = Triangle::new(far, Vector3::default(), Vector3::new(0.0f32, 1.0f32, 0.0f32));
    assert_eq!(triangle.side_a_length(), 3e19f32);
}

#[test]
fn angle_add_wraps_twice_like_operator_plus() {
    // The first wrap lands on 180 through rounding, which is outside [-180, 180).
    let sum = Turn::from_degrees(-180.0f32) + Turn::from_degrees(-1.5258789e-5f32);
    assert_eq!(sum.as_degrees(), -180.0f32);
}

#[test]
fn triangle_is_rectangular_uses_the_widened_float_tolerance() {
    // `<= 0.0001f` in triangle.hpp: for double the tolerance is 0.0000999999975, not 0.0001. With vertex2 at the origin
    // the tested value is twice the dot product of the sides, here 0.000099999998.
    let triangle = Triangle::new(
        Vector3::new(1.0f64, 0.0, 0.0),
        Vector3::default(),
        Vector3::new(0.000049999999f64, 1.0, 0.0),
    );

    assert!(!triangle.is_rectangular());
}

#[test]
fn inverted_4x4_dense_round_trip() {
    let mat = Mat::<4, 4, f64>::from_rows([
        [2.0, -1.0, 3.0, 0.5],
        [1.5, 4.0, -2.0, 1.0],
        [-3.0, 0.25, 1.0, 2.0],
        [0.75, 2.0, -1.0, 3.0],
    ]);

    for product in [mat * mat.inverted().unwrap(), mat.inverted().unwrap() * mat] {
        for row in 0..4 {
            for column in 0..4 {
                let expected = if row == column { 1.0 } else { 0.0 };
                assert!((product.at(row, column) - expected).abs() < 1e-12);
            }
        }
    }
}
