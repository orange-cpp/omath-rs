//! Comprehensive EPA tests. Covers all 3 axis directions, multiple depth levels, penetration-vector round-trips, depth
//! monotonicity, symmetry, asymmetric sizes, repeated solves, tolerance sensitivity, and iteration bookkeeping.

mod common;

use common::expect_near;
use omath::Vector3;
use omath::collision::{
    EpaParams, EpaResult, GjkHitInfo, GjkSettings, epa_solve, is_collide,
    is_collide_with_simplex_info,
};
use omath::engines::source::{Mesh, MeshCollider as Collider};
use omath::primitives::Vertex;

type Vec3 = Vector3<f32>;

const K_DEFAULT_PARAMS: EpaParams = EpaParams {
    max_iterations: 64,
    tolerance: 1e-4,
};
const K_UNIT: Vec3 = Vec3::new(1.0, 1.0, 1.0);

fn vertex(x: f32, y: f32, z: f32) -> Vertex {
    Vertex {
        position: Vector3::new(x, y, z),
        ..Default::default()
    }
}

fn cube_vbo() -> Vec<Vertex> {
    vec![
        vertex(-1.0, -1.0, -1.0),
        vertex(-1.0, -1.0, 1.0),
        vertex(-1.0, 1.0, -1.0),
        vertex(-1.0, 1.0, 1.0),
        vertex(1.0, 1.0, 1.0),
        vertex(1.0, 1.0, -1.0),
        vertex(1.0, -1.0, 1.0),
        vertex(1.0, -1.0, -1.0),
    ]
}

fn make_mesh(origin: Vec3, scale: Vec3) -> Mesh {
    let mut m = Mesh::with_scale(cube_vbo(), vec![], scale);
    m.set_origin(origin);
    m
}

fn make_cube(x: f32, y: f32, z: f32) -> Collider {
    Collider::new(make_mesh(Vec3::new(x, y, z), K_UNIT))
}

fn make_scaled_cube(x: f32, y: f32, z: f32, scale: f32) -> Collider {
    Collider::new(make_mesh(Vec3::new(x, y, z), Vec3::splat(scale)))
}

fn gjk(a: &Collider, b: &Collider) -> GjkHitInfo {
    is_collide_with_simplex_info(a, b, &GjkSettings::default())
}

// Run GJK then EPA; asserts GJK hit and EPA converged.
fn solve_with(a: &Collider, b: &Collider, params: EpaParams) -> EpaResult {
    let GjkHitInfo { hit, simplex } = gjk(a, b);
    assert!(hit, "GJK must detect collision before EPA can run");
    epa_solve(a, b, &simplex, params).expect("EPA must converge")
}

fn solve(a: &Collider, b: &Collider) -> EpaResult {
    solve_with(a, b, K_DEFAULT_PARAMS)
}

// ---------------------------------------------------------------------------
// Normal direction per axis
// ---------------------------------------------------------------------------

// For two unit cubes (half-extent 1) with B offset by d along an axis:
//   depth = 2 - d   (distance from origin to nearest face of Minkowski diff)
//   normal component along that axis ≈ ±1

#[test]
fn normal_along_x_positive() {
    let r = solve(&make_cube(0.0, 0.0, 0.0), &make_cube(0.5, 0.0, 0.0));
    expect_near(r.normal.x.abs(), 1.0f32, 1e-3f32);
    expect_near(r.normal.y, 0.0f32, 1e-3f32);
    expect_near(r.normal.z, 0.0f32, 1e-3f32);
}

#[test]
fn normal_along_x_negative() {
    let r = solve(&make_cube(0.0, 0.0, 0.0), &make_cube(-0.5, 0.0, 0.0));
    expect_near(r.normal.x.abs(), 1.0f32, 1e-3f32);
    expect_near(r.normal.y, 0.0f32, 1e-3f32);
    expect_near(r.normal.z, 0.0f32, 1e-3f32);
}

