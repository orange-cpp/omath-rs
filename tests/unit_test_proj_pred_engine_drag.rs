//! ProjectileFlight and ProjPredEngineDrag: rounds that lose speed to the air.
//!
//! The reference positions below were not computed, they were measured: Team Fortress 2's own vphysics.dll was run
//! offline on the game's real grenade hull, launched with the calls the game makes, and sampled every tick. A pipe there
//! leaves at forward * 1200 + up * 200 (1216.55 units/s, 9.46 degrees above the view) and the Loch-n-Load at
//! forward * 1500 + up * 200, both under gravity 800 at a 0.015 s tick.

mod common;

use common::{expect_float_eq, expect_near};
use omath::engines::source::PredEngineTrait;
use omath::engines::{unity, unreal};
use omath::projectile_prediction::{
    AimAngles, Arc, Launcher, MuzzleOffset, PredEngine, ProjPredEngineDrag,
    ProjPredEngineInterface, ProjPredEngineLegacy, Projectile, ProjectileFlight, Target,
};
use omath::trigonometry::angles;
use omath::{Real, Vector3};

type Flight = ProjectileFlight<PredEngineTrait>;
type Engine = ProjPredEngineDrag;
type Legacy = ProjPredEngineLegacy;

const K_GRAVITY: f32 = 800.0;
const K_TICK: f32 = 0.015;

fn v(x: f32, y: f32, z: f32) -> Vector3<f32> {
    Vector3::new(x, y, z)
}

/// A pipe tumbles, so one number covers its drag.
fn k_pipe() -> Projectile<f32> {
    Projectile {
        launch_speed: 1216.5525,
        gravity_scale: 1.0,
        drag: 2.838e-4,
        max_speed: 2000.0,
        ..Default::default()
    }
}

/// The Loch-n-Load's round is fired without spin and keeps the attitude it left with, so its drag is the hull's own:
/// end-on and side-on.
fn k_loch() -> Projectile<f32> {
    Projectile {
        launch_speed: 1513.2745,
        gravity_scale: 1.0,
        drag_forward: 9.911e-5,
        drag_up: 2.530e-4,
        max_speed: 2000.0,
        ..Default::default()
    }
}

const K_PIPE_PITCH_OFFSET: f32 = 9.46232;
const K_LOCH_PITCH_OFFSET: f32 = 7.59464;

/// Muzzle at the eye, so a flight reads straight off against positions measured from the muzzle.
fn k_pipe_at_eye() -> Launcher<f32> {
    Launcher {
        launch_pitch_offset: K_PIPE_PITCH_OFFSET,
        ..Default::default()
    }
}

fn k_loch_at_eye() -> Launcher<f32> {
    Launcher {
        launch_pitch_offset: K_LOCH_PITCH_OFFSET,
        ..Default::default()
    }
}

/// The way the game really holds the weapon.
fn k_pipe_launcher() -> Launcher<f32> {
    Launcher {
        eye_origin: v(0.0, 0.0, 64.0),
        muzzle_offset: MuzzleOffset {
            forward: 16.0,
            right: 8.0,
            up: -6.0,
        },
        launch_pitch_offset: K_PIPE_PITCH_OFFSET,
        ..Default::default()
    }
}

fn angles_at<T>(pitch: T, yaw: T) -> AimAngles<T> {
    AimAngles { pitch, yaw }
}

fn stationary(x: f32, y: f32, z: f32) -> Target<f32> {
    Target {
        origin: v(x, y, z),
        velocity: Vector3::default(),
        is_airborne: false,
    }
}

fn moving(origin: Vector3<f32>, velocity: Vector3<f32>, is_airborne: bool) -> Target<f32> {
    Target {
        origin,
        velocity,
        is_airborne,
    }
}

fn k_engine() -> Engine {
    Engine::new(K_GRAVITY, K_TICK, 2.13, 5.0)
}

fn fly<E: PredEngine>(
    round: &Projectile<E::Scalar>,
    launcher: &Launcher<E::Scalar>,
    angles: AimAngles<E::Scalar>,
    gravity: E::Scalar,
    step: E::Scalar,
    steps: i32,
) -> Vector3<E::Scalar> {
    let mut flight = ProjectileFlight::<E>::new(round, launcher, angles, gravity, step);
    for _ in 0..steps {
        flight.step();
    }
    flight.position()
}

