//! Invariants every engine's PredEngineTrait has to satisfy, and ProjPredEngineLegacy solving through each of them.
//!
//! Scenarios are laid out along the engine's own axes (K_ABS_FORWARD / RIGHT / UP), so one scenario means the same thing
//! in a Z-up right-handed engine and in a Y-up left-handed one. Each test exists once per engine, in a module named
//! after it, the way the C++ runs them through a typed test suite.

mod common;

use common::expect_near;
use omath::engines::{cry, frostbite, iw, opengl, rage, serious, source, unity, unreal};
use omath::projectile_prediction::{
    AimSolution, Launcher, MuzzleOffset, PredEngine, ProjPredEngineInterface, ProjPredEngineLegacy,
    Projectile, Target,
};
use omath::projection::{CameraEngine, ViewAnglesOf};
use omath::scalar::Scalar;
use omath::trigonometry::angles;
use omath::{Real, Vector3};

trait Engine {
    type Scalar: Real;
    type Trait: PredEngine<Scalar = Self::Scalar>;
    type CameraTrait: CameraEngine<Scalar = Self::Scalar>;

    fn axis_forward() -> Vector3<Self::Scalar>;
    fn axis_right() -> Vector3<Self::Scalar>;
    fn axis_up() -> Vector3<Self::Scalar>;
    fn forward_vector(angles: ViewAnglesOf<Self::CameraTrait>) -> Vector3<Self::Scalar>;
    fn right_vector(angles: ViewAnglesOf<Self::CameraTrait>) -> Vector3<Self::Scalar>;
    fn up_vector(angles: ViewAnglesOf<Self::CameraTrait>) -> Vector3<Self::Scalar>;
}

const K_PITCHES: [f64; 5] = [-60.0, -20.0, 0.0, 35.0, 70.0];
const K_YAWS: [f64; 5] = [-150.0, -45.0, 0.0, 30.0, 120.0];

fn s<T: Real>(value: f64) -> T {
    T::from_f64(value)
}

/// A point given as "this far ahead, this far to the right, this high" in the engine's own axes.
fn world<E: Engine>(forward: f64, right: f64, up: f64) -> Vector3<E::Scalar> {
    E::axis_forward() * s::<E::Scalar>(forward)
        + E::axis_right() * s::<E::Scalar>(right)
        + E::axis_up() * s::<E::Scalar>(up)
}

/// `std::remainder(a - b, 360)`.
fn angle_difference<T: Real>(a: T, b: T) -> f64 {
    let difference = (a - b).to_f64();
    difference - 360.0 * (difference / 360.0).round_ties_even()
}

fn near<T: Real>(actual: T, expected: f64, tolerance: f64) {
    expect_near(actual.to_f64(), expected, tolerance);
}

fn expect_vectors_near<T: Real>(actual: Vector3<T>, expected: Vector3<T>, tolerance: f64) {
    near(actual.x, expected.x.to_f64(), tolerance);
    near(actual.y, expected.y.to_f64(), tolerance);
    near(actual.z, expected.z.to_f64(), tolerance);
}

// Trait invariants

fn zero_angles_basis_is_the_world_axes<E: Engine>() {
    let basis = E::Trait::calc_view_basis(s(0.0), s(0.0));

    expect_vectors_near(basis.forward, E::axis_forward(), 1e-5);
    expect_vectors_near(basis.right, E::axis_right(), 1e-5);
    expect_vectors_near(basis.up, E::axis_up(), 1e-5);
}

fn basis_is_orthonormal_and_keeps_handedness<E: Engine>() {
    let handedness = E::axis_forward()
        .cross(E::axis_right())
        .dot(E::axis_up())
        .to_f64();

    for pitch in K_PITCHES {
        for yaw in K_YAWS {
            let basis = E::Trait::calc_view_basis(s(pitch), s(yaw));

            near(basis.forward.length(), 1.0, 1e-5);
            near(basis.right.length(), 1.0, 1e-5);
            near(basis.up.length(), 1.0, 1e-5);
            near(basis.forward.dot(basis.right), 0.0, 1e-5);
            near(basis.forward.dot(basis.up), 0.0, 1e-5);
            near(basis.right.dot(basis.up), 0.0, 1e-5);
            near(
                basis.forward.cross(basis.right).dot(basis.up),
                handedness,
                1e-5,
            );
        }
    }
}

