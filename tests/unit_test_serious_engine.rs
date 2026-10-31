#[macro_use]
mod projection_support;

use omath::engines::serious;
use omath::primitives::MeshEngine;
use omath::projection::{self, CameraEngine, ViewPort};
use omath::{NdcDepthRange, Vector3};

use projection_support::*;

const K_PI: f32 = std::f32::consts::PI;
const K_GAME_PITCH: f32 = -14.6492;
const K_GAME_HEADING: f32 = 144.5696;
const K_GAME_EYE: Vector3<f32> = Vector3::new(-17.1773, -22.0306, -520.0123);

fn make_angles(pitch: f32, yaw: f32, roll: f32) -> serious::ViewAngles {
    serious::ViewAngles::new(
        serious::PitchAngle::from_degrees(pitch),
        serious::YawAngle::from_degrees(yaw),
        serious::RollAngle::from_degrees(roll),
    )
}

fn expect_vector_near(actual: Vector3<f32>, expected: Vector3<f32>, tolerance: f32) {
    for (result, etalon) in actual.as_array().into_iter().zip(expected.as_array()) {
        expect_near!(result, etalon, tolerance);
    }
}

fn angle_difference(a: f32, b: f32) -> f32 {
    remainder(f64::from(a - b), 360.0) as f32
}

#[test]
fn default_axes() {
    assert_eq!(serious::forward_vector(Default::default()), serious::K_ABS_FORWARD);
    assert_eq!(serious::right_vector(Default::default()), serious::K_ABS_RIGHT);
    assert_eq!(serious::up_vector(Default::default()), serious::K_ABS_UP);
}

#[test]
fn axes_are_right_handed_y_up_looking_down_negative_z() {
    assert_eq!(serious::K_ABS_FORWARD, (Vector3::<f32>::new(0.0, 0.0, -1.0)));
    assert_eq!(serious::K_ABS_RIGHT, (Vector3::<f32>::new(1.0, 0.0, 0.0)));
    assert_eq!(serious::K_ABS_UP, (Vector3::<f32>::new(0.0, 1.0, 0.0)));
    // right x up points backwards in a right-handed system that looks down -Z
    expect_vector_near(serious::K_ABS_RIGHT.cross(serious::K_ABS_UP), -serious::K_ABS_FORWARD, 1e-6);
}

#[test]
fn positive_heading_turns_left() {
    let angles = make_angles(0.0, 90.0, 0.0);

    expect_vector_near(serious::forward_vector(angles), Vector3::new(-1.0, 0.0, 0.0), 1e-5);
    expect_vector_near(serious::right_vector(angles), Vector3::new(0.0, 0.0, -1.0), 1e-5);
    expect_vector_near(serious::up_vector(angles), Vector3::new(0.0, 1.0, 0.0), 1e-5);
}

#[test]
fn positive_pitch_looks_up() {
    let forward = serious::forward_vector(make_angles(30.0, 0.0, 0.0));

    expect_near!(forward.y, 0.5, 1e-5);
    expect_near!(forward.z, -(K_PI / 6.0).cos(), 1e-5);
}

#[test]
fn matches_axes_measured_in_the_game_sample1() {
    // yaw 14.8, pitch -2.5
    let angles = make_angles(-2.5, 14.8, 0.0);

    expect_vector_near(serious::forward_vector(angles), Vector3::new(-0.254, -0.044, -0.966), 2e-3);
    expect_vector_near(serious::right_vector(angles), Vector3::new(0.967, -0.002, -0.254), 2e-3);
    expect_vector_near(serious::up_vector(angles), Vector3::new(-0.010, 0.999, -0.044), 2e-3);
}

