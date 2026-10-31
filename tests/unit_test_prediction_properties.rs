//! Properties of ProjPredEngineLegacy's answers that hold whatever the numbers are: symmetries of the scene, how the
//! answer moves with range and speed, what the horizon and a rejected step do, and how the answer lines up with a
//! camera. Source trait throughout; the per-engine checks live in tests/unit_test_pred_engine_traits_all.rs.

mod common;

use common::{expect_float_eq, expect_near};
use omath::Vector3;
use omath::engines::source::{self, CameraTrait, PredEngineTrait as Trait};
use omath::projectile_prediction::{
    MuzzleOffset, PredEngine, ProjPredEngineInterface, ProjPredEngineLegacy,
};
use omath::projection::{CameraEngine, FieldOfView, ViewPort};
use omath::trigonometry::angles;

type Projectile = omath::projectile_prediction::Projectile<f32>;
type Target = omath::projectile_prediction::Target<f32>;
type Launcher = omath::projectile_prediction::Launcher<f32>;
type Engine = ProjPredEngineLegacy;

const K_GRAVITY: f32 = 800.0;
const K_STEP: f32 = 1.0 / 1000.0;
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

fn make_engine(horizon: f32, tolerance: f32, step: f32) -> Engine {
    Engine::new(K_GRAVITY, step, horizon, tolerance)
}

fn default_engine() -> Engine {
    make_engine(10.0, 5.0, K_STEP)
}

fn pipe_pitch_offset() -> f32 {
    angles::radians_to_degrees(200.0_f32.atan2(1200.0))
}

/// `std::remainder(a - b, 360)`.
fn angle_difference(a: f32, b: f32) -> f32 {
    let difference = a - b;
    difference - 360.0 * (difference / 360.0).round_ties_even()
}

/// Source is Z up, so this turns the scene about the vertical through the origin.
fn rotate_about_up(vector: Vector3<f32>, degrees: f32) -> Vector3<f32> {
    let radians = angles::degrees_to_radians(degrees);
    v(
        vector.x * radians.cos() - vector.y * radians.sin(),
        vector.x * radians.sin() + vector.y * radians.cos(),
        vector.z,
    )
}

// Symmetries

#[test]
fn turning_the_scene_about_the_eye_only_shifts_the_yaw() {
    let round = round(1500.0, 1.0);
    // The muzzle offset turns with the view, so the scene stays symmetric with it in place
    let launcher = Launcher {
        muzzle_offset: K_MUZZLE,
        ..eye(0.0, 0.0, 64.0)
    };
    let target = grounded(v(700.0, 120.0, 90.0), v(-60.0, 45.0, 0.0));

    let engine = default_engine();
    let reference = engine
        .maybe_calculate_aim(&round, &launcher, &target)
        .unwrap();

    for turn in [37.0, 143.0, -90.0, 180.0] {
        let turned = grounded(
            rotate_about_up(target.origin, turn),
            rotate_about_up(target.velocity, turn),
        );

        let aim = engine
            .maybe_calculate_aim(&round, &launcher, &turned)
            .unwrap();

        expect_near(aim.angles.pitch, reference.angles.pitch, 0.02_f32);
        expect_near(
            angle_difference(aim.angles.yaw, reference.angles.yaw + turn),
            0.0_f32,
            0.02_f32,
        );
        expect_near(aim.time_of_flight, reference.time_of_flight, 1.5 * K_STEP);
    }
}

#[test]
fn a_world_offset_does_not_turn_with_the_scene() {
    // Same turn, but the spawn offset is fixed in the world: 40 units to the side when facing +X, 40 units ahead once
    // the scene has turned a quarter. The bearing to the target changes by about atan(40 / 700) on top of the turn,
    // which is the whole difference between world_offset and muzzle_offset.
    let round = round(1500.0, 1.0);
    let launcher = Launcher {
        world_offset: v(0.0, 40.0, 0.0),
        ..eye(0.0, 0.0, 64.0)
    };
    let target = grounded(v(700.0, 0.0, 64.0), Vector3::default());

    let engine = default_engine();
    let reference = engine
        .maybe_calculate_aim(&round, &launcher, &target)
        .unwrap();
    let turned = engine
        .maybe_calculate_aim(
            &round,
            &launcher,
            &grounded(rotate_about_up(target.origin, 90.0), Vector3::default()),
        )
        .unwrap();

    assert!(angle_difference(turned.angles.yaw, reference.angles.yaw + 90.0).abs() > 2.0);
}

