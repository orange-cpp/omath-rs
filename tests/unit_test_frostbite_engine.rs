#[macro_use]
mod projection_support;

use omath::engines::frostbite;
use omath::projection::{self, CameraEngine, ProjectionParams, ScreenStart, ViewPort, ViewPortClipping};
use omath::{NdcDepthRange, Vector3};
use projection_support::Rng;

const K_FOV_TOLERANCE_DEG: f32 = 0.001;
const K_ASPECT_TOLERANCE: f32 = 1e-5;

#[test]
fn units_to_centimeters_basic_values() {
    expect_float_eq!(frostbite::units_to_centimeters(0.0), 0.0);
    expect_float_eq!(frostbite::units_to_centimeters(1.0), 100.0);
    expect_float_eq!(frostbite::units_to_centimeters(100.0), 10000.0);
    expect_float_eq!(frostbite::units_to_centimeters(-250.0), -25000.0);
}

#[test]
fn units_to_meters_basic_values() {
    expect_double_eq!(frostbite::units_to_meters(0.0), 0.0);
    expect_double_eq!(frostbite::units_to_meters(1.0), 1.0);
    expect_double_eq!(frostbite::units_to_meters(123.456), 123.456);
    expect_double_eq!(frostbite::units_to_meters(-42.0), -42.0);
}

#[test]
fn units_to_kilometers_basic_values() {
    expect_near!(frostbite::units_to_kilometers(0.0), 0.0, 1e-15);
    expect_near!(frostbite::units_to_kilometers(1.0), 0.001, 1e-15);
    expect_near!(frostbite::units_to_kilometers(1000.0), 1.0, 1e-12);
    expect_near!(frostbite::units_to_kilometers(-2500.0), -2.5, 1e-12);
}

#[test]
fn centimeters_to_units_basic_values() {
    expect_float_eq!(frostbite::centimeters_to_units(0.0), 0.0);
    expect_float_eq!(frostbite::centimeters_to_units(0.01), 0.0001);
    expect_float_eq!(frostbite::centimeters_to_units(1.0), 0.01);
    expect_float_eq!(frostbite::centimeters_to_units(-2.5), -0.025);
}

#[test]
fn meters_to_units_basic_values() {
    expect_double_eq!(frostbite::meters_to_units(0.0), 0.0);
    expect_double_eq!(frostbite::meters_to_units(1.0), 1.0);
    expect_double_eq!(frostbite::meters_to_units(123.456), 123.456);
    expect_double_eq!(frostbite::meters_to_units(-42.0), -42.0);
}

#[test]
fn kilometers_to_units_basic_values() {
    expect_near!(frostbite::kilometers_to_units(0.0), 0.0, 1e-12);
    expect_near!(frostbite::kilometers_to_units(0.001), 1.0, 1e-12);
    expect_near!(frostbite::kilometers_to_units(1.0), 1000.0, 1e-9);
    expect_near!(frostbite::kilometers_to_units(-2.5), -2500.0, 1e-9);
}

#[test]
fn round_trip_units_centimeters() {
    let units_f: f32 = 12345.678;
    let cm_f = frostbite::units_to_centimeters(units_f);
    let units_f_back = frostbite::centimeters_to_units(cm_f);
    expect_near!(units_f_back, units_f, 1e-3);

    let units_d: f64 = -987654.321;
    let cm_d = frostbite::units_to_centimeters(units_d);
    let units_d_back = frostbite::centimeters_to_units(cm_d);
    expect_near!(units_d_back, units_d, 1e-9);
}

#[test]
fn round_trip_units_meters() {
    let units_f: f32 = 5432.125;
    let m_f = frostbite::units_to_meters(units_f);
    let units_f_back = frostbite::meters_to_units(m_f);
    expect_float_eq!(units_f_back, units_f);

    let units_d: f64 = -123456.789;
    let m_d = frostbite::units_to_meters(units_d);
    let units_d_back = frostbite::meters_to_units(m_d);
    expect_double_eq!(units_d_back, units_d);
}