#[test]
fn matches_axes_measured_in_the_game_sample2() {
    // yaw 90.3, pitch -0.7: looking almost exactly down -X
    let angles = make_angles(-0.7, 90.3, 0.0);

    expect_vector_near(serious::forward_vector(angles), Vector3::new(-1.000, -0.012, 0.005), 2e-3);
    expect_vector_near(serious::right_vector(angles), Vector3::new(-0.004, 0.000, -1.000), 2e-3);
    expect_vector_near(serious::up_vector(angles), Vector3::new(-0.012, 1.000, 0.000), 2e-3);
}

#[test]
fn rotation_matrix_matches_the_engines_make_rotation_matrix() {
    let samples: [(f32, f32, f32, [[f32; 3]; 3]); 3] = [
        (
            30.0,
            20.0,
            10.0,
            [[0.882564, 0.018028, 0.469846], [0.163176, 0.925417, -0.342020], [-0.440970, 0.378522, 0.813798]],
        ),
        (
            -120.0,
            -45.0,
            75.0,
            [[0.462097, 0.641457, -0.612372], [0.683013, 0.183013, 0.707107], [0.565650, -0.745010, -0.353553]],
        ),
        (
            170.0,
            60.0,
            -140.0,
            [[0.657742, -0.748223, 0.086824], [-0.321394, -0.383022, -0.866025], [0.681236, 0.541716, -0.492404]],
        ),
    ];

    for (heading, pitch, banking, expected) in samples {
        let actual = serious::rotation_matrix(make_angles(pitch, heading, banking));

        for row in 0..3 {
            for col in 0..3 {
                expect_near!(actual.at(row, col), expected[row][col], 1e-5);
            }
        }
    }
}

#[test]
fn rotation_matrix_is_heading_then_pitch_then_banking() {
    // The same product written with omath's axis rotations: Ry(heading) * Rx(pitch) * Rz(banking).
    for pitch in [-80.0, -30.0, 0.0, 25.0, 70.0] {
        for yaw in [-170.0, -60.0, 0.0, 45.0, 135.0] {
            for roll in [-120.0, -15.0, 0.0, 30.0, 150.0] {
                let angles = make_angles(pitch, yaw, roll);
                let expected = serious::Mat4X4::rotation_axis_y(angles.yaw)
                    * serious::Mat4X4::rotation_axis_x(angles.pitch)
                    * serious::Mat4X4::rotation_axis_z(angles.roll);
                let actual = serious::rotation_matrix(angles);

                for row in 0..4 {
                    for col in 0..4 {
                        expect_near!(actual.at(row, col), expected.at(row, col), 1e-5);
                    }
                }
            }
        }
    }
}

#[test]
fn roll_rotates_around_the_forward_axis() {
    let angles = make_angles(40.0, 25.0, 33.0);
    let level = make_angles(40.0, 25.0, 0.0);

    // banking must not move where the camera looks, only turn right/up around it
    expect_vector_near(serious::forward_vector(angles), serious::forward_vector(level), 1e-5);
    expect_near!(serious::right_vector(angles).dot(serious::right_vector(level)), (33.0 * K_PI / 180.0).cos(), 1e-5);
}

#[test]
fn extract_rotation_angles_round_trip() {
    for pitch in [-80.0, -30.0, 0.0, 25.0, 70.0] {
        for yaw in [-170.0, -60.0, 0.0, 45.0, 135.0] {
            for roll in [-120.0, -15.0, 0.0, 30.0, 150.0] {
                let angles = make_angles(pitch, yaw, roll);
                let extracted = serious::extract_rotation_angles(&serious::rotation_matrix(angles));

                expect_near!(extracted.pitch.as_degrees(), pitch, 1e-3);
                expect_near!(angle_difference(extracted.yaw.as_degrees(), yaw), 0.0, 1e-3);
                expect_near!(angle_difference(extracted.roll.as_degrees(), roll), 0.0, 1e-3);
            }
        }
    }
}

