mod common;

use common::{expect_float_eq, expect_near};
use omath::Vector3;
use omath::engines::cry::{K_ABS_FORWARD, K_ABS_RIGHT, K_ABS_UP, PredEngineTrait};
use omath::projectile_prediction::{PredEngine, Projectile, Target};
use omath::trigonometry::angles;

fn v(x: f32, y: f32, z: f32) -> Vector3<f32> {
    Vector3::new(x, y, z)
}

fn zero() -> Vector3<f32> {
    Vector3::default()
}

fn projectile(origin: Vector3<f32>, launch_speed: f32, gravity_scale: f32) -> Projectile<f32> {
    Projectile { origin, launch_speed, gravity_scale, ..Default::default() }
}

fn target(origin: Vector3<f32>, velocity: Vector3<f32>, is_airborne: bool) -> Target<f32> {
    Target { origin, velocity, is_airborne }
}

fn predict(p: &Projectile<f32>, time: f32, gravity: f32) -> Vector3<f32> {
    PredEngineTrait::predict_projectile_position(p.origin + p.launch_offset, p, 0.0, 0.0, time, gravity)
}

// predict_projectile_position

#[test]
fn predict_projectile_position_at_time_zero() {
    let mut p = projectile(v(1.0, 2.0, 3.0), 100.0, 1.0);
    p.launch_offset = v(4.0, 5.0, 6.0);

    let pos = predict(&p, 0.0, 9.81);

    // At t=0 no velocity is applied, just origin+offset
    expect_near(pos.x, 5.0_f32, 1e-4_f32);
    expect_near(pos.y, 7.0_f32, 1e-4_f32);
    expect_near(pos.z, 9.0_f32, 1e-4_f32);
}

#[test]
fn predict_projectile_position_zero_angles_forward_is_y() {
    // Cry engine forward = +Y. At pitch=0, yaw=0 the projectile travels along +Y.
    let p = projectile(zero(), 10.0, 0.0); // no gravity so we isolate direction

    let pos = predict(&p, 1.0, 9.81);

    expect_near(pos.x, 0.0_f32, 1e-4_f32);
    expect_near(pos.y, 10.0_f32, 1e-4_f32);
    expect_near(pos.z, 0.0_f32, 1e-4_f32);
}

#[test]
fn predict_projectile_position_gravity_drops_z() {
    let p = projectile(zero(), 10.0, 1.0);

    let pos = predict(&p, 2.0, 9.81);

    // z = 0 - (9.81 * 1) * (4) * 0.5 = -19.62
    expect_near(pos.z, -9.81_f32 * 4.0 * 0.5, 1e-3_f32);
}

#[test]
fn predict_projectile_position_gravity_scale_zero_no_z_drop() {
    let p = projectile(zero(), 10.0, 0.0);

    let pos = predict(&p, 3.0, 9.81);

    expect_near(pos.z, 0.0_f32, 1e-4_f32);
}

#[test]
fn predict_projectile_position_with_launch_offset() {
    let mut p = projectile(v(5.0, 0.0, 0.0), 10.0, 0.0);
    p.launch_offset = v(0.0, 0.0, 2.0);

    let pos = predict(&p, 1.0, 0.0);

    // launch position = {5, 0, 2}, travels along +Y by 10
    expect_near(pos.x, 5.0_f32, 1e-4_f32);
    expect_near(pos.y, 10.0_f32, 1e-4_f32);
    expect_near(pos.z, 2.0_f32, 1e-4_f32);
}

// predict_target_position

#[test]
fn predict_target_position_grounded_stationary() {
    let t = target(v(10.0, 20.0, 5.0), zero(), false);

    let pred = PredEngineTrait::predict_target_position(&t, 5.0, 9.81);

    expect_near(pred.x, 10.0_f32, 1e-6_f32);
    expect_near(pred.y, 20.0_f32, 1e-6_f32);
    expect_near(pred.z, 5.0_f32, 1e-6_f32);
}

#[test]
fn predict_target_position_grounded_moving() {
    let t = target(zero(), v(3.0, 4.0, 0.0), false);

    let pred = PredEngineTrait::predict_target_position(&t, 2.0, 9.81);

    expect_near(pred.x, 6.0_f32, 1e-6_f32);
    expect_near(pred.y, 8.0_f32, 1e-6_f32);
    expect_near(pred.z, 0.0_f32, 1e-6_f32); // grounded: no gravity
}

#[test]
fn predict_target_position_airborne_gravity_drops_z() {
    let t = target(v(0.0, 0.0, 20.0), zero(), true);

    let pred = PredEngineTrait::predict_target_position(&t, 2.0, 9.81);

    // z = 20 - 9.81 * 4 * 0.5 = 20 - 19.62 = 0.38
    expect_near(pred.z, 20.0_f32 - 9.81 * 4.0 * 0.5, 1e-4_f32);
}

#[test]
fn predict_target_position_airborne_moving_with_gravity() {
    let t = target(v(0.0, 0.0, 50.0), v(10.0, 5.0, 0.0), true);

    let pred = PredEngineTrait::predict_target_position(&t, 3.0, 9.81);

    expect_near(pred.x, 30.0_f32, 1e-4_f32);
    expect_near(pred.y, 15.0_f32, 1e-4_f32);
    expect_near(pred.z, 50.0_f32 - 9.81 * 9.0 * 0.5, 1e-4_f32);
}

// calc_vector_2d_distance

