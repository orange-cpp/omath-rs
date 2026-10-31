#[macro_use]
mod projection_support;

use omath::Vector3;
use omath::engines::unreal;
use omath::projection::{self, CameraEngine, ViewPort, ViewPortClipping};
use projection_support::Rng;

#[test]
fn forward_vector() {
    let forward = unreal::forward_vector(Default::default());

    assert_eq!(forward, unreal::K_ABS_FORWARD);
}

#[test]
fn forward_vector_rotation_yaw() {
    let mut angles = unreal::ViewAngles::default();

    angles.yaw = unreal::YawAngle::from_degrees(90.0);

    let forward = unreal::forward_vector(angles);
    expect_near!(forward.x, unreal::K_ABS_RIGHT.x, 0.00001);
    expect_near!(forward.y, unreal::K_ABS_RIGHT.y, 0.00001);
    expect_near!(forward.z, unreal::K_ABS_RIGHT.z, 0.00001);
}

#[test]
fn forward_vector_rotation_pitch() {
    let mut angles = unreal::ViewAngles::default();

    angles.pitch = unreal::PitchAngle::from_degrees(90.0);

    let forward = unreal::forward_vector(angles);
    expect_near!(forward.x, unreal::K_ABS_UP.x, 0.00001);
    expect_near!(forward.y, unreal::K_ABS_UP.y, 0.00001);
    expect_near!(forward.z, unreal::K_ABS_UP.z, 0.00001);
}

#[test]
fn forward_vector_rotation_roll() {
    let mut angles = unreal::ViewAngles::default();

    angles.roll = unreal::RollAngle::from_degrees(90.0);

    let forward = unreal::up_vector(angles);
    expect_near!(forward.x, unreal::K_ABS_RIGHT.x, 0.00001);
    expect_near!(forward.y, unreal::K_ABS_RIGHT.y, 0.00001);
    expect_near!(forward.z, unreal::K_ABS_RIGHT.z, 0.00001);
}

#[test]
fn right_vector() {
    let right = unreal::right_vector(Default::default());

    assert_eq!(right, unreal::K_ABS_RIGHT);
}

#[test]
fn up_vector() {
    let up = unreal::up_vector(Default::default());
    assert_eq!(up, unreal::K_ABS_UP);
}

#[test]
fn project_target_moved_from_camera() {
    let fov = projection::FieldOfView::from_degrees(60.0);
    let cam = unreal::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1280.0, 720.0),
        fov,
        0.01,
        1000.0,
    );

    let mut next_distance: f32 = 0.02;
    while next_distance < 100.0 {
        let distance = next_distance;
        next_distance += 0.01;
        let projected = cam.world_to_screen(Vector3::new(f64::from(distance), 0.0, 0.0));

        assert!(projected.is_ok());

        if !projected.is_ok() {
            continue;
        }

        expect_near!(projected.unwrap().x, 640, 0.00001);
        expect_near!(projected.unwrap().y, 360, 0.00001);
    }
}

#[test]
fn project_target_moved_from_camera_behind() {
    let fov = projection::FieldOfView::from_degrees(60.0);
    let cam = unreal::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1280.0, 720.0),
        fov,
        0.01,
        10000.0,
    );

    let mut next_distance: f32 = 0.02;
    while next_distance < 9000.0 {
        let distance = next_distance;
        next_distance += 100.0;
        let projected = cam.world_to_screen(Vector3::new(f64::from(-distance), 0.0, 0.0));

        assert!(!(projected.is_ok()));
    }
}

#[test]
fn camera_set_and_get_fov() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let mut cam = unreal::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    assert_eq!(cam.get_field_of_view().as_degrees(), 90.0);
    cam.set_field_of_view(projection::FieldOfView::from_degrees(50.0));

    assert_eq!(cam.get_field_of_view().as_degrees(), 50.0);
}

#[test]
fn camera_set_and_get_origin() {
    let mut cam = unreal::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        Default::default(),
        0.01,
        1000.0,
    );

    assert_eq!(cam.get_origin(), Vector3::<f64>::default());
    cam.set_field_of_view(projection::FieldOfView::from_degrees(50.0));

    assert_eq!(cam.get_field_of_view().as_degrees(), 50.0);
}