#[test]
fn round_trip_units_kilometers() {
    let units_f: f32 = 100000.0;
    let km_f = frostbite::units_to_kilometers(units_f);
    let units_f_back = frostbite::kilometers_to_units(km_f);
    expect_near!(units_f_back, units_f, 1e-2);

    let units_d: f64 = -7654321.123;
    let km_d = frostbite::units_to_kilometers(units_d);
    let units_d_back = frostbite::kilometers_to_units(km_d);
    expect_near!(units_d_back, units_d, 1e-6);
}

#[test]
fn conversion_chain_consistency() {
    let units: f64 = 424242.42;

    let cm_direct = frostbite::units_to_centimeters(units);
    let cm_via_units = units * 100.0;
    expect_near!(cm_direct, cm_via_units, 1e-12);

    let km_direct = frostbite::units_to_kilometers(units);
    let km_via_meters = frostbite::units_to_meters(units) / 1000.0;
    expect_near!(km_direct, km_via_meters, 1e-12);
}

#[test]
fn supports_float_and_double() {
    let _: f32 = frostbite::units_to_centimeters(1.0);
    let _: f64 = frostbite::units_to_centimeters(1.0);
    let _: f32 = frostbite::meters_to_units(1.0);
    let _: f64 = frostbite::kilometers_to_units(1.0);
}

#[test]
fn constexpr_conversions() {
    let units: f64 = 1000.0;
    let cm: f64 = frostbite::units_to_centimeters(units);
    let m: f64 = frostbite::units_to_meters(units);
    let km: f64 = frostbite::units_to_kilometers(units);

    assert!(cm == 100000.0);
    assert!(m == 1000.0);
    assert!(km == 1.0);
}

#[test]
fn forward_vector() {
    let forward = frostbite::forward_vector(Default::default());

    assert_eq!(forward, frostbite::K_ABS_FORWARD);
}

#[test]
fn forward_vector_rotation_yaw() {
    let mut angles = frostbite::ViewAngles::default();

    angles.yaw = frostbite::YawAngle::from_degrees(90.0);

    let forward = frostbite::forward_vector(angles);
    expect_near!(forward.x, frostbite::K_ABS_RIGHT.x, 0.00001);
    expect_near!(forward.y, frostbite::K_ABS_RIGHT.y, 0.00001);
    expect_near!(forward.z, frostbite::K_ABS_RIGHT.z, 0.00001);
}

#[test]
fn forward_vector_rotation_pitch() {
    let mut angles = frostbite::ViewAngles::default();

    angles.pitch = frostbite::PitchAngle::from_degrees(-90.0);

    let forward = frostbite::forward_vector(angles);
    expect_near!(forward.x, frostbite::K_ABS_UP.x, 0.00001);
    expect_near!(forward.y, frostbite::K_ABS_UP.y, 0.00001);
    expect_near!(forward.z, frostbite::K_ABS_UP.z, 0.00001);
}

#[test]
fn forward_vector_rotation_roll() {
    let mut angles = frostbite::ViewAngles::default();

    angles.roll = frostbite::RollAngle::from_degrees(-90.0);

    let forward = frostbite::up_vector(angles);
    expect_near!(forward.x, frostbite::K_ABS_RIGHT.x, 0.00001);
    expect_near!(forward.y, frostbite::K_ABS_RIGHT.y, 0.00001);
    expect_near!(forward.z, frostbite::K_ABS_RIGHT.z, 0.00001);
}

#[test]
fn right_vector() {
    let right = frostbite::right_vector(Default::default());

    assert_eq!(right, frostbite::K_ABS_RIGHT);
}

#[test]
fn up_vector() {
    let up = frostbite::up_vector(Default::default());
    assert_eq!(up, frostbite::K_ABS_UP);
}

