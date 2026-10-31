#[macro_use]
mod projection_support;

use omath::Vector3;
use omath::engines::iw;
use omath::projection::{self, CameraEngine, ViewPort, ViewPortClipping};
use projection_support::Rng;

#[test]
fn forward_vector() {
    let forward = iw::forward_vector(Default::default());

    assert_eq!(forward, iw::K_ABS_FORWARD);
}

#[test]
fn right_vector() {
    let right = iw::right_vector(Default::default());

    assert_eq!(right, iw::K_ABS_RIGHT);
}

#[test]
fn up_vector() {
    let up = iw::up_vector(Default::default());
    assert_eq!(up, iw::K_ABS_UP);
}

#[test]
fn forward_vector_rotation_yaw() {
    let angles = iw::ViewAngles {
        yaw: iw::YawAngle::from_degrees(-90.0),
        ..Default::default()
    };

    let forward = iw::forward_vector(angles);
    expect_near!(forward.x, iw::K_ABS_RIGHT.x, 0.00001);
    expect_near!(forward.y, iw::K_ABS_RIGHT.y, 0.00001);
    expect_near!(forward.z, iw::K_ABS_RIGHT.z, 0.00001);
}

#[test]
fn forward_vector_rotation_pitch() {
    let angles = iw::ViewAngles {
        pitch: iw::PitchAngle::from_degrees(-89.0),
        ..Default::default()
    };

    let forward = iw::forward_vector(angles);
    expect_near!(forward.x, iw::K_ABS_UP.x, 0.02);
    expect_near!(forward.y, iw::K_ABS_UP.y, 0.01);
    expect_near!(forward.z, iw::K_ABS_UP.z, 0.01);
}

#[test]
fn forward_vector_rotation_roll() {
    let angles = iw::ViewAngles {
        roll: iw::RollAngle::from_degrees(90.0),
        ..Default::default()
    };

    let forward = iw::up_vector(angles);
    expect_near!(forward.x, iw::K_ABS_RIGHT.x, 0.00001);
    expect_near!(forward.y, iw::K_ABS_RIGHT.y, 0.00001);
    expect_near!(forward.z, iw::K_ABS_RIGHT.z, 0.00001);
}

#[test]
fn project_target_moved_from_camera() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = iw::Camera::new(
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
fn camera_set_and_get_fov() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let mut cam = iw::Camera::new(
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
    let mut cam = iw::Camera::new(
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
    let mut cam = iw::Camera::new(
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

        if (projected_pos.unwrap().x - 0.0).abs() >= 0.001
            || (projected_pos.unwrap().y - 0.0).abs() >= 0.001
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
    let mut cam = iw::Camera::new(
        Vector3::new(rng.uniform(dist), rng.uniform(dist), rng.uniform(dist)),
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
    let mut cam = iw::Camera::new(
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
    let mut cam = iw::Camera::new(
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
    let angles = iw::CameraTrait::calc_look_at_angle(Default::default(), iw::K_ABS_FORWARD);

    // ReSharper disable once CppTooWideScopeInitStatement
    let dir_vector = iw::forward_vector(angles);
    for (result, etalon) in dir_vector
        .as_array()
        .into_iter()
        .zip(iw::K_ABS_FORWARD.as_array())
    {
        expect_near!(result, etalon, 0.0001);
    }
}

#[test]
fn look_at_right() {
    let angles = iw::CameraTrait::calc_look_at_angle(Default::default(), iw::K_ABS_RIGHT);

    // ReSharper disable once CppTooWideScopeInitStatement
    let dir_vector = iw::forward_vector(angles);
    for (result, etalon) in dir_vector
        .as_array()
        .into_iter()
        .zip(iw::K_ABS_RIGHT.as_array())
    {
        expect_near!(result, etalon, 0.0001);
    }
}

#[test]
fn look_at_up() {
    let angles = iw::CameraTrait::calc_look_at_angle(Default::default(), iw::K_ABS_RIGHT);

    // ReSharper disable once CppTooWideScopeInitStatement
    let dir_vector = iw::forward_vector(angles);
    for (result, etalon) in dir_vector
        .as_array()
        .into_iter()
        .zip(iw::K_ABS_RIGHT.as_array())
    {
        expect_near!(result, etalon, 0.0001);
    }
}

#[test]
fn look_at_back() {
    let angles = iw::CameraTrait::calc_look_at_angle(Default::default(), -iw::K_ABS_FORWARD);

    // ReSharper disable once CppTooWideScopeInitStatement
    let dir_vector = iw::forward_vector(angles);
    for (result, etalon) in dir_vector
        .as_array()
        .into_iter()
        .zip((-iw::K_ABS_FORWARD).as_array())
    {
        expect_near!(result, etalon, 0.0001);
    }
}

#[test]
fn look_at_left() {
    let angles = iw::CameraTrait::calc_look_at_angle(Default::default(), -iw::K_ABS_RIGHT);

    // ReSharper disable once CppTooWideScopeInitStatement
    let dir_vector = iw::forward_vector(angles);
    for (result, etalon) in dir_vector
        .as_array()
        .into_iter()
        .zip((-iw::K_ABS_RIGHT).as_array())
    {
        expect_near!(result, etalon, 0.0001);
    }
}

#[test]
fn look_at_down() {
    let angles = iw::CameraTrait::calc_look_at_angle(Default::default(), -iw::K_ABS_UP);

    // ReSharper disable once CppTooWideScopeInitStatement
    let dir_vector = iw::forward_vector(angles);
    expect_near!(dir_vector.z, -0.99984, 0.0001);
    expect_near!(dir_vector.x, -0.017, 0.01);
    expect_near!(dir_vector.y, 0.0, 0.001);
}

#[test]
fn view_angles_as_vector3_zero() {
    let angles: iw::ViewAngles = iw::ViewAngles::default();
    let vec = angles.as_vector3();

    expect_float_eq!(vec.x, 0.0);
    expect_float_eq!(vec.y, 0.0);
    expect_float_eq!(vec.z, 0.0);
}

#[test]
fn view_angles_as_vector3_values() {
    let angles: iw::ViewAngles = iw::ViewAngles::new(
        iw::PitchAngle::from_degrees(45.0),
        iw::YawAngle::from_degrees(-90.0),
        iw::RollAngle::from_degrees(30.0),
    );
    let vec = angles.as_vector3();

    expect_float_eq!(vec.x, 45.0);
    expect_float_eq!(vec.y, -90.0);
    expect_float_eq!(vec.z, 30.0);
}

#[test]
fn view_angles_as_vector3_clamped_pitch() {
    // Pitch is clamped to [-89, 89]
    let angles: iw::ViewAngles = iw::ViewAngles::new(
        iw::PitchAngle::from_degrees(120.0),
        iw::YawAngle::from_degrees(0.0),
        iw::RollAngle::from_degrees(0.0),
    );
    let vec = angles.as_vector3();

    expect_float_eq!(vec.x, 89.0);
}

#[test]
fn view_angles_as_vector3_normalized_yaw() {
    // Yaw is normalized to [-180, 180], 270 wraps to -90
    let angles: iw::ViewAngles = iw::ViewAngles::new(
        iw::PitchAngle::from_degrees(0.0),
        iw::YawAngle::from_degrees(270.0),
        iw::RollAngle::from_degrees(0.0),
    );
    let vec = angles.as_vector3();

    expect_near!(vec.y, -90.0, 0.01);
}