#[test]
fn loook_at_random_all_axis() {
    let mut rng = Rng::new(0x5EED);
    let dist = (-1000.0, 1000.0);

    let fov = projection::FieldOfView::from_degrees(90.0);
    let mut cam = unreal::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.001,
        10000.0,
    );

    let mut failed_points: usize = 0;
    for _i in 0..100 {
        let position_to_look = Vector3::<f64>::new(rng.uniform(dist), rng.uniform(dist), rng.uniform(dist));

        if cam.get_origin().distance_to(position_to_look) < 10.0 {
            continue;
        }
        cam.look_at(position_to_look);

        let projected_pos = cam.world_to_view_port(position_to_look, ViewPortClipping::Auto);

        assert!(projected_pos.is_ok());

        if projected_pos.is_err() {
            continue;
        }

        if (projected_pos.unwrap().x - 0.0).abs() >= 0.0001 || (projected_pos.unwrap().y - 0.0).abs() >= 0.0001 {
            failed_points += 1;
        }
    }
    assert!(failed_points <= 100);
}

#[test]
fn loook_at_random_x_axis() {
    let mut rng = Rng::new(0x5EED);
    let dist = (-1000.0, 1000.0);

    let fov = projection::FieldOfView::from_degrees(90.0);
    let mut cam = unreal::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.001,
        10000.0,
    );

    let mut failed_points: usize = 0;
    for _i in 0..1000 {
        let position_to_look = Vector3::<f64>::new(rng.uniform(dist), rng.uniform(dist), rng.uniform(dist));

        if cam.get_origin().distance_to(position_to_look) < 10.0 {
            continue;
        }

        cam.look_at(position_to_look);

        let projected_pos = cam.world_to_view_port(position_to_look, ViewPortClipping::Auto);

        assert!(projected_pos.is_ok());

        if projected_pos.is_err() {
            continue;
        }

        if (projected_pos.unwrap().x - 0.0).abs() >= 0.01 || (projected_pos.unwrap().y - 0.0).abs() >= 0.01 {
            failed_points += 1;
        }
    }
    assert!(failed_points <= 100);
}

#[test]
fn loook_at_random_y_axis() {
    let mut rng = Rng::new(0x5EED);
    let dist = (-1000.0, 1000.0);

    let fov = projection::FieldOfView::from_degrees(90.0);
    let mut cam = unreal::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.001,
        10000.0,
    );

    let mut failed_points: usize = 0;
    for _i in 0..1000 {
        let position_to_look = Vector3::<f64>::new(0.0, rng.uniform(dist), 0.0);

        if cam.get_origin().distance_to(position_to_look) < 10.0 {
            continue;
        }

        cam.look_at(position_to_look);

        let projected_pos = cam.world_to_view_port(position_to_look, ViewPortClipping::Auto);

        assert!(projected_pos.is_ok());

        if projected_pos.is_err() {
            continue;
        }

        if (projected_pos.unwrap().x - 0.0).abs() >= 0.01 || (projected_pos.unwrap().y - 0.0).abs() >= 0.01 {
            failed_points += 1;
        }
    }
    assert!(failed_points <= 100);
}

#[test]
fn loook_at_random_z_axis() {
    let mut rng = Rng::new(0x5EED);
    let dist = (-1000.0, 1000.0);

    let fov = projection::FieldOfView::from_degrees(90.0);
    let mut cam = unreal::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.001,
        10000.0,
    );

    let mut failed_points: usize = 0;
    for _i in 0..1000 {
        let position_to_look = Vector3::<f64>::new(0.0, 0.0, rng.uniform(dist));

        if cam.get_origin().distance_to(position_to_look) < 10.0 {
            continue;
        }

        cam.look_at(position_to_look);

        let projected_pos = cam.world_to_view_port(position_to_look, ViewPortClipping::Auto);

        assert!(projected_pos.is_ok());

        if projected_pos.is_err() {
            continue;
        }

        if (projected_pos.unwrap().x - 0.0).abs() >= 0.01 || (projected_pos.unwrap().y - 0.0).abs() >= 0.01 {
            failed_points += 1;
        }
    }
    assert!(failed_points <= 100);
}

#[test]
fn units_to_centimeters_basic_values() {
    expect_float_eq!(unreal::units_to_centimeters(0.0), 0.0);
    expect_float_eq!(unreal::units_to_centimeters(1.0), 1.0);
    expect_float_eq!(unreal::units_to_centimeters(250.0), 250.0);
    expect_float_eq!(unreal::units_to_centimeters(-42.5), -42.5);
}

#[test]
fn units_to_meters_basic_values() {
    expect_near!(unreal::units_to_meters(0.0), 0.0, 1e-15);
    expect_near!(unreal::units_to_meters(1.0), 0.01, 1e-15);
    expect_near!(unreal::units_to_meters(100.0), 1.0, 1e-12);
    expect_near!(unreal::units_to_meters(-250.0), -2.5, 1e-12);
}