#[test]
fn project_target_moved_from_camera() {
    let fov = projection::FieldOfView::from_degrees(60.0);
    let cam = frostbite::Camera::new(
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
        let projected = cam.world_to_screen(Vector3::new(0.0, 0.0, distance));

        assert!(projected.is_ok());

        if !projected.is_ok() {
            continue;
        }

        expect_near!(projected.unwrap().x, 640, 0.00001);
        expect_near!(projected.unwrap().y, 360, 0.00001);
    }
}

#[test]
fn project() {
    let fov = projection::FieldOfView::from_degrees(60.0);

    let cam = frostbite::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1280.0, 720.0),
        fov,
        0.03,
        1000.0,
    );
    let proj = cam.world_to_screen_with(Vector3::new(10.0, 3.0, 10.0), ScreenStart::BottomLeftCorner);

    expect_near!(proj.unwrap().x, 1263.538, 0.001);
    expect_near!(proj.unwrap().y, 547.061, 0.001);
}

#[test]
fn camera_set_and_get_fov() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let mut cam = frostbite::Camera::new(
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
    let mut cam = frostbite::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        Default::default(),
        0.01,
        1000.0,
    );

    assert_eq!(cam.get_origin(), Vector3::<f32>::default());
    cam.set_field_of_view(projection::FieldOfView::from_degrees(50.0));

    assert_eq!(cam.get_field_of_view().as_degrees(), 50.0);
}