#[test]
fn extract_rotation_angles_at_straight_up_and_down_keeps_the_direction() {
    // heading and banking are the same rotation at +/-90 pitch, so only the resulting orientation is recoverable
    for pitch in [-90.0, 90.0] {
        let angles = make_angles(pitch, 40.0, 0.0);
        let extracted = serious::extract_rotation_angles(&serious::rotation_matrix(angles));

        expect_near!(extracted.pitch.as_degrees(), pitch, 1e-3);
        expect_vector_near(serious::forward_vector(extracted), serious::forward_vector(angles), 1e-4);
        expect_vector_near(serious::right_vector(extracted), serious::right_vector(angles), 1e-4);
    }
}

#[test]
fn extract_origin_and_scale() {
    let mut mat = serious::rotation_matrix(make_angles(10.0, 20.0, 5.0));
    *mat.at_mut(0, 3) = 11.0;
    *mat.at_mut(1, 3) = -22.0;
    *mat.at_mut(2, 3) = 33.0;

    expect_vector_near(serious::extract_origin(&mat), Vector3::new(11.0, -22.0, 33.0), 1e-5);
    expect_vector_near(serious::extract_scale(&mat), Vector3::new(1.0, 1.0, 1.0), 1e-5);
}

#[test]
fn view_matrix_matches_the_engines_own() {
    let view = serious::calc_view_matrix(make_angles(K_GAME_PITCH, K_GAME_HEADING, 0.0), K_GAME_EYE);

    let expected: [[f32; 4]; 3] =
        [[-0.8148, 0.0, -0.5797, -315.4465], [-0.1466, 0.9675, 0.2061, 125.9520], [0.5609, 0.2529, -0.7883, -394.7441]];
    // the captured entries are printed with 4 decimals
    for row in 0..3 {
        for col in 0..3 {
            expect_near!(view.at(row, col), expected[row][col], 2e-4);
        }
    }

    // the translations are ~500 units, so float rounding alone is ~0.01
    for row in 0..3 {
        expect_near!(view.at(row, 3), expected[row][3], 0.05);
    }
    expect_float_eq!(view.at(3, 3), 1.0);
}

#[test]
fn view_matrix_maps_the_eye_to_the_origin() {
    let angles = make_angles(23.0, -71.0, 12.0);
    let eye: Vector3<f32> = Vector3::new(120.0, -34.0, 560.0);
    let view = serious::calc_view_matrix(angles, eye);

    let in_view_space = |point: Vector3<f32>| {
        return Vector3::<f32>::new(
            view.at(0, 0) * point.x + view.at(0, 1) * point.y + view.at(0, 2) * point.z + view.at(0, 3),
            view.at(1, 0) * point.x + view.at(1, 1) * point.y + view.at(1, 2) * point.z + view.at(1, 3),
            view.at(2, 0) * point.x + view.at(2, 1) * point.y + view.at(2, 2) * point.z + view.at(2, 3),
        );
    };

    expect_vector_near(in_view_space(eye), Vector3::new(0.0, 0.0, 0.0), 1e-3);
    // one unit in front of the eye is one unit down the view-space -Z axis
    expect_vector_near(in_view_space(eye + serious::forward_vector(angles)), Vector3::new(0.0, 0.0, -1.0), 1e-4);
    expect_vector_near(in_view_space(eye + serious::right_vector(angles)), Vector3::new(1.0, 0.0, 0.0), 1e-4);
    expect_vector_near(in_view_space(eye + serious::up_vector(angles)), Vector3::new(0.0, 1.0, 0.0), 1e-4);
}

#[test]
fn camera_recovers_origin_and_angles_from_its_view_matrix() {
    let angles = make_angles(-12.0, 118.0, 0.0);
    let eye: Vector3<f32> = Vector3::new(-300.0, 75.0, 410.0);
    let view = serious::calc_view_matrix(angles, eye);

    expect_vector_near(serious::Camera::calc_origin_from_view_matrix(&view), eye, 1e-3);

    let recovered = serious::Camera::calc_view_angles_from_view_matrix(&view);
    expect_near!(recovered.pitch.as_degrees(), -12.0, 1e-3);
    expect_near!(angle_difference(recovered.yaw.as_degrees(), 118.0), 0.0, 1e-3);
}

