//! The prediction parts of tests/engines/unit_test_traits_engines.cpp, unit_test_rage_engine.cpp and
//! unit_test_serious_engine.cpp. The mesh and camera parts of the same C++ tests are in unit_test_traits_engines.rs.

mod common;

use common::expect_near;
use omath::engines::{cry, frostbite, iw, opengl, rage, serious, source, unity, unreal};
use omath::projectile_prediction::{PredEngine, Projectile, Target};
use omath::scalar::Scalar;
use omath::trigonometry::angles;
use omath::{Real, Vector3};

fn v(x: f32, y: f32, z: f32) -> Vector3<f32> {
    Vector3::new(x, y, z)
}

fn verify_launch_offset_at_time_zero<E: PredEngine>(
    origin: Vector3<E::Scalar>,
    offset: Vector3<E::Scalar>,
) {
    let p = Projectile {
        origin,
        launch_offset: offset,
        launch_speed: E::Scalar::from_f64(100.0),
        gravity_scale: E::Scalar::ONE,
        ..Default::default()
    };

    let zero = E::Scalar::ZERO;
    let pos = E::predict_projectile_position(
        p.origin + p.launch_offset,
        &p,
        zero,
        zero,
        zero,
        E::Scalar::from_f64(9.81),
    );
    let expected = origin + offset;
    expect_near(pos.x.to_f64(), expected.x.to_f64(), 1e-4);
    expect_near(pos.y.to_f64(), expected.y.to_f64(), 1e-4);
    expect_near(pos.z.to_f64(), expected.z.to_f64(), 1e-4);
}

/// Verify that zero offset matches default-initialized offset behavior.
fn verify_zero_offset_matches_default<E: PredEngine>() {
    let at = E::Scalar::from_f64;

    let p = Projectile {
        origin: Vector3::new(at(10.0), at(20.0), at(30.0)),
        launch_offset: Vector3::default(),
        launch_speed: at(50.0),
        gravity_scale: at(1.0),
        ..Default::default()
    };

    let p2 = Projectile {
        origin: Vector3::new(at(10.0), at(20.0), at(30.0)),
        launch_speed: at(50.0),
        gravity_scale: at(1.0),
        ..Default::default()
    };

    let pos1 = E::predict_projectile_position(
        p.origin + p.launch_offset,
        &p,
        at(15.0),
        at(30.0),
        at(1.0),
        at(9.81),
    );
    let pos2 = E::predict_projectile_position(
        p2.origin + p2.launch_offset,
        &p2,
        at(15.0),
        at(30.0),
        at(1.0),
        at(9.81),
    );

    expect_near(pos1.x.to_f64(), pos2.x.to_f64(), 1e-6);
    expect_near(pos1.y.to_f64(), pos2.y.to_f64(), 1e-6);
    expect_near(pos1.z.to_f64(), pos2.z.to_f64(), 1e-6);
}

fn offset_at_time_zero<E: PredEngine>() {
    let at = E::Scalar::from_f64;

    verify_launch_offset_at_time_zero::<E>(
        Vector3::default(),
        Vector3::new(at(5.0), at(3.0), at(-2.0)),
    );
}

#[test]
fn source_offset_at_time_zero() {
    offset_at_time_zero::<source::PredEngineTrait>();
}

#[test]
fn source_zero_offset_matches_default() {
    verify_zero_offset_matches_default::<source::PredEngineTrait>();
}

#[test]
fn frostbite_offset_at_time_zero() {
    offset_at_time_zero::<frostbite::PredEngineTrait>();
}

#[test]
fn frostbite_zero_offset_matches_default() {
    verify_zero_offset_matches_default::<frostbite::PredEngineTrait>();
}

#[test]
fn iw_offset_at_time_zero() {
    offset_at_time_zero::<iw::PredEngineTrait>();
}

#[test]
fn iw_zero_offset_matches_default() {
    verify_zero_offset_matches_default::<iw::PredEngineTrait>();
}

#[test]
fn opengl_offset_at_time_zero() {
    offset_at_time_zero::<opengl::PredEngineTrait>();
}

#[test]
fn opengl_zero_offset_matches_default() {
    verify_zero_offset_matches_default::<opengl::PredEngineTrait>();
}

#[test]
fn unity_offset_at_time_zero() {
    offset_at_time_zero::<unity::PredEngineTrait>();
}

#[test]
fn unity_zero_offset_matches_default() {
    verify_zero_offset_matches_default::<unity::PredEngineTrait>();
}

#[test]
fn unreal_offset_at_time_zero() {
    offset_at_time_zero::<unreal::PredEngineTrait>();
}

