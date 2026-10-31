mod common;

use common::{expect_float_eq, expect_near};
use omath::Vector3;
use omath::engines::source::{CameraTrait, PredEngineTrait as Trait};
use omath::projectile_prediction::{
    Arc, MuzzleOffset, PredEngine, ProjPredEngineInterface, ProjPredEngineLegacy,
};
use omath::projection::CameraEngine;
use omath::trigonometry::angles;

type Projectile = omath::projectile_prediction::Projectile<f32>;
type Target = omath::projectile_prediction::Target<f32>;
type Launcher = omath::projectile_prediction::Launcher<f32>;
type Engine = ProjPredEngineLegacy;

fn v(x: f32, y: f32, z: f32) -> Vector3<f32> {
    Vector3::new(x, y, z)
}

fn target(origin: [f32; 3], velocity: [f32; 3], is_airborne: bool) -> Target {
    Target {
        origin: origin.into_v(),
        velocity: velocity.into_v(),
        is_airborne,
    }
}

trait IntoV {
    fn into_v(self) -> Vector3<f32>;
}

impl IntoV for [f32; 3] {
    fn into_v(self) -> Vector3<f32> {
        v(self[0], self[1], self[2])
    }
}

fn proj(origin: [f32; 3], launch_speed: f32, gravity_scale: f32) -> Projectile {
    Projectile {
        origin: origin.into_v(),
        launch_speed,
        gravity_scale,
        ..Default::default()
    }
}

fn proj_offset(
    origin: [f32; 3],
    offset: [f32; 3],
    launch_speed: f32,
    gravity_scale: f32,
) -> Projectile {
    Projectile {
        launch_offset: offset.into_v(),
        ..proj(origin, launch_speed, gravity_scale)
    }
}

fn round(launch_speed: f32, gravity_scale: f32) -> Projectile {
    Projectile {
        launch_speed,
        gravity_scale,
        ..Default::default()
    }
}

#[test]
fn prediction_test() {
    let target = target([100.0, 0.0, 90.0], [0.0, 0.0, 0.0], false);
    let proj = proj([3.0, 2.0, 1.0], 5000.0, 0.4);
    let view_point =
        Engine::new(400.0, 1.0 / 1000.0, 50.0, 5.0).maybe_calculate_aim_point(&proj, &target);

    let angles = CameraTrait::calc_look_at_angle(proj.origin, view_point.unwrap());

    expect_near(-42.547142, angles.pitch.as_degrees(), 0.01_f32);
    expect_near(-1.181189, angles.yaw.as_degrees(), 0.01_f32);
}

/// Verify aim_angles match angles derived from aim_point via CameraTrait.
fn expect_angles_match_aim_point(
    proj: &Projectile,
    target: &Target,
    gravity: f32,
    step: f32,
    max_time: f32,
    tolerance: f32,
) {
    let angle_eps = 0.01_f32;
    let engine = Engine::new(gravity, step, max_time, tolerance);

    let aim_point = engine
        .maybe_calculate_aim_point(proj, target)
        .expect("aim_point should have a solution");
    let aim_angles = engine
        .maybe_calculate_aim_angles(proj, target)
        .expect("aim_angles should have a solution");

    // Source engine CameraTrait: pitch = -asin(dir.z), yaw = atan2(dir.y, dir.x)
    // PredEngineTrait: pitch = asin(delta.z / dist), yaw = atan2(delta.y, delta.x)
    // So aim_angles.pitch == -camera_pitch, aim_angles.yaw == camera_yaw
    let camera = CameraTrait::calc_look_at_angle(proj.origin, aim_point);

    expect_near(aim_angles.pitch, -camera.pitch.as_degrees(), angle_eps);
    expect_near(aim_angles.yaw, camera.yaw.as_degrees(), angle_eps);
}

#[test]
fn aim_angles_match_aim_point_static_target() {
    let target = target([100.0, 0.0, 90.0], [0.0, 0.0, 0.0], false);
    expect_angles_match_aim_point(
        &proj([3.0, 2.0, 1.0], 5000.0, 0.4),
        &target,
        400.0,
        1.0 / 1000.0,
        50.0,
        5.0,
    );
}