/// Anything computed twice is compared within a tolerance; plain == is kept for values that are copies of one another
/// or exact in any precision.
fn expect_vectors_near<T: Real>(actual: Vector3<T>, expected: Vector3<T>, tolerance: f64) {
    expect_near(actual.x.to_f64(), expected.x.to_f64(), tolerance);
    expect_near(actual.y.to_f64(), expected.y.to_f64(), tolerance);
    expect_near(actual.z.to_f64(), expected.z.to_f64(), tolerance);
}

// Projectile

#[test]
fn has_drag_only_when_a_coefficient_is_set() {
    assert!(
        !Projectile::<f32> {
            launch_speed: 1000.0,
            gravity_scale: 1.0,
            ..Default::default()
        }
        .has_drag()
    );
    assert!(
        !Projectile::<f32> {
            max_speed: 2000.0,
            ..Default::default()
        }
        .has_drag()
    );
    assert!(
        Projectile::<f32> {
            drag: 1e-4,
            ..Default::default()
        }
        .has_drag()
    );
    assert!(
        Projectile::<f32> {
            drag_forward: 1e-4,
            ..Default::default()
        }
        .has_drag()
    );
    assert!(
        Projectile::<f64> {
            drag_up: 1e-4,
            ..Default::default()
        }
        .has_drag()
    );
    assert!(k_pipe().has_drag() && k_loch().has_drag());
}

// ProjectileFlight

#[test]
fn starts_at_the_muzzle() {
    let angles = angles_at(20.0, 35.0);
    let flight = Flight::new(&k_pipe(), &k_pipe_launcher(), angles, K_GRAVITY, K_TICK);

    let muzzle =
        k_pipe_launcher().launch_origin(PredEngineTrait::calc_view_basis(angles.pitch, angles.yaw));

    expect_vectors_near(flight.origin(), muzzle, 1e-3);
    assert_eq!(flight.position(), flight.origin());
    expect_float_eq(flight.time(), 0.0);
}

#[test]
fn without_gravity_or_drag_it_flies_straight() {
    let round = Projectile {
        launch_speed: 1000.0,
        ..Default::default()
    };
    let mut flight = Flight::new(
        &round,
        &Launcher::default(),
        angles_at(0.0, 0.0),
        K_GRAVITY,
        K_TICK,
    );

    for _ in 0..100 {
        flight.step();
    }

    expect_near(flight.position().x, 1500.0_f32, 1e-2_f32);
    expect_near(flight.position().y, 0.0_f32, 1e-3_f32);
    expect_near(flight.position().z, 0.0_f32, 1e-3_f32);
    expect_near(flight.velocity().length(), 1000.0_f32, 1e-3_f32);
    expect_near(flight.time(), 1.5_f32, 1e-6_f32);
}

#[test]
fn gravity_lands_on_the_velocity_before_the_move() {
    // The engine's step is semi-implicit: the velocity it moves with already carries this step's gravity. Over n steps
    // that falls g * dt^2 * n * (n + 1) / 2, a little further than the g * t^2 / 2 of a true parabola.
    let round = Projectile {
        launch_speed: 1000.0,
        gravity_scale: 0.5,
        ..Default::default()
    };
    const STEPS: i32 = 50;

    let position = fly::<PredEngineTrait>(
        &round,
        &Launcher::default(),
        angles_at(0.0, 0.0),
        K_GRAVITY,
        K_TICK,
        STEPS,
    );

    let scaled_gravity = K_GRAVITY * 0.5;
    let fallen = scaled_gravity * K_TICK * K_TICK * STEPS as f32 * (STEPS + 1) as f32 / 2.0;

    expect_near(position.z, -fallen, 1e-2_f32);
    expect_near(position.x, 1000.0 * K_TICK * STEPS as f32, 1e-2_f32);
}

#[test]
fn pitch_offset_raises_the_round_and_leaves_the_muzzle_alone() {
    let round = Projectile {
        launch_speed: 1216.5525,
        ..Default::default()
    };
    let angles = angles_at(10.0, 0.0);

    let flight = Flight::new(&round, &k_pipe_launcher(), angles, K_GRAVITY, K_TICK);
    let velocity = flight.velocity();

    // forward * 1200 + up * 200 in the view frame is one direction, 9.46 degrees above the view
    expect_near(velocity.length(), 1216.5525_f32, 1e-2_f32);
    expect_near(
        angles::radians_to_degrees(velocity.z.atan2(velocity.x)),
        angles.pitch + K_PIPE_PITCH_OFFSET,
        1e-3_f32,
    );

    let plain = Launcher {
        launch_pitch_offset: 0.0,
        ..k_pipe_launcher()
    };
    expect_vectors_near(
        flight.origin(),
        Flight::new(&round, &plain, angles, K_GRAVITY, K_TICK).origin(),
        1e-3,
    );
}