#[test]
fn moving_the_whole_scene_moves_only_the_points() {
    let round = round(1500.0, 1.0);
    let target = grounded(v(700.0, 120.0, 90.0), v(-60.0, 45.0, 0.0));
    let eye = v(0.0, 0.0, 64.0);
    let shift = v(1000.0, -2000.0, 300.0);

    let engine = default_engine();
    let reference = engine
        .maybe_calculate_aim(
            &round,
            &Launcher {
                eye_origin: eye,
                muzzle_offset: K_MUZZLE,
                ..Default::default()
            },
            &target,
        )
        .unwrap();
    let moved = engine
        .maybe_calculate_aim(
            &round,
            &Launcher {
                eye_origin: eye + shift,
                muzzle_offset: K_MUZZLE,
                ..Default::default()
            },
            &grounded(target.origin + shift, target.velocity),
        )
        .unwrap();

    expect_near(moved.angles.pitch, reference.angles.pitch, 0.02_f32);
    expect_near(moved.angles.yaw, reference.angles.yaw, 0.02_f32);
    expect_near(moved.time_of_flight, reference.time_of_flight, 1.5 * K_STEP);
    assert!((moved.aim_point - shift).distance_to(reference.aim_point) <= 1.0);
    assert!(
        (moved.predicted_target_position - shift).distance_to(reference.predicted_target_position)
            <= 0.2
    );
}

#[test]
fn mirroring_the_scene_flips_the_yaw() {
    let round = round(1500.0, 1.0);
    // Nothing off to one side on the shooter, so left and right are the same problem
    let launcher = Launcher {
        muzzle_offset: MuzzleOffset {
            forward: 16.0,
            right: 0.0,
            up: -6.0,
        },
        ..eye(0.0, 0.0, 64.0)
    };
    let target = grounded(v(700.0, 120.0, 90.0), v(-60.0, 45.0, -5.0));
    let mirrored = grounded(v(700.0, -120.0, 90.0), v(-60.0, -45.0, -5.0));

    let engine = default_engine();
    let aim = engine
        .maybe_calculate_aim(&round, &launcher, &target)
        .unwrap();
    let mirror = engine
        .maybe_calculate_aim(&round, &launcher, &mirrored)
        .unwrap();

    expect_near(mirror.angles.pitch, aim.angles.pitch, 1e-3_f32);
    expect_near(mirror.angles.yaw, -aim.angles.yaw, 1e-3_f32);
    expect_float_eq(mirror.time_of_flight, aim.time_of_flight);
}

// Shape of the answer

#[test]
fn time_of_flight_is_a_whole_number_of_steps_and_the_aim_point_is_as_far_as_the_target() {
    let round = round(1100.0, 0.4);
    let launcher = Launcher {
        muzzle_offset: K_MUZZLE,
        ..eye(10.0, -20.0, 64.0)
    };
    let target = grounded(v(900.0, 250.0, 0.0), v(-120.0, 80.0, 0.0));

    for step in [1.0 / 1000.0, 1.0 / 250.0, 1.0 / 64.0] {
        let aim = make_engine(10.0, 20.0, step)
            .maybe_calculate_aim(&round, &launcher, &target)
            .unwrap();

        let steps = aim.time_of_flight / step;
        expect_near(steps, steps.round(), 1e-2_f32);

        expect_near(
            launcher.eye_origin.distance_to(aim.aim_point),
            launcher
                .eye_origin
                .distance_to(aim.predicted_target_position),
            0.05_f32,
        );

        let expected_target = target.origin + target.velocity * aim.time_of_flight;
        assert!(aim.predicted_target_position.distance_to(expected_target) <= 1e-2);
    }
}

