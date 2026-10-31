#[macro_use]
mod projection_support;

use omath::Vector3;
use omath::engines::source;
use omath::projection::{self, CameraEngine, ViewPort, ViewPortClipping};
use projection_support::Rng;

#[test]
fn hammer_units_to_centimeters_basic_values() {
    expect_float_eq!(source::units_to_centimeters(0.0), 0.0);
    expect_float_eq!(source::units_to_centimeters(1.0), 2.54);
    expect_float_eq!(source::units_to_centimeters(10.0), 25.4);
    expect_float_eq!(source::units_to_centimeters(-2.0), -5.08);
}

#[test]
fn hammer_units_to_meters_basic_values() {
    expect_near!(source::units_to_meters(0.0), 0.0, 1e-12);
    expect_near!(source::units_to_meters(1.0), 0.0254, 1e-12);
    expect_near!(source::units_to_meters(100.0), 2.54, 1e-12);
    expect_near!(source::units_to_meters(-4.0), -0.1016, 1e-12);
}

#[test]
fn hammer_units_to_kilometers_basic_values() {
    expect_near!(source::units_to_kilometers(0.0), 0.0, 1e-15);
    expect_near!(source::units_to_kilometers(1.0), 0.0000254, 1e-15);
    expect_near!(source::units_to_kilometers(1000.0), 0.0254, 1e-15);
    expect_near!(source::units_to_kilometers(-10.0), -0.000254, 1e-15);
}

#[test]
fn centimeters_to_hammer_units_basic_values() {
    expect_float_eq!(source::centimeters_to_units(0.0), 0.0);
    expect_near!(source::centimeters_to_units(2.54), 1.0, 1e-6);
    expect_near!(source::centimeters_to_units(25.4), 10.0, 1e-5);
    expect_near!(source::centimeters_to_units(-5.08), -2.0, 1e-6);
}

#[test]
fn meters_to_hammer_units_basic_values() {
    expect_near!(source::meters_to_units(0.0), 0.0, 1e-12);
    expect_near!(source::meters_to_units(0.0254), 1.0, 1e-12);
    expect_near!(source::meters_to_units(2.54), 100.0, 1e-10);
    expect_near!(source::meters_to_units(-0.0508), -2.0, 1e-12);
}

#[test]
fn kilometers_to_hammer_units_basic_values() {
    expect_near!(source::kilometers_to_units(0.0), 0.0, 1e-9);
    expect_near!(source::kilometers_to_units(0.0000254), 1.0, 1e-9);
    expect_near!(source::kilometers_to_units(0.00254), 100.0, 1e-7);
    expect_near!(source::kilometers_to_units(-0.0000508), -2.0, 1e-9);
}

#[test]
fn round_trip_hammer_to_centimeters_to_hammer() {
    let hu_f: f32 = 123.456;
    let cm_f = source::units_to_centimeters(hu_f);
    let hu_f_back = source::centimeters_to_units(cm_f);
    expect_near!(hu_f_back, hu_f, 1e-5);

    let hu_d: f64 = -98765.4321;
    let cm_d = source::units_to_centimeters(hu_d);
    let hu_d_back = source::centimeters_to_units(cm_d);
    expect_near!(hu_d_back, hu_d, 1e-10);
}

#[test]
fn round_trip_hammer_to_meters_to_hammer() {
    let hu_f: f32 = 2500.25;
    let m_f = source::units_to_meters(hu_f);
    let hu_f_back = source::meters_to_units(m_f);
    expect_near!(hu_f_back, hu_f, 1e-4);

    let hu_d: f64 = -42000.125;
    let m_d = source::units_to_meters(hu_d);
    let hu_d_back = source::meters_to_units(m_d);
    expect_near!(hu_d_back, hu_d, 1e-10);
}

#[test]
fn round_trip_hammer_to_kilometers_to_hammer() {
    let hu_f: f32 = 100000.0;
    let km_f = source::units_to_kilometers(hu_f);
    let hu_f_back = source::kilometers_to_units(km_f);
    expect_near!(hu_f_back, hu_f, 1e-2); // looser due to float scaling

    let hu_d: f64 = -1234567.89;
    let km_d = source::units_to_kilometers(hu_d);
    let hu_d_back = source::kilometers_to_units(km_d);
    expect_near!(hu_d_back, hu_d, 1e-7);
}

#[test]
fn conversion_chain_consistency() {
    // hu -> cm -> m -> km should match direct helpers
    let hu = 54321.123;

    let cm = source::units_to_centimeters(hu);
    let m_via_cm = cm / 100.0;
    let km_via_cm = m_via_cm / 1000.0;

    let m_direct = source::units_to_meters(hu);
    let km_direct = source::units_to_kilometers(hu);

    expect_near!(m_direct, m_via_cm, 1e-12);
    expect_near!(km_direct, km_via_cm, 1e-15);
}