/// What the solver leans on: it measures angles with calc_direct_*_angle and turns them back into directions with
/// calc_view_basis, so the two must be inverses. A flipped pitch sign in either fails here.
fn basis_round_trips_through_direct_angles<E: Engine>() {
    for pitch in K_PITCHES {
        for yaw in K_YAWS {
            let forward = E::Trait::calc_view_basis(s(pitch), s(yaw)).forward;

            near(
                E::Trait::calc_direct_pitch_angle(Vector3::default(), forward),
                pitch,
                1e-3,
            );
            near(
                angle_difference(
                    E::Trait::calc_direct_yaw_angle(Vector3::default(), forward),
                    s(yaw),
                ),
                0.0,
                1e-3,
            );
        }
    }
}

/// The muzzle offset is laid out along basis.right and basis.up, so they have to be the engine's own right and up for
/// the view that looks along basis.forward.
fn basis_matches_the_engines_direction_vectors<E: Engine>() {
    for pitch in K_PITCHES {
        for yaw in K_YAWS {
            let basis = E::Trait::calc_view_basis(s(pitch), s(yaw));
            let angles = E::CameraTrait::calc_look_at_angle(Vector3::default(), basis.forward);

            expect_vectors_near(E::forward_vector(angles), basis.forward, 1e-4);
            expect_vectors_near(E::right_vector(angles), basis.right, 1e-4);
            expect_vectors_near(E::up_vector(angles), basis.up, 1e-4);
        }
    }
}

/// forward * a + up * b in the view frame is the view pitched up by atan(b / a): what Launcher::launch_pitch_offset
/// stands on, and it only holds if basis.up really is the direction a positive pitch turns towards.
fn raising_the_pitch_turns_forward_towards_up<E: Engine>() {
    let (forward_speed, up_speed) = (s::<E::Scalar>(1200.0), s::<E::Scalar>(200.0));
    let offset = angles::radians_to_degrees(up_speed.atan2(forward_speed));

    for yaw in K_YAWS {
        let basis = E::Trait::calc_view_basis(s(20.0), s(yaw));
        let game_velocity = basis.forward * forward_speed + basis.up * up_speed;
        let raised = E::Trait::calc_view_basis(s::<E::Scalar>(20.0) + offset, s(yaw)).forward
            * forward_speed.hypot(up_speed);

        near(game_velocity.distance_to(raised), 0.0, 2e-2);
    }
}

fn positive_pitch_launches_upwards<E: Engine>() {
    let round = Projectile {
        launch_speed: s(100.0),
        gravity_scale: s(1.0),
        ..Default::default()
    };
    let origin = world::<E>(5.0, -3.0, 40.0);

    let up_shot =
        E::Trait::predict_projectile_position(origin, &round, s(30.0), s(0.0), s(1.0), s(0.0))
            - origin;
    near(up_shot.dot(E::axis_up()), 50.0, 1e-3);
    near(
        up_shot.dot(E::axis_forward()),
        100.0 * angles::degrees_to_radians(30.0_f64).cos(),
        1e-3,
    );
    near(up_shot.dot(E::axis_right()), 0.0, 1e-3);

    let down_shot =
        E::Trait::predict_projectile_position(origin, &round, s(-30.0), s(0.0), s(1.0), s(0.0))
            - origin;
    near(down_shot.dot(E::axis_up()), -50.0, 1e-3);
}

fn projectile_starts_at_the_launch_origin_and_flies_straight_without_gravity<E: Engine>() {
    let round = Projectile {
        launch_speed: s(250.0),
        gravity_scale: s(0.0),
        ..Default::default()
    };
    let origin = world::<E>(10.0, 20.0, 30.0);

    expect_vectors_near(
        E::Trait::predict_projectile_position(origin, &round, s(15.0), s(40.0), s(0.0), s(800.0)),
        origin,
        1e-5,
    );

    // No gravity scale: gravity passed in must not matter, and the path is a straight line along the basis
    let forward = E::Trait::calc_view_basis(s(15.0), s(40.0)).forward;
    expect_vectors_near(
        E::Trait::predict_projectile_position(origin, &round, s(15.0), s(40.0), s(2.0), s(800.0)),
        origin + forward * s::<E::Scalar>(500.0),
        1e-2,
    );
}

