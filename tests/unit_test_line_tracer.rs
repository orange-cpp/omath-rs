mod common;

use common::expect_near;
use omath::collision::{Ray, can_trace_line, get_ray_hit_point};
use omath::{Triangle, Vector3};

type Vec3 = Vector3<f32>;

fn tri() -> Triangle<Vec3> {
    Triangle::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0))
}

#[test]
fn parallel_ray_returns_end() {
    // Triangle in XY plane
    let ray = Ray { start: Vec3::new(0.0, 0.0, 1.0), end: Vec3::new(1.0, 1.0, 1.0), ..Default::default() };

    // For a ray parallel to the triangle plane the algorithm should return ray.end
    let hit = get_ray_hit_point(&ray, &tri());
    assert!(hit == ray.end);
    assert!(can_trace_line(&ray, &tri()));
}

#[test]
fn misses_triangle_returns_end() {
    // passes above the triangle area
    let ray = Ray { start: Vec3::new(2.0, 2.0, -1.0), end: Vec3::new(2.0, 2.0, 1.0), ..Default::default() };

    let hit = get_ray_hit_point(&ray, &tri());
    assert!(hit == ray.end);
}

#[test]
fn hit_triangle_returns_point_inside_segment() {
    let tri = Triangle::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 0.0, 0.0), Vec3::new(0.0, 2.0, 0.0));
    let ray = Ray { start: Vec3::new(0.25, 0.25, -1.0), end: Vec3::new(0.25, 0.25, 1.0), ..Default::default() };

    let hit = get_ray_hit_point(&ray, &tri);
    // Should return a point between start and end (z approximately 0)
    assert_ne!(hit, ray.end);
    expect_near(hit.z, 0.0f32, 1e-4f32);
    // t_hit should be between 0 and 1 along the ray direction
    let dir = ray.direction_vector();
    // find t such that start + dir * t == hit (only check z comp for stability)
    let t = (hit.z - ray.start.z) / dir.z;
    assert!(t > 0.0);
    assert!(t < 1.0);
}

#[test]
fn infinite_length_early_out() {
    let ray = Ray { start: Vec3::new(0.25, 0.25, 0.0), end: Vec3::new(0.25, 0.25, 1.0), infinite_length: true };

    // If t_hit <= epsilon the algorithm should return ray.end when infinite_length is true.
    // Using start on the triangle plane should produce t_hit <= epsilon.
    let hit = get_ray_hit_point(&ray, &tri());
    assert!(hit == ray.end);
}