#[test]
fn supports_float_and_double_types() {
    let _: f32 = source::units_to_centimeters(1.0);
    let _: f64 = source::units_to_centimeters(1.0);
    let _: f32 = source::meters_to_units(1.0);
    let _: f64 = source::kilometers_to_units(1.0);
}

#[test]
fn constexpr_evaluation() {
    let hu: f64 = 10.0;
    let cm: f64 = source::units_to_centimeters(hu);
    let m: f64 = source::units_to_meters(hu);
    let km: f64 = source::units_to_kilometers(hu);

    assert!(cm == 25.4);
    assert!(m == 0.254);
    assert!(km == 0.000254);
}

#[test]
fn forward_vector() {
    let forward = source::forward_vector(Default::default());

    assert_eq!(forward, source::K_ABS_FORWARD);
}

#[test]
fn right_vector() {
    let right = source::right_vector(Default::default());

    assert_eq!(right, source::K_ABS_RIGHT);
}

#[test]
fn up_vector() {
    let up = source::up_vector(Default::default());
    assert_eq!(up, source::K_ABS_UP);
}

#[test]
fn forward_vector_rotation_yaw() {
    let angles = source::ViewAngles {
        yaw: source::YawAngle::from_degrees(-90.0),
        ..Default::default()
    };

    let forward = source::forward_vector(angles);
    expect_near!(forward.x, source::K_ABS_RIGHT.x, 0.00001);
    expect_near!(forward.y, source::K_ABS_RIGHT.y, 0.00001);
    expect_near!(forward.z, source::K_ABS_RIGHT.z, 0.00001);
}

#[test]
fn forward_vector_rotation_pitch() {
    let angles = source::ViewAngles {
        pitch: source::PitchAngle::from_degrees(-89.0),
        ..Default::default()
    };

    let forward = source::forward_vector(angles);
    expect_near!(forward.x, source::K_ABS_UP.x, 0.02);
    expect_near!(forward.y, source::K_ABS_UP.y, 0.01);
    expect_near!(forward.z, source::K_ABS_UP.z, 0.01);
}

#[test]
fn forward_vector_rotation_roll() {
    let angles = source::ViewAngles {
        roll: source::RollAngle::from_degrees(90.0),
        ..Default::default()
    };

    let forward = source::up_vector(angles);
    expect_near!(forward.x, source::K_ABS_RIGHT.x, 0.00001);
    expect_near!(forward.y, source::K_ABS_RIGHT.y, 0.00001);
    expect_near!(forward.z, source::K_ABS_RIGHT.z, 0.00001);
}

#[test]
fn project_target_moved_from_camera() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let mut next_distance: f32 = 0.02;
    while next_distance < 1000.0 {
        let distance = next_distance;
        next_distance += 0.01;
        let projected = cam.world_to_screen(Vector3::new(distance, 0.0, 0.0));

        assert!(projected.is_ok());

        if !projected.is_ok() {
            continue;
        }

        expect_near!(projected.unwrap().x, 960, 0.00001);
        expect_near!(projected.unwrap().y, 540, 0.00001);
    }
}

#[test]
fn project_target_moved_up() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let mut prev = 1080.0;
    let mut next_distance: f32 = 0.0;
    while next_distance < 10.0 {
        let distance = next_distance;
        next_distance += 1.0;
        let projected = cam.world_to_screen(Vector3::new(100.0, 0.0, distance));
        assert!(projected.is_ok());

        if !projected.is_ok() {
            continue;
        }

        assert!(projected.unwrap().y < prev);

        prev = projected.unwrap().y;
    }
}