#[test]
fn tumbling_drag_takes_its_share_of_the_speed_every_step() {
    let round = Projectile {
        launch_speed: 1000.0,
        drag: 2e-4,
        ..Default::default()
    };
    let mut flight = Flight::new(
        &round,
        &Launcher::default(),
        angles_at(0.0, 0.0),
        K_GRAVITY,
        K_TICK,
    );

    flight.step();
    let after_one = 1000.0 * (1.0 - 2e-4 * 1000.0 * K_TICK);
    expect_near(flight.velocity().length(), after_one, 1e-3_f32);

    // Proportional to the speed it has left, so every step takes a smaller bite than the one before
    flight.step();
    expect_near(
        flight.velocity().length(),
        after_one * (1.0 - 2e-4 * after_one * K_TICK),
        1e-3_f32,
    );
}

#[test]
fn axis_drag_follows_the_rounds_attitude() {
    // Flying along its own nose the round only feels end-on drag...
    let round = Projectile {
        launch_speed: 1000.0,
        drag_forward: 1e-4,
        drag_up: 4e-4,
        ..Default::default()
    };
    let mut nose_first = Flight::new(
        &round,
        &Launcher::default(),
        angles_at(30.0, 0.0),
        K_GRAVITY,
        K_TICK,
    );
    nose_first.step();
    expect_near(
        nose_first.velocity().length(),
        1000.0 * (1.0 - 1e-4 * 1000.0 * K_TICK),
        1e-3_f32,
    );

    // ...and leaving 30 degrees above it, it feels cos(30) of that and sin(30) of the side-on drag
    let tilted = Launcher {
        launch_pitch_offset: 30.0,
        ..Default::default()
    };
    let mut raised = Flight::new(&round, &tilted, angles_at(0.0, 0.0), K_GRAVITY, K_TICK);
    raised.step();

    let rate = 1e-4 * 1000.0 * angles::degrees_to_radians(30.0_f32).cos()
        + 4e-4 * 1000.0 * angles::degrees_to_radians(30.0_f32).sin();
    expect_near(
        raised.velocity().length(),
        1000.0 * (1.0 - rate * K_TICK),
        1e-3_f32,
    );
}

#[test]
fn drag_can_stop_the_round_but_never_turn_it_around() {
    let round = Projectile {
        launch_speed: 1000.0,
        drag: 10.0,
        ..Default::default()
    };
    let mut flight = Flight::new(
        &round,
        &Launcher::default(),
        angles_at(0.0, 0.0),
        K_GRAVITY,
        K_TICK,
    );

    flight.step();

    expect_float_eq(flight.velocity().length(), 0.0);
    assert_eq!(flight.position(), flight.origin());
}

#[test]
fn speed_is_capped_on_launch_and_again_after_every_step() {
    // A fully charged sticky asks for forward * 2400 + up * 200, and VPhysics lets it leave at 2000
    let charged = Projectile {
        launch_speed: 2408.319,
        gravity_scale: 1.0,
        max_speed: 2000.0,
        ..Default::default()
    };
    let launcher = Launcher {
        launch_pitch_offset: 4.76364,
        ..Default::default()
    };
    let mut flight = Flight::new(
        &charged,
        &launcher,
        angles_at(-80.0, 0.0),
        K_GRAVITY,
        K_TICK,
    );

    expect_near(flight.velocity().length(), 2000.0_f32, 1e-2_f32);

    // Fired almost straight down, gravity keeps trying to push it past the cap
    for _ in 0..20 {
        flight.step();
        assert!(flight.velocity().length() <= 2000.0 + 1e-2);
    }

    // Zero means there is no cap
    let uncapped = Projectile {
        launch_speed: 2408.319,
        ..Default::default()
    };
    expect_near(
        Flight::new(
            &uncapped,
            &Launcher::default(),
            angles_at(0.0, 0.0),
            K_GRAVITY,
            K_TICK,
        )
        .velocity()
        .length(),
        2408.319_f32,
        1e-2_f32,
    );
}

