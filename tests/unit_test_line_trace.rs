//! Unit tests for the segment-based Möller–Trumbore line tracer.

use omath::collision::{Ray, can_trace_line, get_ray_hit_point};
use omath::{Triangle, Vector3};

type Vec3 = Vector3<f32>;

const K_TOL: f32 = 1e-5;

fn vec_equal(a: Vec3, b: Vec3) -> bool {
    (a.x - b.x).abs() < K_TOL && (a.y - b.y).abs() < K_TOL && (a.z - b.z).abs() < K_TOL
}

fn ray(start: (f32, f32, f32), end: (f32, f32, f32)) -> Ray {
    Ray {
        start: Vec3::new(start.0, start.1, start.2),
        end: Vec3::new(end.0, end.1, end.2),
        infinite_length: false,
    }
}

// One canonical right-angled triangle in the XY plane.
const fn triangle() -> Triangle<Vec3> {
    Triangle::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0))
}

#[test]
fn various_rays() {
    // true => segment does NOT hit the triangle
    let cases = [
        (ray((0.3, 0.3, -1.0), (0.3, 0.3, 1.0)), false), // hit through centre
        (ray((0.3, 0.3, 1.0), (0.3, 0.3, 2.0)), true),   // parallel above
        (ray((0.3, 0.3, 0.0), (0.3, 0.3, -1.0)), true),  // starts inside, goes away
        (ray((2.0, 2.0, -1.0), (2.0, 2.0, 1.0)), true),  // misses entirely
        (ray((-1.0, -1.0, 0.0), (1.5, 1.5, 0.0)), true), // lies in plane, outside tri
        (ray((-1.0, -1.0, -1.0), (0.0, 0.0, 0.0)), true), // endpoint on vertex
        (ray((-1.0, 0.0, -1.0), (0.5, 0.0, 0.0)), true), // endpoint on edge
    ];

    for (i, (ray, expected_clear)) in cases.iter().enumerate() {
        assert_eq!(can_trace_line(ray, &triangle()), *expected_clear, "case {i}");
    }
}

// Validate that the reported hit point is correct for a genuine intersection.
#[test]
fn hit_point_correct() {
    let ray = ray((0.3, 0.3, -1.0), (0.3, 0.3, 1.0));
    let expected = Vec3::new(0.3, 0.3, 0.0);

    let hit = get_ray_hit_point(&ray, &triangle());
    assert!(!vec_equal(hit, ray.end));
    assert!(vec_equal(hit, expected));
}

// Triangle far beyond the ray should not block.
#[test]
fn distant_triangle_clear() {
    let short_ray = ray((0.0, 0.0, 0.0), (0.0, 0.0, 1.0));
    let distant = Triangle::new(
        Vec3::new(1000.0, 1000.0, 1000.0),
        Vec3::new(1001.0, 1000.0, 1000.0),
        Vec3::new(1000.0, 1001.0, 1000.0),
    );

    assert!(can_trace_line(&short_ray, &distant));
}

#[test]
fn cant_hit() {
    let triangle = Triangle::new(Vec3::new(2.0, 0.0, 0.0), Vec3::new(2.0, 2.0, 0.0), Vec3::new(2.0, 2.0, 2.0));

    let ray = ray((0.0, 0.0, 0.0), (1.0, 0.0, 0.0));

    assert!(can_trace_line(&ray, &triangle));
}

#[test]
fn can_hit() {
    let triangle = Triangle::new(Vec3::new(2.0, 0.0, 0.0), Vec3::new(2.0, 2.0, 0.0), Vec3::new(2.0, 2.0, 2.0));

    let ray = ray((0.0, 0.0, 0.0), (2.1, 0.0, 0.0));

    assert!(!can_trace_line(&ray, &triangle));
}
