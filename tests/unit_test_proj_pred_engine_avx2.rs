//! The port of `ProjPredEngineAvx2` is called [`ProjPredEngineBatched`]. It runs on every build, so the C++ branch of
//! this file for builds without AVX2, `ThrowsWhenAvx2IsNotCompiledIn`, has no counterpart here.

mod common;

use common::{expect_float_eq, expect_near};
use omath::Vector3;
use omath::engines::source::{CameraTrait, PredEngineTrait as Trait};
use omath::projectile_prediction::{
    MuzzleOffset, PredEngine, ProjPredEngineBatched, ProjPredEngineInterface, ProjPredEngineLegacy,
    launcher_from_projectile,
};
use omath::projection::CameraEngine;
use omath::trigonometry::angles;

type Projectile = omath::projectile_prediction::Projectile<f32>;
type Target = omath::projectile_prediction::Target<f32>;
type Launcher = omath::projectile_prediction::Launcher<f32>;
type Avx2Engine = ProjPredEngineBatched;
type LegacyEngine = ProjPredEngineLegacy;

const K_MUZZLE: MuzzleOffset<f32> = MuzzleOffset {
    forward: 16.0,
    right: 8.0,
    up: -6.0,
};

fn v(x: f32, y: f32, z: f32) -> Vector3<f32> {
    Vector3::new(x, y, z)
}

fn round(launch_speed: f32, gravity_scale: f32) -> Projectile {
    Projectile {
        launch_speed,
        gravity_scale,
        ..Default::default()
    }
}

fn eye(x: f32, y: f32, z: f32) -> Launcher {
    Launcher {
        eye_origin: v(x, y, z),
        ..Default::default()
    }
}

fn grounded(origin: Vector3<f32>, velocity: Vector3<f32>) -> Target {
    Target {
        origin,
        velocity,
        is_airborne: false,
    }
}

/// Fire from where the solution says the muzzle is, with the solution's angles, and see how close the projectile gets
/// to the predicted target at the solution's time of flight. The solver picks the first feasible time step, so the miss
/// can be up to one step of travel.
fn expect_avx2_solution_hits(
    proj: &Projectile,
    launcher: &Launcher,
    target: &Target,
    gravity: f32,
    step: f32,
    max_time: f32,
    hit_tolerance: f32,
) {
    let engine = Avx2Engine::new(gravity, step, max_time);

    let solution = engine
        .maybe_calculate_aim(proj, launcher, target)
        .expect("engine must find a solution");

    // The muzzle turns with the view; the round leaves along the view raised by the launcher's pitch offset
    let (pitch, yaw) = (solution.angles.pitch, solution.angles.yaw);
    let launch_origin = launcher.launch_origin(Trait::calc_view_basis(pitch, yaw));
    let proj_pos = Trait::predict_projectile_position(
        launch_origin,
        proj,
        pitch + launcher.launch_pitch_offset,
        yaw,
        solution.time_of_flight,
        gravity,
    );
    let tgt_pos = Trait::predict_target_position(target, solution.time_of_flight, gravity);

    let miss = proj_pos.distance_to(tgt_pos);
    assert!(
        miss <= hit_tolerance,
        "closest approach {miss} at t={}",
        solution.time_of_flight
    );

    let camera = CameraTrait::calc_look_at_angle(launcher.eye_origin, solution.aim_point);
    expect_near(pitch, -camera.pitch.as_degrees(), 0.01_f32);
    expect_near(yaw, camera.yaw.as_degrees(), 0.01_f32);
}

#[test]
fn static_target() {
    let target = grounded(v(300.0, 0.0, 90.0), Vector3::default());

    expect_avx2_solution_hits(
        &round(2000.0, 0.4),
        &eye(3.0, 2.0, 1.0),
        &target,
        400.0,
        1.0 / 1000.0,
        50.0,
        5.0,
    );
}

#[test]
fn moving_target() {
    let target = grounded(v(500.0, 100.0, 0.0), v(-50.0, 20.0, 0.0));

    expect_avx2_solution_hits(
        &round(1500.0, 1.0),
        &eye(0.0, 0.0, 64.0),
        &target,
        800.0,
        1.0 / 1000.0,
        30.0,
        5.0,
    );
}

#[test]
fn airborne_target_with_muzzle_offset() {
    let target = Target {
        origin: v(200.0, 50.0, 300.0),
        velocity: v(10.0, -5.0, -20.0),
        is_airborne: true,
    };
    let launcher = Launcher {
        muzzle_offset: K_MUZZLE,
        ..eye(0.0, 0.0, 64.0)
    };

    expect_avx2_solution_hits(
        &round(1800.0, 0.5),
        &launcher,
        &target,
        400.0,
        1.0 / 1000.0,
        50.0,
        6.0,
    );
}