#[test]
fn farther_targets_take_longer_and_need_more_arc() {
    let round = round(1500.0, 1.0);
    let launcher = eye(0.0, 0.0, 64.0);
    let engine = default_engine();

    let mut previous_time = 0.0;
    let mut previous_pitch = -90.0;
    for distance in [400.0, 800.0, 1200.0, 1600.0, 2000.0] {
        let aim = engine
            .maybe_calculate_aim(
                &round,
                &launcher,
                &grounded(v(distance, 0.0, 64.0), Vector3::default()),
            )
            .unwrap();

        assert!(aim.time_of_flight > previous_time, "distance {distance}");
        assert!(aim.angles.pitch > previous_pitch, "distance {distance}");
        expect_near(aim.angles.yaw, 0.0_f32, 1e-3_f32);

        previous_time = aim.time_of_flight;
        previous_pitch = aim.angles.pitch;
    }
}

#[test]
fn a_faster_round_needs_less_lead_and_less_arc() {
    let launcher = eye(0.0, 0.0, 64.0);
    let crossing = grounded(v(800.0, 0.0, 64.0), v(0.0, 200.0, 0.0));
    let engine = default_engine();

    let slow = engine
        .maybe_calculate_aim(&round(1000.0, 1.0), &launcher, &crossing)
        .unwrap();
    let fast = engine
        .maybe_calculate_aim(&round(3000.0, 1.0), &launcher, &crossing)
        .unwrap();

    assert!(slow.angles.yaw > fast.angles.yaw); // target runs towards +Y, which is a positive yaw in Source
    assert!(fast.angles.yaw > 0.0);
    assert!(slow.angles.pitch > fast.angles.pitch);
    assert!(slow.time_of_flight > fast.time_of_flight);
}

#[test]
fn a_tighter_tolerance_never_gives_a_looser_hit() {
    let round = round(2000.0, 0.5);
    let launcher = Launcher {
        muzzle_offset: K_MUZZLE,
        ..eye(0.0, 0.0, 64.0)
    };
    let target = grounded(v(1200.0, -300.0, 150.0), v(30.0, 60.0, 0.0));

    for tolerance in [40.0, 10.0, 2.0] {
        let aim = make_engine(10.0, tolerance, K_STEP)
            .maybe_calculate_aim(&round, &launcher, &target)
            .unwrap();

        let basis = Trait::calc_view_basis(aim.angles.pitch, aim.angles.yaw);
        let position = Trait::predict_projectile_position(
            launcher.launch_origin(basis),
            &round,
            aim.angles.pitch,
            aim.angles.yaw,
            aim.time_of_flight,
            K_GRAVITY,
        );
        // +1: the muzzle is placed from the previous pass's angles, which costs a hair
        assert!(
            position.distance_to(aim.predicted_target_position) <= tolerance + 1.0,
            "tolerance {tolerance}"
        );
    }
}

// Horizon and rejected steps

#[test]
fn a_horizon_shorter_than_the_flight_finds_nothing() {
    let round = round(1500.0, 0.5);
    let launcher = eye(0.0, 0.0, 64.0);
    let target = grounded(v(3000.0, 0.0, 64.0), Vector3::default());

    assert!(
        make_engine(1.5, 5.0, K_STEP)
            .maybe_calculate_aim(&round, &launcher, &target)
            .is_none()
    );

    let aim = make_engine(3.0, 5.0, K_STEP)
        .maybe_calculate_aim(&round, &launcher, &target)
        .unwrap();
    assert!(aim.time_of_flight > 1.9);
    assert!(aim.time_of_flight < 2.3);
}