#[test]
fn projection_uses_horizontal_fov() {
    let proj = serious::calc_perspective_projection_matrix(90.0, 1.25, 0.2, 1000.0, serious::K_DEFAULT_NDC_DEPTH_RANGE);

    // x scale is 1 / tan(hfov / 2), y scale follows from the aspect ratio - the same ratio the engine's own matrix has
    expect_near!(proj.at(0, 0), 1.0, 1e-5);
    expect_near!(proj.at(1, 1), 1.25, 1e-5);
    expect_near!(proj.at(1, 1) / proj.at(0, 0), 1.25, 1e-5);
    // clip.w = -z_view
    expect_float_eq!(proj.at(3, 2), -1.0);
    expect_float_eq!(proj.at(3, 3), 0.0);
}

#[test]
fn projection_fov_stays_horizontal_when_the_aspect_changes() {
    let narrow =
        serious::calc_perspective_projection_matrix(75.0, 4.0 / 3.0, 0.1, 500.0, serious::K_DEFAULT_NDC_DEPTH_RANGE);
    let wide =
        serious::calc_perspective_projection_matrix(75.0, 16.0 / 9.0, 0.1, 500.0, serious::K_DEFAULT_NDC_DEPTH_RANGE);

    expect_near!(narrow.at(0, 0), wide.at(0, 0), 1e-6);
    assert!(wide.at(1, 1) > narrow.at(1, 1));
}

#[test]
fn depth_range_zero_to_one_maps_near_to_zero_and_far_to_one() {
    let near_plane: f32 = 0.2;
    let far_plane: f32 = 1000.0;
    let proj = serious::calc_perspective_projection_matrix(90.0, 1.0, near_plane, far_plane, NdcDepthRange::ZeroToOne);
    let ndc_depth = |view_z: f32| {
        return (proj.at(2, 2) * view_z + proj.at(2, 3)) / (proj.at(3, 2) * view_z);
    };

    expect_near!(ndc_depth(-near_plane), 0.0, 1e-5);
    expect_near!(ndc_depth(-far_plane), 1.0, 1e-5);
}

#[test]
fn depth_range_negative_one_to_one_maps_near_to_minus_one_and_far_to_one() {
    let near_plane: f32 = 0.2;
    let far_plane: f32 = 1000.0;
    let proj =
        serious::calc_perspective_projection_matrix(90.0, 1.0, near_plane, far_plane, NdcDepthRange::NegativeOneToOne);
    let ndc_depth = |view_z: f32| {
        return (proj.at(2, 2) * view_z + proj.at(2, 3)) / (proj.at(3, 2) * view_z);
    };

    expect_near!(ndc_depth(-near_plane), -1.0, 1e-4);
    expect_near!(ndc_depth(-far_plane), 1.0, 1e-4);
}

#[test]
fn world_to_screen_center_for_point_straight_ahead() {
    let cam = serious::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1280.0, 1024.0),
        projection::FieldOfView::from_degrees(90.0),
        0.2,
        1000.0,
    );

    let projected = cam.world_to_screen(Vector3::new(0.0, 0.0, -10.0));

    assert!(projected.is_ok());
    expect_near!(projected.unwrap().x, 640.0, 1e-3);
    expect_near!(projected.unwrap().y, 512.0, 1e-3);
}