fn gravity_pulls_along_the_engines_down_axis<E: Engine>() {
    let round = Projectile {
        launch_speed: s(300.0),
        gravity_scale: s(0.5),
        ..Default::default()
    };
    let origin = world::<E>(0.0, 0.0, 100.0);

    let weightless =
        E::Trait::predict_projectile_position(origin, &round, s(10.0), s(25.0), s(2.0), s(0.0));
    let falling =
        E::Trait::predict_projectile_position(origin, &round, s(10.0), s(25.0), s(2.0), s(800.0));

    // 0.5 * (800 * 0.5) * 2^2 = 800 straight down, nothing sideways
    expect_vectors_near(
        weightless - falling,
        E::axis_up() * s::<E::Scalar>(800.0),
        1e-2,
    );
}

fn target_prediction_falls_only_when_airborne<E: Engine>() {
    let mut target = Target {
        origin: world::<E>(100.0, 50.0, 300.0),
        velocity: world::<E>(-20.0, 10.0, 5.0),
        is_airborne: false,
    };

    let grounded = E::Trait::predict_target_position(&target, s(2.0), s(800.0));
    expect_vectors_near(
        grounded,
        target.origin + target.velocity * s::<E::Scalar>(2.0),
        1e-3,
    );

    target.is_airborne = true;
    let airborne = E::Trait::predict_target_position(&target, s(2.0), s(800.0));
    expect_vectors_near(
        grounded - airborne,
        E::axis_up() * s::<E::Scalar>(1600.0),
        1e-2,
    );
}

fn splits_a_vector_into_ground_distance_and_height<E: Engine>() {
    let delta = world::<E>(3.0, 4.0, 12.0);
    near(E::Trait::calc_vector_2d_distance(delta), 5.0, 1e-5);
    near(E::Trait::get_vector_height_coordinate(delta), 12.0, 1e-5);

    let below = world::<E>(-6.0, 8.0, -7.0);
    near(E::Trait::calc_vector_2d_distance(below), 10.0, 1e-5);
    near(E::Trait::get_vector_height_coordinate(below), -7.0, 1e-5);
}

fn direct_angles_of_the_axes<E: Engine>() {
    let origin = world::<E>(7.0, -2.0, 9.0);
    let ahead = origin + E::axis_forward() * s::<E::Scalar>(50.0);

    near(E::Trait::calc_direct_pitch_angle(origin, ahead), 0.0, 1e-3);
    near(E::Trait::calc_direct_yaw_angle(origin, ahead), 0.0, 1e-3);
    near(
        E::Trait::calc_direct_pitch_angle(origin, origin + world::<E>(10.0, 0.0, 10.0)),
        45.0,
        1e-3,
    );
    near(
        E::Trait::calc_direct_pitch_angle(origin, origin + world::<E>(10.0, 0.0, -10.0)),
        -45.0,
        1e-3,
    );

    // Turning to the right and to the left are opposite yaws of the same size, whichever sign the engine gives them
    let to_the_right =
        E::Trait::calc_direct_yaw_angle(origin, origin + world::<E>(10.0, 10.0, 0.0));
    let to_the_left =
        E::Trait::calc_direct_yaw_angle(origin, origin + world::<E>(10.0, -10.0, 0.0));
    near(to_the_right.to_f64().abs(), 45.0, 1e-3);
    near(to_the_right + to_the_left, 0.0, 1e-3);
}

// The legacy engine through every trait

struct Scenario<T> {
    projectile: Projectile<T>,
    launcher: Launcher<T>,
    target: Target<T>,
    gravity: T,
    hit_tolerance: T,
}

impl<T: Real> Scenario<T> {
    fn new(projectile: Projectile<T>, launcher: Launcher<T>, target: Target<T>) -> Self {
        Self {
            projectile,
            launcher,
            target,
            gravity: s(800.0),
            hit_tolerance: s(10.0),
        }
    }
}

fn round<T: Real>(launch_speed: f64, gravity_scale: f64) -> Projectile<T> {
    Projectile {
        launch_speed: s(launch_speed),
        gravity_scale: s(gravity_scale),
        ..Default::default()
    }
}

fn eye_at<E: Engine>(forward: f64, right: f64, up: f64) -> Launcher<E::Scalar> {
    Launcher {
        eye_origin: world::<E>(forward, right, up),
        ..Default::default()
    }
}