#[test]
fn camera_set_and_get_fov() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let mut cam = source::Camera::new(
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
    let mut cam = source::Camera::new(
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
    let mut cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.001,
        10000.0,
    );

    let mut failed_points: usize = 0;
    for _i in 0..1000 {
        let position_to_look =
            Vector3::<f32>::new(rng.uniform(dist), rng.uniform(dist), rng.uniform(dist));

        if cam.get_origin().distance_to(position_to_look) < 10.0 {
            continue;
        }

        cam.look_at(position_to_look);

        let projected_pos = cam.world_to_view_port(position_to_look, ViewPortClipping::Auto);

        assert!(projected_pos.is_ok());

        if projected_pos.is_err() {
            continue;
        }

        if (projected_pos.unwrap().x - 0.0).abs() >= 0.0001
            || (projected_pos.unwrap().y - 0.0).abs() >= 0.0001
        {
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
    let mut cam = source::Camera::new(
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

        if (projected_pos.unwrap().x - 0.0).abs() >= 0.01
            || (projected_pos.unwrap().y - 0.0).abs() >= 0.01
        {
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
    let mut cam = source::Camera::new(
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

        if (projected_pos.unwrap().x - 0.0).abs() >= 0.01
            || (projected_pos.unwrap().y - 0.0).abs() >= 0.01
        {
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
    let mut cam = source::Camera::new(
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

        if (projected_pos.unwrap().x - 0.0).abs() >= 0.01
            || (projected_pos.unwrap().y - 0.0).abs() >= 0.025
        {
            failed_points += 1;
        }
    }
    assert!(failed_points <= 100);
}

#[test]
fn look_at_forward() {
    let angles = source::CameraTrait::calc_look_at_angle(Default::default(), source::K_ABS_FORWARD);

    // ReSharper disable once CppTooWideScopeInitStatement
    let dir_vector = source::forward_vector(angles);
    for (result, etalon) in dir_vector
        .as_array()
        .into_iter()
        .zip(source::K_ABS_FORWARD.as_array())
    {
        expect_near!(result, etalon, 0.0001);
    }
}

#[test]
fn look_at_right() {
    let angles = source::CameraTrait::calc_look_at_angle(Default::default(), source::K_ABS_RIGHT);

    // ReSharper disable once CppTooWideScopeInitStatement
    let dir_vector = source::forward_vector(angles);
    for (result, etalon) in dir_vector
        .as_array()
        .into_iter()
        .zip(source::K_ABS_RIGHT.as_array())
    {
        expect_near!(result, etalon, 0.0001);
    }
}

#[test]
fn look_at_up() {
    let angles = source::CameraTrait::calc_look_at_angle(Default::default(), source::K_ABS_RIGHT);

    // ReSharper disable once CppTooWideScopeInitStatement
    let dir_vector = source::forward_vector(angles);
    for (result, etalon) in dir_vector
        .as_array()
        .into_iter()
        .zip(source::K_ABS_RIGHT.as_array())
    {
        expect_near!(result, etalon, 0.0001);
    }
}

#[test]
fn look_at_back() {
    let angles =
        source::CameraTrait::calc_look_at_angle(Default::default(), -source::K_ABS_FORWARD);

    // ReSharper disable once CppTooWideScopeInitStatement
    let dir_vector = source::forward_vector(angles);
    for (result, etalon) in dir_vector
        .as_array()
        .into_iter()
        .zip((-source::K_ABS_FORWARD).as_array())
    {
        expect_near!(result, etalon, 0.0001);
    }
}

#[test]
fn look_at_left() {
    let angles = source::CameraTrait::calc_look_at_angle(Default::default(), -source::K_ABS_RIGHT);

    // ReSharper disable once CppTooWideScopeInitStatement
    let dir_vector = source::forward_vector(angles);
    for (result, etalon) in dir_vector
        .as_array()
        .into_iter()
        .zip((-source::K_ABS_RIGHT).as_array())
    {
        expect_near!(result, etalon, 0.0001);
    }
}

#[test]
fn look_at_down() {
    let angles = source::CameraTrait::calc_look_at_angle(Default::default(), -source::K_ABS_UP);

    // ReSharper disable once CppTooWideScopeInitStatement
    let dir_vector = source::forward_vector(angles);
    expect_near!(dir_vector.z, -0.99984, 0.0001);
    expect_near!(dir_vector.x, -0.017, 0.01);
    expect_near!(dir_vector.y, 0.0, 0.001);
}

#[test]
fn view_angles_as_vector3_zero() {
    let angles: source::ViewAngles = source::ViewAngles::default();
    let vec = angles.as_vector3();

    expect_float_eq!(vec.x, 0.0);
    expect_float_eq!(vec.y, 0.0);
    expect_float_eq!(vec.z, 0.0);
}

#[test]
fn view_angles_as_vector3_values() {
    let angles: source::ViewAngles = source::ViewAngles::new(
        source::PitchAngle::from_degrees(45.0),
        source::YawAngle::from_degrees(-90.0),
        source::RollAngle::from_degrees(30.0),
    );
    let vec = angles.as_vector3();

    expect_float_eq!(vec.x, 45.0);
    expect_float_eq!(vec.y, -90.0);
    expect_float_eq!(vec.z, 30.0);
}

#[test]
fn view_angles_as_vector3_clamped_pitch() {
    // Pitch is clamped to [-89, 89]
    let angles: source::ViewAngles = source::ViewAngles::new(
        source::PitchAngle::from_degrees(120.0),
        source::YawAngle::from_degrees(0.0),
        source::RollAngle::from_degrees(0.0),
    );
    let vec = angles.as_vector3();

    expect_float_eq!(vec.x, 89.0);
}

#[test]
fn view_angles_as_vector3_normalized_yaw() {
    // Yaw is normalized to [-180, 180], 270 wraps to -90
    let angles: source::ViewAngles = source::ViewAngles::new(
        source::PitchAngle::from_degrees(0.0),
        source::YawAngle::from_degrees(270.0),
        source::RollAngle::from_degrees(0.0),
    );
    let vec = angles.as_vector3();

    expect_near!(vec.y, -90.0, 0.01);
}