#[test]
fn aim_angles_match_aim_point_moving_target() {
    let target = target([500.0, 100.0, 0.0], [-50.0, 20.0, 0.0], false);
    expect_angles_match_aim_point(
        &proj([0.0, 0.0, 0.0], 3000.0, 1.0),
        &target,
        800.0,
        1.0 / 500.0,
        30.0,
        10.0,
    );
}

#[test]
fn aim_angles_match_aim_point_airborne_target() {
    let target = target([200.0, 50.0, 300.0], [10.0, -5.0, -20.0], true);
    expect_angles_match_aim_point(
        &proj([0.0, 0.0, 0.0], 4000.0, 0.5),
        &target,
        400.0,
        1.0 / 1000.0,
        50.0,
        10.0,
    );
}

#[test]
fn aim_angles_match_aim_point_high_arc() {
    // Target nearly directly above: high pitch angle
    let target = target([10.0, 0.0, 500.0], [0.0, 0.0, 0.0], false);
    expect_angles_match_aim_point(
        &proj([0.0, 0.0, 0.0], 5000.0, 0.3),
        &target,
        400.0,
        1.0 / 1000.0,
        50.0,
        5.0,
    );
}

#[test]
fn aim_angles_match_aim_point_negative_yaw() {
    // Target behind and to the left: negative yaw quadrant
    let target = target([-200.0, -150.0, 10.0], [0.0, 0.0, 0.0], false);
    expect_angles_match_aim_point(
        &proj([0.0, 0.0, 0.0], 5000.0, 0.4),
        &target,
        400.0,
        1.0 / 1000.0,
        50.0,
        5.0,
    );
}

#[test]
fn aim_angles_match_aim_point_with_launch_offset() {
    let target = target([200.0, 0.0, 50.0], [0.0, 0.0, 0.0], false);
    let proj = proj_offset([0.0, 0.0, 0.0], [5.0, 0.0, -3.0], 5000.0, 0.4);
    expect_angles_match_aim_point(&proj, &target, 400.0, 1.0 / 1000.0, 50.0, 5.0);
}

/// Simulate projectile flight using aim_angles and verify it reaches the target. Steps the projectile forward in small
/// increments, simultaneously predicts target position, and checks that the minimum distance is within hit_tolerance.
fn expect_projectile_hits_target(
    proj: &Projectile,
    target: &Target,
    gravity: f32,
    engine_step: f32,
    max_time: f32,
    engine_tolerance: f32,
    hit_tolerance: f32,
) {
    let sim_step = 1.0_f32 / 2000.0;
    let engine = Engine::new(gravity, engine_step, max_time, engine_tolerance);

    let aim_angles = engine
        .maybe_calculate_aim_angles(proj, target)
        .expect("engine must find a solution");

    let mut min_dist = f32::MAX;
    let mut best_time = 0.0;

    let mut t = 0.0;
    while t <= max_time {
        let proj_pos = Trait::predict_projectile_position(
            proj.origin + proj.launch_offset,
            proj,
            aim_angles.pitch,
            aim_angles.yaw,
            t,
            gravity,
        );
        let tgt_pos = Trait::predict_target_position(target, t, gravity);
        let dist = proj_pos.distance_to(tgt_pos);

        if dist < min_dist {
            min_dist = dist;
            best_time = t;
        }

        // Early exit once distance starts increasing significantly after approaching
        if dist > min_dist + hit_tolerance * 10.0 && min_dist < hit_tolerance * 100.0 {
            break;
        }

        t += sim_step;
    }

    assert!(
        min_dist <= hit_tolerance,
        "Projectile must reach target. Closest approach: {min_dist} at t={best_time}"
    );
}

// Simulation hit tests: no launch offset

#[test]
fn hits_static_target_no_offset() {
    let target = target([100.0, 0.0, 90.0], [0.0, 0.0, 0.0], false);
    expect_projectile_hits_target(
        &proj([3.0, 2.0, 1.0], 5000.0, 0.4),
        &target,
        400.0,
        1.0 / 1000.0,
        50.0,
        5.0,
        10.0,
    );
}

