#[macro_use]
mod projection_support;

use omath::Vector3;
use omath::engines::rage;
use omath::primitives::MeshEngine;
use omath::projection::{self, CameraEngine, ViewPort};

fn expect_rage_vector_near(actual: Vector3<f32>, expected: Vector3<f32>) {
    for (result, etalon) in actual.as_array().into_iter().zip(expected.as_array()) {
        expect_near!(result, etalon, 0.0001_f32);
    }
}

#[test]
fn forward_vector() {
    let forward = rage::forward_vector(Default::default());

    assert_eq!(forward, rage::K_ABS_FORWARD);
}

#[test]
fn right_vector() {
    let right = rage::right_vector(Default::default());

    assert_eq!(right, rage::K_ABS_RIGHT);
}

#[test]
fn up_vector() {
    let up = rage::up_vector(Default::default());
    assert_eq!(up, rage::K_ABS_UP);
}

#[test]
fn look_at_forward() {
    let angles = rage::CameraTrait::calc_look_at_angle(Default::default(), rage::K_ABS_FORWARD);

    expect_rage_vector_near(rage::forward_vector(angles), rage::K_ABS_FORWARD);
}

#[test]
fn look_at_right() {
    let angles = rage::CameraTrait::calc_look_at_angle(Default::default(), rage::K_ABS_RIGHT);

    expect_rage_vector_near(rage::forward_vector(angles), rage::K_ABS_RIGHT);
}

#[test]
fn look_at_up() {
    let angles = rage::CameraTrait::calc_look_at_angle(Default::default(), rage::K_ABS_UP);

    expect_rage_vector_near(rage::forward_vector(angles), rage::K_ABS_UP);
}

#[test]
fn project_target_moved_from_camera() {
    let fov = projection::FieldOfView::from_degrees(60.0);
    let cam = rage::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1280.0, 720.0),
        fov,
        0.01,
        1000.0,
    );

    let projected = cam.world_to_screen(Vector3::new(0.0, 10.0, 0.0));

    assert!(projected.is_ok());
    expect_near!(projected.unwrap().x, 640.0, 0.0001);
    expect_near!(projected.unwrap().y, 360.0, 0.0001);
}

#[test]
fn mesh_trait_forwards_rotation_matrix() {
    let angles: rage::ViewAngles = rage::ViewAngles::new(
        rage::PitchAngle::from_degrees(20.0),
        rage::YawAngle::from_degrees(-35.0),
        rage::RollAngle::from_degrees(15.0),
    );

    assert_eq!(rage::MeshTrait::rotation_matrix(&angles), rage::rotation_matrix(angles));
}
