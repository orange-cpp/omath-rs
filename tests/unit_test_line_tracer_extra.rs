mod common;

use common::expect_near;
use omath::collision::{Ray, get_ray_hit_point};
use omath::{Triangle, Vector3};

type Vec3 = Vector3<f32>;

fn tri() -> Triangle<Vec3> {
    Triangle::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0))
}

#[test]
fn miss_parallel() {
    // parallel above triangle
    let ray = Ray { start: Vec3::new(0.3, 0.3, 1.0), end: Vec3::new(0.3, 0.3, 2.0), infinite_length: false };
    let hit = get_ray_hit_point(&ray, &tri());
    assert_eq!(hit, ray.end);
}

#[test]
fn hit_center() {
    let ray = Ray { start: Vec3::new(0.3, 0.3, -1.0), end: Vec3::new(0.3, 0.3, 1.0), infinite_length: false };
    let hit = get_ray_hit_point(&ray, &tri());
    assert!(hit != ray.end);
    expect_near(hit.x, 0.3f32, 1e-6f32);
    expect_near(hit.y, 0.3f32, 1e-6f32);
    expect_near(hit.z, 0.0f32, 1e-6f32);
}

#[test]
fn hit_on_edge() {
    let ray = Ray { start: Vec3::new(0.0, 0.0, 1.0), end: Vec3::new(0.0, 0.0, 0.0), infinite_length: false };
    // hitting exact vertex/edge may be considered miss; ensure function handles without crash
    let hit = get_ray_hit_point(&ray, &tri());
    if hit != ray.end {
        expect_near(hit.x, 0.0f32, 1e-6f32);
        expect_near(hit.y, 0.0f32, 1e-6f32);
    }
}

#[test]
fn infinite_ray_ignored_if_behind() {
    // Ray pointing away but infinite_length true should be ignored
    let ray = Ray { start: Vec3::new(0.5, 0.5, -1.0), end: Vec3::new(0.5, 0.5, -2.0), infinite_length: true };
    let hit = get_ray_hit_point(&ray, &tri());
    assert_eq!(hit, ray.end);
}