#[test]
fn hits_moving_target_no_offset() {
    let target = target([500.0, 100.0, 0.0], [-50.0, 20.0, 0.0], false);
    expect_projectile_hits_target(
        &proj([0.0, 0.0, 0.0], 3000.0, 1.0),
        &target,
        800.0,
        1.0 / 500.0,
        30.0,
        10.0,
        15.0,
    );
}

#[test]
fn hits_airborne_target_no_offset() {
    let target = target([200.0, 50.0, 300.0], [10.0, -5.0, -20.0], true);
    expect_projectile_hits_target(
        &proj([0.0, 0.0, 0.0], 4000.0, 0.5),
        &target,
        400.0,
        1.0 / 1000.0,
        50.0,
        10.0,
        15.0,
    );
}

#[test]
fn hits_high_target_no_offset() {
    let target = target([10.0, 0.0, 500.0], [0.0, 0.0, 0.0], false);
    expect_projectile_hits_target(
        &proj([0.0, 0.0, 0.0], 5000.0, 0.3),
        &target,
        400.0,
        1.0 / 1000.0,
        50.0,
        5.0,
        10.0,
    );
}

#[test]
fn hits_negative_yaw_target_no_offset() {
    let target = target([-200.0, -150.0, 10.0], [0.0, 0.0, 0.0], false);
    expect_projectile_hits_target(
        &proj([0.0, 0.0, 0.0], 5000.0, 0.4),
        &target,
        400.0,
        1.0 / 1000.0,
        50.0,
        5.0,
        10.0,
    );
}

// Simulation hit tests: with launch offset

#[test]
fn hits_static_target_small_offset() {
    let target = target([200.0, 0.0, 50.0], [0.0, 0.0, 0.0], false);
    let proj = proj_offset([0.0, 0.0, 0.0], [5.0, 0.0, -3.0], 5000.0, 0.4);
    expect_projectile_hits_target(&proj, &target, 400.0, 1.0 / 1000.0, 50.0, 5.0, 10.0);
}

#[test]
fn hits_static_target_large_x_offset() {
    let target = target([300.0, 100.0, 0.0], [0.0, 0.0, 0.0], false);
    let proj = proj_offset([0.0, 0.0, 0.0], [20.0, 0.0, 0.0], 5000.0, 0.4);
    expect_projectile_hits_target(&proj, &target, 400.0, 1.0 / 1000.0, 50.0, 5.0, 10.0);
}

#[test]
fn hits_static_target_large_y_offset() {
    let target = target([150.0, -200.0, 30.0], [0.0, 0.0, 0.0], false);
    let proj = proj_offset([0.0, 0.0, 0.0], [0.0, 15.0, 0.0], 5000.0, 0.4);
    expect_projectile_hits_target(&proj, &target, 400.0, 1.0 / 1000.0, 50.0, 5.0, 10.0);
}

#[test]
fn hits_static_target_large_z_offset() {
    let target = target([100.0, 0.0, 200.0], [0.0, 0.0, 0.0], false);
    let proj = proj_offset([0.0, 0.0, 0.0], [0.0, 0.0, -10.0], 5000.0, 0.4);
    expect_projectile_hits_target(&proj, &target, 400.0, 1.0 / 1000.0, 50.0, 5.0, 10.0);
}

#[test]
fn hits_static_target_all_axes_offset() {
    let target = target([250.0, 80.0, 60.0], [0.0, 0.0, 0.0], false);
    let proj = proj_offset([10.0, 5.0, 20.0], [8.0, -4.0, -6.0], 5000.0, 0.4);
    expect_projectile_hits_target(&proj, &target, 400.0, 1.0 / 1000.0, 50.0, 5.0, 10.0);
}

#[test]
fn hits_moving_target_with_offset() {
    let target = target([400.0, 0.0, 50.0], [-30.0, 10.0, 5.0], false);
    let proj = proj_offset([0.0, 0.0, 0.0], [10.0, -5.0, 2.0], 3000.0, 0.8);
    expect_projectile_hits_target(&proj, &target, 800.0, 1.0 / 500.0, 30.0, 10.0, 15.0);
}

