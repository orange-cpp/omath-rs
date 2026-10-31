//! Tests for PredEngineTrait.

mod common;

use common::expect_near;
use omath::Vector3;
use omath::engines::source::PredEngineTrait;
use omath::projectile_prediction::{PredEngine, Projectile, Target};
use omath::trigonometry::angles;

fn v(x: f32, y: f32, z: f32) -> Vector3<f32> {
    Vector3::new(x, y, z)
}

#[test]
fn predict_projectile_position_basic() {
    let p = Projectile { launch_speed: 10.0, gravity_scale: 1.0, ..Default::default() };

    let pos = PredEngineTrait::predict_projectile_position(p.origin + p.launch_offset, &p, 0.0, 0.0, 1.0, 9.81);
    // With zero pitch and yaw forward vector is along X; expect x ~10, z reduced by gravity*0.5
    expect_near(pos.x, 10.0_f32, 1e-3_f32);
    expect_near(pos.z, -9.81_f32 * 0.5, 1e-3_f32);
}

#[test]
fn predict_target_position_airborne() {
    let t = Target { origin: v(0.0, 0.0, 10.0), velocity: v(1.0, 0.0, 0.0), is_airborne: true };

    let pred = PredEngineTrait::predict_target_position(&t, 2.0, 9.81);
    expect_near(pred.x, 2.0_f32, 1e-6_f32);
    // z should have been reduced by gravity* t^2
    expect_near(pred.z, 10.0_f32 - 9.81 * 4.0 * 0.5, 1e-6_f32);
}

#[test]
fn calc_vector2d_distance() {
    expect_near(PredEngineTrait::calc_vector_2d_distance(v(3.0, 4.0, 0.0)), 5.0_f32, 1e-6_f32);
}

#[test]
fn calc_view_basis() {
    // Zero angles: Source looks down +X with +Z up
    let level = PredEngineTrait::calc_view_basis(0.0, 0.0);
    expect_near(level.forward.x, 1.0_f32, 1e-5_f32);
    expect_near(level.forward.z, 0.0_f32, 1e-5_f32);
    expect_near(level.up.z, 1.0_f32, 1e-5_f32);

    // Trait pitch is positive upwards even though the engine's own pitch is negative upwards
    let raised = PredEngineTrait::calc_view_basis(45.0, 0.0);
    expect_near(raised.forward.x, angles::degrees_to_radians(45.0_f32).cos(), 1e-5_f32);
    expect_near(raised.forward.z, angles::degrees_to_radians(45.0_f32).sin(), 1e-5_f32);
    expect_near(raised.forward.dot(raised.right), 0.0_f32, 1e-5_f32);
    expect_near(raised.forward.dot(raised.up), 0.0_f32, 1e-5_f32);

    // Yaw swings forward towards +Y
    let turned = PredEngineTrait::calc_view_basis(0.0, 90.0);
    expect_near(turned.forward.y, 1.0_f32, 1e-5_f32);
}

#[test]
fn predict_projectile_position_with_launch_offset() {
    let p =
        Projectile { launch_offset: v(5.0, 3.0, -2.0), launch_speed: 10.0, gravity_scale: 1.0, ..Default::default() };

    // At time=0, projectile should be at launch_pos = origin + offset
    let pos_t0 = PredEngineTrait::predict_projectile_position(p.origin + p.launch_offset, &p, 0.0, 0.0, 0.0, 9.81);
    expect_near(pos_t0.x, 5.0_f32, 1e-4_f32);
    expect_near(pos_t0.y, 3.0_f32, 1e-4_f32);
    expect_near(pos_t0.z, -2.0_f32, 1e-4_f32);

    // At time=1 with zero pitch/yaw, should travel along X from the offset position
    let pos_t1 = PredEngineTrait::predict_projectile_position(p.origin + p.launch_offset, &p, 0.0, 0.0, 1.0, 9.81);
    expect_near(pos_t1.x, 5.0_f32 + 10.0, 1e-3_f32);
    expect_near(pos_t1.y, 3.0_f32, 1e-3_f32);
    expect_near(pos_t1.z, -2.0_f32 - 9.81 * 0.5, 1e-3_f32);
}

#[test]
fn zero_launch_offset_matches_original_behavior() {
    let p = Projectile {
        origin: v(10.0, 20.0, 30.0),
        launch_offset: v(0.0, 0.0, 0.0),
        launch_speed: 15.0,
        gravity_scale: 0.5,
        ..Default::default()
    };

    let p_no_offset =
        Projectile { origin: v(10.0, 20.0, 30.0), launch_speed: 15.0, gravity_scale: 0.5, ..Default::default() };

    let pos1 = PredEngineTrait::predict_projectile_position(p.origin + p.launch_offset, &p, 30.0, 45.0, 2.0, 9.81);
    let pos2 = PredEngineTrait::predict_projectile_position(
        p_no_offset.origin + p_no_offset.launch_offset,
        &p_no_offset,
        30.0,
        45.0,
        2.0,
        9.81,
    );
    expect_near(pos1.x, pos2.x, 1e-6_f32);
    expect_near(pos1.y, pos2.y, 1e-6_f32);
    expect_near(pos1.z, pos2.z, 1e-6_f32);
}

#[test]
fn direct_angles() {
    let origin = v(0.0, 0.0, 0.0);
    let target = v(0.0, 1.0, 1.0);
    // yaw should be 90 degrees (pointing along y)
    expect_near(PredEngineTrait::calc_direct_yaw_angle(origin, target), 90.0_f32, 1e-3_f32);
    // pitch should be asin(z/distance)
    let dist = origin.distance_to(target);
    expect_near(
        PredEngineTrait::calc_direct_pitch_angle(origin, target),
        angles::radians_to_degrees(((target.z - origin.z) / dist).asin()),
        1e-3_f32,
    );
}