fn muzzle<T: Real>(forward: f64, right: f64, up: f64) -> MuzzleOffset<T> {
    MuzzleOffset {
        forward: s(forward),
        right: s(right),
        up: s(up),
    }
}

/// Solves, then fires the trait's own projectile from where the solution puts the muzzle and checks it arrives. Returns
/// the solution so a test can assert more about it.
fn solve_and_check<E: Engine>(scenario: &Scenario<E::Scalar>) -> AimSolution<E::Scalar> {
    let engine = ProjPredEngineLegacy::<E::Trait>::new(
        scenario.gravity,
        E::Scalar::ONE / s::<E::Scalar>(1000.0),
        s(10.0),
        s(5.0),
    );
    let aim = engine
        .maybe_calculate_aim(&scenario.projectile, &scenario.launcher, &scenario.target)
        .expect("engine found no solution");

    let basis = E::Trait::calc_view_basis(aim.angles.pitch, aim.angles.yaw);
    let launch_origin = scenario.launcher.launch_origin(basis);
    let projectile_position = E::Trait::predict_projectile_position(
        launch_origin,
        &scenario.projectile,
        aim.angles.pitch + scenario.launcher.launch_pitch_offset,
        aim.angles.yaw,
        aim.time_of_flight,
        scenario.gravity,
    );
    let target_position =
        E::Trait::predict_target_position(&scenario.target, aim.time_of_flight, scenario.gravity);

    assert!(
        projectile_position.distance_to(target_position) <= scenario.hit_tolerance,
        "fired from the solved muzzle with the solved angles the round has to arrive"
    );
    expect_vectors_near(aim.predicted_target_position, target_position, 1e-3);

    // The aim point sits on the eye's view ray, as far out as the target
    let eye = scenario.launcher.eye_origin;
    near(
        E::Trait::calc_direct_pitch_angle(eye, aim.aim_point),
        aim.angles.pitch.to_f64(),
        0.01,
    );
    near(
        angle_difference(
            E::Trait::calc_direct_yaw_angle(eye, aim.aim_point),
            aim.angles.yaw,
        ),
        0.0,
        0.01,
    );
    near(
        eye.distance_to(aim.aim_point),
        eye.distance_to(target_position).to_f64(),
        0.05,
    );

    aim
}

fn engine_hits_an_elevated_target_with_an_upward_pitch<E: Engine>() {
    let scenario = Scenario::new(
        round(1500.0, 1.0),
        eye_at::<E>(0.0, 0.0, 64.0),
        Target {
            origin: world::<E>(800.0, 200.0, 250.0),
            velocity: Vector3::default(),
            is_airborne: false,
        },
    );

    let aim = solve_and_check::<E>(&scenario);
    assert!(aim.angles.pitch > E::Scalar::ZERO);
    assert!(aim.time_of_flight > E::Scalar::ZERO);
}

fn engine_hits_a_target_below_with_a_downward_pitch<E: Engine>() {
    let scenario = Scenario::new(
        round(2000.0, 0.5),
        eye_at::<E>(0.0, 0.0, 600.0),
        Target {
            origin: world::<E>(700.0, -250.0, 0.0),
            velocity: Vector3::default(),
            is_airborne: false,
        },
    );

    let aim = solve_and_check::<E>(&scenario);
    assert!(aim.angles.pitch < E::Scalar::ZERO);
}

fn engine_leads_a_moving_target_from_an_offset_muzzle<E: Engine>() {
    let scenario = Scenario::new(
        round(1100.0, 0.4),
        Launcher {
            muzzle_offset: muzzle(16.0, 8.0, -6.0),
            ..eye_at::<E>(0.0, 0.0, 64.0)
        },
        Target {
            origin: world::<E>(600.0, -150.0, 0.0),
            velocity: world::<E>(-40.0, 90.0, 0.0),
            is_airborne: false,
        },
    );

    let aim = solve_and_check::<E>(&scenario);

    // It aims where the target will be, not where it is
    assert!(
        aim.predicted_target_position
            .distance_to(scenario.target.origin)
            > s(30.0)
    );
}

