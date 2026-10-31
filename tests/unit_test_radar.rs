#[macro_use]
mod projection_support;

use omath::Vector3;
use omath::algorithm;
use omath::engines::{cry, frostbite, iw, opengl, rage, source, unity, unreal};
use omath::projection::{CameraEngine, FieldOfView, ViewPort};

use omath::projection::Camera;
use omath::scalar::Scalar;
use std::cell::Cell;

fn make_camera<E: CameraEngine>(origin: Vector3<E::Scalar>) -> Camera<E> {
    Camera::new(
        origin,
        Default::default(),
        ViewPort::new(1280.0, 720.0),
        FieldOfView::from_degrees(90.0),
        E::Scalar::from_f64(0.01),
        E::Scalar::from_f64(1000.0),
    )
}

fn verify_world_to_radar_uses_engine_world_axes<E: CameraEngine>(
    world_forward: Vector3<E::Scalar>,
    world_right: Vector3<E::Scalar>,
) {
    let num = E::Scalar::from_f64;
    let origin = Vector3::new(num(10.0), num(-20.0), num(5.0));
    let world_distance = num(20.0);
    let scale = num(0.5);
    let radar_distance = (world_distance * scale).to_f64() as f32;

    let camera = make_camera::<E>(origin);

    let forward_position = origin + world_forward * world_distance;
    let forward_radar = algorithm::world_to_radar(&camera, forward_position, scale, None);

    expect_near!(forward_radar.x, 0.0_f32, 1e-4_f32);
    expect_near!(forward_radar.y, -radar_distance, 1e-4_f32);

    let right_position = origin + world_right * world_distance;
    let right_radar = algorithm::world_to_radar(&camera, right_position, scale, None);

    expect_near!(right_radar.x, radar_distance, 1e-4_f32);
    expect_near!(right_radar.y, 0.0_f32, 1e-4_f32);
}

fn verify_world_to_radar_uses_changed_camera_yaw<E: CameraEngine>(
    yaw_degrees: f32,
    world_forward: Vector3<E::Scalar>,
    world_right: Vector3<E::Scalar>,
) {
    let num = E::Scalar::from_f64;
    let origin = Vector3::new(num(10.0), num(-20.0), num(5.0));
    let world_distance = num(20.0);
    let scale = num(0.5);
    let radar_distance = (world_distance * scale).to_f64() as f32;

    let mut camera = make_camera::<E>(origin);

    let mut angles = camera.get_view_angles();
    angles.yaw = omath::Angle::from_degrees(num(f64::from(yaw_degrees)));
    camera.set_view_angles(angles);

    let forward_position = origin + world_right * world_distance;
    let forward_radar = algorithm::world_to_radar(&camera, forward_position, scale, None);

    expect_near!(forward_radar.x, 0.0_f32, 1e-4_f32);
    expect_near!(forward_radar.y, -radar_distance, 1e-4_f32);

    let left_position = origin + world_forward * world_distance;
    let left_radar = algorithm::world_to_radar(&camera, left_position, scale, None);

    expect_near!(left_radar.x, -radar_distance, 1e-4_f32);
    expect_near!(left_radar.y, 0.0_f32, 1e-4_f32);
}

fn verify_world_to_radar_ignores_camera_pitch<E: CameraEngine>(world_forward: Vector3<E::Scalar>) {
    let num = E::Scalar::from_f64;
    let origin = Vector3::new(num(10.0), num(-20.0), num(5.0));
    let world_distance = num(20.0);
    let scale = num(0.5);
    let radar_distance = (world_distance * scale).to_f64() as f32;

    let mut camera = make_camera::<E>(origin);

    let mut angles = camera.get_view_angles();
    angles.pitch = omath::Angle::from_degrees(num(45.0));
    camera.set_view_angles(angles);

    let forward_position = origin + world_forward * world_distance;
    let forward_radar = algorithm::world_to_radar(&camera, forward_position, scale, None);

    expect_near!(forward_radar.x, 0.0_f32, 1e-4_f32);
    expect_near!(forward_radar.y, -radar_distance, 1e-4_f32);
}

fn verify_world_to_radar_uses_custom_distance<E: CameraEngine>(world_forward: Vector3<E::Scalar>) {
    let num = E::Scalar::from_f64;
    let origin = Vector3::new(num(10.0), num(-20.0), num(5.0));
    let world_distance = num(20.0);
    let custom_distance = num(6.0);
    let scale = num(0.5);
    let radar_distance = (custom_distance * scale).to_f64() as f32;

    let camera = make_camera::<E>(origin);

    let forward_position = origin + world_forward * world_distance;
    let custom_distance_called = Cell::new(false);
    let calc_distance = |distance_origin: Vector3<E::Scalar>, distance_position: Vector3<E::Scalar>| {
        custom_distance_called.set(true);
        assert_eq!(distance_origin, origin);
        assert_eq!(distance_position, forward_position);
        custom_distance
    };
    let forward_radar = algorithm::world_to_radar(&camera, forward_position, scale, Some(&calc_distance));

    assert!(custom_distance_called.get());
    expect_near!(forward_radar.x, 0.0_f32, 1e-4_f32);
    expect_near!(forward_radar.y, -radar_distance, 1e-4_f32);
}