#[test]
fn hits_airborne_target_with_offset() {
    let target = target([150.0, 80.0, 250.0], [5.0, -10.0, -30.0], true);
    let proj = proj_offset([0.0, 0.0, 50.0], [3.0, 7.0, -5.0], 4000.0, 0.5);
    expect_projectile_hits_target(&proj, &target, 400.0, 1.0 / 1000.0, 50.0, 10.0, 15.0);
}

#[test]
fn hits_negative_yaw_target_with_offset() {
    let target = target([-200.0, -150.0, 10.0], [0.0, 0.0, 0.0], false);
    let proj = proj_offset([0.0, 0.0, 0.0], [-5.0, 3.0, 2.0], 5000.0, 0.4);
    expect_projectile_hits_target(&proj, &target, 400.0, 1.0 / 1000.0, 50.0, 5.0, 10.0);
}

#[test]
fn aim_angles_returns_nullopt_when_no_solution() {
    let target = target([100000.0, 0.0, 0.0], [0.0, 0.0, 0.0], false);
    let proj = proj([0.0, 0.0, 0.0], 1.0, 1.0);

    let engine = Engine::new(9.81, 0.1, 2.0, 5.0);

    assert!(engine.maybe_calculate_aim_point(&proj, &target).is_none());
    assert!(engine.maybe_calculate_aim_angles(&proj, &target).is_none());
}

// Regression tests for the legacy engine cleanup: fixed step count, pitch against a double reference, vertical shots.

#[test]
fn zero_time_step_returns_nullopt_instead_of_hanging() {
    let target = target([100.0, 0.0, 90.0], [0.0, 0.0, 0.0], false);
    let proj = proj([3.0, 2.0, 1.0], 5000.0, 0.4);

    assert!(
        Engine::new(400.0, 0.0, 50.0, 5.0)
            .maybe_calculate_aim_point(&proj, &target)
            .is_none()
    );
    assert!(
        Engine::new(400.0, -1.0, 50.0, 5.0)
            .maybe_calculate_aim_angles(&proj, &target)
            .is_none()
    );
    assert!(
        Engine::new(400.0, 1.0 / 1000.0, 0.0, 5.0)
            .maybe_calculate_aim_point(&proj, &target)
            .is_none()
    );
}

#[test]
fn high_speed_pitch_matches_double_reference() {
    // Static target, so the solved pitch is the closed form for a fixed (horizontal distance, height) pair.
    let target = target([100.0, 0.0, 90.0], [0.0, 0.0, 0.0], false);
    let proj = proj([3.0, 2.0, 1.0], 5000.0, 0.4);

    let aim_angles = Engine::new(400.0, 1.0 / 1000.0, 50.0, 5.0)
        .maybe_calculate_aim_angles(&proj, &target)
        .unwrap();

    let v_ = 5000.0_f64;
    let g = 400.0_f64 * 0.4;
    let x = (100.0_f64 - 3.0).hypot(0.0 - 2.0);
    let y = 90.0_f64 - 1.0;
    let reference = ((v_ * v_ - (v_ * v_ * v_ * v_ - g * (g * x * x + 2.0 * y * v_ * v_)).sqrt())
        / (g * x))
        .atan()
        * 180.0
        / std::f64::consts::PI;

    // At this speed v^2 and sqrt(D) are nearly equal and their difference keeps few float digits: the pitch comes out
    // about 0.002 degrees off the double reference, which over these 97 units is 0.003 of a unit.
    expect_near(aim_angles.pitch, reference as f32, 1e-2_f32);
}

