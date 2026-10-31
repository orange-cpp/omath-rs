mod common;

use common::expect_near;
use omath::collision::{Ray, get_ray_hit_point};
use omath::{Triangle, Vector3};

type Vec3 = Vector3<f32>;

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

fn tri() -> Triangle<Vec3> {
    Triangle::new(v(0.0, 0.0, 0.0), v(1.0, 0.0, 0.0), v(0.0, 1.0, 0.0))
}

fn segment(start: Vec3, end: Vec3) -> Ray {
    Ray {
        start,
        end,
        ..Default::default()
    }
}

#[test]
fn u_greater_than_one_returns_end() {
    // choose ray so barycentric u > 1
    let ray = segment(v(2.0, -1.0, -1.0), v(2.0, -1.0, 1.0));

    let hit = get_ray_hit_point(&ray, &tri());
    assert_eq!(hit, ray.end);
}

#[test]
fn v_greater_than_one_returns_end() {
    // choose ray so barycentric v > 1
    let ray = segment(v(-1.0, 2.0, -1.0), v(-1.0, 2.0, 1.0));

    let hit = get_ray_hit_point(&ray, &tri());
    assert_eq!(hit, ray.end);
}

#[test]
fn u_plus_v_greater_than_one_returns_end() {
    // Ray aimed so u+v > 1 (outside triangle region)
    let ray = segment(v(1.0, 1.0, -1.0), v(1.0, 1.0, 1.0));

    let hit = get_ray_hit_point(&ray, &tri());
    assert_eq!(hit, ray.end);
}

#[test]
fn direction_vector_normalized_produces_unit_length() {
    let r = segment(v(0.0, 0.0, 0.0), v(0.0, 3.0, 4.0));
    let dir = r.direction_vector_normalized();
    expect_near(dir.length(), 1.0f32, 1e-6f32);
}

#[test]
fn zero_length_ray_handled() {
    let ray = segment(v(0.0, 0.0, 0.0), v(0.0, 0.0, 0.0));

    // Zero-length ray: direction length == 0; algorithm should handle without crash
    let hit = get_ray_hit_point(&ray, &tri());
    assert_eq!(hit, ray.end);
}