#[test]
fn unreal_zero_offset_matches_default() {
    verify_zero_offset_matches_default::<unreal::PredEngineTrait>();
}

#[test]
fn cry_engine_offset_at_time_zero() {
    offset_at_time_zero::<cry::PredEngineTrait>();
}

#[test]
fn cry_engine_zero_offset_matches_default() {
    verify_zero_offset_matches_default::<cry::PredEngineTrait>();
}

/// Test that offset shifts the projectile position at t>0 as well.
#[test]
fn offset_shifts_trajectory() {
    let p_no_offset = Projectile {
        launch_speed: 100.0,
        gravity_scale: 1.0,
        ..Default::default()
    };

    let p_with_offset = Projectile {
        launch_offset: v(10.0, 5.0, -3.0),
        ..p_no_offset
    };

    let pos1 = source::PredEngineTrait::predict_projectile_position(
        p_no_offset.origin + p_no_offset.launch_offset,
        &p_no_offset,
        20.0,
        45.0,
        2.0,
        9.81,
    );
    let pos2 = source::PredEngineTrait::predict_projectile_position(
        p_with_offset.origin + p_with_offset.launch_offset,
        &p_with_offset,
        20.0,
        45.0,
        2.0,
        9.81,
    );

    // The difference should be exactly the launch offset
    expect_near(pos2.x - pos1.x, 10.0_f32, 1e-4_f32);
    expect_near(pos2.y - pos1.y, 5.0_f32, 1e-4_f32);
    expect_near(pos2.z - pos1.z, -3.0_f32, 1e-4_f32);
}

fn plain_projectile<T: Real>() -> Projectile<T> {
    Projectile {
        launch_speed: T::from_f64(10.0),
        gravity_scale: T::ONE,
        ..Default::default()
    }
}

fn zero_angle_shot<E: PredEngine>(gravity: E::Scalar) -> Vector3<E::Scalar> {
    let p = plain_projectile();
    let zero = E::Scalar::ZERO;

    E::predict_projectile_position(
        p.origin + p.launch_offset,
        &p,
        zero,
        zero,
        E::Scalar::ONE,
        gravity,
    )
}

fn expect_basis_is_orthogonal_and_looks_up_at_45<E: PredEngine<Scalar = f32>>() {
    let basis = E::calc_view_basis(45.0, 0.0);
    expect_near(
        E::get_vector_height_coordinate(basis.forward),
        angles::degrees_to_radians(45.0_f32).sin(),
        1e-5_f32,
    );
    expect_near(basis.forward.dot(basis.right), 0.0_f32, 1e-5_f32);
    expect_near(basis.forward.dot(basis.up), 0.0_f32, 1e-5_f32);
}

// Generic tests for PredEngineTrait behaviour across engines

#[test]
fn frostbite_pred_and_mesh_and_camera() {
    type E = frostbite::PredEngineTrait;

    let pos = zero_angle_shot::<E>(9.81);
    expect_near(pos.x, 0.0_f32, 1e-4_f32);
    expect_near(pos.z, 10.0_f32, 1e-4_f32);
    expect_near(pos.y, -9.81_f32 * 0.5, 1e-4_f32);

    let mut t = Target {
        origin: v(0.0, 5.0, 0.0),
        velocity: v(2.0, 0.0, 0.0),
        is_airborne: true,
    };
    let pred = E::predict_target_position(&t, 2.0, 9.81);
    expect_near(pred.x, 4.0_f32, 1e-6_f32);
    expect_near(pred.y, 5.0_f32 - 9.81 * (2.0 * 2.0) * 0.5, 1e-6_f32);

    // Also test non-airborne path (no gravity applied)
    t.is_airborne = false;
    let pred_ground = E::predict_target_position(&t, 2.0, 9.81);
    expect_near(pred_ground.x, 4.0_f32, 1e-6_f32);
    expect_near(pred_ground.y, 5.0_f32, 1e-6_f32);

    expect_near(
        E::calc_vector_2d_distance(v(3.0, 0.0, 4.0)),
        5.0_f32,
        1e-6_f32,
    );
    expect_near(
        E::get_vector_height_coordinate(v(1.0, 2.5, 3.0)),
        2.5_f32,
        1e-6_f32,
    );

    expect_basis_is_orthogonal_and_looks_up_at_45::<E>();

    // Direct angles
    let origin = v(0.0, 0.0, 0.0);
    let view_to = v(0.0, 1.0, 1.0);
    let dir = (view_to - origin).normalized();
    expect_near(
        E::calc_direct_pitch_angle(origin, view_to),
        angles::radians_to_degrees(dir.y.asin()),
        1e-3_f32,
    );
    expect_near(
        E::calc_direct_yaw_angle(origin, view_to),
        angles::radians_to_degrees(dir.x.atan2(dir.z)),
        1e-3_f32,
    );
}

