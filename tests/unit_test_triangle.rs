mod common;

use common::*;
use omath::{Triangle, Vector3};

fn close_to(actual: f32, expected: f32, epsilon: f32) -> bool {
    (actual - expected).abs() <= epsilon
}

#[test]
fn constructor() {
    let t = Triangle::new(
        Vector3::new(1.0f32, 2.0f32, 3.0f32),
        Vector3::new(4.0f32, 5.0f32, 6.0f32),
        Vector3::new(7.0f32, 8.0f32, 9.0f32),
    );

    expect_float_eq(t.vertex1.x, 1.0f32);
    expect_float_eq(t.vertex1.y, 2.0f32);
    expect_float_eq(t.vertex1.z, 3.0f32);

    expect_float_eq(t.vertex2.x, 4.0f32);
    expect_float_eq(t.vertex2.y, 5.0f32);
    expect_float_eq(t.vertex2.z, 6.0f32);

    expect_float_eq(t.vertex3.x, 7.0f32);
    expect_float_eq(t.vertex3.y, 8.0f32);
    expect_float_eq(t.vertex3.z, 9.0f32);
}

#[test]
fn calculate_normal() {
    let t1 = Triangle::new(
        Vector3::new(0.0f32, 0.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
    );
    let t3 = Triangle::new(
        Vector3::new(0.0f32, 0.0, 0.0),
        Vector3::new(2.0, 0.0, 0.0),
        Vector3::new(0.0, 2.0, 0.0),
    );

    // For t1, the normal should point in the +Z direction (0, 0, 1) or (0, 0, -1)
    let normal_t1 = t1.calculate_normal();
    // Check if it's normalized and pointed along Z (sign can differ, so use absolute check)
    expect_near((normal_t1.z).abs(), 1.0f32, 1e-5f32);
    expect_near(normal_t1.length(), 1.0f32, 1e-5f32);

    // For t3, we expect the normal to be along +Z as well
    let normal_t3 = t3.calculate_normal();
    expect_near((normal_t3.z).abs(), 1.0f32, 1e-5f32);
}

#[test]
fn side_lengths() {
    let t1 = Triangle::new(
        Vector3::new(0.0f32, 0.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
    );
    let t3 = Triangle::new(
        Vector3::new(0.0f32, 0.0, 0.0),
        Vector3::new(2.0, 0.0, 0.0),
        Vector3::new(0.0, 2.0, 0.0),
    );

    // For t1 side lengths
    expect_float_eq(t1.side_a_length(), (1.0f32).sqrt());
    expect_float_eq(t1.side_b_length(), (1.0f32 + 1.0f32).sqrt());
    // Actually, for t1: vertex2=(1,0,0), vertex3=(0,1,0)
    // Dist between (0,1,0) and (1,0,0) = sqrt((1-0)^2 + (0-1)^2) = sqrt(1 + 1) = sqrt(2)
    expect_float_eq(t1.side_b_length(), (2.0f32).sqrt());

    // For t3, side a = distance between vertex1=(0,0,0) and vertex2=(2,0,0), which is 2
    // side b = distance between vertex3=(0,2,0) and vertex2=(2,0,0), which is sqrt(2^2 + (-2)^2)= sqrt(8)= 2.828...
    // We'll just check side a first:
    expect_float_eq(t3.side_a_length(), 2.0f32);
    // Then side b:
    expect_float_eq(t3.side_b_length(), (8.0f32).sqrt());
}

#[test]
fn side_vectors() {
    let t1 = Triangle::new(
        Vector3::new(0.0f32, 0.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
    );

    let side_a_t1 = t1.side_a_vector();
    expect_float_eq(side_a_t1.x, 0.0f32 - 1.0f32);
    expect_float_eq(side_a_t1.y, 0.0f32 - 0.0f32);
    expect_float_eq(side_a_t1.z, 0.0f32 - 0.0f32);

    let side_b_t1 = t1.side_b_vector();
    expect_float_eq(side_b_t1.x, 0.0f32 - 1.0f32);
    expect_float_eq(side_b_t1.y, 1.0f32 - 0.0f32);
    expect_float_eq(side_b_t1.z, 0.0f32 - 0.0f32);
}

#[test]
fn is_rectangular() {
    assert!(
        Triangle::new(
            Vector3::new(2.0f32, 0.0, 0.0),
            Vector3::default(),
            Vector3::new(0.0, 2.0, 0.0)
        )
        .is_rectangular()
    );
}

#[test]
fn mid_point() {
    let t1 = Triangle::new(
        Vector3::new(0.0f32, 0.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
    );
    let t2 = Triangle::new(
        Vector3::new(1.0f32, 2.0, 3.0),
        Vector3::new(4.0, 5.0, 6.0),
        Vector3::new(7.0, 8.0, 9.0),
    );

    // For t1, midpoint of (0,0,0), (1,0,0), (0,1,0)
    let mid1 = t1.mid_point();
    expect_float_eq(mid1.x, (0.0f32 + 1.0f32 + 0.0f32) / 3.0f32);
    expect_float_eq(mid1.y, (0.0f32 + 0.0f32 + 1.0f32) / 3.0f32);
    expect_float_eq(mid1.z, 0.0f32);

    // For t2, midpoint of (1,2,3), (4,5,6), (7,8,9)
    let mid2 = t2.mid_point();
    expect_float_eq(mid2.x, (1.0f32 + 4.0f32 + 7.0f32) / 3.0f32);
    expect_float_eq(mid2.y, (2.0f32 + 5.0f32 + 8.0f32) / 3.0f32);
    expect_float_eq(mid2.z, (3.0f32 + 6.0f32 + 9.0f32) / 3.0f32);
}

// The file-scope static_assert of the C++ test, as a runtime check.
#[test]
fn static_assert_helpers() {
    let triangle = Triangle::new(
        Vector3::new(3.0f32, 0.0f32, 0.0f32),
        Vector3::new(0.0f32, 0.0f32, 0.0f32),
        Vector3::new(0.0f32, 4.0f32, 0.0f32),
    );
    let normal = triangle.calculate_normal();
    let mid_point = triangle.mid_point();

    assert!(close_to(triangle.side_a_length(), 3.0f32, 1e-5f32));
    assert!(close_to(triangle.side_b_length(), 4.0f32, 1e-5f32));
    assert!(close_to(triangle.hypot(), 5.0f32, 1e-5f32));
    assert!(triangle.is_rectangular());
    assert!(close_to(normal.length(), 1.0f32, 1e-5f32));
    assert!(close_to(normal.z, -1.0f32, 1e-5f32));
    assert!(close_to(mid_point.x, 1.0f32, 1e-5f32));
    assert!(close_to(mid_point.y, 4.0f32 / 3.0f32, 1e-5f32));
    assert!(close_to(mid_point.z, 0.0f32, 1e-5f32));
}