#[test]
fn high_arc_reaches_the_target_steeper_and_later() {
    let target = target([800.0, 100.0, 64.0], [0.0, 0.0, 0.0], false);
    let proj = proj([0.0, 0.0, 64.0], 1200.0, 1.0);

    let engine = Engine::new(800.0, 1.0 / 1000.0, 5.0, 5.0);
    let low = engine
        .with_arc(Arc::Low)
        .maybe_calculate_aim_angles(&proj, &target)
        .unwrap();
    let high = engine
        .with_arc(Arc::High)
        .maybe_calculate_aim_angles(&proj, &target)
        .unwrap();

    // The two roots of the same quadratic: their tangents multiply to 2 y v^2 / (g x^2) + 1, which is 1 on the level
    let pi = std::f32::consts::PI;
    expect_near(
        (low.pitch * pi / 180.0).tan() * (high.pitch * pi / 180.0).tan(),
        1.0_f32,
        1e-2_f32,
    );
    assert!(high.pitch > 45.0);
    assert!(low.pitch < 45.0);
    expect_near(high.yaw, low.yaw, 1e-3_f32);
}

#[test]
fn without_gravity_the_arcs_agree() {
    let target = target([800.0, 100.0, 200.0], [0.0, 0.0, 0.0], false);
    let proj = proj([0.0, 0.0, 64.0], 1100.0, 0.0);

    let engine = Engine::new(800.0, 1.0 / 1000.0, 5.0, 5.0);
    let low = engine
        .with_arc(Arc::Low)
        .maybe_calculate_aim_angles(&proj, &target)
        .unwrap();
    let high = engine
        .with_arc(Arc::High)
        .maybe_calculate_aim_angles(&proj, &target)
        .unwrap();

    expect_near(high.pitch, low.pitch, 1e-4_f32);
    expect_near(high.yaw, low.yaw, 1e-4_f32);
}

#[test]
fn vertical_shots_use_direct_pitch() {
    let proj = proj([0.0, 0.0, 0.0], 500.0, 1.0);
    let engine = Engine::new(400.0, 1.0 / 1000.0, 5.0, 5.0);

    let above = target([0.0, 0.0, 100.0], [0.0, 0.0, 0.0], false);
    let up = engine.maybe_calculate_aim_angles(&proj, &above).unwrap();
    expect_float_eq(up.pitch, 90.0);

    let below = target([0.0, 0.0, -100.0], [0.0, 0.0, 0.0], false);
    let down = engine.maybe_calculate_aim_angles(&proj, &below).unwrap();
    expect_float_eq(down.pitch, -90.0);
}

#[test]
fn aim_point_and_angles_agree_with_lateral_offset() {
    // The aim point lies on the eye's aim ray, so the camera angles towards it are the launch angles even when the
    // muzzle sits 50 units to the side and fires along a different bearing than the camera-to-target line.
    let target = target([200.0, 0.0, 0.0], [0.0, 0.0, 0.0], false);
    let proj = proj_offset([0.0, 0.0, 0.0], [0.0, 50.0, 0.0], 5000.0, 0.4);

    let engine = Engine::new(400.0, 1.0 / 1000.0, 50.0, 5.0);
    let aim_angles = engine.maybe_calculate_aim_angles(&proj, &target).unwrap();
    let aim_point = engine.maybe_calculate_aim_point(&proj, &target).unwrap();

    let expected_yaw = angles::radians_to_degrees((0.0_f32 - 50.0).atan2(200.0 - 0.0));
    expect_near(aim_angles.yaw, expected_yaw, 0.05_f32);

    let camera = CameraTrait::calc_look_at_angle(proj.origin, aim_point);
    expect_near(aim_angles.pitch, -camera.pitch.as_degrees(), 0.01_f32);
    expect_near(aim_angles.yaw, camera.yaw.as_degrees(), 0.01_f32);
}

#[test]
fn engine_is_assignable() {
    fn is_copy<T: Copy>() {}

    is_copy::<Engine>();
}

// The Launcher API: view-relative muzzle offset