#[test]
fn world_to_screen_half_fov_lands_on_the_screen_edge() {
    let cam = serious::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1280.0, 1024.0),
        projection::FieldOfView::from_degrees(90.0),
        0.2,
        1000.0,
    );

    // 90 degrees horizontal fov: 10 units right at 10 units ahead is exactly the right edge, on the middle row
    let right_edge = cam.world_to_screen_unclipped(Vector3::new(10.0, 0.0, -10.0));
    assert!(right_edge.is_ok());
    expect_near!(right_edge.unwrap().x, 1280.0, 1e-2);
    expect_near!(right_edge.unwrap().y, 512.0, 1e-2);

    // the vertical half-extent follows the aspect ratio: 10 / 1.25 = 8 units up reaches the top edge
    let top_edge = cam.world_to_screen_unclipped(Vector3::new(0.0, 8.0, -10.0));
    assert!(top_edge.is_ok());
    expect_near!(top_edge.unwrap().x, 640.0, 1e-2);
    expect_near!(top_edge.unwrap().y, 0.0, 1e-2);
}

#[test]
fn world_to_screen_right_is_right_and_up_is_up() {
    let cam = serious::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1280.0, 1024.0),
        projection::FieldOfView::from_degrees(90.0),
        0.2,
        1000.0,
    );

    let right = cam.world_to_screen(Vector3::new(3.0, 0.0, -10.0));
    let up = cam.world_to_screen(Vector3::new(0.0, 3.0, -10.0));

    assert!(right.is_ok());
    assert!(up.is_ok());
    assert!(right.unwrap().x > 640.0);
    assert!(up.unwrap().y < 512.0); // screen Y grows downwards from the top-left corner
}

#[test]
fn world_to_screen_rejects_points_behind_the_camera() {
    let cam = serious::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1280.0, 1024.0),
        projection::FieldOfView::from_degrees(90.0),
        0.2,
        1000.0,
    );

    assert!(!(cam.world_to_screen(Vector3::new(0.0, 0.0, 10.0)).is_ok()));
}

#[test]
fn world_to_screen_follows_heading() {
    // heading +90 turns left, so the camera looks down -X and a point there is dead centre
    let cam = serious::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        make_angles(0.0, 90.0, 0.0),
        ViewPort::new(1280.0, 1024.0),
        projection::FieldOfView::from_degrees(90.0),
        0.2,
        1000.0,
    );

    let ahead = cam.world_to_screen(Vector3::new(-10.0, 0.0, 0.0));
    assert!(ahead.is_ok());
    expect_near!(ahead.unwrap().x, 640.0, 1e-2);
    expect_near!(ahead.unwrap().y, 512.0, 1e-2);

    // the old forward direction (-Z) is now on the camera's right (a point exactly 90 degrees off-axis has no depth)
    let to_the_right = cam.world_to_screen(Vector3::new(-10.0, 0.0, -5.0));
    assert!(to_the_right.is_ok());
    assert!(to_the_right.unwrap().x > 640.0);
}

#[test]
fn world_to_screen_matches_the_engines_own_matrices() {
    // Same capture as ViewMatrixMatchesTheEnginesOwn, with a 5:4 viewport and projection scales sx = 0.9959,
    // sy = 1.2448. Pushing the world origin through the engine's own view rows gives:
    //   view = (-315.4465 * sx, 125.9520 * sy, w = 394.7441)  ->  ndc = (-0.79584, 0.39718)
    //   screen (1280 x 1024, origin top-left) = (130.66, 308.64)
    let hfov: f32 = 2.0 * (1.0_f32 / 0.9959).atan() * 180.0 / K_PI;
    let cam = serious::Camera::new(
        K_GAME_EYE,
        make_angles(K_GAME_PITCH, K_GAME_HEADING, 0.0),
        ViewPort::new(1280.0, 1024.0),
        projection::FieldOfView::from_degrees(hfov),
        0.2,
        100000.0,
    );

    let projected = cam.world_to_screen(Vector3::new(0.0, 0.0, 0.0));

    assert!(projected.is_ok());
    expect_near!(projected.unwrap().x, 130.66, 0.5);
    expect_near!(projected.unwrap().y, 308.64, 0.5);
}

