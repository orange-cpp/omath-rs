mod common;

use common::expect_near;
use omath::Vector3;
use omath::collision::{
    EpaParams, GjkSettings, epa_solve, is_collide, is_collide_with_simplex_info,
};
use omath::engines::source::{Mesh, MeshCollider as Collider};
use omath::primitives::Vertex;

fn vertex(x: f32, y: f32, z: f32) -> Vertex {
    Vertex {
        position: Vector3::new(x, y, z),
        ..Default::default()
    }
}

// Unit cube [-1,1]^3
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

fn collide(a: &Collider, mesh: &Mesh) -> bool {
    is_collide(a, &Collider::new(mesh.clone()))
}

#[test]
fn test_collision_true() {
    let mut a = Mesh::with_scale(cube_vbo(), vec![], Vector3::splat(1.0));
    let mut b = Mesh::with_scale(cube_vbo(), vec![], Vector3::splat(1.0));

    // Overlap along +X by 0.5
    a.set_origin(Vector3::new(0.0, 0.0, 0.0));
    b.set_origin(Vector3::new(0.5, 0.0, 0.0));

    let (collider_a, collider_b) = (Collider::new(a), Collider::new(b.clone()));

    // GJK
    let gjk = is_collide_with_simplex_info(&collider_a, &collider_b, &GjkSettings::default());
    assert!(gjk.hit, "GJK should report collision");

    // EPA
    let params = EpaParams {
        max_iterations: 64,
        tolerance: 1e-4,
    };
    let epa =
        epa_solve(&collider_a, &collider_b, &gjk.simplex, params).expect("EPA should converge");

    // Normal is unit
    expect_near(epa.normal.dot(epa.normal), 1.0f32, 1e-5f32);

    // For this setup, depth ≈ 1.5 (2 - 0.5)
    expect_near(epa.depth, 1.5f32, 1e-3f32);

    // Normal axis sanity: near X axis
    expect_near(epa.normal.x.abs(), 1.0f32, 1e-3f32);
    expect_near(epa.normal.y, 0.0f32, 1e-3f32);
    expect_near(epa.normal.z, 0.0f32, 1e-3f32);

    // Try both signs with a tiny margin (avoid grazing contacts)
    let margin = 1.0f32 + 1e-3f32;
    let pen = epa.penetration_vector;

    let mut b_plus = b.clone();
    b_plus.set_origin(b_plus.get_origin() + pen * margin);
    let mut b_minus = b.clone();
    b_minus.set_origin(b_minus.get_origin() - pen * margin);

    let sep_plus = !collide(&collider_a, &b_plus);
    let sep_minus = !collide(&collider_a, &b_minus);

    // Exactly one direction should separate
    assert_ne!(
        sep_plus, sep_minus,
        "Exactly one of ±penetration must separate"
    );

    // Optional: pick the resolving direction and assert round-trip
    let resolve = if sep_plus {
        pen * margin
    } else {
        -pen * margin
    };

    let mut b_resolved = b.clone();
    b_resolved.set_origin(b_resolved.get_origin() + resolve);
    assert!(
        !collide(&collider_a, &b_resolved),
        "Resolved position should be non-colliding"
    );

    // Moving the other way should still collide
    let mut b_wrong = b.clone();
    b_wrong.set_origin(b_wrong.get_origin() - resolve);
    assert!(collide(&collider_a, &b_wrong));
}

#[test]
fn test_collision_true2() {
    let mut a = Mesh::with_scale(cube_vbo(), vec![], Vector3::splat(1.0));
    let mut b = Mesh::with_scale(cube_vbo(), vec![], Vector3::splat(1.0));

    // Overlap along +X by 0.5
    a.set_origin(Vector3::new(0.0, 0.0, 0.0));
    b.set_origin(Vector3::new(0.5, 0.0, 0.0));

    let (collider_a, collider_b) = (Collider::new(a), Collider::new(b.clone()));

    // --- GJK must detect collision and provide simplex ---
    let gjk = is_collide_with_simplex_info(&collider_a, &collider_b, &GjkSettings::default());
    assert!(gjk.hit, "GJK should report collision for overlapping cubes");

    // --- EPA penetration ---
    let params = EpaParams {
        max_iterations: 64,
        tolerance: 1e-4,
    };
    let epa =
        epa_solve(&collider_a, &collider_b, &gjk.simplex, params).expect("EPA should converge");

    // Normal is unit-length
    expect_near(epa.normal.dot(epa.normal), 1.0f32, 1e-5f32);

    // For centers at 0 and +0.5 and half-extent 1 -> depth ≈ 1.5
    expect_near(epa.depth, 1.5f32, 1e-3f32);

    // Axis sanity: mostly X
    expect_near(epa.normal.x.abs(), 1.0f32, 1e-3f32);
    expect_near(epa.normal.y, 0.0f32, 1e-3f32);
    expect_near(epa.normal.z, 0.0f32, 1e-3f32);

    let margin = 1.0f32 + 1e-3f32; // tiny slack to avoid grazing
    let pen = epa.normal * epa.depth;

    // Apply once: B + pen must separate; the opposite must still collide
    let mut b_resolved = b.clone();
    b_resolved.set_origin(b_resolved.get_origin() + pen * margin);
    assert!(
        !collide(&collider_a, &b_resolved),
        "Applying penetration should separate"
    );

    let mut b_wrong = b.clone();
    b_wrong.set_origin(b_wrong.get_origin() - pen * margin);
    assert!(
        collide(&collider_a, &b_wrong),
        "Opposite direction should still intersect"
    );

    // Some book-keeping sanity
    assert!(epa.iterations > 0);
    assert!(epa.iterations < params.max_iterations);
    assert!(epa.num_faces >= 4);
    assert!(epa.num_vertices > 4);
}
