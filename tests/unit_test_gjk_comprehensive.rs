//! Comprehensive GJK tests. Covers all 6 axis directions, diagonal cases, boundary touching, asymmetric sizes, nesting,
//! symmetry, simplex info, far separation.

use omath::Vector3;
use omath::collision::{GjkHitInfo, GjkSettings, is_collide, is_collide_with_simplex_info};
use omath::engines::source::{Mesh, MeshCollider as Collider};
use omath::primitives::Vertex;

type Vec3 = Vector3<f32>;

fn vertex(x: f32, y: f32, z: f32) -> Vertex {
    Vertex { position: Vector3::new(x, y, z), ..Default::default() }
}

fn make_scaled_cube(x: f32, y: f32, z: f32, scale: f32) -> Collider {
    // Unit cube [-1, 1]^3 in local space.
    let vbo = vec![
        vertex(-1.0, -1.0, -1.0),
        vertex(-1.0, -1.0, 1.0),
        vertex(-1.0, 1.0, -1.0),
        vertex(-1.0, 1.0, 1.0),
        vertex(1.0, 1.0, 1.0),
        vertex(1.0, 1.0, -1.0),
        vertex(1.0, -1.0, 1.0),
        vertex(1.0, -1.0, -1.0),
    ];
    let mut m = Mesh::with_scale(vbo, vec![], Vec3::splat(scale));
    m.set_origin(Vec3::new(x, y, z));
    Collider::new(m)
}

fn make_cube(x: f32, y: f32, z: f32) -> Collider {
    make_scaled_cube(x, y, z, 1.0)
}

fn collide_with_origin_cube(x: f32, y: f32, z: f32) -> bool {
    is_collide(&make_cube(0.0, 0.0, 0.0), &make_cube(x, y, z))
}

fn simplex_info_with_origin_cube(x: f32, y: f32, z: f32) -> GjkHitInfo {
    is_collide_with_simplex_info(&make_cube(0.0, 0.0, 0.0), &make_cube(x, y, z), &GjkSettings::default())
}

fn tet_vbo() -> Vec<Vertex> {
    vec![
        vertex(0.0, 1.0, 0.0),
        vertex(-1.0, -1.0, 1.0),
        vertex(1.0, -1.0, 1.0),
        vertex(0.0, -1.0, -1.0),
    ]
}

// ---------------------------------------------------------------------------
// Separation — expect false
// ---------------------------------------------------------------------------

#[test]
fn separated_along_pos_x() {
    // A extends to x=1, B starts at x=1.1 → clear gap
    assert!(!collide_with_origin_cube(2.1, 0.0, 0.0));
}

#[test]
fn separated_along_neg_x() {
    // B to the left of A
    assert!(!collide_with_origin_cube(-2.1, 0.0, 0.0));
}

#[test]
fn separated_along_pos_y() {
    assert!(!collide_with_origin_cube(0.0, 2.1, 0.0));
}

#[test]
fn separated_along_neg_y() {
    assert!(!collide_with_origin_cube(0.0, -2.1, 0.0));
}

#[test]
fn separated_along_pos_z() {
    assert!(!collide_with_origin_cube(0.0, 0.0, 2.1));
}

#[test]
fn separated_along_neg_z() {
    assert!(!collide_with_origin_cube(0.0, 0.0, -2.1));
}

#[test]
fn separated_along_diagonal() {
    // All components exceed 2.0 — no overlap on any axis
    assert!(!collide_with_origin_cube(2.1, 2.1, 2.1));
}

#[test]
fn separated_large_distance() {
    assert!(!collide_with_origin_cube(100.0, 0.0, 0.0));
}

#[test]
fn separated_asymmetric_sizes() {
    // Big (scale 2, half-ext 2), small (scale 0.5, half-ext 0.5) at 2.6 → gap of 0.1
    assert!(!is_collide(&make_scaled_cube(0.0, 0.0, 0.0, 2.0), &make_scaled_cube(2.6, 0.0, 0.0, 0.5)));
}

// ---------------------------------------------------------------------------
// Overlap — expect true
// ---------------------------------------------------------------------------

#[test]
fn overlapping_along_pos_x() {
    // B offset 1.5 → overlap depth 0.5 in X
    assert!(collide_with_origin_cube(1.5, 0.0, 0.0));
}

#[test]
fn overlapping_along_neg_x() {
    assert!(collide_with_origin_cube(-1.5, 0.0, 0.0));
}

#[test]
fn overlapping_along_pos_z() {
    assert!(collide_with_origin_cube(0.0, 0.0, 1.5));
}

#[test]
fn overlapping_along_neg_z() {
    assert!(collide_with_origin_cube(0.0, 0.0, -1.5));
}

#[test]
fn overlapping_along_diagonal_xy() {
    // Minkowski sum extends ±2 on each axis; offset (1,1,0) is inside
    assert!(collide_with_origin_cube(1.0, 1.0, 0.0));
}

#[test]
fn overlapping_along_diagonal_xyz() {
    // All three axes overlap: (1,1,1) is inside the Minkowski sum
    assert!(collide_with_origin_cube(1.0, 1.0, 1.0));
}