#[test]
fn units_to_kilometers_basic_values() {
    expect_near!(unreal::units_to_kilometers(0.0), 0.0, 1e-18);
    expect_near!(unreal::units_to_kilometers(1.0), 0.00001, 1e-18);
    expect_near!(unreal::units_to_kilometers(100000.0), 1.0, 1e-12);
    expect_near!(unreal::units_to_kilometers(-250000.0), -2.5, 1e-12);
}

#[test]
fn centimeters_to_units_basic_values() {
    expect_float_eq!(unreal::centimeters_to_units(0.0), 0.0);
    expect_float_eq!(unreal::centimeters_to_units(1.0), 1.0);
    expect_float_eq!(unreal::centimeters_to_units(250.0), 250.0);
    expect_float_eq!(unreal::centimeters_to_units(-42.5), -42.5);
}

#[test]
fn meters_to_units_basic_values() {
    expect_near!(unreal::meters_to_units(0.0), 0.0, 1e-12);
    expect_near!(unreal::meters_to_units(0.01), 1.0, 1e-12);
    expect_near!(unreal::meters_to_units(1.0), 100.0, 1e-9);
    expect_near!(unreal::meters_to_units(-2.5), -250.0, 1e-9);
}

#[test]
fn kilometers_to_units_basic_values() {
    expect_near!(unreal::kilometers_to_units(0.0), 0.0, 1e-9);
    expect_near!(unreal::kilometers_to_units(0.00001), 1.0, 1e-9);
    expect_near!(unreal::kilometers_to_units(1.0), 100000.0, 1e-6);
    expect_near!(unreal::kilometers_to_units(-2.5), -250000.0, 1e-3);
}

#[test]
fn round_trip_units_centimeters() {
    let units_f: f32 = 12345.678;
    let cm_f = unreal::units_to_centimeters(units_f);
    let units_f_back = unreal::centimeters_to_units(cm_f);
    expect_float_eq!(units_f_back, units_f);

    let units_d: f64 = -987654.321;
    let cm_d = unreal::units_to_centimeters(units_d);
    let units_d_back = unreal::centimeters_to_units(cm_d);
    expect_double_eq!(units_d_back, units_d);
}

#[test]
fn round_trip_units_meters() {
    let units_f: f32 = 5432.125;
    let m_f = unreal::units_to_meters(units_f);
    let units_f_back = unreal::meters_to_units(m_f);
    expect_near!(units_f_back, units_f, 1e-3);

    let units_d: f64 = -123456.789;
    let m_d = unreal::units_to_meters(units_d);
    let units_d_back = unreal::meters_to_units(m_d);
    expect_near!(units_d_back, units_d, 1e-9);
}

#[test]
fn round_trip_units_kilometers() {
    let units_f: f32 = 100000.0;
    let km_f = unreal::units_to_kilometers(units_f);
    let units_f_back = unreal::kilometers_to_units(km_f);
    expect_near!(units_f_back, units_f, 1e-2);

    let units_d: f64 = -7654321.123;
    let km_d = unreal::units_to_kilometers(units_d);
    let units_d_back = unreal::kilometers_to_units(km_d);
    expect_near!(units_d_back, units_d, 1e-6);
}

#[test]
fn conversion_chain_consistency() {
    let units: f64 = 424242.42;

    let cm_direct = unreal::units_to_centimeters(units);
    let cm_expected = units; // 1 uu == 1 cm
    expect_near!(cm_direct, cm_expected, 1e-12);

    let m_direct = unreal::units_to_meters(units);
    let m_via_cm = cm_direct / 100.0;
    expect_near!(m_direct, m_via_cm, 1e-12);

    let km_direct = unreal::units_to_kilometers(units);
    let km_via_m = m_direct / 1000.0;
    expect_near!(km_direct, km_via_m, 1e-15);
}

#[test]
fn supports_float_and_double() {
    let _: f32 = unreal::units_to_centimeters(1.0);
    let _: f64 = unreal::units_to_centimeters(1.0);
    let _: f32 = unreal::meters_to_units(1.0);
    let _: f64 = unreal::kilometers_to_units(1.0);
}

#[test]
fn constexpr_conversions() {
    let units: f64 = 100000.0;
    let cm: f64 = unreal::units_to_centimeters(units);
    let m: f64 = unreal::units_to_meters(units);
    let km: f64 = unreal::units_to_kilometers(units);

    assert!(cm == 100000.0);
    assert!(m == 1000.0);
    assert!(km == 1.0);
}

