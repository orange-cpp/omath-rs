#[macro_use]
mod projection_support;

use omath::Vector3;
use omath::collision::{Ray, get_ray_hit_point};
use omath::engines::{opengl, unreal};
use omath::primitives::{create_box, create_plane};

#[test]
fn test() {
    let result = create_box::<opengl::MeshTrait>(
        Vector3::new(0.0, 30.0, 0.0),
        Vector3::default(),
        opengl::K_ABS_FORWARD,
        opengl::K_ABS_RIGHT,
        4.0,
    );

    assert_eq!(result.vertex_buffer.len(), 8);
    assert_eq!(result.element_buffer_object.len(), 12);

    let ray = Ray { start: Vector3::new(0.0, 0.0, 0.0), end: Vector3::new(-100.0, 0.0, 0.0), ..Default::default() };
    let _ = get_ray_hit_point(&ray, &result);
}

// The tests below have no C++ counterpart: nothing in the ported C++ files reaches the world-space side of `Mesh`.

#[test]
fn box_corners_and_bounds() {
    let mesh: opengl::BoxMesh = create_box::<opengl::MeshTrait>(
        Vector3::new(0.0, 8.0, 0.0),
        Vector3::default(),
        opengl::K_ABS_FORWARD,
        opengl::K_ABS_RIGHT,
        4.0,
    );

    // side_size is 2: forward (-Z) and right (+X) offsets around the bottom and top points.
    assert_eq!(mesh.vertex_buffer[0], Vector3::new(2.0, 0.0, -2.0));
    assert_eq!(mesh.vertex_buffer[7], Vector3::new(-2.0, 8.0, 2.0));

    let bounds = mesh.local_bounds();
    assert_eq!(bounds.min, Vector3::new(-2.0, 0.0, -2.0));
    assert_eq!(bounds.max, Vector3::new(2.0, 8.0, 2.0));
}

#[test]
fn mesh_transform_applies_scale_then_rotation_then_origin() {
    let mut mesh: opengl::BoxMesh = create_box::<opengl::MeshTrait>(
        Vector3::new(0.0, 8.0, 0.0),
        Vector3::default(),
        opengl::K_ABS_FORWARD,
        opengl::K_ABS_RIGHT,
        4.0,
    );

    mesh.set_scale(Vector3::splat(2.0));
    mesh.set_rotation(opengl::ViewAngles::new(
        opengl::PitchAngle::from_degrees(0.0),
        opengl::YawAngle::from_degrees(90.0),
        opengl::RollAngle::from_degrees(0.0),
    ));
    mesh.set_origin(Vector3::new(10.0, 20.0, 30.0));

    // Yaw around +Y takes +X to -Z.
    let world = mesh.vertex_position_to_world_space(Vector3::new(1.0, 1.0, 0.0));
    expect_near!(world.x, 10.0_f32, 1e-5_f32);
    expect_near!(world.y, 22.0_f32, 1e-5_f32);
    expect_near!(world.z, 28.0_f32, 1e-5_f32);

    // The transform is cached, so a setter has to drop it.
    mesh.set_origin(Vector3::default());
    let moved = mesh.vertex_position_to_world_space(Vector3::new(1.0, 1.0, 0.0));
    expect_near!(moved.y, 2.0_f32, 1e-5_f32);

    let face = mesh.make_face_in_world_space(mesh.element_buffer_object[0]);
    let local = mesh.make_face_in_local_space(mesh.element_buffer_object[0]);
    assert_eq!(face.vertex1, mesh.vertex_position_to_world_space(local.vertex1));
}

#[test]
fn unreal_meshes_work_in_world_space() {
    let mut mesh: unreal::BoxMesh = create_box::<unreal::MeshTrait>(
        Vector3::new(0.0, 0.0, 8.0),
        Vector3::default(),
        unreal::K_ABS_FORWARD,
        unreal::K_ABS_RIGHT,
        4.0,
    );

    mesh.set_origin(Vector3::new(1.0, 2.0, 3.0));
    assert_eq!(mesh.vertex_position_to_world_space(Vector3::default()), Vector3::new(1.0, 2.0, 3.0));
}

#[test]
fn plane_keeps_the_cpp_indices() {
    let mesh = create_plane::<opengl::MeshTrait>(
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        2.0,
    );

    assert_eq!(mesh.vertex_buffer[3], Vector3::new(1.0, 2.0, 0.0));
    assert_eq!(mesh.element_buffer_object, [Vector3::new(1, 1, 2), Vector3::new(0, 1, 3)]);
}