#[test]
fn follows_source_physics_for_a_round_that_holds_its_attitude() {
    // Nothing random about this one, so the engine's own numbers can be asked for almost exactly
    let measured = [
        (20.0, 40, 765.4679_f32, 257.2263_f32),
        (20.0, 100, 1757.2307, 96.5640),
        (-10.0, 40, 860.71, -178.6139),
        (-10.0, 100, 1951.4204, -891.4063),
        (45.0, 40, 525.6462, 544.0996),
        (45.0, 100, 1230.2048, 767.2341),
    ];

    for (view_pitch, steps, x, z) in measured {
        let position = fly::<PredEngineTrait>(
            &k_loch(),
            &k_loch_at_eye(),
            angles_at(view_pitch, 0.0),
            K_GRAVITY,
            K_TICK,
            steps,
        );

        expect_near(position.x, x, 0.1_f32);
        expect_near(position.z, z, 0.1_f32);
        expect_near(position.y, 0.0_f32, 1e-3_f32);
    }
}

#[test]
fn follows_source_physics_for_a_tumbling_round() {
    // The game spins every pipe differently, which moves these by about one percent. The numbers are the middle of that
    // spread, and a parabola would be 145 and 430 units past them by the last two.
    let measured = [
        (0.0, 40, 653.008_f32, -29.273_f32),
        (0.0, 100, 1436.293, -544.110),
        (30.0, 40, 514.455, 284.513),
        (30.0, 100, 1161.768, 147.345),
    ];

    for (view_pitch, steps, x, z) in measured {
        let position = fly::<PredEngineTrait>(
            &k_pipe(),
            &k_pipe_at_eye(),
            angles_at(view_pitch, 0.0),
            K_GRAVITY,
            K_TICK,
            steps,
        );

        expect_near(position.x, x, 2.0_f32);
        expect_near(position.z, z, 2.0_f32);
    }
}

#[test]
fn gravity_pulls_along_whichever_axis_the_engine_calls_up() {
    // Unity keeps height on y and looks along z
    let round = Projectile {
        launch_speed: 40.0,
        gravity_scale: 1.0,
        drag: 0.01,
        ..Default::default()
    };

    let position = fly::<unity::PredEngineTrait>(
        &round,
        &Launcher::default(),
        angles_at(0.0, 0.0),
        9.81,
        0.02,
        50,
    );

    assert!(position.y < -4.0);
    expect_near(position.x, 0.0_f32, 1e-3_f32);
    assert!(position.z > 25.0);
    assert!(position.z < 40.0); // drag kept it short of the 40 a vacuum would give
}

#[test]
fn works_in_double_precision() {
    // Unreal's trait is a double one: x ahead, z up, centimetres
    let round = Projectile::<f64> {
        launch_speed: 3000.0,
        gravity_scale: 1.0,
        drag: 1e-5,
        ..Default::default()
    };

    let position = fly::<unreal::PredEngineTrait>(
        &round,
        &Launcher::default(),
        angles_at(0.0, 0.0),
        980.0,
        1.0 / 60.0,
        60,
    );

    let _: f64 = position.x;
    assert!(position.x > 2500.0);
    assert!(position.x < 3000.0);
    assert!(position.z < -450.0);
}

// ProjPredEngineDrag

#[test]
fn lands_the_round_on_a_stationary_target() {
    let target = stationary(900.0, 150.0, 40.0);

    let solution = k_engine()
        .maybe_calculate_aim(&k_pipe(), &k_pipe_launcher(), &target)
        .unwrap();

    let arrival = k_engine().predict_projectile_position(
        &k_pipe(),
        &k_pipe_launcher(),
        solution.angles,
        solution.time_of_flight,
    );

    assert!(arrival.distance_to(target.origin) < 0.5);
    assert_eq!(solution.predicted_target_position, target.origin);
}

#[test]
fn needs_more_loft_and_more_time_than_a_parabola() {
    let target = stationary(1000.0, 0.0, 64.0);

    let vacuum_round = Projectile {
        drag: 0.0,
        ..k_pipe()
    };

    let with_drag = k_engine()
        .maybe_calculate_aim(&k_pipe(), &k_pipe_launcher(), &target)
        .unwrap();
    let in_vacuum = Legacy::new(K_GRAVITY, 0.001, 2.13, 5.0)
        .maybe_calculate_aim(&vacuum_round, &k_pipe_launcher(), &target)
        .unwrap();

    assert!(with_drag.angles.pitch > in_vacuum.angles.pitch + 3.0);
    assert!(with_drag.time_of_flight > in_vacuum.time_of_flight + 0.1);
}