#[test]
fn look_at_forward() {
    let angles = unreal::CameraTrait::calc_look_at_angle(Default::default(), unreal::K_ABS_FORWARD);

    // ReSharper disable once CppTooWideScopeInitStatement
    let dir_vector = unreal::forward_vector(angles);
    for (result, etalon) in dir_vector.as_array().into_iter().zip(unreal::K_ABS_FORWARD.as_array()) {
        expect_near!(result, etalon, 0.0001);
    }
}

#[test]
fn look_at_right() {
    let angles = unreal::CameraTrait::calc_look_at_angle(Default::default(), unreal::K_ABS_RIGHT);

    // ReSharper disable once CppTooWideScopeInitStatement
    let dir_vector = unreal::forward_vector(angles);
    for (result, etalon) in dir_vector.as_array().into_iter().zip(unreal::K_ABS_RIGHT.as_array()) {
        expect_near!(result, etalon, 0.0001);
    }
}

#[test]
fn look_at_up() {
    let angles = unreal::CameraTrait::calc_look_at_angle(Default::default(), unreal::K_ABS_RIGHT);

    // ReSharper disable once CppTooWideScopeInitStatement
    let dir_vector = unreal::forward_vector(angles);
    for (result, etalon) in dir_vector.as_array().into_iter().zip(unreal::K_ABS_RIGHT.as_array()) {
        expect_near!(result, etalon, 0.0001);
    }
}

#[test]
fn look_at_back() {
    let angles = unreal::CameraTrait::calc_look_at_angle(Default::default(), -unreal::K_ABS_FORWARD);

    // ReSharper disable once CppTooWideScopeInitStatement
    let dir_vector = unreal::forward_vector(angles);
    for (result, etalon) in dir_vector.as_array().into_iter().zip((-unreal::K_ABS_FORWARD).as_array()) {
        expect_near!(result, etalon, 0.0001);
    }
}

#[test]
fn look_at_left() {
    let angles = unreal::CameraTrait::calc_look_at_angle(Default::default(), -unreal::K_ABS_RIGHT);

    // ReSharper disable once CppTooWideScopeInitStatement
    let dir_vector = unreal::forward_vector(angles);
    for (result, etalon) in dir_vector.as_array().into_iter().zip((-unreal::K_ABS_RIGHT).as_array()) {
        expect_near!(result, etalon, 0.0001);
    }
}

#[test]
fn look_at_down() {
    let angles = unreal::CameraTrait::calc_look_at_angle(Default::default(), -unreal::K_ABS_UP);

    // ReSharper disable once CppTooWideScopeInitStatement
    let dir_vector = unreal::forward_vector(angles);
    for (result, etalon) in dir_vector.as_array().into_iter().zip((-unreal::K_ABS_UP).as_array()) {
        expect_near!(result, etalon, 0.0001);
    }
}

#[test]
fn view_angles_as_vector3_zero() {
    let angles: unreal::ViewAngles = unreal::ViewAngles::default();
    let vec = angles.as_vector3();

    expect_float_eq!(vec.x as f32, 0.0);
    expect_float_eq!(vec.y as f32, 0.0);
    expect_float_eq!(vec.z as f32, 0.0);
}

#[test]
fn view_angles_as_vector3_values() {
    let angles: unreal::ViewAngles = unreal::ViewAngles::new(
        unreal::PitchAngle::from_degrees(45.0),
        unreal::YawAngle::from_degrees(-90.0),
        unreal::RollAngle::from_degrees(30.0),
    );
    let vec = angles.as_vector3();

    expect_float_eq!(vec.x as f32, 45.0);
    expect_float_eq!(vec.y as f32, -90.0);
    expect_float_eq!(vec.z as f32, 30.0);
}

#[test]
fn view_angles_as_vector3_clamped_pitch() {
    let angles: unreal::ViewAngles = unreal::ViewAngles::new(
        unreal::PitchAngle::from_degrees(120.0),
        unreal::YawAngle::from_degrees(0.0),
        unreal::RollAngle::from_degrees(0.0),
    );
    let vec = angles.as_vector3();

    expect_float_eq!(vec.x as f32, 90.0);
}

#[test]
fn view_angles_as_vector3_normalized_yaw() {
    let angles: unreal::ViewAngles = unreal::ViewAngles::new(
        unreal::PitchAngle::from_degrees(0.0),
        unreal::YawAngle::from_degrees(270.0),
        unreal::RollAngle::from_degrees(0.0),
    );
    let vec = angles.as_vector3();

    expect_near!(vec.y, -90.0, 0.01);
}