fn engine_hits_a_falling_target<E: Engine>() {
    let scenario = Scenario::new(
        round(1800.0, 0.5),
        eye_at::<E>(0.0, 0.0, 64.0),
        Target {
            origin: world::<E>(400.0, 120.0, 500.0),
            velocity: world::<E>(15.0, -10.0, -30.0),
            is_airborne: true,
        },
    );

    let aim = solve_and_check::<E>(&scenario);
    assert!(
        aim.predicted_target_position.dot(E::axis_up()) < scenario.target.origin.dot(E::axis_up())
    );
}

/// A rocket has no arc to hide behind: from the rotated muzzle the solved direction has to run through the target.
fn rocket_from_an_offset_muzzle_flies_through_the_target<E: Engine>() {
    let scenario = Scenario::new(
        round(1100.0, 0.0),
        Launcher {
            muzzle_offset: muzzle(23.5, 12.0, -3.0),
            ..eye_at::<E>(0.0, 0.0, 64.0)
        },
        Target {
            origin: world::<E>(900.0, 300.0, 150.0),
            velocity: Vector3::default(),
            is_airborne: false,
        },
    );

    let aim = solve_and_check::<E>(&scenario);

    let basis = E::Trait::calc_view_basis(aim.angles.pitch, aim.angles.yaw);
    let to_target = scenario.target.origin - scenario.launcher.launch_origin(basis);
    let off_the_ray = to_target - basis.forward * to_target.dot(basis.forward);
    // One refinement pass leaves a second-order error; placing the muzzle only once would be off by a third of a unit
    // here
    assert!(off_the_ray.length() <= s(0.05));

    // and the eye does not look straight at the target: the muzzle is off to the side
    let direct_yaw =
        E::Trait::calc_direct_yaw_angle(scenario.launcher.eye_origin, scenario.target.origin);
    assert!(angle_difference(aim.angles.yaw, direct_yaw).abs() > 0.3);
}

/// The muzzle turns with the view while the round leaves along the view raised by the offset. Placing the muzzle from
/// the launch pitch instead would put it several units away from where the game spawns the round.
fn tilted_rocket_from_an_offset_muzzle_flies_through_the_target<E: Engine>() {
    let scenario = Scenario::new(
        round(1100.0, 0.0),
        Launcher {
            muzzle_offset: muzzle(23.5, 12.0, -3.0),
            launch_pitch_offset: s(10.0),
            ..eye_at::<E>(0.0, 0.0, 64.0)
        },
        Target {
            origin: world::<E>(900.0, 300.0, 150.0),
            velocity: Vector3::default(),
            is_airborne: false,
        },
    );

    let aim = solve_and_check::<E>(&scenario);

    let view = E::Trait::calc_view_basis(aim.angles.pitch, aim.angles.yaw);
    let launch_direction = E::Trait::calc_view_basis(
        aim.angles.pitch + scenario.launcher.launch_pitch_offset,
        aim.angles.yaw,
    )
    .forward;
    let to_target = scenario.target.origin - scenario.launcher.launch_origin(view);
    let off_the_ray = to_target - launch_direction * to_target.dot(launch_direction);
    assert!(off_the_ray.length() <= s(0.05));

    // The eye looks well below the line the round takes
    near(
        aim.angles.pitch + s(10.0),
        E::Trait::calc_direct_pitch_angle(
            scenario.launcher.launch_origin(view),
            scenario.target.origin,
        )
        .to_f64(),
        1e-3,
    );
}

/// TF2-style pipe: velocity = forward * 1200 + up * 200 in the view frame, given to the engine as a pitch offset.
fn pipe_style_launch_fired_the_way_the_game_does_hits<E: Engine>() {
    let forward_speed = s::<E::Scalar>(1200.0);
    let up_speed = s::<E::Scalar>(200.0);

    let scenario = Scenario::new(
        Projectile {
            launch_speed: forward_speed.hypot(up_speed),
            gravity_scale: s(0.5),
            ..Default::default()
        },
        Launcher {
            muzzle_offset: muzzle(16.0, 8.0, -6.0),
            launch_pitch_offset: angles::radians_to_degrees(up_speed.atan2(forward_speed)),
            ..eye_at::<E>(0.0, 0.0, 64.0)
        },
        Target {
            origin: world::<E>(600.0, 150.0, 0.0),
            velocity: world::<E>(-40.0, 30.0, 0.0),
            is_airborne: false,
        },
    );

    let aim = solve_and_check::<E>(&scenario);

    let basis = E::Trait::calc_view_basis(aim.angles.pitch, aim.angles.yaw);
    let time = aim.time_of_flight;
    let drop =
        s::<E::Scalar>(0.5) * scenario.gravity * scenario.projectile.gravity_scale * time * time;
    let position = scenario.launcher.launch_origin(basis)
        + (basis.forward * forward_speed + basis.up * up_speed) * time
        - E::axis_up() * drop;

    assert!(
        position.distance_to(E::Trait::predict_target_position(
            &scenario.target,
            time,
            scenario.gravity
        )) <= s(6.0)
    );
}

