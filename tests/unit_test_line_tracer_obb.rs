mod common;

use common::expect_near;
use omath::Vector3;
use omath::collision::{Ray, get_ray_hit_point};
use omath::primitives::{Aabb, Obb};
use std::f32::consts::{PI, SQRT_2};

type Vec3 = Vector3<f32>;
type OrientedBounds = Obb<f32>;

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

fn make_ray(start: Vec3, end: Vec3) -> Ray {
    Ray {
        start,
        end,
        infinite_length: false,
    }
}

const fn axis_aligned_obb(center: Vec3, half_extents: Vec3) -> OrientedBounds {
    OrientedBounds {
        center,
        axis_x: Vec3::new(1.0, 0.0, 0.0),
        axis_y: Vec3::new(0.0, 1.0, 0.0),
        axis_z: Vec3::new(0.0, 0.0, 1.0),
        half_extents,
    }
}

fn rotated_around_z(center: Vec3, half_extents: Vec3, radians: f32) -> OrientedBounds {
    let (c, s) = (radians.cos(), radians.sin());
    OrientedBounds {
        center,
        axis_x: v(c, s, 0.0),
        axis_y: v(-s, c, 0.0),
        axis_z: v(0.0, 0.0, 1.0),
        half_extents,
    }
}

fn rotated_around_y(center: Vec3, half_extents: Vec3, radians: f32) -> OrientedBounds {
    let (c, s) = (radians.cos(), radians.sin());
    OrientedBounds {
        center,
        axis_x: v(c, 0.0, -s),
        axis_y: v(0.0, 1.0, 0.0),
        axis_z: v(s, 0.0, c),
        half_extents,
    }
}

const K_ZERO: Vec3 = Vec3::new(0.0, 0.0, 0.0);
const K_ONE: Vec3 = Vec3::new(1.0, 1.0, 1.0);

// --- axis-aligned OrientedBounds behaves like Bounds ---

#[test]
fn axis_aligned_hit_along_z() {
    let box_ = axis_aligned_obb(K_ZERO, K_ONE);
    let ray = make_ray(v(0.0, 0.0, -5.0), v(0.0, 0.0, 5.0));

    let hit = get_ray_hit_point(&ray, &box_);
    assert_ne!(hit, ray.end);
    expect_near(hit.x, 0.0f32, 1e-4f32);
    expect_near(hit.y, 0.0f32, 1e-4f32);
    expect_near(hit.z, -1.0f32, 1e-4f32);
}

#[test]
fn axis_aligned_hit_along_x() {
    let box_ = axis_aligned_obb(K_ZERO, K_ONE);
    let ray = make_ray(v(-5.0, 0.0, 0.0), v(5.0, 0.0, 0.0));

    let hit = get_ray_hit_point(&ray, &box_);
    assert_ne!(hit, ray.end);
    expect_near(hit.x, -1.0f32, 1e-4f32);
}

#[test]
fn miss_returns_end() {
    let box_ = axis_aligned_obb(K_ZERO, K_ONE);
    let ray = make_ray(v(0.0, 5.0, -5.0), v(0.0, 5.0, 5.0));

    let hit = get_ray_hit_point(&ray, &box_);
    assert_eq!(hit, ray.end);
}

#[test]
fn ray_too_short_returns_end() {
    let box_ = axis_aligned_obb(v(4.0, 0.0, 0.0), K_ONE);
    let ray = make_ray(K_ZERO, v(2.0, 0.0, 0.0));

    let hit = get_ray_hit_point(&ray, &box_);
    assert_eq!(hit, ray.end);
}

#[test]
fn infinite_ray_hits() {
    let box_ = axis_aligned_obb(v(4.0, 0.0, 0.0), K_ONE);
    let ray = Ray {
        start: K_ZERO,
        end: v(2.0, 0.0, 0.0),
        infinite_length: true,
    };

    let hit = get_ray_hit_point(&ray, &box_);
    assert_ne!(hit, ray.end);
    expect_near(hit.x, 3.0f32, 1e-4f32);
}

#[test]
fn ray_starts_inside_box() {
    let box_ = axis_aligned_obb(K_ZERO, K_ONE);
    let ray = make_ray(K_ZERO, v(5.0, 0.0, 0.0));

    let hit = get_ray_hit_point(&ray, &box_);
    assert_ne!(hit, ray.end);
    expect_near(hit.x, 0.0f32, 1e-4f32);
    expect_near(hit.y, 0.0f32, 1e-4f32);
    expect_near(hit.z, 0.0f32, 1e-4f32);
}

