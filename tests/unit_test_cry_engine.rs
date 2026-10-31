#[macro_use]
mod projection_support;

use omath::engines::cry;
use omath::projection::{self, CameraEngine, FieldOfView, ViewPort, ViewPortClipping};
use omath::{NdcDepthRange, Vector3};
use projection_support::Rng;

fn expect_vec_near(actual: Vector3<f32>, expected: Vector3<f32>, epsilon: f32) {
    expect_near!(actual.x, expected.x, epsilon);
    expect_near!(actual.y, expected.y, epsilon);
    expect_near!(actual.z, expected.z, epsilon);
}

// The C++ checks these with static_assert, to show that the math works at compile time.

#[test]
fn zero_angles_give_the_basis_vectors() {
    let angles = cry::ViewAngles::default();
    let rotation = cry::rotation_matrix(angles);

    expect_vec_near(cry::forward_vector(angles), cry::K_ABS_FORWARD, 1e-5);
    expect_vec_near(cry::right_vector(angles), cry::K_ABS_RIGHT, 1e-5);
    expect_vec_near(cry::up_vector(angles), cry::K_ABS_UP, 1e-5);
    for diagonal in 0..4 {
        expect_near!(rotation.at(diagonal, diagonal), 1.0_f32, 1e-5);
    }
}

#[test]
fn view_matrix_at_zero_angles() {
    let view = cry::calc_view_matrix(Default::default(), Default::default());

    expect_near!(view.at(0, 0), 1.0_f32, 1e-5);
    expect_near!(view.at(1, 2), 1.0_f32, 1e-5);
    expect_near!(view.at(2, 1), 1.0_f32, 1e-5);
    expect_near!(view.at(3, 3), 1.0_f32, 1e-5);
}

#[test]
fn transform_extraction() {
    let transform = cry::Mat4X4::translation(Vector3::new(1.0, 2.0, 3.0))
        * cry::Mat4X4::scale(Vector3::new(2.0, 3.0, 4.0));

    expect_vec_near(
        cry::extract_origin(&transform),
        Vector3::new(1.0, 2.0, 3.0),
        1e-5,
    );
    expect_vec_near(
        cry::extract_scale(&transform),
        Vector3::new(2.0, 3.0, 4.0),
        1e-5,
    );
}

fn expect_cry_projection(projection: &cry::Mat4X4) {
    expect_near!(projection.at(0, 0), 1.0_f32, 1e-5);
    expect_near!(projection.at(1, 1), 1.0_f32, 1e-5);
    expect_near!(projection.at(2, 2), 1.1_f32, 1e-5);
    expect_near!(projection.at(2, 3), -1.1_f32, 1e-5);
    expect_near!(projection.at(3, 2), 1.0_f32, 1e-5);
}

#[test]
fn projection_matrix() {
    expect_cry_projection(&cry::calc_perspective_projection_matrix(
        90.0,
        1.0,
        1.0,
        11.0,
        NdcDepthRange::ZeroToOne,
    ));
}

#[test]
fn camera_trait() {
    let angles = cry::CameraTrait::calc_look_at_angle(Default::default(), cry::K_ABS_FORWARD);
    let view = cry::CameraTrait::calc_view_matrix(angles, Default::default());
    let projection = cry::CameraTrait::calc_projection_matrix(
        FieldOfView::from_degrees(90.0),
        ViewPort::new(1.0, 1.0),
        1.0,
        11.0,
        NdcDepthRange::ZeroToOne,
    );

    expect_near!(angles.pitch.as_degrees(), 0.0_f32, 1e-5);
    expect_near!(angles.yaw.as_degrees(), 0.0_f32, 1e-5);
    expect_near!(angles.roll.as_degrees(), 0.0_f32, 1e-5);
    expect_near!(view.at(0, 0), 1.0_f32, 1e-5);
    expect_near!(view.at(2, 1), 1.0_f32, 1e-5);
    expect_cry_projection(&projection);
}

#[test]
fn units() {
    assert_eq!(cry::units_to_centimeters(1.0_f32), 100.0);
    assert_eq!(cry::units_to_meters(2.0_f32), 2.0);
    assert_eq!(cry::units_to_kilometers(2000.0_f32), 2.0);
    assert_eq!(cry::centimeters_to_units(100.0_f32), 1.0);
    assert_eq!(cry::meters_to_units(2.0_f32), 2.0);
    assert_eq!(cry::kilometers_to_units(2.0_f32), 2000.0);
}

#[test]
fn look_at_forward() {
    let angles = cry::CameraTrait::calc_look_at_angle(Default::default(), cry::K_ABS_FORWARD);

    // ReSharper disable once CppTooWideScopeInitStatement
    let dir_vector = cry::forward_vector(angles);
    for (result, etalon) in dir_vector
        .as_array()
        .into_iter()
        .zip(cry::K_ABS_FORWARD.as_array())
    {
        expect_near!(result, etalon, 0.0001);
    }
}

#[test]
fn look_at_right() {
    let angles = cry::CameraTrait::calc_look_at_angle(Default::default(), cry::K_ABS_RIGHT);

    // ReSharper disable once CppTooWideScopeInitStatement
    let dir_vector = cry::forward_vector(angles);
    for (result, etalon) in dir_vector
        .as_array()
        .into_iter()
        .zip(cry::K_ABS_RIGHT.as_array())
    {
        expect_near!(result, etalon, 0.0001);
    }
}