#[test]
fn normal_along_y_positive() {
    let r = solve(&make_cube(0.0, 0.0, 0.0), &make_cube(0.0, 0.5, 0.0));
    expect_near(r.normal.x, 0.0f32, 1e-3f32);
    expect_near(r.normal.y.abs(), 1.0f32, 1e-3f32);
    expect_near(r.normal.z, 0.0f32, 1e-3f32);
}

#[test]
fn normal_along_y_negative() {
    let r = solve(&make_cube(0.0, 0.0, 0.0), &make_cube(0.0, -0.5, 0.0));
    expect_near(r.normal.x, 0.0f32, 1e-3f32);
    expect_near(r.normal.y.abs(), 1.0f32, 1e-3f32);
    expect_near(r.normal.z, 0.0f32, 1e-3f32);
}

#[test]
fn normal_along_z_positive() {
    let r = solve(&make_cube(0.0, 0.0, 0.0), &make_cube(0.0, 0.0, 0.5));
    expect_near(r.normal.x, 0.0f32, 1e-3f32);
    expect_near(r.normal.y, 0.0f32, 1e-3f32);
    expect_near(r.normal.z.abs(), 1.0f32, 1e-3f32);
}

#[test]
fn normal_along_z_negative() {
    let r = solve(&make_cube(0.0, 0.0, 0.0), &make_cube(0.0, 0.0, -0.5));
    expect_near(r.normal.x, 0.0f32, 1e-3f32);
    expect_near(r.normal.y, 0.0f32, 1e-3f32);
    expect_near(r.normal.z.abs(), 1.0f32, 1e-3f32);
}

// ---------------------------------------------------------------------------
// Depth correctness  (depth = 2 - offset for unit cubes)
// ---------------------------------------------------------------------------

#[test]
fn depth_shallow_overlap() {
    // offset 1.9 → depth 0.1
    let r = solve(&make_cube(0.0, 0.0, 0.0), &make_cube(1.9, 0.0, 0.0));
    expect_near(r.depth, 0.1f32, 1e-2f32);
}

#[test]
fn depth_quarter_overlap() {
    // offset 1.5 → depth 0.5
    let r = solve(&make_cube(0.0, 0.0, 0.0), &make_cube(1.5, 0.0, 0.0));
    expect_near(r.depth, 0.5f32, 1e-2f32);
}

#[test]
fn depth_half_overlap() {
    // offset 1.0 → depth 1.0
    let r = solve(&make_cube(0.0, 0.0, 0.0), &make_cube(1.0, 0.0, 0.0));
    expect_near(r.depth, 1.0f32, 1e-2f32);
}

#[test]
fn depth_three_quarter_overlap() {
    // offset 0.5 → depth 1.5
    let r = solve(&make_cube(0.0, 0.0, 0.0), &make_cube(0.5, 0.0, 0.0));
    expect_near(r.depth, 1.5f32, 1e-2f32);
}

#[test]
fn depth_along_y_half_overlap() {
    let r = solve(&make_cube(0.0, 0.0, 0.0), &make_cube(0.0, 1.0, 0.0));
    expect_near(r.depth, 1.0f32, 1e-2f32);
}

#[test]
fn depth_along_z_half_overlap() {
    let r = solve(&make_cube(0.0, 0.0, 0.0), &make_cube(0.0, 0.0, 1.0));
    expect_near(r.depth, 1.0f32, 1e-2f32);
}

// ---------------------------------------------------------------------------
// Depth monotonicity — deeper overlap → larger depth
// ---------------------------------------------------------------------------