/// The projectile leaves from eye + basis(angles) * offset, so the returned angles have to hit from exactly there.
/// Simulate that with the trait at the solution's own time of flight.
#[allow(clippy::too_many_arguments)]
fn expect_launcher_solution_hits(
    proj: &Projectile,
    launcher: &Launcher,
    target: &Target,
    gravity: f32,
    engine_step: f32,
    max_time: f32,
    engine_tolerance: f32,
    hit_tolerance: f32,
) {
    let engine = Engine::new(gravity, engine_step, max_time, engine_tolerance);

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

    assert!(
        proj_pos.distance_to(tgt_pos) <= hit_tolerance,
        "projectile fired from the rotated muzzle must reach the target"
    );
    expect_near(solution.predicted_target_position.x, tgt_pos.x, 1e-3_f32);
    expect_near(solution.predicted_target_position.y, tgt_pos.y, 1e-3_f32);
    expect_near(solution.predicted_target_position.z, tgt_pos.z, 1e-3_f32);

    // The aim point sits on the eye's aim ray
    let camera = CameraTrait::calc_look_at_angle(launcher.eye_origin, solution.aim_point);
    expect_near(pitch, -camera.pitch.as_degrees(), 0.01_f32);
    expect_near(yaw, camera.yaw.as_degrees(), 0.01_f32);
}

/// Source-style spawn: 16 forward, 8 to the right, 6 down from the eye.
const K_SOURCE_MUZZLE: MuzzleOffset<f32> = MuzzleOffset {
    forward: 16.0,
    right: 8.0,
    up: -6.0,
};

fn eye(x: f32, y: f32, z: f32) -> Launcher {
    Launcher {
        eye_origin: v(x, y, z),
        ..Default::default()
    }
}

fn muzzle_launcher() -> Launcher {
    Launcher {
        muzzle_offset: K_SOURCE_MUZZLE,
        ..eye(0.0, 0.0, 64.0)
    }
}

#[test]
fn static_target_with_muzzle_offset() {
    let target = target([200.0, 0.0, 50.0], [0.0, 0.0, 0.0], false);
    expect_launcher_solution_hits(
        &round(5000.0, 0.4),
        &muzzle_launcher(),
        &target,
        400.0,
        1.0 / 1000.0,
        50.0,
        5.0,
        10.0,
    );
}

#[test]
fn moving_target_with_muzzle_offset() {
    let target = target([500.0, 100.0, 0.0], [-50.0, 20.0, 0.0], false);
    expect_launcher_solution_hits(
        &round(3000.0, 1.0),
        &muzzle_launcher(),
        &target,
        800.0,
        1.0 / 500.0,
        30.0,
        10.0,
        15.0,
    );
}

#[test]
fn side_target_with_muzzle_offset() {
    // Target off to the left so the rotated muzzle ends up well away from where a world-space offset would put it
    let target = target([-50.0, 300.0, 80.0], [10.0, -5.0, 0.0], false);
    expect_launcher_solution_hits(
        &round(2500.0, 1.0),
        &muzzle_launcher(),
        &target,
        800.0,
        1.0 / 1000.0,
        30.0,
        10.0,
        15.0,
    );
}

#[test]
fn world_and_view_offsets_combine() {
    let target = target([300.0, -120.0, 20.0], [0.0, 0.0, 0.0], false);
    let launcher = Launcher {
        muzzle_offset: K_SOURCE_MUZZLE,
        world_offset: v(0.0, 0.0, -10.0),
        ..eye(10.0, 5.0, 64.0)
    };

    expect_launcher_solution_hits(
        &round(4000.0, 0.5),
        &launcher,
        &target,
        400.0,
        1.0 / 1000.0,
        50.0,
        5.0,
        10.0,
    );
}

#[test]
fn muzzle_offset_changes_the_answer() {
    // A muzzle 8 units to the right must fire on a different bearing than one at the eye
    let target = target([300.0, 0.0, 64.0], [0.0, 0.0, 0.0], false);
    let proj = round(5000.0, 0.4);
    let engine = Engine::new(400.0, 1.0 / 1000.0, 50.0, 5.0);

    let centred = engine
        .maybe_calculate_aim(&proj, &eye(0.0, 0.0, 64.0), &target)
        .unwrap();
    let offset = engine
        .maybe_calculate_aim(
            &proj,
            &Launcher {
                muzzle_offset: MuzzleOffset {
                    forward: 0.0,
                    right: 8.0,
                    up: 0.0,
                },
                ..eye(0.0, 0.0, 64.0)
            },
            &target,
        )
        .unwrap();

    // Source right is -Y at zero yaw, so the muzzle sits at y = -8 and must yaw towards +Y by atan(8 / 300)
    let expected_shift = angles::radians_to_degrees(8.0_f32.atan2(300.0));
    expect_near(centred.angles.yaw, 0.0_f32, 0.05_f32);
    expect_near(
        offset.angles.yaw - centred.angles.yaw,
        expected_shift,
        0.05_f32,
    );
}