#[test]
fn look_at_up() {
    let angles = cry::CameraTrait::calc_look_at_angle(Default::default(), cry::K_ABS_UP);

    // ReSharper disable once CppTooWideScopeInitStatement
    let dir_vector = cry::forward_vector(angles);
    for (result, etalon) in dir_vector
        .as_array()
        .into_iter()
        .zip(cry::K_ABS_UP.as_array())
    {
        expect_near!(result, etalon, 0.0001);
    }
}

#[test]
fn look_at_back() {
    let angles = cry::CameraTrait::calc_look_at_angle(Default::default(), -cry::K_ABS_FORWARD);

    // ReSharper disable once CppTooWideScopeInitStatement
    let dir_vector = cry::forward_vector(angles);
    for (result, etalon) in dir_vector
        .as_array()
        .into_iter()
        .zip((-cry::K_ABS_FORWARD).as_array())
    {
        expect_near!(result, etalon, 0.0001);
    }
}

#[test]
fn look_at_left() {
    let angles = cry::CameraTrait::calc_look_at_angle(Default::default(), -cry::K_ABS_RIGHT);

    // ReSharper disable once CppTooWideScopeInitStatement
    let dir_vector = cry::forward_vector(angles);
    for (result, etalon) in dir_vector
        .as_array()
        .into_iter()
        .zip((-cry::K_ABS_RIGHT).as_array())
    {
        expect_near!(result, etalon, 0.0001);
    }
}

#[test]
fn look_at_down() {
    let angles = cry::CameraTrait::calc_look_at_angle(Default::default(), -cry::K_ABS_UP);

    // ReSharper disable once CppTooWideScopeInitStatement
    let dir_vector = cry::forward_vector(angles);
    for (result, etalon) in dir_vector
        .as_array()
        .into_iter()
        .zip((-cry::K_ABS_UP).as_array())
    {
        expect_near!(result, etalon, 0.0001);
    }
}

#[test]
fn right_vector() {
    let right = cry::right_vector(Default::default());

    assert_eq!(right, cry::K_ABS_RIGHT);
}

#[test]
fn up_vector() {
    let up = cry::up_vector(Default::default());
    assert_eq!(up, cry::K_ABS_UP);
}

#[test]
fn project_target_moved_from_camera() {
    let fov = projection::FieldOfView::from_degrees(60.0);
    let cam = cry::Camera::new(
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
        let projected = cam.world_to_screen(Vector3::new(0.0, distance, 0.0));

        assert!(projected.is_ok());

        if !projected.is_ok() {
            continue;
        }

        expect_near!(projected.unwrap().x, 640, 0.00001);
        expect_near!(projected.unwrap().y, 360, 0.00001);
    }
}

#[test]
fn camera_set_and_get_fov() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let mut cam = cry::Camera::new(
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
    let mut cam = cry::Camera::new(
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
    let mut cam = cry::Camera::new(
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
    let mut cam = cry::Camera::new(
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

        if (projected_pos.unwrap().x - 0.0).abs() >= 0.001
            || (projected_pos.unwrap().y - 0.0).abs() >= 0.001
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
    let mut cam = cry::Camera::new(
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
    let mut cam = cry::Camera::new(
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
            || (projected_pos.unwrap().y - 0.0).abs() >= 0.01
        {
            failed_points += 1;
        }
    }
    assert!(failed_points <= 100);
}

#[test]
fn view_angles_as_vector3_zero() {
    let angles: cry::ViewAngles = cry::ViewAngles::default();
    let vec = angles.as_vector3();

    expect_float_eq!(vec.x, 0.0);
    expect_float_eq!(vec.y, 0.0);
    expect_float_eq!(vec.z, 0.0);
}

#[test]
fn view_angles_as_vector3_values() {
    let angles: cry::ViewAngles = cry::ViewAngles::new(
        cry::PitchAngle::from_degrees(45.0),
        cry::YawAngle::from_degrees(-90.0),
        cry::RollAngle::from_degrees(30.0),
    );
    let vec = angles.as_vector3();

    expect_float_eq!(vec.x, 45.0);
    expect_float_eq!(vec.y, -90.0);
    expect_float_eq!(vec.z, 30.0);
}

#[test]
fn view_angles_as_vector3_clamped_pitch() {
    // Pitch is clamped to [-90, 90]
    let angles: cry::ViewAngles = cry::ViewAngles::new(
        cry::PitchAngle::from_degrees(120.0),
        cry::YawAngle::from_degrees(0.0),
        cry::RollAngle::from_degrees(0.0),
    );
    let vec = angles.as_vector3();

    expect_float_eq!(vec.x, 90.0);
}

#[test]
fn view_angles_as_vector3_normalized_yaw() {
    // Yaw is normalized to [-180, 180], 270 wraps to -90
    let angles: cry::ViewAngles = cry::ViewAngles::new(
        cry::PitchAngle::from_degrees(0.0),
        cry::YawAngle::from_degrees(270.0),
        cry::RollAngle::from_degrees(0.0),
    );
    let vec = angles.as_vector3();

    expect_near!(vec.y, -90.0, 0.01);
}