#[test]
fn without_drag_it_agrees_with_the_closed_form() {
    let target = stationary(600.0, -200.0, 100.0);

    let vacuum_round = Projectile {
        drag: 0.0,
        ..k_pipe()
    };

    let stepped = k_engine()
        .maybe_calculate_aim(&vacuum_round, &k_pipe_launcher(), &target)
        .unwrap();
    let closed_form = Legacy::new(K_GRAVITY, 0.001, 2.13, 5.0)
        .maybe_calculate_aim(&vacuum_round, &k_pipe_launcher(), &target)
        .unwrap();

    // What is left between them is the extra the stepped fall drops: see gravity_lands_on_the_velocity_before_the_move
    expect_near(stepped.angles.pitch, closed_form.angles.pitch, 0.5_f32);
    expect_near(stepped.angles.yaw, closed_form.angles.yaw, 0.05_f32);
    expect_near(stepped.time_of_flight, closed_form.time_of_flight, 0.01_f32);
}

#[test]
fn leads_a_moving_target() {
    let target = moving(v(800.0, 0.0, 64.0), v(-120.0, 300.0, 0.0), false);

    let solution = k_engine()
        .maybe_calculate_aim(&k_pipe(), &k_pipe_launcher(), &target)
        .unwrap();

    // It aims at where the target will be when the round gets there, and the round does get there
    let expected =
        PredEngineTrait::predict_target_position(&target, solution.time_of_flight, K_GRAVITY);
    expect_vectors_near(solution.predicted_target_position, expected, 1e-3);
    assert!(solution.predicted_target_position.y > 150.0);

    let arrival = k_engine().predict_projectile_position(
        &k_pipe(),
        &k_pipe_launcher(),
        solution.angles,
        solution.time_of_flight,
    );
    assert!(arrival.distance_to(expected) < 0.5);
}

#[test]
fn leads_an_airborne_target_down_its_fall() {
    let target = moving(v(700.0, 100.0, 300.0), v(50.0, -80.0, 200.0), true);

    let solution = k_engine()
        .maybe_calculate_aim(&k_loch(), &k_pipe_launcher(), &target)
        .unwrap();

    let expected =
        PredEngineTrait::predict_target_position(&target, solution.time_of_flight, K_GRAVITY);
    let arrival = k_engine().predict_projectile_position(
        &k_loch(),
        &k_pipe_launcher(),
        solution.angles,
        solution.time_of_flight,
    );

    assert!(arrival.distance_to(expected) < 0.5);
}

#[test]
fn lands_a_target_flung_at_the_shooter_faster_than_the_round_flies() {
    // Blown towards the shooter at 1400 units/s, against a pipe that covers ground at about 1000. Feeding each arrival
    // time back in as the next guess overshoots by more every pass here, so the time has to be solved for rather than
    // iterated on.
    let target = moving(v(1068.0, 0.0, 250.0), v(-1422.0, -224.0, 354.0), true);

    let solution = k_engine()
        .maybe_calculate_aim(&k_pipe(), &k_pipe_launcher(), &target)
        .unwrap();

    let expected =
        PredEngineTrait::predict_target_position(&target, solution.time_of_flight, K_GRAVITY);
    let arrival = k_engine().predict_projectile_position(
        &k_pipe(),
        &k_pipe_launcher(),
        solution.angles,
        solution.time_of_flight,
    );

    assert!(arrival.distance_to(expected) < 0.5);
}

#[test]
fn catches_a_target_fleeing_nearly_as_fast_as_the_round() {
    // A rocket has neither drag nor drop, and flying one a step at a time is exact for it. The target is going away at
    // two thirds of its speed, which is a shot that can be made but takes a long time to close on.
    let rocket = Projectile {
        launch_speed: 1980.0,
        ..Default::default()
    };
    let launcher = Launcher {
        eye_origin: v(0.0, 0.0, 64.0),
        muzzle_offset: MuzzleOffset {
            forward: 23.5,
            right: 12.0,
            up: -3.0,
        },
        ..Default::default()
    };
    let target = moving(v(1280.0, 0.0, 250.0), v(1263.0, -283.0, 528.0), true);

    let engine = Engine::new(K_GRAVITY, K_TICK, 3.0, 5.0);

    let solution = engine
        .maybe_calculate_aim(&rocket, &launcher, &target)
        .unwrap();

    let expected =
        PredEngineTrait::predict_target_position(&target, solution.time_of_flight, K_GRAVITY);
    let arrival = engine.predict_projectile_position(
        &rocket,
        &launcher,
        solution.angles,
        solution.time_of_flight,
    );

    assert!(arrival.distance_to(expected) < 0.5);
    assert!(solution.time_of_flight > 1.5);
}

