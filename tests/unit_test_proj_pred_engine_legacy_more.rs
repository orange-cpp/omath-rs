mod common;

use common::{expect_float_eq, expect_near};
use omath::Vector3;
use omath::projectile_prediction::{
    Launcher, PredEngine, ProjPredEngineInterface, ProjPredEngineLegacy, Projectile, Target, ViewBasis,
};

fn v(x: f32, y: f32, z: f32) -> Vector3<f32> {
    Vector3::new(x, y, z)
}

/// World axes for every angle: enough for the engine to build launch origins and aim points from.
fn fake_basis() -> ViewBasis<f32> {
    ViewBasis { forward: v(1.0, 0.0, 0.0), right: v(0.0, 1.0, 0.0), up: v(0.0, 0.0, 1.0) }
}

fn projectile(launch_speed: f32, gravity_scale: f32) -> Projectile<f32> {
    Projectile { launch_speed, gravity_scale, ..Default::default() }
}

fn target(x: f32) -> Target<f32> {
    Target { origin: v(x, 0.0, 0.0), velocity: Vector3::default(), is_airborne: false }
}

/// Fake engine trait where gravity is effectively zero and projectile prediction always hits the target.
struct FakeEngineZeroGravity;

impl PredEngine for FakeEngineZeroGravity {
    type Scalar = f32;

    fn predict_target_position(t: &Target<f32>, _: f32, _: f32) -> Vector3<f32> {
        t.origin
    }
    fn predict_projectile_position(
        _: Vector3<f32>,
        _: &Projectile<f32>,
        _: f32,
        _: f32,
        _: f32,
        _: f32,
    ) -> Vector3<f32> {
        // Return a fixed point matching typical target used in the test
        v(100.0, 0.0, 0.0)
    }
    fn calc_vector_2d_distance(delta: Vector3<f32>) -> f32 {
        delta.x.hypot(delta.y)
    }
    fn get_vector_height_coordinate(vec: Vector3<f32>) -> f32 {
        vec.z
    }
    fn calc_view_basis(_: f32, _: f32) -> ViewBasis<f32> {
        fake_basis()
    }
    fn calc_direct_pitch_angle(_: Vector3<f32>, _: Vector3<f32>) -> f32 {
        12.5
    }
    fn calc_direct_yaw_angle(_: Vector3<f32>, _: Vector3<f32>) -> f32 {
        0.0
    }
}

#[test]
fn zero_gravity_uses_direct_pitch_and_returns_aim_point_on_forward_ray() {
    let engine = ProjPredEngineLegacy::<FakeEngineZeroGravity>::new(9.8, 0.1, 5.0, 1e-3);

    let res = engine.maybe_calculate_aim_point(&projectile(10.0, 0.0), &target(100.0)).unwrap();
    // eye + fake forward * distance(eye, target)
    expect_near(res.x, 100.0_f32, 1e-4_f32);
    expect_near(res.y, 0.0_f32, 1e-6_f32);
    expect_near(res.z, 0.0_f32, 1e-6_f32);
}

#[test]
fn zero_gravity_aim_angles_returns_pitch_and_yaw() {
    let engine = ProjPredEngineLegacy::<FakeEngineZeroGravity>::new(9.8, 0.1, 5.0, 1e-3);

    let res = engine.maybe_calculate_aim_angles(&projectile(10.0, 0.0), &target(100.0)).unwrap();
    // FakeEngineZeroGravity::calc_direct_pitch_angle returns 12.5
    expect_near(res.pitch, 12.5_f32, 1e-6_f32);
    // FakeEngineZeroGravity::calc_direct_yaw_angle returns 0
    expect_near(res.yaw, 0.0_f32, 1e-6_f32);
}

#[test]
fn solution_carries_target_and_time_of_flight() {
    let engine = ProjPredEngineLegacy::<FakeEngineZeroGravity>::new(9.8, 0.1, 5.0, 1e-3);

    let res = engine.maybe_calculate_aim(&projectile(10.0, 0.0), &Launcher::default(), &target(100.0)).unwrap();
    expect_float_eq(res.time_of_flight, 0.0); // the fake hits at the very first step
    expect_near(res.predicted_target_position.x, 100.0_f32, 1e-6_f32);
    expect_near(res.angles.pitch, 12.5_f32, 1e-6_f32);
}

/// Fake trait producing no valid launch angle (root < 0).
struct FakeEngineNoSolution;

impl PredEngine for FakeEngineNoSolution {
    type Scalar = f32;

    fn predict_target_position(t: &Target<f32>, _: f32, _: f32) -> Vector3<f32> {
        t.origin
    }
    fn predict_projectile_position(
        _: Vector3<f32>,
        _: &Projectile<f32>,
        _: f32,
        _: f32,
        _: f32,
        _: f32,
    ) -> Vector3<f32> {
        v(0.0, 0.0, 0.0)
    }
    fn calc_vector_2d_distance(_: Vector3<f32>) -> f32 {
        10000.0
    }
    fn get_vector_height_coordinate(_: Vector3<f32>) -> f32 {
        0.0
    }
    fn calc_view_basis(_: f32, _: f32) -> ViewBasis<f32> {
        fake_basis()
    }
    fn calc_direct_pitch_angle(_: Vector3<f32>, _: Vector3<f32>) -> f32 {
        0.0
    }
    fn calc_direct_yaw_angle(_: Vector3<f32>, _: Vector3<f32>) -> f32 {
        0.0
    }
}

#[test]
fn no_solution_root_returns_nullopt() {
    // Very slow projectile and large distance -> quadratic root negative
    let engine = ProjPredEngineLegacy::<FakeEngineNoSolution>::new(9.8, 0.5, 2.0, 1.0);

    assert!(engine.maybe_calculate_aim_point(&projectile(1.0, 1.0), &target(10000.0)).is_none());
    assert!(engine.maybe_calculate_aim_angles(&projectile(1.0, 1.0), &target(10000.0)).is_none());
}

/// Fake trait where an angle exists but the projectile does not reach target (miss).
struct FakeEngineAngleButMiss;

impl PredEngine for FakeEngineAngleButMiss {
    type Scalar = f32;

    fn predict_target_position(t: &Target<f32>, _: f32, _: f32) -> Vector3<f32> {
        t.origin
    }
    fn predict_projectile_position(
        _: Vector3<f32>,
        _: &Projectile<f32>,
        _: f32,
        _: f32,
        _: f32,
        _: f32,
    ) -> Vector3<f32> {
        // always return a point far from the target
        v(0.0, 0.0, 1000.0)
    }
    fn calc_vector_2d_distance(delta: Vector3<f32>) -> f32 {
        delta.x.hypot(delta.y)
    }
    fn get_vector_height_coordinate(vec: Vector3<f32>) -> f32 {
        vec.z
    }
    fn calc_view_basis(_: f32, _: f32) -> ViewBasis<f32> {
        fake_basis()
    }
    fn calc_direct_pitch_angle(_: Vector3<f32>, _: Vector3<f32>) -> f32 {
        1.0
    }
    fn calc_direct_yaw_angle(_: Vector3<f32>, _: Vector3<f32>) -> f32 {
        0.0
    }
}

#[test]
fn angle_computed_but_miss_returns_nullopt() {
    let engine = ProjPredEngineLegacy::<FakeEngineAngleButMiss>::new(9.8, 0.1, 1.0, 0.1);

    assert!(engine.maybe_calculate_aim_point(&projectile(100.0, 1.0), &target(10.0)).is_none());
}