#[test]
fn fully_nested_small_inside_big() {
    // Small cube (half-ext 0.5) fully inside big cube (half-ext 2)
    assert!(is_collide(&make_scaled_cube(0.0, 0.0, 0.0, 2.0), &make_scaled_cube(0.0, 0.0, 0.0, 0.5)));
}

#[test]
fn fully_nested_off_center() {
    // Small at (0.5, 0, 0) still fully inside big (half-ext 2)
    assert!(is_collide(&make_scaled_cube(0.0, 0.0, 0.0, 2.0), &make_scaled_cube(0.5, 0.0, 0.0, 0.5)));
}

#[test]
fn overlapping_asymmetric_sizes() {
    // Big (scale 2, half-ext 2) and small (scale 0.5, half-ext 0.5) at 2.0 → overlap 0.5 in X
    assert!(is_collide(&make_scaled_cube(0.0, 0.0, 0.0, 2.0), &make_scaled_cube(2.0, 0.0, 0.0, 0.5)));
}

// ---------------------------------------------------------------------------
// Boundary cases
// ---------------------------------------------------------------------------

#[test]
fn boundary_case_just_colliding() {
    // B at 1.999 — 0.001 overlap in X
    assert!(collide_with_origin_cube(1.999, 0.0, 0.0));
}

#[test]
fn boundary_case_just_separated() {
    // B at 2.001 — 0.001 gap in X
    assert!(!collide_with_origin_cube(2.001, 0.0, 0.0));
}

// ---------------------------------------------------------------------------
// Symmetry
// ---------------------------------------------------------------------------

#[test]
fn symmetry_when_colliding() {
    let a = make_cube(0.0, 0.0, 0.0);
    let b = make_cube(1.5, 0.0, 0.0);
    assert_eq!(is_collide(&a, &b), is_collide(&b, &a));
}

#[test]
fn symmetry_when_separated() {
    let a = make_cube(0.0, 0.0, 0.0);
    let b = make_cube(2.1, 0.5, 0.0);
    assert_eq!(is_collide(&a, &b), is_collide(&b, &a));
}

#[test]
fn symmetry_diagonal_separation() {
    let a = make_cube(0.0, 0.0, 0.0);
    let b = make_cube(1.5, 1.5, 1.5);
    assert_eq!(is_collide(&a, &b), is_collide(&b, &a));
}

// ---------------------------------------------------------------------------
// Simplex info
// ---------------------------------------------------------------------------

#[test]
fn simplex_info_hit_produces_simplex4() {
    // On collision the simplex must be a full tetrahedron (4 points)
    let GjkHitInfo { hit, simplex } = simplex_info_with_origin_cube(0.5, 0.0, 0.0);
    assert!(hit);
    assert_eq!(simplex.size(), 4);
}

#[test]
fn simplex_info_miss_produces_less_than4() {
    // On non-collision the simplex can never be a full tetrahedron
    let GjkHitInfo { hit, simplex } = simplex_info_with_origin_cube(2.1, 0.0, 0.0);
    assert!(!hit);
    assert!(simplex.size() < 4);
}

#[test]
fn simplex_info_hit_along_y() {
    let GjkHitInfo { hit, simplex } = simplex_info_with_origin_cube(0.0, 1.5, 0.0);
    assert!(hit);
    assert_eq!(simplex.size(), 4);
}

#[test]
fn simplex_info_hit_along_z() {
    let GjkHitInfo { hit, simplex } = simplex_info_with_origin_cube(0.0, 0.0, 1.5);
    assert!(hit);
    assert_eq!(simplex.size(), 4);
}

#[test]
fn simplex_info_miss_along_diagonal() {
    let GjkHitInfo { hit, simplex } = simplex_info_with_origin_cube(2.1, 2.1, 2.1);
    assert!(!hit);
    assert!(simplex.size() < 4);
}

// ---------------------------------------------------------------------------
// Non-trivial geometry — tetrahedron shaped colliders
// ---------------------------------------------------------------------------

#[test]
fn tetrahedron_shapes_overlapping() {
    // A rough tetrahedron mesh; two of them close enough to overlap
    let m_a = Mesh::new(tet_vbo(), vec![]);
    let mut m_b = Mesh::new(tet_vbo(), vec![]);
    m_b.set_origin(Vec3::new(0.5, 0.0, 0.0));

    assert!(is_collide(&Collider::new(m_a), &Collider::new(m_b)));
}

#[test]
fn tetrahedron_shapes_separated() {
    let m_a = Mesh::new(tet_vbo(), vec![]);
    let mut m_b = Mesh::new(tet_vbo(), vec![]);
    m_b.set_origin(Vec3::new(3.0, 0.0, 0.0));

    assert!(!is_collide(&Collider::new(m_a), &Collider::new(m_b)));
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn deterministic_same_result_on_repeated_calls() {
    let a = make_cube(0.0, 0.0, 0.0);
    let b = make_cube(1.2, 0.3, 0.1);
    let first = is_collide(&a, &b);
    for _ in 0..10 {
        assert_eq!(is_collide(&a, &b), first);
    }
}