#[test]
fn source_engine_camera() {
    verify_world_to_radar_uses_engine_world_axes::<source::CameraTrait>(source::K_ABS_FORWARD, source::K_ABS_RIGHT);
    verify_world_to_radar_uses_changed_camera_yaw::<source::CameraTrait>(
        -90.0,
        source::K_ABS_FORWARD,
        source::K_ABS_RIGHT,
    );
    verify_world_to_radar_ignores_camera_pitch::<source::CameraTrait>(source::K_ABS_FORWARD);
    verify_world_to_radar_uses_custom_distance::<source::CameraTrait>(source::K_ABS_FORWARD);
}

#[test]
fn iw_engine_camera() {
    verify_world_to_radar_uses_engine_world_axes::<iw::CameraTrait>(iw::K_ABS_FORWARD, iw::K_ABS_RIGHT);
    verify_world_to_radar_uses_changed_camera_yaw::<iw::CameraTrait>(-90.0, iw::K_ABS_FORWARD, iw::K_ABS_RIGHT);
    verify_world_to_radar_ignores_camera_pitch::<iw::CameraTrait>(iw::K_ABS_FORWARD);
    verify_world_to_radar_uses_custom_distance::<iw::CameraTrait>(iw::K_ABS_FORWARD);
}

#[test]
fn frostbite_engine_camera() {
    verify_world_to_radar_uses_engine_world_axes::<frostbite::CameraTrait>(
        frostbite::K_ABS_FORWARD,
        frostbite::K_ABS_RIGHT,
    );
    verify_world_to_radar_uses_changed_camera_yaw::<frostbite::CameraTrait>(
        90.0,
        frostbite::K_ABS_FORWARD,
        frostbite::K_ABS_RIGHT,
    );
    verify_world_to_radar_ignores_camera_pitch::<frostbite::CameraTrait>(frostbite::K_ABS_FORWARD);
    verify_world_to_radar_uses_custom_distance::<frostbite::CameraTrait>(frostbite::K_ABS_FORWARD);
}

#[test]
fn opengl_engine_camera() {
    verify_world_to_radar_uses_engine_world_axes::<opengl::CameraTrait>(opengl::K_ABS_FORWARD, opengl::K_ABS_RIGHT);
    verify_world_to_radar_uses_changed_camera_yaw::<opengl::CameraTrait>(
        -90.0,
        opengl::K_ABS_FORWARD,
        opengl::K_ABS_RIGHT,
    );
    verify_world_to_radar_ignores_camera_pitch::<opengl::CameraTrait>(opengl::K_ABS_FORWARD);
    verify_world_to_radar_uses_custom_distance::<opengl::CameraTrait>(opengl::K_ABS_FORWARD);
}

#[test]
fn unity_engine_camera() {
    verify_world_to_radar_uses_engine_world_axes::<unity::CameraTrait>(unity::K_ABS_FORWARD, unity::K_ABS_RIGHT);
    verify_world_to_radar_uses_changed_camera_yaw::<unity::CameraTrait>(90.0, unity::K_ABS_FORWARD, unity::K_ABS_RIGHT);
    verify_world_to_radar_ignores_camera_pitch::<unity::CameraTrait>(unity::K_ABS_FORWARD);
    verify_world_to_radar_uses_custom_distance::<unity::CameraTrait>(unity::K_ABS_FORWARD);
}

#[test]
fn cry_engine_camera() {
    verify_world_to_radar_uses_engine_world_axes::<cry::CameraTrait>(cry::K_ABS_FORWARD, cry::K_ABS_RIGHT);
    verify_world_to_radar_uses_changed_camera_yaw::<cry::CameraTrait>(-90.0, cry::K_ABS_FORWARD, cry::K_ABS_RIGHT);
    verify_world_to_radar_ignores_camera_pitch::<cry::CameraTrait>(cry::K_ABS_FORWARD);
    verify_world_to_radar_uses_custom_distance::<cry::CameraTrait>(cry::K_ABS_FORWARD);
}

#[test]
fn rage_engine_camera() {
    verify_world_to_radar_uses_engine_world_axes::<rage::CameraTrait>(rage::K_ABS_FORWARD, rage::K_ABS_RIGHT);
    verify_world_to_radar_uses_changed_camera_yaw::<rage::CameraTrait>(-90.0, rage::K_ABS_FORWARD, rage::K_ABS_RIGHT);
    verify_world_to_radar_ignores_camera_pitch::<rage::CameraTrait>(rage::K_ABS_FORWARD);
    verify_world_to_radar_uses_custom_distance::<rage::CameraTrait>(rage::K_ABS_FORWARD);
}

#[test]
fn unreal_engine_camera() {
    verify_world_to_radar_uses_engine_world_axes::<unreal::CameraTrait>(unreal::K_ABS_FORWARD, unreal::K_ABS_RIGHT);
    verify_world_to_radar_uses_changed_camera_yaw::<unreal::CameraTrait>(
        90.0,
        unreal::K_ABS_FORWARD,
        unreal::K_ABS_RIGHT,
    );
    verify_world_to_radar_ignores_camera_pitch::<unreal::CameraTrait>(unreal::K_ABS_FORWARD);
    verify_world_to_radar_uses_custom_distance::<unreal::CameraTrait>(unreal::K_ABS_FORWARD);
}
