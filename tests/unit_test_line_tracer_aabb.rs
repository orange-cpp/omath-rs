mod common;

use common::expect_near;
use omath::Vector3;
use omath::collision::{Ray, get_ray_hit_point};
use omath::primitives::Aabb;

type Vec3 = Vector3<f32>;
type AABB = Aabb<f32>;

fn make_ray(start: Vec3, end: Vec3, infinite_length: bool) -> Ray {
    Ray { start, end, infinite_length }
}

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

fn unit_box() -> AABB {
    AABB::new(v(-1.0, -1.0, -1.0), v(1.0, 1.0, 1.0))
}

// Ray passing straight through the center along Z axis
#[test]
fn hit_center_along_z() {
    let ray = make_ray(v(0.0, 0.0, -5.0), v(0.0, 0.0, 5.0), false);

    let hit = get_ray_hit_point(&ray, &unit_box());
    assert_ne!(hit, ray.end);
    expect_near(hit.z, -1.0f32, 1e-4f32);
    expect_near(hit.x, 0.0f32, 1e-4f32);
    expect_near(hit.y, 0.0f32, 1e-4f32);
}

// Ray passing straight through the center along X axis
#[test]
fn hit_center_along_x() {
    let ray = make_ray(v(-5.0, 0.0, 0.0), v(5.0, 0.0, 0.0), false);

    let hit = get_ray_hit_point(&ray, &unit_box());
    assert_ne!(hit, ray.end);
    expect_near(hit.x, -1.0f32, 1e-4f32);
}

// Ray that misses entirely (too far in Y)
#[test]
fn miss_returns_end() {
    let ray = make_ray(v(0.0, 5.0, -5.0), v(0.0, 5.0, 5.0), false);

    let hit = get_ray_hit_point(&ray, &unit_box());
    assert_eq!(hit, ray.end);
}

// Ray that stops short before reaching the box
#[test]
fn ray_too_short_returns_end() {
    let box_ = AABB::new(v(3.0, -1.0, -1.0), v(5.0, 1.0, 1.0));
    let ray = make_ray(v(0.0, 0.0, 0.0), v(2.0, 0.0, 0.0), false);

    let hit = get_ray_hit_point(&ray, &box_);
    assert_eq!(hit, ray.end);
}

// Infinite ray that starts before the box should hit
#[test]
fn infinite_ray_hits() {
    let box_ = AABB::new(v(3.0, -1.0, -1.0), v(5.0, 1.0, 1.0));
    let ray = make_ray(v(0.0, 0.0, 0.0), v(2.0, 0.0, 0.0), true);

    let hit = get_ray_hit_point(&ray, &box_);
    assert_ne!(hit, ray.end);
    expect_near(hit.x, 3.0f32, 1e-4f32);
}

// Ray starting inside the box — t_min=0, so hit point equals ray.start
#[test]
fn ray_starts_inside_box() {
    let ray = make_ray(v(0.0, 0.0, 0.0), v(0.0, 0.0, 5.0), false);

    let hit = get_ray_hit_point(&ray, &unit_box());
    assert_ne!(hit, ray.end);
    // t_min is clamped to 0, so hit == start
    expect_near(hit.x, 0.0f32, 1e-4f32);
    expect_near(hit.y, 0.0f32, 1e-4f32);
    expect_near(hit.z, 0.0f32, 1e-4f32);
}

// Ray parallel to XY plane, pointing along X, at Z outside the box
#[test]
fn parallel_ray_outside_slab_misses() {
    // Z component of ray is 3.0 — outside box's Z slab
    let ray = make_ray(v(-5.0, 0.0, 3.0), v(5.0, 0.0, 3.0), false);

    let hit = get_ray_hit_point(&ray, &unit_box());
    assert_eq!(hit, ray.end);
}

// Ray parallel to XY plane, pointing along X, at Z inside the box
#[test]
fn parallel_ray_inside_slab_hits() {
    let ray = make_ray(v(-5.0, 0.0, 0.0), v(5.0, 0.0, 0.0), false);

    let hit = get_ray_hit_point(&ray, &unit_box());
    assert_ne!(hit, ray.end);
    expect_near(hit.x, -1.0f32, 1e-4f32);
}

// Diagonal ray hitting a corner region
#[test]
fn diagonal_ray_hits() {
    let box_ = AABB::new(v(0.0, 0.0, 0.0), v(2.0, 2.0, 2.0));
    let ray = make_ray(v(-1.0, -1.0, -1.0), v(3.0, 3.0, 3.0), false);

    let hit = get_ray_hit_point(&ray, &box_);
    assert_ne!(hit, ray.end);
    // Entry point should be at (0,0,0)
    expect_near(hit.x, 0.0f32, 1e-4f32);
    expect_near(hit.y, 0.0f32, 1e-4f32);
    expect_near(hit.z, 0.0f32, 1e-4f32);
}
