//! Not ported from C++: pins the places where the collision port had to take care to match the headers, plus the one
//! place it deliberately does not.

use omath::collision::{Collider, Ray, Simplex, get_ray_hit_point};
use omath::engines::source;
use omath::primitives::{Aabb, Vertex};
use omath::{Triangle, Vector3};

#[test]
fn ties_pick_the_first_vertex_as_max_element_does() {
    let vertex = |x, y| Vertex {
        position: Vector3::new(x, y, 0.0f32),
        ..Default::default()
    };
    let mesh = source::Mesh::new(
        vec![
            vertex(1.0, 1.0),
            vertex(1.0, 2.0),
            vertex(1.0, 3.0),
            vertex(0.0, 9.0),
        ],
        vec![],
    );
    let collider = source::MeshCollider::new(mesh);

    assert_eq!(
        collider
            .find_furthest_vertex(Vector3::new(1.0, 0.0, 0.0))
            .position
            .y,
        1.0
    );
    assert_eq!(
        collider
            .find_abs_furthest_vertex_position(Vector3::new(1.0, 0.0, 0.0))
            .y,
        1.0
    );
}

#[test]
fn triangle_epsilon_is_f32_for_f64_rays() {
    let triangle = Triangle::new(
        Vector3::new(0.0f64, 0.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
    );
    // t_hit is 5e-8: well above the f64 epsilon, but below the f32 one that C++ uses for every scalar type.
    let ray = Ray {
        start: Vector3::new(0.25, 0.25, 5e-8),
        end: Vector3::new(0.25, 0.25, -1.0 + 5e-8),
        infinite_length: false,
    };

    assert_eq!(get_ray_hit_point(&ray, &triangle), ray.end);
}

#[test]
fn slab_epsilon_follows_the_scalar_type() {
    let aabb = Aabb::<f64>::new(Vector3::new(1.0, -1.0, -1.0), Vector3::new(2.0, 1.0, 1.0));
    // Direction along X is below the f32 epsilon but not the f64 one, so this is not a parallel ray.
    let ray = Ray {
        start: Vector3::new(0.0, 0.0, 0.0),
        end: Vector3::new(1e-8, 0.0, 0.0),
        infinite_length: true,
    };

    assert_eq!(get_ray_hit_point(&ray, &aabb).x, 1.0);
}

#[test]
fn infinite_ray_past_its_end_misses_a_triangle() {
    let triangle = Triangle::new(
        Vector3::new(0.0f32, 0.0, 5.0),
        Vector3::new(1.0, 0.0, 5.0),
        Vector3::new(0.0, 1.0, 5.0),
    );
    let ray = Ray {
        start: Vector3::new(0.25, 0.25, 0.0),
        end: Vector3::new(0.25, 0.25, 1.0),
        infinite_length: true,
    };

    // Questionable, since the slab tests do honour `infinite_length`, but it is what the header does.
    assert_eq!(get_ray_hit_point(&ray, &triangle), ray.end);
}

#[test]
fn handle_line_with_tiny_segment_stays_defined() {
    // Undefined in C++, where `any_perp` reaches `std::unreachable` for a segment this short.
    let mut simplex = Simplex::default();
    simplex.assign(&[
        Vector3::new(2e-8f32, 0.0, 0.0),
        Vector3::new(1e-8, 0.0, 0.0),
    ]);

    let mut direction = Vector3::default();
    assert!(!simplex.handle(&mut direction));
    assert_eq!(direction, Vector3::new(1.0, 0.0, 0.0));
}
