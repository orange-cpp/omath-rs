//! Launcher, MuzzleOffset and the parts of ProjPredEngineInterface that do not need a real solver behind them.

mod common;

use common::expect_float_eq;
use omath::Vector3;
use omath::projectile_prediction::{
    AimAngles, AimSolution, Launcher, MuzzleOffset, ProjPredEngineInterface, ProjPredEngineLegacy,
    Projectile, Target, ViewBasis, launcher_from_projectile,
};
use std::cell::{Cell, RefCell};

fn v(x: f32, y: f32, z: f32) -> Vector3<f32> {
    Vector3::new(x, y, z)
}

const K_WORLD_AXES: ViewBasis<f32> = ViewBasis {
    forward: Vector3::new(1.0, 0.0, 0.0),
    right: Vector3::new(0.0, 1.0, 0.0),
    up: Vector3::new(0.0, 0.0, 1.0),
};

#[test]
fn defaults_carry_no_offsets() {
    let launcher = Launcher {
        eye_origin: v(1.0, 2.0, 3.0),
        ..Default::default()
    };

    assert!(launcher.muzzle_offset.is_zero());
    assert_eq!(launcher.world_offset, v(0.0, 0.0, 0.0));
    assert_eq!(launcher.launch_pitch_offset, 0.0);
    assert_eq!(launcher.launch_origin(K_WORLD_AXES), v(1.0, 2.0, 3.0));
}

#[test]
fn launch_origin_adds_every_part() {
    let launcher = Launcher {
        eye_origin: v(1.0, 2.0, 3.0),
        muzzle_offset: MuzzleOffset {
            forward: 10.0,
            right: 20.0,
            up: 30.0,
        },
        world_offset: v(100.0, 200.0, 300.0),
        ..Default::default()
    };

    assert_eq!(launcher.launch_origin(K_WORLD_AXES), v(111.0, 222.0, 333.0));
}

#[test]
fn muzzle_offset_follows_the_basis_and_world_offset_does_not() {
    // "16 ahead, 8 to the right, 6 down" is a different world position once the view has turned; the world part stays
    // where it was.
    let turned = ViewBasis {
        forward: v(0.0, 1.0, 0.0),
        right: v(1.0, 0.0, 0.0),
        up: v(0.0, 0.0, 1.0),
    };
    let launcher = Launcher {
        eye_origin: v(0.0, 0.0, 64.0),
        muzzle_offset: MuzzleOffset {
            forward: 16.0,
            right: 8.0,
            up: -6.0,
        },
        world_offset: v(0.0, 0.0, -10.0),
        ..Default::default()
    };

    assert_eq!(launcher.launch_origin(K_WORLD_AXES), v(16.0, 8.0, 48.0));
    assert_eq!(launcher.launch_origin(turned), v(8.0, 16.0, 48.0));
}

#[test]
fn pitch_offset_does_not_move_the_muzzle() {
    // It tilts the round, not the weapon: the launch origin only ever depends on the basis handed in
    let plain = Launcher {
        eye_origin: v(0.0, 0.0, 64.0),
        muzzle_offset: MuzzleOffset {
            forward: 16.0,
            right: 8.0,
            up: -6.0,
        },
        ..Default::default()
    };
    let tilted = Launcher {
        launch_pitch_offset: 9.46,
        ..plain
    };

    assert_eq!(
        plain.launch_origin(K_WORLD_AXES),
        tilted.launch_origin(K_WORLD_AXES)
    );
}

#[test]
fn muzzle_offset_is_zero_only_when_every_component_is() {
    assert!(MuzzleOffset::<f32>::default().is_zero());
    assert!(
        MuzzleOffset::<f32> {
            forward: -0.0,
            ..Default::default()
        }
        .is_zero()
    );
    assert!(
        !MuzzleOffset::<f32> {
            forward: 1e-6,
            ..Default::default()
        }
        .is_zero()
    );
    assert!(
        !MuzzleOffset::<f32> {
            right: -8.0,
            ..Default::default()
        }
        .is_zero()
    );
    assert!(
        !MuzzleOffset::<f32> {
            up: 0.5,
            ..Default::default()
        }
        .is_zero()
    );
    assert!(
        !MuzzleOffset::<f64> {
            up: 1e-300,
            ..Default::default()
        }
        .is_zero()
    );
}

