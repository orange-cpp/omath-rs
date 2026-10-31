use omath::Vector3;
use omath::collision::Collider;
use omath::engines::source;
use omath::primitives::Vertex;

fn vertex(x: f32, y: f32, z: f32) -> Vertex {
    Vertex {
        position: Vector3::new(x, y, z),
        ..Default::default()
    }
}

#[test]
fn check_to_world() {
    let mut mesh = source::Mesh::new(
        vec![vertex(1.0, 1.0, 1.0), vertex(-1.0, -1.0, -1.0)],
        vec![],
    );
    mesh.set_origin(Vector3::new(0.0, 2.0, 0.0));
    let collider = source::MeshCollider::new(mesh);

    let vertex = collider.find_abs_furthest_vertex_position(Vector3::new(1.0, 0.0, 0.0));

    assert_eq!(vertex, Vector3::new(1.0, 3.0, 1.0));
}

#[test]
fn find_furthest_vertex() {
    let mesh = source::Mesh::new(
        vec![vertex(1.0, 1.0, 1.0), vertex(-1.0, -1.0, -1.0)],
        vec![],
    );
    let collider = source::MeshCollider::new(mesh);

    let vertex = collider
        .find_furthest_vertex(Vector3::new(1.0, 0.0, 0.0))
        .position;

    assert_eq!(vertex, Vector3::new(1.0, 1.0, 1.0));
}