#[test]
fn look_at_forward() {
    let angles = serious::CameraTrait::calc_look_at_angle(Default::default(), serious::K_ABS_FORWARD);

    expect_vector_near(serious::forward_vector(angles), serious::K_ABS_FORWARD, 1e-4);
}

#[test]
fn look_at_right() {
    let angles = serious::CameraTrait::calc_look_at_angle(Default::default(), serious::K_ABS_RIGHT);

    expect_vector_near(serious::forward_vector(angles), serious::K_ABS_RIGHT, 1e-4);
}

#[test]
fn look_at_up() {
    let angles = serious::CameraTrait::calc_look_at_angle(Default::default(), serious::K_ABS_UP);

    expect_vector_near(serious::forward_vector(angles), serious::K_ABS_UP, 1e-4);
}

#[test]
fn look_at_arbitrary_directions_points_the_forward_vector_there() {
    for target in [
        Vector3::<f32>::new(30.0, 10.0, -40.0),
        Vector3::<f32>::new(-25.0, -15.0, 5.0),
        Vector3::<f32>::new(-3.0, 20.0, 18.0),
        Vector3::<f32>::new(44.0, -7.0, 12.0),
    ] {
        let angles = serious::CameraTrait::calc_look_at_angle(Default::default(), target);

        expect_vector_near(serious::forward_vector(angles), target.normalized(), 1e-4);
    }
}

#[test]
fn look_at_puts_the_target_in_the_middle_of_the_screen() {
    let mut cam = serious::Camera::new(
        Vector3::new(10.0, 5.0, -20.0),
        Default::default(),
        ViewPort::new(1280.0, 1024.0),
        projection::FieldOfView::from_degrees(90.0),
        0.2,
        1000.0,
    );
    let target: Vector3<f32> = Vector3::new(-40.0, 30.0, -90.0);
    cam.look_at(target);

    let projected = cam.world_to_screen(target);
    assert!(projected.is_ok());
    expect_near!(projected.unwrap().x, 640.0, 1e-2);
    expect_near!(projected.unwrap().y, 512.0, 1e-2);
}

#[test]
fn mesh_trait_forwards_rotation_matrix() {
    let angles = make_angles(20.0, -35.0, 15.0);
    let from_trait = serious::MeshTrait::rotation_matrix(&angles);
    let direct = serious::rotation_matrix(angles);

    // Not EXPECT_EQ: the compiler may fold one call at compile time and run the other through libm (or contract
    // multiply-adds differently), so the last bit can differ between the two even though the math is the same.
    for row in 0..4 {
        for col in 0..4 {
            expect_near!(from_trait.at(row, col), direct.at(row, col), 1e-6);
        }
    }
}

#[test]
fn one_unit_is_one_meter() {
    expect_float_eq!(serious::units_to_meters(1.0), 1.0);
    expect_float_eq!(serious::units_to_centimeters(1.0), 100.0);
    expect_double_eq!(serious::units_to_kilometers(2500.0), 2.5);

    expect_float_eq!(serious::meters_to_units(1.0), 1.0);
    expect_float_eq!(serious::centimeters_to_units(250.0), 2.5);
    expect_double_eq!(serious::kilometers_to_units(2.5), 2500.0);
}

#[test]
fn unit_conversions_round_trip() {
    let units: f64 = -123456.789;

    expect_near!(serious::centimeters_to_units(serious::units_to_centimeters(units)), units, 1e-6);
    expect_near!(serious::meters_to_units(serious::units_to_meters(units)), units, 1e-9);
    expect_near!(serious::kilometers_to_units(serious::units_to_kilometers(units)), units, 1e-6);
}

#[test]
fn unit_conversions_are_constexpr_and_keep_their_type() {
    assert!(serious::units_to_centimeters(1.0) == 100.0);
    let _: f32 = serious::units_to_meters(1.0);
    let _: f64 = serious::units_to_meters(1.0);
}