#[test]
fn wrappers_match_launcher_with_world_offset() {
    use omath::projectile_prediction::launcher_from_projectile;

    let target = target([200.0, 0.0, 50.0], [0.0, 0.0, 0.0], false);
    let proj = proj_offset([0.0, 0.0, 64.0], [5.0, 0.0, -3.0], 5000.0, 0.4);
    let engine = Engine::new(400.0, 1.0 / 1000.0, 50.0, 5.0);

    let solution = engine
        .maybe_calculate_aim(&proj, &launcher_from_projectile(&proj), &target)
        .unwrap();
    let point = engine.maybe_calculate_aim_point(&proj, &target).unwrap();
    let angles = engine.maybe_calculate_aim_angles(&proj, &target).unwrap();

    assert_eq!(solution.aim_point, point);
    expect_float_eq(solution.angles.pitch, angles.pitch);
    expect_float_eq(solution.angles.yaw, angles.yaw);
}

// Launch pitch offset: weapons that fire above the crosshair, such as TF2's pipe and sticky launchers

/// TF2 sets a pipe's velocity to forward * 1200 + up * 200 in the view frame.
fn pipe_speed() -> f32 {
    1200.0_f32.hypot(200.0)
}

fn pipe_pitch_offset() -> f32 {
    angles::radians_to_degrees(200.0_f32.atan2(1200.0))
}

#[test]
fn pitch_offset_is_forward_plus_up_velocity() {
    // The equivalence the feature rests on: forward * a + up * b in the view frame is the view direction pitched up by
    // atan(b / a), at a speed of hypot(a, b).
    let basis = Trait::calc_view_basis(20.0, 35.0);
    let game_velocity = basis.forward * 1200.0 + basis.up * 200.0;
    let offset_velocity =
        Trait::calc_view_basis(20.0 + pipe_pitch_offset(), 35.0).forward * pipe_speed();

    expect_near(pipe_pitch_offset(), 9.4623_f32, 1e-3_f32);
    expect_near(pipe_speed(), 1216.55_f32, 1e-2_f32);
    expect_near(
        game_velocity.distance_to(offset_velocity),
        0.0_f32,
        1e-2_f32,
    );
}

#[test]
fn pipe_launcher_fired_like_the_game_hits() {
    let gravity = 800.0_f32;
    let target = target([600.0, 150.0, 0.0], [-40.0, 30.0, 0.0], false);
    let proj = round(pipe_speed(), 0.5);
    let launcher = Launcher {
        launch_pitch_offset: pipe_pitch_offset(),
        ..muzzle_launcher()
    };

    let aim = Engine::new(gravity, 1.0 / 1000.0, 5.0, 5.0)
        .maybe_calculate_aim(&proj, &launcher, &target)
        .unwrap();

    // Fire the way the game does rather than through the offset: forward * 1200 + up * 200 in the view frame, from the
    // view-relative muzzle, with the view angles the engine returned.
    let basis = Trait::calc_view_basis(aim.angles.pitch, aim.angles.yaw);
    let time = aim.time_of_flight;
    let mut position =
        launcher.launch_origin(basis) + (basis.forward * 1200.0 + basis.up * 200.0) * time;
    position.z -= 0.5 * gravity * proj.gravity_scale * time * time;

    assert!(position.distance_to(Trait::predict_target_position(&target, time, gravity)) <= 6.0);

    // Left out, the same shot lands far away: the engine would aim the crosshair where the round has to go
    let no_offset = muzzle_launcher();
    let naive = Engine::new(gravity, 1.0 / 1000.0, 5.0, 5.0)
        .maybe_calculate_aim(&proj, &no_offset, &target)
        .unwrap();
    let naive_basis = Trait::calc_view_basis(naive.angles.pitch, naive.angles.yaw);
    let naive_time = naive.time_of_flight;
    let mut naive_position = no_offset.launch_origin(naive_basis)
        + (naive_basis.forward * 1200.0 + naive_basis.up * 200.0) * naive_time;
    naive_position.z -= 0.5 * gravity * proj.gravity_scale * naive_time * naive_time;
    assert!(
        naive_position.distance_to(Trait::predict_target_position(&target, naive_time, gravity))
            > 50.0
    );
}