#[test]
fn reach_is_the_one_the_game_gives_the_pipe() {
    // Measured in the game's physics, a pipe fired at a level target gets 1352 units and no further. A parabola says
    // 1850, which is 500 units of shots that cannot be made.
    let launcher = Launcher {
        eye_origin: Vector3::default(),
        ..k_pipe_launcher()
    };

    let at = |distance: f32| {
        k_engine().maybe_calculate_aim(&k_pipe(), &launcher, &stationary(distance, 0.0, 0.0))
    };

    assert!(at(1340.0).is_some());
    assert!(at(1365.0).is_none());
    assert!(at(1800.0).is_none());
    assert!(at(5000.0).is_none());
}

#[test]
fn keeps_to_the_horizon() {
    let target = stationary(1200.0, 0.0, 64.0);

    let unhurried = k_engine()
        .maybe_calculate_aim(&k_pipe(), &k_pipe_launcher(), &target)
        .unwrap();
    assert!(unhurried.time_of_flight > 1.2);

    // The same shot, asked of an engine whose rounds are gone by then
    assert!(
        Engine::new(K_GRAVITY, K_TICK, 1.2, 5.0)
            .maybe_calculate_aim(&k_pipe(), &k_pipe_launcher(), &target)
            .is_none()
    );
}

#[test]
fn reports_view_angles_with_the_aim_point_on_their_ray() {
    let target = stationary(600.0, 300.0, 200.0);

    let solution = k_engine()
        .maybe_calculate_aim(&k_pipe(), &k_pipe_launcher(), &target)
        .unwrap();

    let eye = k_pipe_launcher().eye_origin;
    expect_near(
        PredEngineTrait::calc_direct_pitch_angle(eye, solution.aim_point),
        solution.angles.pitch,
        1e-2_f32,
    );
    expect_near(
        PredEngineTrait::calc_direct_yaw_angle(eye, solution.aim_point),
        solution.angles.yaw,
        1e-2_f32,
    );
    expect_near(
        eye.distance_to(solution.aim_point),
        eye.distance_to(target.origin),
        1e-2_f32,
    );

    // The offset comes off the view, not off the round: the same shot from a weapon that fires along its crosshair needs
    // the view that much higher. Not to the last hundredth, because the muzzle hangs off the view and so sits in a
    // slightly different place for the two.
    let along_the_crosshair = Launcher {
        launch_pitch_offset: 0.0,
        ..k_pipe_launcher()
    };

    let plain = k_engine()
        .maybe_calculate_aim(&k_pipe(), &along_the_crosshair, &target)
        .unwrap();
    expect_near(
        plain.angles.pitch,
        solution.angles.pitch + K_PIPE_PITCH_OFFSET,
        0.3_f32,
    );
    expect_near(plain.time_of_flight, solution.time_of_flight, 0.01_f32);
}

#[test]
fn refuses_a_view_the_engine_will_not_allow() {
    // Almost straight overhead: the pitch that would do it is past what Source lets a player look at
    let target = stationary(6.0, 0.0, 464.0);
    let launcher = Launcher {
        eye_origin: v(0.0, 0.0, 64.0),
        ..Default::default()
    };

    assert!(
        k_engine()
            .maybe_calculate_aim(&k_pipe(), &launcher, &target)
            .is_none()
    );
}

#[test]
fn settings_that_cannot_run_give_no_solution() {
    let target = stationary(500.0, 0.0, 64.0);

    let solve = |engine: Engine| engine.maybe_calculate_aim(&k_pipe(), &k_pipe_launcher(), &target);

    assert!(solve(Engine::new(K_GRAVITY, 0.0, 2.0, 5.0)).is_none());
    assert!(solve(Engine::new(K_GRAVITY, -K_TICK, 2.0, 5.0)).is_none());
    assert!(solve(Engine::new(K_GRAVITY, K_TICK, 0.0, 5.0)).is_none());

    let dud = Projectile {
        gravity_scale: 1.0,
        drag: 2.838e-4,
        ..Default::default()
    };
    assert!(
        k_engine()
            .maybe_calculate_aim(&dud, &k_pipe_launcher(), &target)
            .is_none()
    );
}