#[test]
fn ray_behind_box_returns_end() {
    let box_ = axis_aligned_obb(v(4.0, 0.0, 0.0), K_ONE);
    let ray = make_ray(v(10.0, 0.0, 0.0), v(20.0, 0.0, 0.0));

    let hit = get_ray_hit_point(&ray, &box_);
    assert_eq!(hit, ray.end);
}

// --- rotated OrientedBounds ---

#[test]
fn rotated_box_hit_on_rotated_face() {
    // Box centred at the origin, rotated 45° around Z. After rotation, the box's "near" face (originally x=-1) is now
    // perpendicular to the (1, 1, 0)/√2 direction. A ray approaching from +X (along world -X) first hits the box at the
    // rotated face — at x = √2 ≈ 1.414.
    let box_ = rotated_around_z(K_ZERO, K_ONE, PI / 4.0);
    let ray = make_ray(v(5.0, 0.0, 0.0), v(-5.0, 0.0, 0.0));

    let hit = get_ray_hit_point(&ray, &box_);
    assert_ne!(hit, ray.end);
    expect_near(hit.x, SQRT_2, 1e-4f32);
    expect_near(hit.y, 0.0f32, 1e-4f32);
    expect_near(hit.z, 0.0f32, 1e-4f32);
}

#[test]
fn rotated_around_y_box_hit_on_rotated_face() {
    let box_ = rotated_around_y(K_ZERO, K_ONE, PI / 4.0);
    let ray = make_ray(v(0.0, 0.0, 5.0), v(0.0, 0.0, -5.0));

    let hit = get_ray_hit_point(&ray, &box_);
    assert_ne!(hit, ray.end);
    expect_near(hit.x, 0.0f32, 1e-4f32);
    expect_near(hit.y, 0.0f32, 1e-4f32);
    expect_near(hit.z, SQRT_2, 1e-4f32);
}

#[test]
fn rotated_box_misses_where_aabb_would_hit() {
    // A unit cube rotated 45° around Z has an XY footprint that is a diamond reaching (±√2, 0) and (0, ±√2). The Bounds
    // envelope spans x,y ∈ [-√2, √2], but at y just below √2 the diamond is essentially a point. A ray at y = 1.43 is
    // outside the diamond entirely (|x| + |y| ≤ √2 ⇒ |x| ≤ √2 - 1.43 < 0), yet it would still pass through the Bounds
    // envelope of the rotated box.
    let box_ = rotated_around_z(K_ZERO, K_ONE, PI / 4.0);
    let ray = make_ray(v(-5.0, 1.43, 0.0), v(5.0, 1.43, 0.0));

    let hit = get_ray_hit_point(&ray, &box_);
    assert_eq!(hit, ray.end);
}

#[test]
fn rotated_thin_box_hit_from_the_side() {
    // Long, thin axis-aligned slab along X, rotated 90° around Z so it now points along Y. A ray from +X straight back
    // along -X must miss (the slab is thin in X), but a ray along -Y from +Y must hit.
    let box_ = rotated_around_z(K_ZERO, v(5.0, 0.5, 1.0), PI / 2.0);

    let ray_along_x = make_ray(v(10.0, 0.0, 0.0), v(-10.0, 0.0, 0.0));
    let hit_x = get_ray_hit_point(&ray_along_x, &box_);
    assert_ne!(hit_x, ray_along_x.end);
    expect_near(hit_x.x, 0.5f32, 1e-4f32); // hit on the rotated slab's narrow side

    let ray_along_y = make_ray(v(0.0, 10.0, 0.0), v(0.0, -10.0, 0.0));
    let hit_y = get_ray_hit_point(&ray_along_y, &box_);
    assert_ne!(hit_y, ray_along_y.end);
    expect_near(hit_y.y, 5.0f32, 1e-4f32); // hit on the long end at y=+5
}

#[test]
fn rotated_and_translated_box_hit() {
    let box_ = rotated_around_z(v(10.0, 5.0, 0.0), K_ONE, PI / 4.0);
    // Ray approaches the rotated box from +X.
    let ray = make_ray(v(20.0, 5.0, 0.0), v(0.0, 5.0, 0.0));

    let hit = get_ray_hit_point(&ray, &box_);
    assert_ne!(hit, ray.end);
    expect_near(hit.x, 10.0f32 + SQRT_2, 1e-4f32);
    expect_near(hit.y, 5.0f32, 1e-4f32);
}