#[test]
fn pitch_offset_with_side_and_airborne_targets() {
    let proj = round(pipe_speed(), 0.5);
    let launcher = Launcher {
        launch_pitch_offset: pipe_pitch_offset(),
        ..muzzle_launcher()
    };

    let side = target([-50.0, 300.0, 80.0], [10.0, -5.0, 0.0], false);
    expect_launcher_solution_hits(
        &proj,
        &launcher,
        &side,
        800.0,
        1.0 / 1000.0,
        30.0,
        5.0,
        10.0,
    );

    let airborne = target([400.0, -100.0, 300.0], [10.0, -5.0, -20.0], true);
    expect_launcher_solution_hits(
        &proj,
        &launcher,
        &airborne,
        800.0,
        1.0 / 1000.0,
        30.0,
        5.0,
        10.0,
    );

    // A weapon that fires below the crosshair
    let downward = Launcher {
        launch_pitch_offset: -4.0,
        ..muzzle_launcher()
    };
    expect_launcher_solution_hits(
        &proj,
        &downward,
        &side,
        800.0,
        1.0 / 1000.0,
        30.0,
        5.0,
        10.0,
    );
}

#[test]
fn pitch_offset_shifts_view_pitch_by_exactly_the_offset() {
    // Without a rotating muzzle the launch is the same shot either way; only the view pitch that produces it moves
    let target = target([500.0, 100.0, 40.0], [0.0, 0.0, 0.0], false);
    let proj = round(pipe_speed(), 0.5);
    let engine = Engine::new(800.0, 1.0 / 1000.0, 5.0, 5.0);

    let plain = engine
        .maybe_calculate_aim(&proj, &eye(0.0, 0.0, 64.0), &target)
        .unwrap();
    let offset = engine
        .maybe_calculate_aim(
            &proj,
            &Launcher {
                launch_pitch_offset: pipe_pitch_offset(),
                ..eye(0.0, 0.0, 64.0)
            },
            &target,
        )
        .unwrap();

    expect_near(
        offset.angles.pitch,
        plain.angles.pitch - pipe_pitch_offset(),
        1e-4_f32,
    );
    expect_float_eq(offset.angles.yaw, plain.angles.yaw);
    expect_float_eq(offset.time_of_flight, plain.time_of_flight);

    // The aim point follows the view, not the launch
    let camera = CameraTrait::calc_look_at_angle(v(0.0, 0.0, 64.0), offset.aim_point);
    expect_near(offset.angles.pitch, -camera.pitch.as_degrees(), 0.01_f32);
}

#[test]
fn unreachable_view_pitch_is_rejected() {
    // Almost straight down: the launch pitch is about -88, so with a +9.46 offset the view would have to sit near
    // -97.5. Source stops at -89, so nobody can set that shot up.
    let below = target([30.0, 0.0, 0.0], [0.0, 0.0, 0.0], false);
    let proj = round(pipe_speed(), 0.5);
    let engine = Engine::new(800.0, 1.0 / 1000.0, 5.0, 5.0);

    let plain = engine
        .maybe_calculate_aim(&proj, &eye(0.0, 0.0, 1000.0), &below)
        .unwrap();
    assert!(plain.angles.pitch < -85.0);

    assert!(
        engine
            .maybe_calculate_aim(
                &proj,
                &Launcher {
                    launch_pitch_offset: pipe_pitch_offset(),
                    ..eye(0.0, 0.0, 1000.0)
                },
                &below
            )
            .is_none()
    );
}