#[test]
fn loook_at_random_all_axis() {
    let mut rng = Rng::new(0x5EED);
    let dist = (-1000.0, 1000.0);

    let fov = projection::FieldOfView::from_degrees(90.0);
    let mut cam = frostbite::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.001,
        10000.0,
    );

    let mut failed_points: usize = 0;

    for _i in 0..1000 {
        let position_to_look = Vector3::<f32>::new(rng.uniform(dist), rng.uniform(dist), rng.uniform(dist));

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
    let mut cam = frostbite::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.001,
        10000.0,
    );

    let mut failed_points: usize = 0;
    for _i in 0..1000 {
        let position_to_look = Vector3::<f32>::new(rng.uniform(dist), 0.0, 0.0);
        if cam.get_origin().distance_to(position_to_look) < 10.0 {
            continue;
        }

        cam.look_at(position_to_look);

        let projected_pos = cam.world_to_view_port(position_to_look, ViewPortClipping::Auto);

        assert!(projected_pos.is_ok());

        if projected_pos.is_err() {
            continue;
        }

        if (projected_pos.unwrap().x - 0.0).abs() >= 0.001 || (projected_pos.unwrap().y - 0.0).abs() >= 0.001 {
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
    let mut cam = frostbite::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.001,
        10000.0,
    );

    let mut failed_points: usize = 0;
    for _i in 0..1000 {
        let position_to_look = Vector3::<f32>::new(0.0, rng.uniform(dist), 0.0);
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
    let mut cam = frostbite::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.001,
        10000.0,
    );

    let mut failed_points: usize = 0;
    for _i in 0..1000 {
        let position_to_look = Vector3::<f32>::new(0.0, 0.0, rng.uniform(dist));
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
fn look_at_right() {
    let angles = frostbite::CameraTrait::calc_look_at_angle(Default::default(), frostbite::K_ABS_RIGHT);

    // ReSharper disable once CppTooWideScopeInitStatement
    let dir_vector = frostbite::forward_vector(angles);
    for (result, etalon) in dir_vector.as_array().into_iter().zip(frostbite::K_ABS_RIGHT.as_array()) {
        expect_near!(result, etalon, 0.0001);
    }
}

#[test]
fn look_at_up() {
    let angles = frostbite::CameraTrait::calc_look_at_angle(Default::default(), frostbite::K_ABS_RIGHT);

    // ReSharper disable once CppTooWideScopeInitStatement
    let dir_vector = frostbite::forward_vector(angles);
    for (result, etalon) in dir_vector.as_array().into_iter().zip(frostbite::K_ABS_RIGHT.as_array()) {
        expect_near!(result, etalon, 0.0001);
    }
}

#[test]
fn look_at_back() {
    let angles = frostbite::CameraTrait::calc_look_at_angle(Default::default(), -frostbite::K_ABS_FORWARD);

    // ReSharper disable once CppTooWideScopeInitStatement
    let dir_vector = frostbite::forward_vector(angles);
    for (result, etalon) in dir_vector.as_array().into_iter().zip((-frostbite::K_ABS_FORWARD).as_array()) {
        expect_near!(result, etalon, 0.0001);
    }
}

#[test]
fn look_at_left() {
    let angles = frostbite::CameraTrait::calc_look_at_angle(Default::default(), -frostbite::K_ABS_RIGHT);

    // ReSharper disable once CppTooWideScopeInitStatement
    let dir_vector = frostbite::forward_vector(angles);
    for (result, etalon) in dir_vector.as_array().into_iter().zip((-frostbite::K_ABS_RIGHT).as_array()) {
        expect_near!(result, etalon, 0.0001);
    }
}

#[test]
fn look_at_down() {
    let angles = frostbite::CameraTrait::calc_look_at_angle(Default::default(), -frostbite::K_ABS_UP);

    // ReSharper disable once CppTooWideScopeInitStatement
    let dir_vector = frostbite::forward_vector(angles);
    for (result, etalon) in dir_vector.as_array().into_iter().zip((-frostbite::K_ABS_UP).as_array()) {
        expect_near!(result, etalon, 0.0001);
    }
}

#[test]
fn view_angles_as_vector3_zero() {
    let angles: frostbite::ViewAngles = frostbite::ViewAngles::default();
    let vec = angles.as_vector3();

    expect_float_eq!(vec.x, 0.0);
    expect_float_eq!(vec.y, 0.0);
    expect_float_eq!(vec.z, 0.0);
}

#[test]
fn view_angles_as_vector3_values() {
    let angles: frostbite::ViewAngles = frostbite::ViewAngles::new(
        frostbite::PitchAngle::from_degrees(45.0),
        frostbite::YawAngle::from_degrees(-90.0),
        frostbite::RollAngle::from_degrees(30.0),
    );
    let vec = angles.as_vector3();

    expect_float_eq!(vec.x, 45.0);
    expect_float_eq!(vec.y, -90.0);
    expect_float_eq!(vec.z, 30.0);
}

#[test]
fn view_angles_as_vector3_clamped_pitch() {
    let angles: frostbite::ViewAngles = frostbite::ViewAngles::new(
        frostbite::PitchAngle::from_degrees(120.0),
        frostbite::YawAngle::from_degrees(0.0),
        frostbite::RollAngle::from_degrees(0.0),
    );
    let vec = angles.as_vector3();

    expect_float_eq!(vec.x, 90.0);
}

#[test]
fn view_angles_as_vector3_normalized_yaw() {
    let angles: frostbite::ViewAngles = frostbite::ViewAngles::new(
        frostbite::PitchAngle::from_degrees(0.0),
        frostbite::YawAngle::from_degrees(270.0),
        frostbite::RollAngle::from_degrees(0.0),
    );
    let vec = angles.as_vector3();

    expect_near!(vec.y, -90.0, 0.01);
}

#[test]
fn extract_projection_params_basic_round_trip() {
    // Build a matrix with known inputs and verify both outputs are recovered.
    let fov_deg: f32 = 60.0;
    let aspect: f32 = 16.0 / 9.0;
    let mat = frostbite::calc_perspective_projection_matrix(fov_deg, aspect, 0.1, 1000.0, NdcDepthRange::ZeroToOne);

    let ProjectionParams { fov, aspect_ratio: ar } = frostbite::Camera::extract_projection_params(&mat);

    expect_near!(fov.as_degrees(), fov_deg, K_FOV_TOLERANCE_DEG);
    expect_near!(ar, aspect, K_ASPECT_TOLERANCE);
}

#[test]
fn extract_projection_params_neg_one_to_one_depth_range() {
    // The FOV/aspect encoding in rows 0 and 1 is identical for both NDC
    // depth ranges, so extraction must work the same way.
    let fov_deg: f32 = 75.0;
    let aspect: f32 = 4.0 / 3.0;
    let mat =
        frostbite::calc_perspective_projection_matrix(fov_deg, aspect, 0.1, 500.0, NdcDepthRange::NegativeOneToOne);

    let ProjectionParams { fov, aspect_ratio: ar } = frostbite::Camera::extract_projection_params(&mat);

    expect_near!(fov.as_degrees(), fov_deg, K_FOV_TOLERANCE_DEG);
    expect_near!(ar, aspect, K_ASPECT_TOLERANCE);
}

#[test]
fn extract_projection_params_fov45() {
    let fov_deg: f32 = 45.0;
    let aspect: f32 = 16.0 / 9.0;
    let mat = frostbite::calc_perspective_projection_matrix(
        fov_deg,
        aspect,
        0.01,
        1000.0,
        frostbite::K_DEFAULT_NDC_DEPTH_RANGE,
    );

    let ProjectionParams { fov, aspect_ratio: ar } = frostbite::Camera::extract_projection_params(&mat);

    expect_near!(fov.as_degrees(), fov_deg, K_FOV_TOLERANCE_DEG);
    expect_near!(ar, aspect, K_ASPECT_TOLERANCE);
}

#[test]
fn extract_projection_params_fov90() {
    let fov_deg: f32 = 90.0;
    let aspect: f32 = 16.0 / 9.0;
    let mat = frostbite::calc_perspective_projection_matrix(
        fov_deg,
        aspect,
        0.01,
        1000.0,
        frostbite::K_DEFAULT_NDC_DEPTH_RANGE,
    );

    let ProjectionParams { fov, aspect_ratio: ar } = frostbite::Camera::extract_projection_params(&mat);

    expect_near!(fov.as_degrees(), fov_deg, K_FOV_TOLERANCE_DEG);
    expect_near!(ar, aspect, K_ASPECT_TOLERANCE);
}

#[test]
fn extract_projection_params_fov120() {
    let fov_deg: f32 = 120.0;
    let aspect: f32 = 16.0 / 9.0;
    let mat = frostbite::calc_perspective_projection_matrix(
        fov_deg,
        aspect,
        0.01,
        1000.0,
        frostbite::K_DEFAULT_NDC_DEPTH_RANGE,
    );

    let ProjectionParams { fov, aspect_ratio: ar } = frostbite::Camera::extract_projection_params(&mat);

    expect_near!(fov.as_degrees(), fov_deg, K_FOV_TOLERANCE_DEG);
    expect_near!(ar, aspect, K_ASPECT_TOLERANCE);
}

#[test]
fn extract_projection_params_aspect_ratio_4by3() {
    let fov_deg: f32 = 60.0;
    let aspect: f32 = 4.0 / 3.0;
    let mat = frostbite::calc_perspective_projection_matrix(
        fov_deg,
        aspect,
        0.1,
        500.0,
        frostbite::K_DEFAULT_NDC_DEPTH_RANGE,
    );

    let ProjectionParams { fov, aspect_ratio: ar } = frostbite::Camera::extract_projection_params(&mat);

    expect_near!(fov.as_degrees(), fov_deg, K_FOV_TOLERANCE_DEG);
    expect_near!(ar, aspect, K_ASPECT_TOLERANCE);
}

#[test]
fn extract_projection_params_aspect_ratio_ultrawide() {
    let fov_deg: f32 = 90.0;
    let aspect: f32 = 21.0 / 9.0;
    let mat = frostbite::calc_perspective_projection_matrix(
        fov_deg,
        aspect,
        0.1,
        500.0,
        frostbite::K_DEFAULT_NDC_DEPTH_RANGE,
    );

    let ProjectionParams { fov, aspect_ratio: ar } = frostbite::Camera::extract_projection_params(&mat);

    expect_near!(fov.as_degrees(), fov_deg, K_FOV_TOLERANCE_DEG);
    expect_near!(ar, aspect, K_ASPECT_TOLERANCE);
}

#[test]
fn extract_projection_params_aspect_ratio_square() {
    let fov_deg: f32 = 90.0;
    let aspect: f32 = 1.0;
    let mat = frostbite::calc_perspective_projection_matrix(
        fov_deg,
        aspect,
        0.1,
        500.0,
        frostbite::K_DEFAULT_NDC_DEPTH_RANGE,
    );

    let ProjectionParams { fov, aspect_ratio: ar } = frostbite::Camera::extract_projection_params(&mat);

    expect_near!(fov.as_degrees(), fov_deg, K_FOV_TOLERANCE_DEG);
    expect_near!(ar, aspect, K_ASPECT_TOLERANCE);
}

#[test]
fn extract_projection_params_fov_and_aspect_are_independent() {
    // Changing only FOV must not affect recovered aspect ratio, and vice versa.
    let aspect: f32 = 16.0 / 9.0;

    for fov_deg in [45.0, 60.0, 90.0, 110.0] {
        let mat = frostbite::calc_perspective_projection_matrix(
            fov_deg,
            aspect,
            0.1,
            1000.0,
            frostbite::K_DEFAULT_NDC_DEPTH_RANGE,
        );
        let ProjectionParams { fov, aspect_ratio: ar } = frostbite::Camera::extract_projection_params(&mat);

        expect_near!(fov.as_degrees(), fov_deg, K_FOV_TOLERANCE_DEG);
        expect_near!(ar, aspect, K_ASPECT_TOLERANCE);
    }
}

#[test]
fn extract_projection_params_via_camera_round_trip() {
    // End-to-end: construct a Camera, retrieve its projection matrix, then
    // recover the FOV and aspect ratio and compare against the original inputs.
    let fov_in = projection::FieldOfView::from_degrees(90.0);
    let aspect: f32 = 1920.0 / 1080.0;

    let cam = frostbite::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov_in,
        0.01,
        1000.0,
    );

    let ProjectionParams { fov: fov_out, aspect_ratio: ar_out } =
        frostbite::Camera::extract_projection_params(&cam.get_projection_matrix());

    expect_near!(fov_out.as_degrees(), fov_in.as_degrees(), K_FOV_TOLERANCE_DEG);
    expect_near!(ar_out, aspect, K_ASPECT_TOLERANCE);
}

#[test]
fn extract_projection_params_via_camera_after_fov_change() {
    // Verify that the extracted FOV tracks the camera's FOV after set_field_of_view().
    let mut cam = frostbite::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        projection::FieldOfView::from_degrees(60.0),
        0.01,
        1000.0,
    );

    cam.set_field_of_view(projection::FieldOfView::from_degrees(110.0));

    let ProjectionParams { fov, aspect_ratio: ar } =
        frostbite::Camera::extract_projection_params(&cam.get_projection_matrix());

    expect_near!(fov.as_degrees(), 110.0, K_FOV_TOLERANCE_DEG);
    expect_near!(ar, 1920.0 / 1080.0, K_ASPECT_TOLERANCE);
}

#[test]
fn extract_projection_params_via_camera_after_viewport_change() {
    // Verify that the extracted aspect ratio tracks the viewport after set_view_port().
    let mut cam = frostbite::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        projection::FieldOfView::from_degrees(90.0),
        0.01,
        1000.0,
    );

    cam.set_view_port(ViewPort::new(1280.0, 720.0));

    let ProjectionParams { fov, aspect_ratio: ar } =
        frostbite::Camera::extract_projection_params(&cam.get_projection_matrix());

    expect_near!(fov.as_degrees(), 90.0, K_FOV_TOLERANCE_DEG);
    expect_near!(ar, 1280.0 / 720.0, K_ASPECT_TOLERANCE);
}