fn engine_reports_no_solution_for_an_unreachable_target<E: Engine>() {
    let engine = ProjPredEngineLegacy::<E::Trait>::new(
        s(800.0),
        E::Scalar::ONE / s::<E::Scalar>(100.0),
        s(5.0),
        s(5.0),
    );

    let slow = round(10.0, 1.0);
    let launcher = eye_at::<E>(0.0, 0.0, 64.0);
    let far_away = Target {
        origin: world::<E>(50000.0, 0.0, 0.0),
        velocity: Vector3::default(),
        is_airborne: false,
    };

    assert!(
        engine
            .maybe_calculate_aim(&slow, &launcher, &far_away)
            .is_none()
    );
}

macro_rules! test_engine {
    ($module:ident, $engine:ident, $scalar:ty) => {
        mod $module {
            use super::*;

            struct TestEngine;

            impl Engine for TestEngine {
                type Scalar = $scalar;
                type Trait = $engine::PredEngineTrait;
                type CameraTrait = $engine::CameraTrait;

                fn axis_forward() -> Vector3<$scalar> {
                    $engine::K_ABS_FORWARD
                }
                fn axis_right() -> Vector3<$scalar> {
                    $engine::K_ABS_RIGHT
                }
                fn axis_up() -> Vector3<$scalar> {
                    $engine::K_ABS_UP
                }
                fn forward_vector(angles: ViewAnglesOf<Self::CameraTrait>) -> Vector3<$scalar> {
                    $engine::forward_vector(angles)
                }
                fn right_vector(angles: ViewAnglesOf<Self::CameraTrait>) -> Vector3<$scalar> {
                    $engine::right_vector(angles)
                }
                fn up_vector(angles: ViewAnglesOf<Self::CameraTrait>) -> Vector3<$scalar> {
                    $engine::up_vector(angles)
                }
            }

            test_engine!(@tests
                zero_angles_basis_is_the_world_axes,
                basis_is_orthonormal_and_keeps_handedness,
                basis_round_trips_through_direct_angles,
                basis_matches_the_engines_direction_vectors,
                raising_the_pitch_turns_forward_towards_up,
                positive_pitch_launches_upwards,
                projectile_starts_at_the_launch_origin_and_flies_straight_without_gravity,
                gravity_pulls_along_the_engines_down_axis,
                target_prediction_falls_only_when_airborne,
                splits_a_vector_into_ground_distance_and_height,
                direct_angles_of_the_axes,
                engine_hits_an_elevated_target_with_an_upward_pitch,
                engine_hits_a_target_below_with_a_downward_pitch,
                engine_leads_a_moving_target_from_an_offset_muzzle,
                engine_hits_a_falling_target,
                rocket_from_an_offset_muzzle_flies_through_the_target,
                tilted_rocket_from_an_offset_muzzle_flies_through_the_target,
                pipe_style_launch_fired_the_way_the_game_does_hits,
                engine_reports_no_solution_for_an_unreachable_target,
            );
        }
    };
    (@tests $($test:ident),* $(,)?) => {
        $(
            #[test]
            fn $test() {
                super::$test::<TestEngine>();
            }
        )*
    };
}

test_engine!(source_engine, source, f32);
test_engine!(iw_engine, iw, f32);
test_engine!(rage_engine, rage, f32);
test_engine!(cry_engine, cry, f32);
test_engine!(unity_engine, unity, f32);
test_engine!(frostbite_engine, frostbite, f32);
test_engine!(opengl_engine, opengl, f32);
test_engine!(serious_engine, serious, f32);
test_engine!(unreal_engine, unreal, f64);
