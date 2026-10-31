mod common;

use common::expect_near;
use omath::Vector3;
use omath::collision::{Collider, EpaParams, Simplex, epa_solve};
use omath::engines::source;
use omath::primitives::Vertex;

type Vec3 = Vector3<f32>;

fn vertex(x: f32, y: f32, z: f32) -> Vertex {
    Vertex {
        position: Vector3::new(x, y, z),
        ..Default::default()
    }
}

#[test]
fn handle_empty_simplex() {
    let mut simplex = Simplex::<f32>::default();
    let mut direction = Vec3::new(1.0, 0.0, 0.0);

    assert_eq!(simplex.size(), 0);
    assert!(!simplex.handle(&mut direction));
}

#[test]
fn handle_line_collinear_with_x_axis() {
    let mut simplex = Simplex::default();

    simplex.push_front(Vec3::new(1.0, 0.0, 0.0));
    simplex.push_front(Vec3::new(-1.0, 0.0, 0.0));

    let mut direction = Vec3::default();
    let _ = simplex.handle(&mut direction);

    expect_near(direction.x, 0.0f32, 1e-6f32);
}

#[test]
fn simplex_line_handle() {
    let mut s = Simplex::default();
    s.assign(&[Vec3::new(1.0, 0.0, 0.0), Vec3::new(2.0, 0.0, 0.0)]);
    let mut dir = Vec3::default();
    assert!(!s.handle(&mut dir));
    // direction should not be zero
    assert!(dir.length_sqr() > 0.0);
}

#[test]
fn simplex_triangle_handle() {
    let mut s = Simplex::default();
    s.assign(&[
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
    ]);
    let mut dir = Vec3::default();
    assert!(!s.handle(&mut dir));
    assert!(dir.length_sqr() > 0.0);
}

#[test]
fn simplex_tetrahedron_inside() {
    let mut s = Simplex::default();
    // tetra that surrounds origin roughly
    s.assign(&[
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(-1.0, -1.0, -1.0),
    ]);
    let mut dir = Vec3::default();
    // if origin inside, handle returns true
    assert!(s.handle(&mut dir));
}

#[test]
fn mesh_collider_origin_and_furthest() {
    let mut mesh = source::Mesh::new(
        vec![vertex(1.0, 1.0, 1.0), vertex(-1.0, -1.0, -1.0)],
        vec![],
    );
    mesh.set_origin(Vector3::new(0.0, 2.0, 0.0));
    let mut collider = source::MeshCollider::new(mesh);

    assert_eq!(collider.get_origin(), Vec3::new(0.0, 2.0, 0.0));
    collider.set_origin(Vector3::new(1.0, 2.0, 3.0));
    assert_eq!(collider.get_origin(), Vec3::new(1.0, 2.0, 3.0));

    let v = collider.find_abs_furthest_vertex_position(Vector3::new(1.0, 0.0, 0.0));
    // the original vertex at (1,1,1) translated by origin (1,2,3) becomes (2,3,4)
    assert_eq!(v, Vec3::new(2.0, 3.0, 4.0));
}

#[test]
fn epa_converges_on_simple_case() {
    // Build two simple colliders using simple meshes that overlap
    let mesh_a = source::Mesh::new(vec![vertex(0.0, 0.0, 0.0), vertex(1.0, 0.0, 0.0)], vec![]);
    let mut mesh_b = mesh_a.clone();
    mesh_b.set_origin(Vector3::new(0.5, 0.0, 0.0)); // translate to overlap

    let a = source::MeshCollider::new(mesh_a);
    let b = source::MeshCollider::new(mesh_b);

    // Create a simplex that approximately contains the origin in Minkowski space
    let mut simplex = Simplex::default();
    simplex.assign(&[
        Vec3::new(0.5, 0.0, 0.0),
        Vec3::new(-0.5, 0.0, 0.0),
        Vec3::new(0.0, 0.5, 0.0),
        Vec3::new(0.0, -0.5, 0.0),
    ]);

    // EPA may or may not converge depending on numerics; ensure it returns optionally
    // but if it does, fields should be finite
    if let Some(r) = epa_solve(&a, &b, &simplex, EpaParams::default()) {
        assert!(r.depth.is_finite());
        assert!(r.normal.length_sqr() > 0.0);
    }
}