#[test]
fn iw_pred_and_mesh_and_camera() {
    type E = iw::PredEngineTrait;

    let pos = zero_angle_shot::<E>(9.81);
    expect_near(pos.x, 10.0_f32, 1e-4_f32);
    expect_near(pos.z, -9.81_f32 * 0.5, 1e-4_f32);

    let mut t = Target {
        origin: v(0.0, 0.0, 5.0),
        velocity: v(0.0, 0.0, 2.0),
        is_airborne: true,
    };
    let pred = E::predict_target_position(&t, 2.0, 9.81);
    // predicted = origin + velocity * t -> z = 5 + 2*2 = 9; then gravity applied
    expect_near(pred.z, 9.0_f32 - 9.81 * (2.0 * 2.0) * 0.5, 1e-6_f32);

    expect_near(
        E::calc_vector_2d_distance(v(3.0, 4.0, 0.0)),
        5.0_f32,
        1e-6_f32,
    );
    expect_near(
        E::get_vector_height_coordinate(v(1.0, 2.5, 3.0)),
        3.0_f32,
        1e-6_f32,
    );

    expect_basis_is_orthogonal_and_looks_up_at_45::<E>();

    let origin = v(0.0, 0.0, 0.0);
    let view_to = v(1.0, 1.0, 1.0);
    let dist = origin.distance_to(view_to);
    expect_near(
        E::calc_direct_pitch_angle(origin, view_to),
        angles::radians_to_degrees(((view_to.z - origin.z) / dist).asin()),
        1e-3_f32,
    );

    let delta = view_to - origin;
    expect_near(
        E::calc_direct_yaw_angle(origin, view_to),
        angles::radians_to_degrees(delta.y.atan2(delta.x)),
        1e-3_f32,
    );

    // non-airborne
    t.is_airborne = false;
    let pred_ground_iw = E::predict_target_position(&t, 2.0, 9.81);
    expect_near(pred_ground_iw.z, 9.0_f32, 1e-6_f32);
}

#[test]
fn opengl_pred_and_mesh_and_camera() {
    type E = opengl::PredEngineTrait;

    let pos = zero_angle_shot::<E>(9.81);
    expect_near(pos.z, -10.0_f32, 1e-4_f32);
    expect_near(pos.y, -9.81_f32 * 0.5, 1e-4_f32);

    let mut t = Target {
        origin: v(0.0, 5.0, 0.0),
        velocity: v(2.0, 0.0, 0.0),
        is_airborne: true,
    };
    let pred = E::predict_target_position(&t, 2.0, 9.81);
    expect_near(pred.x, 4.0_f32, 1e-6_f32);
    expect_near(pred.y, 5.0_f32 - 9.81 * (2.0 * 2.0) * 0.5, 1e-6_f32);

    expect_near(
        E::calc_vector_2d_distance(v(3.0, 0.0, 4.0)),
        5.0_f32,
        1e-6_f32,
    );
    expect_near(
        E::get_vector_height_coordinate(v(1.0, 2.5, 3.0)),
        2.5_f32,
        1e-6_f32,
    );

    expect_basis_is_orthogonal_and_looks_up_at_45::<E>();

    let origin = v(0.0, 0.0, 0.0);
    let view_to = v(0.0, 1.0, 1.0);
    let dir = (view_to - origin).normalized();
    expect_near(
        E::calc_direct_pitch_angle(origin, view_to),
        angles::radians_to_degrees(dir.y.asin()),
        1e-3_f32,
    );
    expect_near(
        E::calc_direct_yaw_angle(origin, view_to),
        angles::radians_to_degrees(-dir.x.atan2(-dir.z)),
        1e-3_f32,
    );

    // non-airborne
    t.is_airborne = false;
    let pred_ground_gl = E::predict_target_position(&t, 2.0, 9.81);
    expect_near(pred_ground_gl.x, 4.0_f32, 1e-6_f32);
}