#[test]
fn agrees_with_legacy_engine() {
    let target = grounded(v(400.0, -150.0, 30.0), v(-30.0, 10.0, 0.0));
    let proj = round(2500.0, 0.6);
    let launcher = Launcher {
        muzzle_offset: K_MUZZLE,
        ..eye(0.0, 0.0, 64.0)
    };

    let avx2 = Avx2Engine::new(400.0, 1.0 / 1000.0, 50.0)
        .maybe_calculate_aim(&proj, &launcher, &target)
        .unwrap();
    let legacy = LegacyEngine::new(400.0, 1.0 / 1000.0, 50.0, 5.0)
        .maybe_calculate_aim(&proj, &launcher, &target)
        .unwrap();

    expect_near(avx2.angles.pitch, legacy.angles.pitch, 0.25_f32);
    expect_near(avx2.angles.yaw, legacy.angles.yaw, 0.25_f32);
    expect_near(avx2.time_of_flight, legacy.time_of_flight, 3.0_f32 / 1000.0);
}

#[test]
fn pipe_launcher_with_pitch_offset() {
    // TF2-style: velocity = forward * 1200 + up * 200 in the view frame
    let pitch_offset = angles::radians_to_degrees(200.0_f32.atan2(1200.0));
    let target = grounded(v(600.0, 150.0, 0.0), v(-40.0, 30.0, 0.0));
    let proj = round(1200.0_f32.hypot(200.0), 0.5);
    let launcher = Launcher {
        muzzle_offset: K_MUZZLE,
        launch_pitch_offset: pitch_offset,
        ..eye(0.0, 0.0, 64.0)
    };

    expect_avx2_solution_hits(&proj, &launcher, &target, 800.0, 1.0 / 1000.0, 5.0, 5.0);

    // Same shot through the legacy engine: the view pitch must carry the same offset
    let avx2 = Avx2Engine::new(800.0, 1.0 / 1000.0, 5.0)
        .maybe_calculate_aim(&proj, &launcher, &target)
        .unwrap();
    let legacy = LegacyEngine::new(800.0, 1.0 / 1000.0, 5.0, 5.0)
        .maybe_calculate_aim(&proj, &launcher, &target)
        .unwrap();
    expect_near(avx2.angles.pitch, legacy.angles.pitch, 0.25_f32);
    expect_near(avx2.angles.yaw, legacy.angles.yaw, 0.25_f32);
}

#[test]
fn unreachable_view_pitch_is_rejected() {
    let pitch_offset = angles::radians_to_degrees(200.0_f32.atan2(1200.0));
    let below = grounded(v(30.0, 0.0, 0.0), Vector3::default());
    let proj = round(1200.0_f32.hypot(200.0), 0.5);
    let engine = Avx2Engine::new(800.0, 1.0 / 1000.0, 5.0);

    assert!(
        engine
            .maybe_calculate_aim(&proj, &eye(0.0, 0.0, 1000.0), &below)
            .is_some()
    );
    assert!(
        engine
            .maybe_calculate_aim(
                &proj,
                &Launcher {
                    launch_pitch_offset: pitch_offset,
                    ..eye(0.0, 0.0, 1000.0)
                },
                &below
            )
            .is_none()
    );
}

#[test]
fn grounded_target_is_not_pulled_down_by_the_scan() {
    // Regression: the vectorised feasibility scan used to apply gravity to every target, airborne or not. With a round
    // that falls slower than the target supposedly did, the scan threw the true flight time away and accepted a later
    // one, for which the solved pitch belongs to a slower round. Over a flight this long that is a miss of hundreds of
    // units.
    let target = grounded(v(2500.0, 400.0, 0.0), Vector3::default());
    let proj = round(1500.0, 0.2);
    let launcher = eye(0.0, 0.0, 64.0);

    expect_avx2_solution_hits(&proj, &launcher, &target, 800.0, 1.0 / 1000.0, 10.0, 6.0);

    let avx2 = Avx2Engine::new(800.0, 1.0 / 1000.0, 10.0)
        .maybe_calculate_aim(&proj, &launcher, &target)
        .unwrap();
    let legacy = LegacyEngine::new(800.0, 1.0 / 1000.0, 10.0, 5.0)
        .maybe_calculate_aim(&proj, &launcher, &target)
        .unwrap();
    expect_near(avx2.angles.pitch, legacy.angles.pitch, 0.25_f32);
    expect_near(avx2.angles.yaw, legacy.angles.yaw, 0.25_f32);
    expect_near(avx2.time_of_flight, legacy.time_of_flight, 5.0_f32 / 1000.0);
}