#[test]
fn is_an_ordinary_value_behind_the_interface() {
    fn is_copy<T: Copy>() {}

    is_copy::<Engine>();

    let target = moving(v(700.0, 120.0, 90.0), v(-60.0, 45.0, 0.0), false);

    let engine = k_engine();
    let through_base: &dyn ProjPredEngineInterface<f32> = &engine;
    let copy = engine;

    let direct = engine
        .maybe_calculate_aim(&k_pipe(), &k_pipe_launcher(), &target)
        .unwrap();
    let dispatched = through_base
        .maybe_calculate_aim(&k_pipe(), &k_pipe_launcher(), &target)
        .unwrap();
    let from_copy = copy
        .maybe_calculate_aim(&k_pipe(), &k_pipe_launcher(), &target)
        .unwrap();

    // The solver iterates to a tolerance, so two differently compiled runs of it may stop a refinement apart. A copy or a
    // dispatch that lost the engine's settings would be out by degrees, not by this.
    expect_near(direct.angles.pitch, dispatched.angles.pitch, 0.02_f32);
    expect_near(direct.angles.yaw, dispatched.angles.yaw, 0.02_f32);
    expect_near(direct.time_of_flight, from_copy.time_of_flight, 5e-3_f32);
    expect_vectors_near(direct.aim_point, from_copy.aim_point, 0.5);
}

#[test]
fn solves_in_an_engine_with_height_on_y() {
    type Trait = unity::PredEngineTrait;

    // Metres, Unity's 50 Hz physics, and a round that sheds about a third of its speed on the way
    let round = Projectile {
        launch_speed: 40.0,
        gravity_scale: 1.0,
        drag: 0.01,
        ..Default::default()
    };
    let launcher = Launcher {
        eye_origin: v(0.0, 1.7, 0.0),
        muzzle_offset: MuzzleOffset {
            forward: 0.4,
            right: 0.15,
            up: -0.1,
        },
        ..Default::default()
    };
    let target = moving(v(8.0, 1.0, 30.0), v(-3.0, 0.0, 1.0), false);

    let engine = ProjPredEngineDrag::<Trait>::new(9.81, 0.02, 5.0, 0.05);

    let solution = engine
        .maybe_calculate_aim(&round, &launcher, &target)
        .unwrap();

    let expected = Trait::predict_target_position(&target, solution.time_of_flight, 9.81);
    let arrival = engine.predict_projectile_position(
        &round,
        &launcher,
        solution.angles,
        solution.time_of_flight,
    );

    assert!(arrival.distance_to(expected) < 0.01);
    assert!(solution.angles.pitch > Trait::calc_direct_pitch_angle(launcher.eye_origin, expected));
}

// The high arc

/// Stickies have no fuse, so a lob has the time it needs.
fn k_low_arc() -> Engine {
    Engine::new(K_GRAVITY, K_TICK, 4.0, 5.0).with_arc(Arc::Low)
}

fn k_high_arc() -> Engine {
    Engine::new(K_GRAVITY, K_TICK, 4.0, 5.0).with_arc(Arc::High)
}

#[test]
fn high_arc_drops_onto_the_target_from_above() {
    let target = stationary(800.0, 100.0, 64.0);

    let low = k_low_arc()
        .maybe_calculate_aim(&k_pipe(), &k_pipe_launcher(), &target)
        .unwrap();
    let high = k_high_arc()
        .maybe_calculate_aim(&k_pipe(), &k_pipe_launcher(), &target)
        .unwrap();

    assert!(high.angles.pitch > low.angles.pitch + 20.0);
    assert!(high.time_of_flight > low.time_of_flight + 0.5);
    expect_near(high.angles.yaw, low.angles.yaw, 0.1_f32);

    // It gets there, and it is on its way down when it does
    let arrival = k_high_arc().predict_projectile_position(
        &k_pipe(),
        &k_pipe_launcher(),
        high.angles,
        high.time_of_flight,
    );
    let a_little_before = k_high_arc().predict_projectile_position(
        &k_pipe(),
        &k_pipe_launcher(),
        high.angles,
        high.time_of_flight - 0.05,
    );

    assert!(arrival.distance_to(target.origin) < 0.5);
    assert!(a_little_before.z > arrival.z + 20.0);
}

#[test]
fn high_arc_leads_a_moving_target() {
    let target = moving(v(700.0, 0.0, 64.0), v(100.0, 250.0, 0.0), false);

    let solution = k_high_arc()
        .maybe_calculate_aim(&k_pipe(), &k_pipe_launcher(), &target)
        .unwrap();

    let expected =
        PredEngineTrait::predict_target_position(&target, solution.time_of_flight, K_GRAVITY);
    let arrival = k_high_arc().predict_projectile_position(
        &k_pipe(),
        &k_pipe_launcher(),
        solution.angles,
        solution.time_of_flight,
    );

    expect_vectors_near(solution.predicted_target_position, expected, 1e-2);
    assert!(arrival.distance_to(expected) < 0.5);
    assert!(solution.angles.pitch > 45.0);
}