#[test]
fn depth_monotonic_along_x() {
    let d1 = solve(&make_cube(0.0, 0.0, 0.0), &make_cube(1.9, 0.0, 0.0)).depth; // ~0.1
    let d2 = solve(&make_cube(0.0, 0.0, 0.0), &make_cube(1.5, 0.0, 0.0)).depth; // ~0.5
    let d3 = solve(&make_cube(0.0, 0.0, 0.0), &make_cube(1.0, 0.0, 0.0)).depth; // ~1.0
    let d4 = solve(&make_cube(0.0, 0.0, 0.0), &make_cube(0.5, 0.0, 0.0)).depth; // ~1.5

    assert!(d1 < d2);
    assert!(d2 < d3);
    assert!(d3 < d4);
}

// ---------------------------------------------------------------------------
// Normal is a unit vector
// ---------------------------------------------------------------------------

#[test]
fn normal_is_unit_along_x() {
    let r = solve(&make_cube(0.0, 0.0, 0.0), &make_cube(0.5, 0.0, 0.0));
    expect_near(r.normal.dot(r.normal), 1.0f32, 1e-5f32);
}

#[test]
fn normal_is_unit_along_y() {
    let r = solve(&make_cube(0.0, 0.0, 0.0), &make_cube(0.0, 1.2, 0.0));
    expect_near(r.normal.dot(r.normal), 1.0f32, 1e-5f32);
}

#[test]
fn normal_is_unit_along_z() {
    let r = solve(&make_cube(0.0, 0.0, 0.0), &make_cube(0.0, 0.0, 0.8));
    expect_near(r.normal.dot(r.normal), 1.0f32, 1e-5f32);
}

// ---------------------------------------------------------------------------
// Penetration vector = normal * depth
// ---------------------------------------------------------------------------

#[test]
fn penetration_vector_length_equals_depth() {
    let r = solve(&make_cube(0.0, 0.0, 0.0), &make_cube(0.5, 0.0, 0.0));
    let pen_len = r.penetration_vector.dot(r.penetration_vector).sqrt();
    expect_near(pen_len, r.depth, 1e-5f32);
}

#[test]
fn penetration_vector_direction_parallel_to_normal() {
    let r = solve(&make_cube(0.0, 0.0, 0.0), &make_cube(0.0, 1.0, 0.0));
    // penetration_vector = normal * depth → cross product must be ~zero
    let cross = r.penetration_vector.cross(r.normal);
    expect_near(cross.dot(cross), 0.0f32, 1e-8f32);
}

// ---------------------------------------------------------------------------
// Round-trip: applying penetration_vector separates the shapes
// ---------------------------------------------------------------------------

#[test]
fn round_trip_along_x() {
    let a = make_cube(0.0, 0.0, 0.0);
    let mesh_b = make_mesh(Vec3::new(0.5, 0.0, 0.0), K_UNIT);
    let b = Collider::new(mesh_b.clone());

    let r = solve(&a, &b);
    let margin = 1.0f32 + 1e-3f32;

    // Move B along the penetration vector; it should separate from A
    let mesh_sep = make_mesh(mesh_b.get_origin() + r.penetration_vector * margin, K_UNIT);
    assert!(
        !is_collide(&a, &Collider::new(mesh_sep)),
        "Applying pen vector must separate"
    );

    // Moving the wrong way must still collide
    let mesh_wrong = make_mesh(mesh_b.get_origin() - r.penetration_vector * margin, K_UNIT);
    assert!(
        is_collide(&a, &Collider::new(mesh_wrong)),
        "Opposite direction must still collide"
    );
}

#[test]
fn round_trip_along_y() {
    let a = make_cube(0.0, 0.0, 0.0);
    let mesh_b = make_mesh(Vec3::new(0.0, 0.8, 0.0), K_UNIT);
    let b = Collider::new(mesh_b.clone());

    let r = solve(&a, &b);
    let margin = 1.0f32 + 1e-3f32;

    let mesh_sep = make_mesh(mesh_b.get_origin() + r.penetration_vector * margin, K_UNIT);
    assert!(!is_collide(&a, &Collider::new(mesh_sep)));

    let mesh_wrong = make_mesh(mesh_b.get_origin() - r.penetration_vector * margin, K_UNIT);
    assert!(is_collide(&a, &Collider::new(mesh_wrong)));
}