#[test]
fn unity_pred_and_mesh_and_camera() {
    type E = unity::PredEngineTrait;

    let pos = zero_angle_shot::<E>(9.81);
    expect_near(pos.z, 10.0_f32, 1e-4_f32);
    expect_near(pos.y, -9.81_f32 * 0.5, 1e-4_f32);

    let mut t = Target {
        origin: v(0.0, 5.0, 0.0),
        velocity: v(2.0, 0.0, 0.0),
        is_airborne: true,
    };
    let pred = E::predict_target_position(&t, 2.0, 9.81);
    expect_near(pred.x, 4.0_f32, 1e-6_f32);
    expect_near(pred.y, 5.0_f32 - 9.81 * (2.0 * 2.0) * 0.5, 1e-6_f32);

    expect_near(
        E::calc_vector_2d_distance(v(3.0, 0.0, 4.0)),
        5.0_f32,
        1e-6_f32,
    );
    expect_near(
        E::get_vector_height_coordinate(v(1.0, 2.5, 3.0)),
        2.5_f32,
        1e-6_f32,
    );

    expect_basis_is_orthogonal_and_looks_up_at_45::<E>();

    let origin = v(0.0, 0.0, 0.0);
    let view_to = v(0.0, 1.0, 1.0);
    let dir = (view_to - origin).normalized();
    expect_near(
        E::calc_direct_pitch_angle(origin, view_to),
        angles::radians_to_degrees(dir.y.asin()),
        1e-3_f32,
    );
    expect_near(
        E::calc_direct_yaw_angle(origin, view_to),
        angles::radians_to_degrees(dir.x.atan2(dir.z)),
        1e-3_f32,
    );

    // non-airborne
    t.is_airborne = false;
    let pred_ground_unity = E::predict_target_position(&t, 2.0, 9.81);
    expect_near(pred_ground_unity.x, 4.0_f32, 1e-6_f32);
}

#[test]
fn unreal_pred_and_mesh_and_camera() {
    type E = unreal::PredEngineTrait;

    let pos = zero_angle_shot::<E>(9.81);
    expect_near(pos.x, 10.0, 1e-4);
    expect_near(pos.z, -9.81 * 0.5, 1e-4);

    let mut t = Target {
        origin: Vector3::new(0.0, 0.0, 5.0),
        velocity: Vector3::new(2.0, 0.0, 0.0),
        is_airborne: true,
    };
    let pred = E::predict_target_position(&t, 2.0, 9.81);
    expect_near(pred.x, 4.0, 1e-6);
    expect_near(pred.z, 5.0 - 9.81 * (2.0 * 2.0) * 0.5, 1e-6);

    expect_near(
        E::calc_vector_2d_distance(Vector3::new(3.0, 4.0, 0.0)),
        5.0,
        1e-6,
    );
    expect_near(
        E::get_vector_height_coordinate(Vector3::new(1.0, 2.5, 3.0)),
        3.0,
        1e-6,
    );

    let basis = E::calc_view_basis(45.0, 0.0);
    expect_near(
        E::get_vector_height_coordinate(basis.forward),
        angles::degrees_to_radians(45.0_f64).sin(),
        1e-9,
    );
    expect_near(basis.forward.dot(basis.right), 0.0, 1e-9);
    expect_near(basis.forward.dot(basis.up), 0.0, 1e-9);

    let origin = Vector3::new(0.0, 0.0, 0.0);
    let view_to = Vector3::new(1.0, 1.0, 1.0);
    let dir: Vector3<f64> = (view_to - origin).normalized();
    expect_near(
        E::calc_direct_pitch_angle(origin, view_to),
        angles::radians_to_degrees(dir.z.asin()),
        1e-3,
    );
    expect_near(
        E::calc_direct_yaw_angle(origin, view_to),
        angles::radians_to_degrees(dir.y.atan2(dir.x)),
        1e-3,
    );

    // non-airborne
    t.is_airborne = false;
    let pred_ground_unreal = E::predict_target_position(&t, 2.0, 9.81);
    expect_near(pred_ground_unreal.x, 4.0, 1e-6);
}

#[test]
fn pred_engine_trait_uses_z_as_height() {
    type E = rage::PredEngineTrait;

    let pos = zero_angle_shot::<E>(9.81);

    expect_near(pos.x, 0.0_f32, 0.0001_f32);
    expect_near(pos.y, 10.0_f32, 0.0001_f32);
    expect_near(pos.z, -9.81_f32 * 0.5, 0.0001_f32);
    expect_near(
        E::get_vector_height_coordinate(v(1.0, 2.0, 3.0)),
        3.0_f32,
        0.0001_f32,
    );
}

#[test]
fn pred_engine_trait_uses_y_as_height() {
    type E = serious::PredEngineTrait;

    let pos = zero_angle_shot::<E>(9.81);

    // heading and pitch 0 fire down -Z, and gravity pulls along -Y
    expect_near(pos.x, 0.0_f32, 1e-4_f32);
    expect_near(pos.y, -9.81_f32 * 0.5, 1e-4_f32);
    expect_near(pos.z, -10.0_f32, 1e-4_f32);
    expect_near(
        E::get_vector_height_coordinate(v(1.0, 2.0, 3.0)),
        2.0_f32,
        1e-4_f32,
    );
}