#[test]
fn calc_vector_2d_distance_3_4_5() {
    expect_near(PredEngineTrait::calc_vector_2d_distance(v(3.0, 4.0, 999.0)), 5.0_f32, 1e-5_f32);
}

#[test]
fn calc_vector_2d_distance_zero_vector() {
    expect_near(PredEngineTrait::calc_vector_2d_distance(zero()), 0.0_f32, 1e-6_f32);
}

#[test]
fn calc_vector_2d_distance_z_ignored() {
    // Z does not affect the 2D distance
    expect_near(
        PredEngineTrait::calc_vector_2d_distance(v(0.0, 5.0, 100.0)),
        PredEngineTrait::calc_vector_2d_distance(v(0.0, 5.0, 0.0)),
        1e-6_f32,
    );
}

// get_vector_height_coordinate

#[test]
fn get_vector_height_coordinate_returns_z() {
    // Cry engine up = +Z
    expect_float_eq(PredEngineTrait::get_vector_height_coordinate(v(1.0, 2.0, 7.0)), 7.0);
}

// calc_direct_pitch_angle

#[test]
fn calc_direct_pitch_angle_flat() {
    // Target at same height -> pitch = 0
    expect_near(PredEngineTrait::calc_direct_pitch_angle(zero(), v(0.0, 100.0, 0.0)), 0.0_f32, 1e-4_f32);
}

#[test]
fn calc_direct_pitch_angle_looking_up() {
    // Target at 45 degrees above (equal XY distance and Z height)
    // direction to {0, 1, 1} normalized = {0, 0.707, 0.707}, asin(0.707) = 45 degrees
    expect_near(PredEngineTrait::calc_direct_pitch_angle(zero(), v(0.0, 1.0, 1.0)), 45.0_f32, 1e-3_f32);
}

#[test]
fn calc_direct_pitch_angle_looking_down() {
    // Target directly below
    expect_near(PredEngineTrait::calc_direct_pitch_angle(v(0.0, 0.0, 10.0), zero()), -90.0_f32, 1e-3_f32);
}

#[test]
fn calc_direct_pitch_angle_looking_directly_up() {
    expect_near(PredEngineTrait::calc_direct_pitch_angle(zero(), v(0.0, 0.0, 100.0)), 90.0_f32, 1e-3_f32);
}

// calc_direct_yaw_angle

#[test]
fn calc_direct_yaw_angle_forward_along_y() {
    // Cry engine forward = +Y -> yaw = 0
    expect_near(PredEngineTrait::calc_direct_yaw_angle(zero(), v(0.0, 100.0, 0.0)), 0.0_f32, 1e-4_f32);
}

#[test]
fn calc_direct_yaw_angle_along_positive_x() {
    // direction = {1, 0, 0}, yaw = -atan2(1, 0) = -90 degrees
    expect_near(PredEngineTrait::calc_direct_yaw_angle(zero(), v(100.0, 0.0, 0.0)), -90.0_f32, 1e-3_f32);
}

#[test]
fn calc_direct_yaw_angle_along_negative_x() {
    // direction = {-1, 0, 0}, yaw = -atan2(-1, 0) = 90 degrees
    expect_near(PredEngineTrait::calc_direct_yaw_angle(zero(), v(-100.0, 0.0, 0.0)), 90.0_f32, 1e-3_f32);
}

#[test]
fn calc_direct_yaw_angle_backward_along_neg_y() {
    // direction = {0, -1, 0}, yaw = -atan2(0, -1) = +-180 degrees
    let yaw = PredEngineTrait::calc_direct_yaw_angle(zero(), v(0.0, -100.0, 0.0));
    expect_near(yaw.abs(), 180.0_f32, 1e-3_f32);
}

#[test]
fn calc_direct_yaw_angle_off_origin_camera() {
    // Same relative direction regardless of camera position
    let yaw_a = PredEngineTrait::calc_direct_yaw_angle(zero(), v(0.0, 100.0, 0.0));
    let yaw_b = PredEngineTrait::calc_direct_yaw_angle(v(50.0, 50.0, 0.0), v(50.0, 150.0, 0.0));
    expect_near(yaw_a, yaw_b, 1e-4_f32);
}

// calc_view_basis

#[test]
fn calc_view_basis_zero_angles_is_world_axes() {
    let basis = PredEngineTrait::calc_view_basis(0.0, 0.0);

    expect_near(basis.forward.x, K_ABS_FORWARD.x, 1e-5_f32);
    expect_near(basis.forward.y, K_ABS_FORWARD.y, 1e-5_f32);
    expect_near(basis.forward.z, K_ABS_FORWARD.z, 1e-5_f32);
    expect_near(basis.right.x, K_ABS_RIGHT.x, 1e-5_f32);
    expect_near(basis.up.z, K_ABS_UP.z, 1e-5_f32);
}

#[test]
fn calc_view_basis_positive_pitch_looks_up() {
    // Trait pitch is positive upwards: forward gains height, the frame stays orthonormal
    let basis = PredEngineTrait::calc_view_basis(45.0, 0.0);

    expect_near(basis.forward.z, angles::degrees_to_radians(45.0_f32).sin(), 1e-5_f32);
    expect_near(basis.forward.length(), 1.0_f32, 1e-5_f32);
    expect_near(basis.forward.dot(basis.right), 0.0_f32, 1e-5_f32);
    expect_near(basis.forward.dot(basis.up), 0.0_f32, 1e-5_f32);
}