#[test]
fn round_trip_along_z() {
    let a = make_cube(0.0, 0.0, 0.0);
    let mesh_b = make_mesh(Vec3::new(0.0, 0.0, 1.2), K_UNIT);
    let b = Collider::new(mesh_b.clone());

    let r = solve(&a, &b);
    let margin = 1.0f32 + 1e-3f32;

    let mesh_sep = make_mesh(mesh_b.get_origin() + r.penetration_vector * margin, K_UNIT);
    assert!(!is_collide(&a, &Collider::new(mesh_sep)));
}

// ---------------------------------------------------------------------------
// Symmetry — swapping A and B preserves depth
// ---------------------------------------------------------------------------

#[test]
fn symmetry_depth_is_independent_of_order() {
    let a = make_cube(0.0, 0.0, 0.0);
    let b = make_cube(0.5, 0.0, 0.0);

    let depth_ab = solve(&a, &b).depth;
    let depth_ba = solve(&b, &a).depth;

    expect_near(depth_ab, depth_ba, 1e-2f32);
}

#[test]
fn symmetry_normals_are_opposite() {
    let a = make_cube(0.0, 0.0, 0.0);
    let b = make_cube(0.5, 0.0, 0.0);

    let n_ab = solve(&a, &b).normal;
    let n_ba = solve(&b, &a).normal;

    // The normals should be anti-parallel: n_ab · n_ba ≈ -1
    expect_near(n_ab.dot(n_ba), -1.0f32, 1e-3f32);
}

// ---------------------------------------------------------------------------
// Asymmetric sizes
// ---------------------------------------------------------------------------

#[test]
fn large_vs_small_depth_correct() {
    // Big (half-ext 2) at origin, small (half-ext 0.5) at (2.0, 0, 0)
    // Minkowski diff closest face in X at distance 0.5
    let r = solve(
        &make_scaled_cube(0.0, 0.0, 0.0, 2.0),
        &make_scaled_cube(2.0, 0.0, 0.0, 0.5),
    );
    expect_near(r.depth, 0.5f32, 1e-2f32);
    expect_near(r.normal.x.abs(), 1.0f32, 1e-3f32);
}

#[test]
fn large_vs_small_round_trip() {
    let a = make_scaled_cube(0.0, 0.0, 0.0, 2.0);

    let small = Vec3::splat(0.5);
    let mesh_b = make_mesh(Vec3::new(2.0, 0.0, 0.0), small);
    let b = Collider::new(mesh_b.clone());

    let r = solve(&a, &b);
    let margin = 1.0f32 + 1e-3f32;

    let mesh_sep = make_mesh(mesh_b.get_origin() + r.penetration_vector * margin, small);
    assert!(!is_collide(&a, &Collider::new(mesh_sep)));
}

// ---------------------------------------------------------------------------
// Memory resource variants. There is no allocator parameter here, so these are repeated plain solves.
// ---------------------------------------------------------------------------

#[test]
fn monotonic_buffer_converges_correctly() {
    let a = make_cube(0.0, 0.0, 0.0);
    let b = make_cube(0.5, 0.0, 0.0);
    let GjkHitInfo { hit, simplex } = gjk(&a, &b);
    assert!(hit);

    let r = epa_solve(&a, &b, &simplex, K_DEFAULT_PARAMS).unwrap();
    expect_near(r.depth, 1.5f32, 1e-2f32);
}

