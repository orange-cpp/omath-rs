mod common;

use common::expect_near;
use omath::Vector3;
use omath::collision::{Ray, get_ray_hit_point};
use omath::engines::source::{self, Mesh};
use omath::primitives::Vertex;
use std::f32::consts::PI;

type Vec3 = Vector3<f32>;

fn make_ray(start: Vec3, end: Vec3) -> Ray {
    Ray { start, end, infinite_length: false }
}

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

// Unit cube spanning [-1, 1]^3 as 12 triangles, so a ray through it crosses two faces and the tracer has to pick the
// nearer one.
fn make_cube() -> Mesh {
    let vbo = [
        v(-1.0, -1.0, -1.0),
        v(1.0, -1.0, -1.0),
        v(1.0, 1.0, -1.0),
        v(-1.0, 1.0, -1.0),
        v(-1.0, -1.0, 1.0),
        v(1.0, -1.0, 1.0),
        v(1.0, 1.0, 1.0),
        v(-1.0, 1.0, 1.0),
    ]
    .map(|position| Vertex { position, ..Default::default() })
    .to_vec();

    let ebo = vec![
        Vector3::new(0, 1, 2),
        Vector3::new(0, 2, 3), // -Z
        Vector3::new(4, 6, 5),
        Vector3::new(4, 7, 6), // +Z
        Vector3::new(0, 5, 1),
        Vector3::new(0, 4, 5), // -Y
        Vector3::new(3, 2, 6),
        Vector3::new(3, 6, 7), // +Y
        Vector3::new(0, 3, 7),
        Vector3::new(0, 7, 4), // -X
        Vector3::new(1, 5, 6),
        Vector3::new(1, 6, 2), // +X
    ];

    Mesh::new(vbo, ebo)
}

fn rotation() -> source::ViewAngles {
    source::ViewAngles::new(
        source::PitchAngle::from_degrees(25.0),
        source::YawAngle::from_degrees(40.0),
        source::RollAngle::from_degrees(15.0),
    )
}

// The world-space algorithm the local-space tracer replaced, kept here as an oracle: transform every face into world
// space, intersect, keep the nearest. Any disagreement with the tracer means the local-space shortcut is not the
// identity it claims to be.
fn reference_trace(ray: &Ray, mesh: &Mesh) -> Vec3 {
    let mut best = ray.end;

    for &face in &mesh.element_buffer_object {
        let hit = get_ray_hit_point(ray, &mesh.make_face_in_world_space(face));

        if hit.distance_to(ray.start) < best.distance_to(ray.start) {
            best = hit;
        }
    }

    best
}

#[test]
fn hits_nearest_face() {
    let mesh = make_cube();
    let ray = make_ray(v(0.0, 0.0, -5.0), v(0.0, 0.0, 5.0));

    let hit = get_ray_hit_point(&ray, &mesh);

    assert_ne!(hit, ray.end);
    // -Z face, not the +Z one four units further along
    expect_near(hit.z, -1.0f32, 1e-4f32);
    expect_near(hit.x, 0.0f32, 1e-4f32);
    expect_near(hit.y, 0.0f32, 1e-4f32);
}

#[test]
fn miss_returns_ray_end() {
    let mesh = make_cube();
    let ray = make_ray(v(5.0, 5.0, -5.0), v(5.0, 5.0, 5.0));

    assert_eq!(get_ray_hit_point(&ray, &mesh), ray.end);
}

// A ray that stops before reaching the mesh is a miss, not a hit past its own end point.
#[test]
fn ray_too_short_returns_ray_end() {
    let mesh = make_cube();
    let ray = make_ray(v(0.0, 0.0, -5.0), v(0.0, 0.0, -3.0));

    assert_eq!(get_ray_hit_point(&ray, &mesh), ray.end);
}

#[test]
fn translated_mesh_shifts_hit() {
    let mut mesh = make_cube();
    mesh.set_origin(v(0.0, 0.0, 10.0));

    let ray = make_ray(v(0.0, 0.0, -5.0), v(0.0, 0.0, 20.0));
    let hit = get_ray_hit_point(&ray, &mesh);

    assert_ne!(hit, ray.end);
    expect_near(hit.z, 9.0f32, 1e-4f32);
}