#[test]
fn works_in_double_precision() {
    let axes = ViewBasis::<f64> {
        forward: Vector3::new(1.0, 0.0, 0.0),
        right: Vector3::new(0.0, 1.0, 0.0),
        up: Vector3::new(0.0, 0.0, 1.0),
    };
    let launcher = Launcher::<f64> {
        eye_origin: Vector3::new(0.5, 0.25, 0.125),
        muzzle_offset: MuzzleOffset {
            forward: 1.0,
            right: 2.0,
            up: 4.0,
        },
        ..Default::default()
    };

    assert_eq!(launcher.launch_origin(axes), Vector3::new(1.5, 2.25, 4.125));
}

#[test]
fn input_and_output_types_are_aggregates() {
    // Struct literals are the intended way to fill these in
    let _ = Launcher::<f32> {
        eye_origin: Vector3::default(),
        muzzle_offset: MuzzleOffset {
            forward: 0.0,
            right: 0.0,
            up: 0.0,
        },
        world_offset: Vector3::default(),
        launch_pitch_offset: 0.0,
    };
    let _ = ViewBasis::<f32> {
        forward: Vector3::default(),
        right: Vector3::default(),
        up: Vector3::default(),
    };
    let _ = AimSolution::<f32> {
        angles: AimAngles {
            pitch: 0.0,
            yaw: 0.0,
        },
        aim_point: Vector3::default(),
        predicted_target_position: Vector3::default(),
        time_of_flight: 0.0,
    };
    let _ = Launcher::<f64>::default();
}

#[test]
fn projectile_needs_only_its_physical_properties() {
    // New code describes the round and leaves the legacy origin fields alone.
    let round = Projectile::<f32> {
        launch_speed: 1100.0,
        gravity_scale: 0.5,
        ..Default::default()
    };

    assert_eq!(round.origin, v(0.0, 0.0, 0.0));
    assert_eq!(round.launch_offset, v(0.0, 0.0, 0.0));
}

// ProjPredEngineInterface

/// Remembers what it was asked and answers with fixed numbers.
struct RecordingEngine {
    last_launcher: RefCell<Option<Launcher<f32>>>,
    calls: Cell<i32>,
    has_solution: bool,
}

impl RecordingEngine {
    fn new(has_solution: bool) -> Self {
        Self {
            last_launcher: RefCell::new(None),
            calls: Cell::new(0),
            has_solution,
        }
    }
}

impl ProjPredEngineInterface<f32> for RecordingEngine {
    fn maybe_calculate_aim(
        &self,
        _: &Projectile<f32>,
        launcher: &Launcher<f32>,
        _: &Target<f32>,
    ) -> Option<AimSolution<f32>> {
        *self.last_launcher.borrow_mut() = Some(*launcher);
        self.calls.set(self.calls.get() + 1);

        self.has_solution.then_some(AimSolution {
            angles: AimAngles {
                pitch: 11.0,
                yaw: 22.0,
            },
            aim_point: v(3.0, 4.0, 5.0),
            predicted_target_position: v(6.0, 7.0, 8.0),
            time_of_flight: 0.5,
        })
    }
}

fn k_legacy_projectile() -> Projectile<f32> {
    Projectile {
        origin: v(1.0, 2.0, 3.0),
        launch_offset: v(4.0, 5.0, 6.0),
        launch_speed: 1000.0,
        gravity_scale: 1.0,
        ..Default::default()
    }
}

fn k_target() -> Target<f32> {
    Target {
        origin: v(100.0, 0.0, 0.0),
        velocity: Vector3::default(),
        is_airborne: false,
    }
}