#[test]
fn high_arc_is_refused_when_the_lob_outlasts_the_horizon() {
    let target = stationary(800.0, 100.0, 64.0);

    // A pipe's fuse is 2.13 s, and no lob to 800 units is back down by then
    assert!(
        k_engine()
            .maybe_calculate_aim(&k_pipe(), &k_pipe_launcher(), &target)
            .is_some()
    );
    assert!(
        Engine::new(K_GRAVITY, K_TICK, 2.13, 5.0)
            .with_arc(Arc::High)
            .maybe_calculate_aim(&k_pipe(), &k_pipe_launcher(), &target)
            .is_none()
    );
}

#[test]
fn high_arc_is_refused_where_the_low_arc_is_too() {
    // Past the pipe's reach (see reach_is_the_one_the_game_gives_the_pipe) there is no arc at all
    let target = stationary(1800.0, 0.0, 64.0);

    assert!(
        k_low_arc()
            .maybe_calculate_aim(&k_pipe(), &k_pipe_launcher(), &target)
            .is_none()
    );
    assert!(
        k_high_arc()
            .maybe_calculate_aim(&k_pipe(), &k_pipe_launcher(), &target)
            .is_none()
    );
}

#[test]
fn without_gravity_the_arcs_are_one_and_the_same() {
    let rocket = Projectile {
        launch_speed: 1100.0,
        ..Default::default()
    };
    let launcher = Launcher {
        eye_origin: v(0.0, 0.0, 64.0),
        muzzle_offset: MuzzleOffset {
            forward: 23.5,
            right: 12.0,
            up: 0.0,
        },
        ..Default::default()
    };
    let target = moving(v(1500.0, 300.0, 200.0), v(0.0, 200.0, 0.0), false);

    let low = k_low_arc()
        .maybe_calculate_aim(&rocket, &launcher, &target)
        .unwrap();
    let high = k_high_arc()
        .maybe_calculate_aim(&rocket, &launcher, &target)
        .unwrap();

    expect_near(high.angles.pitch, low.angles.pitch, 1e-3_f32);
    expect_near(high.angles.yaw, low.angles.yaw, 1e-3_f32);
    expect_near(high.time_of_flight, low.time_of_flight, 1e-3_f32);
}

fn double_precision_scene() -> (Projectile<f64>, Launcher<f64>, Target<f64>) {
    (
        Projectile {
            launch_speed: 3000.0,
            gravity_scale: 1.0,
            drag: 1e-5,
            ..Default::default()
        },
        Launcher {
            eye_origin: Vector3::new(0.0, 0.0, 170.0),
            muzzle_offset: MuzzleOffset {
                forward: 40.0,
                right: 15.0,
                up: 0.0,
            },
            ..Default::default()
        },
        Target {
            origin: Vector3::new(4000.0, 500.0, 300.0),
            velocity: Vector3::new(0.0, -300.0, 0.0),
            is_airborne: false,
        },
    )
}

#[test]
fn solves_the_high_arc_in_double_precision() {
    type Trait = unreal::PredEngineTrait;

    let (round, launcher, target) = double_precision_scene();

    let engine = ProjPredEngineDrag::<Trait>::new(980.0, 1.0 / 60.0, 8.0, 5.0).with_arc(Arc::High);

    let solution = engine
        .maybe_calculate_aim(&round, &launcher, &target)
        .unwrap();

    let expected = Trait::predict_target_position(&target, solution.time_of_flight, 980.0);
    let arrival = engine.predict_projectile_position(
        &round,
        &launcher,
        solution.angles,
        solution.time_of_flight,
    );

    assert!(arrival.distance_to(expected) < 0.5);
    assert!(solution.angles.pitch > 45.0);
}

#[test]
fn solves_in_double_precision() {
    type Trait = unreal::PredEngineTrait;

    let (round, launcher, target) = double_precision_scene();

    let engine = ProjPredEngineDrag::<Trait>::new(980.0, 1.0 / 60.0, 5.0, 5.0);

    let solution = engine
        .maybe_calculate_aim(&round, &launcher, &target)
        .unwrap();

    let expected = Trait::predict_target_position(&target, solution.time_of_flight, 980.0);
    let arrival = engine.predict_projectile_position(
        &round,
        &launcher,
        solution.angles,
        solution.time_of_flight,
    );

    assert!(arrival.distance_to(expected) < 0.5);
}