#[test]
fn a_rejected_step_does_not_end_the_scan() {
    // A tolerance this wide lets every step through the reach check, so the only thing deciding the answer is whether
    // the view pitch can be set. The target starts almost straight below (view about -97.7 with the offset, past
    // Source's -89) and walks out from under the shooter; the first step it can be aimed at wins.
    let step = 0.05;
    let round = round(1100.0, 0.0);
    let target = grounded(v(30.0, 0.0, 0.0), v(300.0, 0.0, 0.0));
    let eye_origin = v(0.0, 0.0, 1000.0);
    let offset = pipe_pitch_offset();

    let engine = Engine::new(K_GRAVITY, step, 5.0, 1e9);

    let plain = engine
        .maybe_calculate_aim(&round, &eye(0.0, 0.0, 1000.0), &target)
        .unwrap();
    expect_float_eq(plain.time_of_flight, 0.0);

    let aim = engine
        .maybe_calculate_aim(
            &round,
            &Launcher {
                launch_pitch_offset: offset,
                ..eye(0.0, 0.0, 1000.0)
            },
            &target,
        )
        .unwrap();
    assert!(aim.time_of_flight > 0.0);
    assert!(aim.angles.pitch >= -89.1);

    // ...and it is the first such step: one step earlier the view would still have been out of reach
    let one_step_earlier =
        Trait::predict_target_position(&target, aim.time_of_flight - step, K_GRAVITY);
    assert!(Trait::calc_direct_pitch_angle(eye_origin, one_step_earlier) - offset < -89.1);
}

#[test]
fn an_offset_decides_whether_an_overhead_shot_can_be_set_up() {
    // Straight up needs a launch pitch of 90. A weapon that throws 9.46 degrees above the crosshair gets there from a
    // view of 80.54; one that throws below it would need a view of 99.46.
    let round = round(500.0, 1.0);
    let overhead = grounded(v(0.0, 0.0, 100.0), Vector3::default());
    let engine = Engine::new(400.0, K_STEP, 5.0, 5.0);
    let offset = pipe_pitch_offset();

    let upward = engine
        .maybe_calculate_aim(
            &round,
            &Launcher {
                launch_pitch_offset: offset,
                ..Default::default()
            },
            &overhead,
        )
        .unwrap();
    expect_near(upward.angles.pitch, 90.0 - offset, 0.01_f32);

    assert!(
        engine
            .maybe_calculate_aim(
                &round,
                &Launcher {
                    launch_pitch_offset: -offset,
                    ..Default::default()
                },
                &overhead
            )
            .is_none()
    );
}

#[test]
fn without_gravity_the_view_is_the_direct_angle_minus_the_offset() {
    let rocket = round(1100.0, 0.0);
    let eye_origin = v(0.0, 0.0, 64.0);
    let target = grounded(v(600.0, -250.0, 300.0), Vector3::default());

    for offset in [0.0, 5.0, -5.0, pipe_pitch_offset()] {
        let aim = default_engine()
            .maybe_calculate_aim(
                &rocket,
                &Launcher {
                    launch_pitch_offset: offset,
                    ..eye(0.0, 0.0, 64.0)
                },
                &target,
            )
            .unwrap();

        expect_near(
            aim.angles.pitch,
            Trait::calc_direct_pitch_angle(eye_origin, target.origin) - offset,
            1e-4_f32,
        );
        expect_near(
            aim.angles.yaw,
            Trait::calc_direct_yaw_angle(eye_origin, target.origin),
            1e-4_f32,
        );
    }
}

// Offsets and the old call style

#[test]
fn a_world_offset_below_the_eye_aims_higher() {
    let round = round(1500.0, 0.5);
    let target = grounded(v(900.0, 100.0, 64.0), Vector3::default());
    let engine = default_engine();

    let from_the_eye = engine
        .maybe_calculate_aim(&round, &eye(0.0, 0.0, 64.0), &target)
        .unwrap();
    let from_below = engine
        .maybe_calculate_aim(
            &round,
            &Launcher {
                world_offset: v(0.0, 0.0, -40.0),
                ..eye(0.0, 0.0, 64.0)
            },
            &target,
        )
        .unwrap();

    // 40 units lower over 900 is about 2.5 degrees more elevation
    expect_near(
        from_below.angles.pitch - from_the_eye.angles.pitch,
        angles::radians_to_degrees(40.0_f32.atan2(905.0)),
        0.1_f32,
    );
    expect_near(from_below.angles.yaw, from_the_eye.angles.yaw, 1e-3_f32);

    // The aim point still belongs to the eye, not to the muzzle
    let camera = CameraTrait::calc_look_at_angle(v(0.0, 0.0, 64.0), from_below.aim_point);
    expect_near(
        -camera.pitch.as_degrees(),
        from_below.angles.pitch,
        0.01_f32,
    );
    expect_near(camera.yaw.as_degrees(), from_below.angles.yaw, 0.01_f32);
}