#[test]
fn launcher_from_projectile_maps_the_legacy_fields() {
    let launcher = launcher_from_projectile(&k_legacy_projectile());

    assert_eq!(launcher.eye_origin, v(1.0, 2.0, 3.0));
    assert_eq!(launcher.world_offset, v(4.0, 5.0, 6.0));
    assert!(launcher.muzzle_offset.is_zero());
    assert_eq!(launcher.launch_pitch_offset, 0.0);
}

#[test]
fn wrappers_solve_through_a_launcher_built_from_the_legacy_fields() {
    let engine = RecordingEngine::new(true);

    let point = engine.maybe_calculate_aim_point(&k_legacy_projectile(), &k_target());
    assert_eq!(point, Some(v(3.0, 4.0, 5.0)));

    let last_launcher = engine.last_launcher.borrow().unwrap();
    assert_eq!(last_launcher.eye_origin, v(1.0, 2.0, 3.0));
    assert_eq!(last_launcher.world_offset, v(4.0, 5.0, 6.0));
    assert!(last_launcher.muzzle_offset.is_zero());
    expect_float_eq(last_launcher.launch_pitch_offset, 0.0);

    let angles = engine
        .maybe_calculate_aim_angles(&k_legacy_projectile(), &k_target())
        .unwrap();
    expect_float_eq(angles.pitch, 11.0);
    expect_float_eq(angles.yaw, 22.0);

    // Each wrapper is a full solve: callers that want both should ask for the solution once
    assert_eq!(engine.calls.get(), 2);
}

#[test]
fn wrappers_pass_no_solution_through() {
    let engine = RecordingEngine::new(false);

    assert!(
        engine
            .maybe_calculate_aim_point(&k_legacy_projectile(), &k_target())
            .is_none()
    );
    assert!(
        engine
            .maybe_calculate_aim_angles(&k_legacy_projectile(), &k_target())
            .is_none()
    );
    assert_eq!(engine.calls.get(), 2);
}

#[test]
fn cannot_be_sliced_through_a_base_reference() {
    // A trait object cannot be copied, which is the slicing the C++ protected special members rule out. Concrete
    // engines stay ordinary values.
    fn is_copy<T: Copy>() {}

    is_copy::<ProjPredEngineLegacy>();

    let _: &dyn ProjPredEngineInterface<f32> = &RecordingEngine::new(true);
}

#[test]
fn legacy_engine_answers_the_same_through_the_interface() {
    let engine = ProjPredEngineLegacy::<omath::engines::source::PredEngineTrait>::new(
        800.0,
        1.0 / 1000.0,
        10.0,
        5.0,
    );
    let through_base: &dyn ProjPredEngineInterface<f32> = &engine;

    let round = Projectile {
        launch_speed: 1500.0,
        gravity_scale: 1.0,
        ..Default::default()
    };
    let launcher = Launcher {
        eye_origin: v(0.0, 0.0, 64.0),
        muzzle_offset: MuzzleOffset {
            forward: 16.0,
            right: 8.0,
            up: -6.0,
        },
        ..Default::default()
    };
    let target = Target {
        origin: v(700.0, 120.0, 90.0),
        velocity: v(-60.0, 45.0, 0.0),
        is_airborne: false,
    };

    let direct = engine
        .maybe_calculate_aim(&round, &launcher, &target)
        .unwrap();
    let dispatched = through_base
        .maybe_calculate_aim(&round, &launcher, &target)
        .unwrap();

    expect_float_eq(direct.angles.pitch, dispatched.angles.pitch);
    expect_float_eq(direct.angles.yaw, dispatched.angles.yaw);
    expect_float_eq(direct.time_of_flight, dispatched.time_of_flight);
    assert_eq!(direct.aim_point, dispatched.aim_point);

    // A copy is its own engine with the same settings
    let copy = engine;
    let from_copy = copy
        .maybe_calculate_aim(&round, &launcher, &target)
        .unwrap();
    expect_float_eq(from_copy.angles.pitch, direct.angles.pitch);
}