#[test]
fn airborne_target_falls_during_the_flight() {
    // The same target, airborne or not, must be aimed at differently: lower when it is falling
    let proj = round(1500.0, 0.2);
    let launcher = eye(0.0, 0.0, 64.0);
    let on_the_ground = grounded(v(1500.0, 0.0, 600.0), Vector3::default());
    let falling = Target {
        is_airborne: true,
        ..on_the_ground
    };

    let engine = Avx2Engine::new(800.0, 1.0 / 1000.0, 10.0);
    let at_grounded = engine
        .maybe_calculate_aim(&proj, &launcher, &on_the_ground)
        .unwrap();
    let at_falling = engine
        .maybe_calculate_aim(&proj, &launcher, &falling)
        .unwrap();

    assert!(at_falling.angles.pitch < at_grounded.angles.pitch - 5.0);
    assert!(at_falling.predicted_target_position.z < 600.0 - 100.0);
    expect_float_eq(at_grounded.predicted_target_position.z, 600.0);

    expect_avx2_solution_hits(&proj, &launcher, &falling, 800.0, 1.0 / 1000.0, 10.0, 6.0);
}

#[test]
fn a_horizon_shorter_than_the_flight_finds_nothing() {
    let target = grounded(v(3000.0, 0.0, 64.0), Vector3::default());
    let proj = round(1500.0, 0.5);
    let launcher = eye(0.0, 0.0, 64.0);

    assert!(
        Avx2Engine::new(800.0, 1.0 / 1000.0, 1.5)
            .maybe_calculate_aim(&proj, &launcher, &target)
            .is_none()
    );

    let aim = Avx2Engine::new(800.0, 1.0 / 1000.0, 3.0)
        .maybe_calculate_aim(&proj, &launcher, &target)
        .unwrap();
    assert!(aim.time_of_flight > 1.9);
    assert!(aim.time_of_flight < 2.3);
}

#[test]
fn world_offset_moves_the_launch_origin() {
    // Firing from 40 units lower has to aim higher to reach the same spot
    let target = grounded(v(900.0, 100.0, 64.0), Vector3::default());
    let proj = round(1500.0, 0.5);
    let from_the_eye = eye(0.0, 0.0, 64.0);
    let from_below = Launcher {
        world_offset: v(0.0, 0.0, -40.0),
        ..eye(0.0, 0.0, 64.0)
    };

    let engine = Avx2Engine::new(800.0, 1.0 / 1000.0, 10.0);
    let high = engine
        .maybe_calculate_aim(&proj, &from_the_eye, &target)
        .unwrap();
    let low = engine
        .maybe_calculate_aim(&proj, &from_below, &target)
        .unwrap();
    assert!(low.angles.pitch > high.angles.pitch + 1.0);

    expect_avx2_solution_hits(&proj, &from_below, &target, 800.0, 1.0 / 1000.0, 10.0, 5.0);
}

#[test]
fn wrappers_and_the_interface_give_the_same_answer() {
    let target = grounded(v(500.0, 100.0, 0.0), v(-50.0, 20.0, 0.0));
    let proj = Projectile {
        origin: v(0.0, 0.0, 64.0),
        launch_offset: v(5.0, 0.0, -3.0),
        ..round(1500.0, 1.0)
    };

    let engine = Avx2Engine::new(800.0, 1.0 / 1000.0, 10.0);
    let through_base: &dyn ProjPredEngineInterface<f32> = &engine;

    let solution = engine
        .maybe_calculate_aim(&proj, &launcher_from_projectile(&proj), &target)
        .unwrap();
    let dispatched = through_base
        .maybe_calculate_aim(&proj, &launcher_from_projectile(&proj), &target)
        .unwrap();
    let point = engine.maybe_calculate_aim_point(&proj, &target).unwrap();
    let angles = through_base
        .maybe_calculate_aim_angles(&proj, &target)
        .unwrap();

    assert_eq!(solution.aim_point, dispatched.aim_point);
    assert_eq!(solution.aim_point, point);
    expect_float_eq(solution.angles.pitch, angles.pitch);
    expect_float_eq(solution.angles.yaw, angles.yaw);
}

#[test]
fn out_of_range_returns_nullopt() {
    let target = grounded(v(100000.0, 0.0, 0.0), Vector3::default());
    let proj = round(1.0, 1.0);

    let engine = Avx2Engine::new(400.0, 1.0 / 1000.0, 5.0);
    assert!(
        engine
            .maybe_calculate_aim(&proj, &eye(0.0, 0.0, 0.0), &target)
            .is_none()
    );
    assert!(engine.maybe_calculate_aim_point(&proj, &target).is_none());
}

/// Not in the C++, where these settings never get out of the scan.
#[test]
fn settings_that_cannot_run_give_no_solution() {
    let target = grounded(v(300.0, 0.0, 90.0), Vector3::default());
    let proj = round(2000.0, 0.4);
    let launcher = eye(3.0, 2.0, 1.0);

    assert!(
        Avx2Engine::new(400.0, 0.0, 50.0)
            .maybe_calculate_aim(&proj, &launcher, &target)
            .is_none()
    );
    assert!(
        Avx2Engine::new(400.0, -1.0, 50.0)
            .maybe_calculate_aim(&proj, &launcher, &target)
            .is_none()
    );
    assert!(
        Avx2Engine::new(400.0, 1.0 / 1000.0, f32::NAN)
            .maybe_calculate_aim(&proj, &launcher, &target)
            .is_none()
    );
}