// Non-uniform scale is the case the local-space shortcut has to earn: distances are not preserved by the transform,
// only the ray parameter t is.
#[test]
fn non_uniform_scale_hits_scaled_surface() {
    let mut mesh = make_cube();
    mesh.set_scale(v(1.0, 1.0, 4.0));

    let ray = make_ray(v(0.0, 0.0, -20.0), v(0.0, 0.0, 20.0));
    let hit = get_ray_hit_point(&ray, &mesh);

    assert_ne!(hit, ray.end);
    expect_near(hit.z, -4.0f32, 1e-3f32);
}

#[test]
fn degenerate_scale_returns_ray_end() {
    let mut mesh = make_cube();
    mesh.set_scale(v(0.0, 0.0, 0.0));

    let ray = make_ray(v(0.0, 0.0, -5.0), v(0.0, 0.0, 5.0));

    assert_eq!(get_ray_hit_point(&ray, &mesh), ray.end);
}

// local_bounds is the broadphase callers are meant to hoist, so it has to be tight and independent of the transform.
#[test]
fn local_bounds_cover_vertex_buffer() {
    let mut mesh = make_cube();
    mesh.set_origin(v(100.0, 100.0, 100.0));
    mesh.set_scale(v(5.0, 5.0, 5.0));

    let bounds = mesh.local_bounds();

    assert_eq!(bounds.min, v(-1.0, -1.0, -1.0));
    assert_eq!(bounds.max, v(1.0, 1.0, 1.0));
}

// A slab test against the hoisted bounds must not reject a ray the full trace would have hit.
#[test]
fn local_bounds_do_not_reject_real_hits() {
    let mesh = make_cube();
    let bounds = mesh.local_bounds();

    for i in 0..32 {
        let angle = 2.0 * PI * i as f32 / 32.0;
        let offset = v(angle.cos() * 6.0, (i % 5) as f32 - 2.0, angle.sin() * 6.0);

        let ray = make_ray(offset, -offset);

        if get_ray_hit_point(&ray, &mesh) != ray.end {
            assert_ne!(get_ray_hit_point(&ray, &bounds), ray.end, "ray {i}");
        }
    }
}

// The load-bearing test: agreement with the world-space reference across a transform that combines translation,
// rotation and non-uniform scale, over rays hitting from every direction.
#[test]
fn matches_world_space_reference() {
    let mut mesh = make_cube();
    mesh.set_origin(v(3.0, -2.0, 7.0));
    mesh.set_scale(v(2.0, 0.5, 1.5));
    mesh.set_rotation(rotation());

    let center = mesh.get_origin();

    for i in 0..64 {
        let angle = 2.0 * PI * i as f32 / 64.0;
        let offset = v(angle.cos() * 12.0, (i % 7) as f32 - 3.0, angle.sin() * 12.0);

        let ray = make_ray(center + offset, center - offset);

        let actual = get_ray_hit_point(&ray, &mesh);
        let expected = reference_trace(&ray, &mesh);

        expect_near(actual.x, expected.x, 1e-3f32);
        expect_near(actual.y, expected.y, 1e-3f32);
        expect_near(actual.z, expected.z, 1e-3f32);
    }
}

// Same agreement check for rays that miss, where both paths must produce the ray.end sentinel exactly.
#[test]
fn matches_world_space_reference_on_misses() {
    let mut mesh = make_cube();
    mesh.set_origin(v(3.0, -2.0, 7.0));
    mesh.set_rotation(rotation());

    for i in 0..32 {
        let angle = 2.0 * PI * i as f32 / 32.0;
        let start = mesh.get_origin() + v(angle.cos() * 40.0, 30.0, angle.sin() * 40.0);

        let ray = make_ray(start, start + v(0.0, 5.0, 0.0));

        assert_eq!(get_ray_hit_point(&ray, &mesh), ray.end, "ray {i}");
        assert_eq!(reference_trace(&ray, &mesh), ray.end, "ray {i}");
    }
}