#[test]
fn monotonic_buffer_multiple_release_cycles() {
    let a = make_cube(0.0, 0.0, 0.0);
    let b = make_cube(0.5, 0.0, 0.0);
    let GjkHitInfo { hit, simplex } = gjk(&a, &b);
    assert!(hit);

    let mut first_depth = 0.0f32;
    for i in 0..5 {
        let r = epa_solve(&a, &b, &simplex, K_DEFAULT_PARAMS)
            .unwrap_or_else(|| panic!("solve must converge on iteration {i}"));
        if i == 0 {
            first_depth = r.depth;
        } else {
            expect_near(r.depth, first_depth, 1e-6f32);
        }
    }
}

#[test]
fn default_resource_converges_correctly() {
    let a = make_cube(0.0, 0.0, 0.0);
    let b = make_cube(1.0, 0.0, 0.0);
    let GjkHitInfo { hit, simplex } = gjk(&a, &b);
    assert!(hit);

    let r = epa_solve(&a, &b, &simplex, EpaParams::default()).unwrap();
    expect_near(r.depth, 1.0f32, 1e-2f32);
}

// ---------------------------------------------------------------------------
// Tolerance sensitivity
// ---------------------------------------------------------------------------

#[test]
fn tighter_tolerance_more_accurate_depth() {
    let a = make_cube(0.0, 0.0, 0.0);
    let b = make_cube(1.0, 0.0, 0.0);
    let GjkHitInfo { hit, simplex } = gjk(&a, &b);
    assert!(hit);

    let loose = EpaParams {
        max_iterations: 64,
        tolerance: 1e-2,
    };
    let tight = EpaParams {
        max_iterations: 64,
        tolerance: 1e-5,
    };

    let r_loose = epa_solve(&a, &b, &simplex, loose).unwrap();
    let r_tight = epa_solve(&a, &b, &simplex, tight).unwrap();

    // Tighter tolerance must yield a result at least as accurate
    assert!((r_tight.depth - 1.0).abs() <= (r_loose.depth - 1.0).abs() + 1e-4);
}

// ---------------------------------------------------------------------------
// Bookkeeping fields
// ---------------------------------------------------------------------------

#[test]
fn bookkeeping_iterations_in_bounds() {
    let r = solve(&make_cube(0.0, 0.0, 0.0), &make_cube(0.5, 0.0, 0.0));

    assert!(r.iterations > 0);
    assert!(r.iterations <= K_DEFAULT_PARAMS.max_iterations);
}

#[test]
fn bookkeeping_faces_and_vertices_grow() {
    let r = solve(&make_cube(0.0, 0.0, 0.0), &make_cube(0.5, 0.0, 0.0));

    // Started with a tetrahedron (4 faces, 4 vertices); EPA must have expanded it
    assert!(r.num_faces >= 4);
    assert!(r.num_vertices >= 4);
}

#[test]
fn bookkeeping_max_iterations_respected() {
    let a = make_cube(0.0, 0.0, 0.0);
    let b = make_cube(0.5, 0.0, 0.0);
    let GjkHitInfo { hit, simplex } = gjk(&a, &b);
    assert!(hit);

    let tight = EpaParams {
        max_iterations: 3,
        tolerance: 1e-10,
    };

    // Must return something (fallback best-face path) and respect the cap
    if let Some(r) = epa_solve(&a, &b, &simplex, tight) {
        assert!(r.iterations <= tight.max_iterations);
    }
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn deterministic_same_result_on_repeated_calls() {
    let a = make_cube(0.0, 0.0, 0.0);
    let b = make_cube(0.7, 0.0, 0.0);
    let GjkHitInfo { hit, simplex } = gjk(&a, &b);
    assert!(hit);

    let first = epa_solve(&a, &b, &simplex, EpaParams::default()).unwrap();

    for _ in 0..5 {
        let r = epa_solve(&a, &b, &simplex, EpaParams::default()).unwrap();
        expect_near(r.depth, first.depth, 1e-6f32);
        expect_near(r.normal.x, first.normal.x, 1e-6f32);
        expect_near(r.normal.y, first.normal.y, 1e-6f32);
        expect_near(r.normal.z, first.normal.z, 1e-6f32);
    }
}