#[test]
fn ray_starts_inside_rotated_box() {
    let box_ = rotated_around_z(v(2.0, 3.0, 0.0), v(2.0, 1.0, 1.0), PI / 6.0);
    let ray = make_ray(v(2.0, 3.0, 0.0), v(10.0, 3.0, 0.0));

    let hit = get_ray_hit_point(&ray, &box_);
    assert_ne!(hit, ray.end);
    expect_near(hit.x, 2.0f32, 1e-4f32);
    expect_near(hit.y, 3.0f32, 1e-4f32);
    expect_near(hit.z, 0.0f32, 1e-4f32);
}

#[test]
fn tangent_ray_hits_rotated_box() {
    let box_ = rotated_around_z(K_ZERO, K_ONE, PI / 4.0);
    let ray = make_ray(v(-5.0, SQRT_2, 0.0), v(5.0, SQRT_2, 0.0));

    let hit = get_ray_hit_point(&ray, &box_);
    assert_ne!(hit, ray.end);
    expect_near(hit.x, 0.0f32, 1e-4f32);
    expect_near(hit.y, SQRT_2, 1e-4f32);
    expect_near(hit.z, 0.0f32, 1e-4f32);
}

#[test]
fn degenerate_plane_box_can_be_hit() {
    let box_ = axis_aligned_obb(K_ZERO, v(1.0, 1.0, 0.0));
    let ray = make_ray(v(0.0, 0.0, -5.0), v(0.0, 0.0, 5.0));

    let hit = get_ray_hit_point(&ray, &box_);
    assert_ne!(hit, ray.end);
    expect_near(hit.x, 0.0f32, 1e-4f32);
    expect_near(hit.y, 0.0f32, 1e-4f32);
    expect_near(hit.z, 0.0f32, 1e-4f32);
}

#[test]
fn parallel_ray_outside_misses() {
    // Ray runs parallel to a slab face, completely outside the slab.
    let box_ = axis_aligned_obb(K_ZERO, K_ONE);
    let ray = make_ray(v(-5.0, 2.0, 0.0), v(5.0, 2.0, 0.0));

    let hit = get_ray_hit_point(&ray, &box_);
    assert_eq!(hit, ray.end);
}

#[test]
fn parallel_ray_inside_hits() {
    // Ray runs parallel to a slab face but inside the slab — should still hit the entry plane.
    let box_ = axis_aligned_obb(K_ZERO, K_ONE);
    let ray = make_ray(v(-5.0, 0.5, 0.0), v(5.0, 0.5, 0.0));

    let hit = get_ray_hit_point(&ray, &box_);
    assert_ne!(hit, ray.end);
    expect_near(hit.x, -1.0f32, 1e-4f32);
}

#[test]
fn matches_aabb_for_axis_aligned_box() {
    let cases = [
        (K_ZERO, K_ONE, v(-5.0, 0.0, 0.0), v(5.0, 0.0, 0.0)),
        (K_ZERO, K_ONE, v(0.0, -5.0, 0.0), v(0.0, 5.0, 0.0)),
        (v(4.0, 0.0, 0.0), K_ONE, K_ZERO, v(2.0, 0.0, 0.0)), // too short
        (K_ZERO, K_ONE, v(-5.0, 5.0, 0.0), v(5.0, 5.0, 0.0)), // miss
        (
            v(2.0, 3.0, -1.0),
            Vec3::splat(0.5),
            K_ZERO,
            v(10.0, 15.0, -5.0),
        ), // diagonal
    ];

    for (center, half, start, end) in cases {
        let aabb = Aabb::<f32>::new(center - half, center + half);
        let obb = axis_aligned_obb(center, half);
        let ray = make_ray(start, end);

        let aabb_hit = get_ray_hit_point(&ray, &aabb);
        let obb_hit = get_ray_hit_point(&ray, &obb);

        expect_near(aabb_hit.x, obb_hit.x, 1e-4f32);
        expect_near(aabb_hit.y, obb_hit.y, 1e-4f32);
        expect_near(aabb_hit.z, obb_hit.z, 1e-4f32);
    }
}