#[test]
fn old_style_aim_point_stays_on_the_ray_the_old_formula_gave() {
    use omath::projectile_prediction::launcher_from_projectile;

    // Before Launcher the aim point was (target.x, target.y, eye.z + ground_distance * tan(pitch)). The wrapper now
    // returns a point at the target's distance along the view ray instead. Code that only looks at it (sets angles from
    // it, projects it) must see no difference, so the two points have to lie on one ray from the eye.
    let round = Projectile {
        origin: v(3.0, 2.0, 1.0),
        ..round(3000.0, 0.6)
    };
    let target = grounded(v(900.0, -350.0, 240.0), v(-35.0, 50.0, 0.0));
    let engine = default_engine();

    let point = engine.maybe_calculate_aim_point(&round, &target).unwrap();
    let angles_result = engine.maybe_calculate_aim_angles(&round, &target).unwrap();
    let solution = engine
        .maybe_calculate_aim(&round, &launcher_from_projectile(&round), &target)
        .unwrap();

    let predicted = solution.predicted_target_position;
    let ground_distance = Trait::calc_vector_2d_distance(predicted - round.origin);
    let old_point = v(
        predicted.x,
        predicted.y,
        round.origin.z + ground_distance * angles::degrees_to_radians(angles_result.pitch).tan(),
    );

    let new_direction = (point - round.origin).normalized();
    let old_direction = (old_point - round.origin).normalized();
    assert!(new_direction.distance_to(old_direction) <= 1e-4);
}

// Together with a camera

#[test]
fn the_solved_view_puts_the_aim_point_under_the_crosshair() {
    let pipe = round(1200.0_f32.hypot(200.0), 0.5);
    let launcher = Launcher {
        muzzle_offset: K_MUZZLE,
        launch_pitch_offset: pipe_pitch_offset(),
        ..eye(0.0, 0.0, 64.0)
    };
    let target = grounded(v(600.0, 150.0, 0.0), v(-40.0, 30.0, 0.0));

    let aim = default_engine()
        .maybe_calculate_aim(&pipe, &launcher, &target)
        .unwrap();

    // Source's own pitch is negative upwards
    let camera = source::Camera::new(
        launcher.eye_origin,
        source::ViewAngles::new(
            source::PitchAngle::from_degrees(-aim.angles.pitch),
            source::YawAngle::from_degrees(aim.angles.yaw),
            source::RollAngle::from_degrees(0.0),
        ),
        ViewPort::new(1920.0, 1080.0),
        FieldOfView::from_degrees(90.0),
        0.01,
        10000.0,
    );

    let crosshair = camera.world_to_screen(aim.aim_point).unwrap();
    expect_near(crosshair.x, 960.0_f32, 0.5_f32);
    expect_near(crosshair.y, 540.0_f32, 0.5_f32);

    // The target itself is not under the crosshair: the round leaves above the view, so the target shows up above the
    // centre of the screen (smaller y with a top-left origin).
    let target_on_screen = camera
        .world_to_screen(aim.predicted_target_position)
        .unwrap();
    assert!(target_on_screen.y < 540.0 - 20.0);

    // look_at() towards the aim point is the other way to get the same view
    let looked_at = CameraTrait::calc_look_at_angle(launcher.eye_origin, aim.aim_point);
    expect_near(looked_at.pitch.as_degrees(), -aim.angles.pitch, 0.01_f32);
    expect_near(looked_at.yaw.as_degrees(), aim.angles.yaw, 0.01_f32);
}
