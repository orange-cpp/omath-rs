use omath::Vector3;
use omath::collision::is_collide;
use omath::engines::source::{Mesh, MeshCollider};
use omath::primitives::Vertex;

fn vertex(x: f32, y: f32, z: f32) -> Vertex {
    Vertex {
        position: Vector3::new(x, y, z),
        ..Default::default()
    }
}

fn mesh() -> Mesh {
    Mesh::new(
        vec![
            vertex(-1.0, -1.0, -1.0),
            vertex(-1.0, -1.0, 1.0),
            vertex(-1.0, 1.0, -1.0),
            vertex(-1.0, 1.0, 1.0),
            vertex(1.0, 1.0, 1.0),
            vertex(1.0, 1.0, -1.0),
            vertex(1.0, -1.0, 1.0),
            vertex(1.0, -1.0, -1.0),
        ],
        vec![],
    )
}

#[test]
fn test_collision_true() {
    let collider_a = MeshCollider::new(mesh());

    let mut mesh_b = mesh();
    mesh_b.set_origin(Vector3::new(0.0, 0.5, 0.0));

    let collider_b = MeshCollider::new(mesh_b);

    assert!(is_collide(&collider_a, &collider_b));
}

#[test]
fn test_collision_false() {
    let collider_a = MeshCollider::new(mesh());

    let mut mesh_b = mesh();
    mesh_b.set_origin(Vector3::new(0.0, 2.1, 0.0));

    let collider_b = MeshCollider::new(mesh_b);

    assert!(!is_collide(&collider_a, &collider_b));
}

#[test]
fn test_collision_equal_origin() {
    let collider_a = MeshCollider::new(mesh());
    let collider_b = MeshCollider::new(mesh());

    assert!(is_collide(&collider_a, &collider_b));
}
