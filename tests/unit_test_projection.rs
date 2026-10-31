#[macro_use]
mod projection_support;

use omath::engines::{cry, frostbite, iw, opengl, rage, serious, source, unity, unreal};
use omath::primitives::Aabb;
use omath::projection::{self, CameraEngine, Error, FieldOfView, ScreenStart, ViewPort};
use omath::{Mat, NdcDepthRange, RowMajor, Triangle, Vector2, Vector3};
use projection_support::Rng;

use omath::projection::{Camera, Mat4X4Of, ViewAnglesOf};
use omath::scalar::Scalar;
use omath::{Angle, Real};
use projection_support::*;
use std::marker::PhantomData;

fn random_visible_component<T: Real>(rng: &mut Rng, dist: (T, T)) -> T {
    let value = rng.uniform(dist);
    let fifty = T::from_f64(50.0);
    if value.abs_value() < fifty {
        return if value < T::ZERO { value - fifty } else { value + fifty };
    }
    value
}

fn verify_random_look_at_targets_project_to_screen_center<E: CameraEngine>(
    origin: Vector3<E::Scalar>,
    view_angles: ViewAnglesOf<E>,
    view_port: ViewPort,
    near_plane: E::Scalar,
    far_plane: E::Scalar,
    seed: u64,
) {
    let mut rng = Rng::new(seed);
    let dist = (E::Scalar::from_f64(-500.0), E::Scalar::from_f64(500.0));

    let fov = FieldOfView::from_degrees(75.0);
    let mut cam = Camera::<E>::new(origin, view_angles, view_port, fov, near_plane, far_plane);
    let screen_center_x = f64::from(view_port.width / 2.0);
    let screen_center_y = f64::from(view_port.height / 2.0);

    for _ in 0..100 {
        let target = origin
            + Vector3::new(
                random_visible_component(&mut rng, dist),
                random_visible_component(&mut rng, dist),
                random_visible_component(&mut rng, dist),
            );

        cam.look_at(target);

        let projected = cam.world_to_screen(target).unwrap();
        expect_near!(projected.x.to_f64(), screen_center_x, 0.1);
        expect_near!(projected.y.to_f64(), screen_center_y, 0.1);
    }
}

fn make_camera<E: CameraEngine>(angles: ViewAnglesOf<E>, fov: FieldOfView) -> Camera<E> {
    Camera::new(
        Default::default(),
        angles,
        ViewPort::new(1280.0, 1024.0),
        fov,
        E::Scalar::from_f64(0.1),
        E::Scalar::from_f64(1000.0),
    )
}

/// The FOV `extract_projection_params` returns is the kind the camera was built with - horizontal for Unreal and Serious,
/// horizontal at 4:3 for Source/IW, vertical elsewhere - so feeding it back reproduces the matrix.
fn expect_projection_params_round_trip<E: CameraEngine>(fov_degrees: f32) {
    let cam = make_camera::<E>(Default::default(), FieldOfView::from_degrees(fov_degrees));
    let params = Camera::<E>::extract_projection_params(&cam.get_projection_matrix());

    expect_near!(params.fov.as_degrees(), fov_degrees, 1e-3_f32);
    expect_near!(params.aspect_ratio.to_f64(), 1280.0_f32 / 1024.0, 1e-4_f32);
}

/// Rebuilding the view matrix from the recovered angles must give the original matrix back, and for angles inside the
/// engine's ranges (away from the +/-90 pitch singularity) the angles themselves come back too.
fn expect_view_angles_round_trip<E: CameraEngine>(angles: ViewAnglesOf<E>) {
    let fov = FieldOfView::from_degrees(90.0);
    let cam = make_camera::<E>(angles, fov);
    let view = cam.get_view_matrix();

    let recovered = Camera::<E>::calc_view_angles_from_view_matrix(&view);

    let rebuilt = make_camera::<E>(recovered, fov).get_view_matrix();

    let angle_difference = |a: E::Scalar, b: E::Scalar| remainder(a.to_f64() - b.to_f64(), 360.0);
    expect_near!(angle_difference(recovered.pitch.as_degrees(), angles.pitch.as_degrees()), 0.0, 1e-2);
    expect_near!(angle_difference(recovered.yaw.as_degrees(), angles.yaw.as_degrees()), 0.0, 1e-2);
    expect_near!(angle_difference(recovered.roll.as_degrees(), angles.roll.as_degrees()), 0.0, 1e-2);

    for row in 0..3 {
        for col in 0..3 {
            expect_near!(rebuilt.at(row, col).to_f64(), view.at(row, col).to_f64(), 1e-4_f32);
        }
    }
}

fn expect_view_angles_round_trip_grid<E: CameraEngine>() {
    let degrees = |value: f32| E::Scalar::from_f64(f64::from(value));

    for pitch in [-70.0, -20.0, 0.0, 35.0, 80.0] {
        for yaw in [-170.0, -90.0, -40.0, 0.0, 90.0, 100.0] {
            for roll in [-150.0, -35.0, 0.0, 20.0, 120.0] {
                expect_view_angles_round_trip::<E>(ViewAnglesOf::<E>::new(
                    Angle::from_degrees(degrees(pitch)),
                    Angle::from_degrees(degrees(yaw)),
                    Angle::from_degrees(degrees(roll)),
                ));
            }
        }
    }
}

/// An engine with `extract_rotation_angles` taken away, to exercise the camera's look-at fallback.
struct WithoutRotationExtraction<E>(PhantomData<E>);

impl<E: CameraEngine> CameraEngine for WithoutRotationExtraction<E> {
    type Scalar = E::Scalar;
    type Order = E::Order;
    type PitchRange = E::PitchRange;
    type YawRange = E::YawRange;
    type RollRange = E::RollRange;

    const DEPTH_RANGE: NdcDepthRange = E::DEPTH_RANGE;
    const AXES: projection::CameraAxes = E::AXES;

    fn calc_look_at_angle(cam_origin: Vector3<E::Scalar>, look_at: Vector3<E::Scalar>) -> ViewAnglesOf<E> {
        E::calc_look_at_angle(cam_origin, look_at)
    }

    fn calc_view_matrix(angles: ViewAnglesOf<E>, cam_origin: Vector3<E::Scalar>) -> Mat4X4Of<E> {
        E::calc_view_matrix(angles, cam_origin)
    }

    fn calc_projection_matrix(
        fov: FieldOfView,
        view_port: ViewPort,
        near_plane: E::Scalar,
        far_plane: E::Scalar,
        ndc_depth_range: NdcDepthRange,
    ) -> Mat4X4Of<E> {
        E::calc_projection_matrix(fov, view_port, near_plane, far_plane, ndc_depth_range)
    }
}

/// A custom engine that rolls around a world axis (roll applied last, Rz(roll) * Ry(yaw) * Rx(pitch)).
struct WorldAxisRollTrait;

impl CameraEngine for WorldAxisRollTrait {
    type Scalar = f32;
    type Order = RowMajor;
    type PitchRange = omath::engines::Clamped90;
    type YawRange = omath::engines::Normalized180;
    type RollRange = omath::engines::Normalized180;

    const DEPTH_RANGE: NdcDepthRange = NdcDepthRange::ZeroToOne;
    const AXES: projection::CameraAxes = projection::CameraAxes { inverted_forward: true, inverted_right: false };

    fn calc_look_at_angle(cam_origin: Vector3<f32>, look_at: Vector3<f32>) -> unity::ViewAngles {
        <unity::CameraTrait as CameraEngine>::calc_look_at_angle(cam_origin, look_at)
    }

    fn calc_view_matrix(angles: unity::ViewAngles, cam_origin: Vector3<f32>) -> unity::Mat4X4 {
        let rotation = unity::Mat4X4::rotation_zyx(angles.roll, angles.yaw, angles.pitch);
        Mat::camera_view(
            -rotation.rotate_vector(unity::K_ABS_FORWARD),
            rotation.rotate_vector(unity::K_ABS_RIGHT),
            rotation.rotate_vector(unity::K_ABS_UP),
            cam_origin,
        )
    }

    fn calc_projection_matrix(
        fov: FieldOfView,
        view_port: ViewPort,
        near_plane: f32,
        far_plane: f32,
        ndc_depth_range: NdcDepthRange,
    ) -> unity::Mat4X4 {
        <unity::CameraTrait as CameraEngine>::calc_projection_matrix(
            fov,
            view_port,
            near_plane,
            far_plane,
            ndc_depth_range,
        )
    }
}

/// Roll turns the camera around its own forward axis: it must not change where the camera looks.
fn expect_roll_keeps_forward<E: CameraEngine>() {
    let degrees = |value: f32| E::Scalar::from_f64(f64::from(value));

    for roll in [-120.0, -30.0, 45.0, 160.0] {
        let make = |roll_degrees: f32| {
            make_camera::<E>(
                ViewAnglesOf::<E>::new(
                    Angle::from_degrees(degrees(35.0)),
                    Angle::from_degrees(degrees(-60.0)),
                    Angle::from_degrees(degrees(roll_degrees)),
                ),
                FieldOfView::from_degrees(90.0),
            )
        };

        let level_forward = make(0.0).get_forward();
        let rolled_forward = make(roll).get_forward();
        expect_near!(rolled_forward.dot(level_forward).to_f64(), 1.0_f32, 1e-5_f32);
    }
}

#[test]
fn projection() {
    let fov = FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let projected = cam.world_to_screen(Vector3::new(1000.0, 0.0, 50.0));
    let result = cam.screen_to_world(projected.unwrap());
    let result2 = cam.world_to_screen(result.unwrap());

    assert_eq!(projected.unwrap().xy(), result2.unwrap().xy());
    expect_near!(projected.unwrap().x, 960.0, 0.001);
    expect_near!(projected.unwrap().y, 504.0, 0.001);
    expect_near!(projected.unwrap().z, 1.0, 0.001);
}

#[test]
fn screen_to_ndc_top_left() {
    let fov = FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let ndc_top_left = cam.screen_to_ndc(Vector3::new(1500.0, 300.0, 1.0), ScreenStart::TopLeftCorner);
    expect_near!(ndc_top_left.x, 0.5625, 0.0001);
    expect_near!(ndc_top_left.y, 0.4444, 0.0001);
}

#[test]
fn screen_to_ndc_bottom_left() {
    let fov = projection::FieldOfView::from_degrees(60.0);

    let cam = unity::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1280.0, 720.0),
        fov,
        0.03,
        1000.0,
    );

    let ndc_bottom_left =
        cam.screen_to_ndc(Vector3::new(1263.53833, 547.061523, 0.99405992), ScreenStart::BottomLeftCorner);
    expect_near!(ndc_bottom_left.x, 0.974278628, 0.0001);
    expect_near!(ndc_bottom_left.y, 0.519615293, 0.0001);
}

#[test]
fn unclipped_world_to_screen_in_bounds() {
    let fov = FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let projected = cam.world_to_screen_unclipped(Vector3::new(1000.0, 0.0, 50.0));
    assert!(projected.is_ok());
    expect_near!(projected.unwrap().x, 960.0, 0.001);
    expect_near!(projected.unwrap().y, 504.0, 0.001);
}

#[test]
fn unclipped_world_to_screen_matches_world_to_screen_when_in_bounds() {
    let fov = FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let w2s = cam.world_to_screen(Vector3::new(1000.0, 0.0, 50.0));
    let no_clip = cam.world_to_screen_unclipped(Vector3::new(1000.0, 0.0, 50.0));

    assert!(w2s.is_ok());
    assert!(no_clip.is_ok());
    expect_near!(w2s.unwrap().x, no_clip.unwrap().x, 0.001);
    expect_near!(w2s.unwrap().y, no_clip.unwrap().y, 0.001);
    expect_near!(w2s.unwrap().z, no_clip.unwrap().z, 0.001);
}

#[test]
fn unclipped_world_to_screen_rejects_behind_camera() {
    let fov = FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let projected = cam.world_to_screen_unclipped(Vector3::new(-1000.0, 0.0, 0.0));
    assert!(!(projected.is_ok()));
    assert_eq!(projected.unwrap_err(), Error::PerspectiveDividerLessEqZero);
}

#[test]
fn unclipped_world_to_screen_allows_out_of_bounds_ndc() {
    let fov = FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    // Point far to the side exceeds NDC [-1,1] bounds but unclipped returns it anyway
    let projected = cam.world_to_screen_unclipped(Vector3::new(100.0, 5000.0, 0.0));
    assert!(projected.is_ok());
}

#[test]
fn world_to_screen_rejects_out_of_bounds_ndc() {
    let fov = FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    // Same point that unclipped allows — clipped world_to_screen rejects it
    let projected = cam.world_to_screen(Vector3::new(100.0, 5000.0, 0.0));
    assert!(!(projected.is_ok()));
}

#[test]
fn unclipped_world_to_screen_bottom_left_corner() {
    let fov = FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let top_left = cam.world_to_screen_unclipped_with(Vector3::new(1000.0, 0.0, 50.0), ScreenStart::TopLeftCorner);
    let bottom_left =
        cam.world_to_screen_unclipped_with(Vector3::new(1000.0, 0.0, 50.0), ScreenStart::BottomLeftCorner);

    assert!(top_left.is_ok());
    assert!(bottom_left.is_ok());
    // X should be identical, Y should differ (mirrored around center)
    expect_near!(top_left.unwrap().x, bottom_left.unwrap().x, 0.001);
    expect_near!(top_left.unwrap().y + bottom_left.unwrap().y, 1080.0, 0.001);
}

#[test]
fn unclipped_world_to_screen_round_trip() {
    let mut rng = Rng::new(42);
    let dist_fwd = (100.0, 900.0);
    let dist_side = (-400.0, 400.0);
    let dist_up = (-200.0, 200.0);

    let fov = FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    for _i in 0..100 {
        let world_pos: Vector3<f32> = Vector3::new(rng.uniform(dist_fwd), rng.uniform(dist_side), rng.uniform(dist_up));
        let screen = cam.world_to_screen_unclipped(world_pos);
        if !screen.is_ok() {
            continue;
        }

        let back_to_world = cam.screen_to_world(screen.unwrap());
        assert!(back_to_world.is_ok());

        let back_to_screen = cam.world_to_screen_unclipped(back_to_world.unwrap());
        assert!(back_to_screen.is_ok());

        expect_near!(screen.unwrap().x, back_to_screen.unwrap().x, 0.01);
        expect_near!(screen.unwrap().y, back_to_screen.unwrap().y, 0.01);
    }
}

#[test]
fn unclipped_world_to_screen_unity_engine() {
    let fov = projection::FieldOfView::from_degrees(60.0);
    let cam = unity::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1280.0, 720.0),
        fov,
        0.03,
        1000.0,
    );

    // Point directly in front
    let projected = cam.world_to_screen_unclipped_with(Vector3::new(0.0, 0.0, 500.0), ScreenStart::BottomLeftCorner);
    assert!(projected.is_ok());
    expect_near!(projected.unwrap().x, 640.0, 0.5);
    expect_near!(projected.unwrap().y, 360.0, 0.5);
}

#[test]
fn screen_to_world_top_left_corner() {
    let mut rng = Rng::new(0x5EED);

    let dist_x = (1.0, 1900.0);
    let dist_y = (1.0, 1070.0);

    let fov = FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    for _i in 0..100 {
        let initial_screen_cords = Vector2::new(rng.uniform(dist_x), rng.uniform(dist_y));

        let world_cords = cam.screen_to_world_2d_with(initial_screen_cords, ScreenStart::TopLeftCorner);
        let screen_cords = cam.world_to_screen_with(world_cords.unwrap(), ScreenStart::TopLeftCorner);

        expect_near!(screen_cords.unwrap().x, initial_screen_cords.x, 0.001);
        expect_near!(screen_cords.unwrap().y, initial_screen_cords.y, 0.001);
    }
}

#[test]
fn screen_to_world_bottom_left_corner() {
    let mut rng = Rng::new(0x5EED);

    let dist_x = (1.0, 1900.0);
    let dist_y = (1.0, 1070.0);

    let fov = FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    for _i in 0..100 {
        let initial_screen_cords = Vector2::new(rng.uniform(dist_x), rng.uniform(dist_y));

        let world_cords = cam.screen_to_world_2d_with(initial_screen_cords, ScreenStart::BottomLeftCorner);
        let screen_cords = cam.world_to_screen_with(world_cords.unwrap(), ScreenStart::BottomLeftCorner);

        expect_near!(screen_cords.unwrap().x, initial_screen_cords.x, 0.001);
        expect_near!(screen_cords.unwrap().y, initial_screen_cords.y, 0.001);
    }
}

#[test]
fn aabb_inside_frustum_not_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    // Small box_ directly in front of camera (Source Engine: +X forward, +Y left, +Z up)
    let aabb: Aabb<f32> = Aabb::new(Vector3::new(90.0, -1.0, -1.0), Vector3::new(110.0, 1.0, 1.0));
    assert!(!(cam.is_aabb_culled_by_frustum(&aabb)));
}

#[test]
fn aabb_behind_camera_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    // Box entirely behind the camera
    let aabb: Aabb<f32> = Aabb::new(Vector3::new(-200.0, -1.0, -1.0), Vector3::new(-100.0, 1.0, 1.0));
    assert!(cam.is_aabb_culled_by_frustum(&aabb));
}

#[test]
fn aabb_beyond_far_plane_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    // Box beyond far plane (1000)
    let aabb: Aabb<f32> = Aabb::new(Vector3::new(1500.0, -1.0, -1.0), Vector3::new(2000.0, 1.0, 1.0));
    assert!(cam.is_aabb_culled_by_frustum(&aabb));
}

#[test]
fn aabb_far_left_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    // Box far to the side, outside the frustum
    let aabb: Aabb<f32> = Aabb::new(Vector3::new(90.0, 4000.0, -1.0), Vector3::new(110.0, 5000.0, 1.0));
    assert!(cam.is_aabb_culled_by_frustum(&aabb));
}

#[test]
fn aabb_far_right_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    // Box far to the other side, outside the frustum
    let aabb: Aabb<f32> = Aabb::new(Vector3::new(90.0, -5000.0, -1.0), Vector3::new(110.0, -4000.0, 1.0));
    assert!(cam.is_aabb_culled_by_frustum(&aabb));
}

#[test]
fn aabb_above_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    // Box far above the frustum
    let aabb: Aabb<f32> = Aabb::new(Vector3::new(90.0, -1.0, 5000.0), Vector3::new(110.0, 1.0, 6000.0));
    assert!(cam.is_aabb_culled_by_frustum(&aabb));
}

#[test]
fn aabb_partially_inside_not_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    // Large box_ that straddles the frustum boundary — partially inside
    let aabb: Aabb<f32> = Aabb::new(Vector3::new(50.0, -5000.0, -5000.0), Vector3::new(500.0, 5000.0, 5000.0));
    assert!(!(cam.is_aabb_culled_by_frustum(&aabb)));
}

#[test]
fn aabb_straddles_near_plane_not_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    // Box that straddles the near plane — partially in front
    let aabb: Aabb<f32> = Aabb::new(Vector3::new(-5.0, -1.0, -1.0), Vector3::new(5.0, 1.0, 1.0));
    assert!(!(cam.is_aabb_culled_by_frustum(&aabb)));
}

#[test]
fn aabb_straddles_far_plane_not_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    // Box that straddles the far plane
    let aabb: Aabb<f32> = Aabb::new(Vector3::new(900.0, -1.0, -1.0), Vector3::new(1100.0, 1.0, 1.0));
    assert!(!(cam.is_aabb_culled_by_frustum(&aabb)));
}

#[test]
fn aabb_culled_unity_engine() {
    let fov = projection::FieldOfView::from_degrees(60.0);
    let cam = unity::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1280.0, 720.0),
        fov,
        0.03,
        1000.0,
    );

    // Box in front — not culled
    let inside: Aabb<f32> = Aabb::new(Vector3::new(-1.0, -1.0, 50.0), Vector3::new(1.0, 1.0, 100.0));
    assert!(!(cam.is_aabb_culled_by_frustum(&inside)));

    // Box behind — culled
    let behind: Aabb<f32> = Aabb::new(Vector3::new(-1.0, -1.0, -200.0), Vector3::new(1.0, 1.0, -100.0));
    assert!(cam.is_aabb_culled_by_frustum(&behind));
}

#[test]
fn aabb_below_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    // Box far below the frustum (Source Engine: +Z up)
    let aabb: Aabb<f32> = Aabb::new(Vector3::new(90.0, -1.0, -6000.0), Vector3::new(110.0, 1.0, -5000.0));
    assert!(cam.is_aabb_culled_by_frustum(&aabb));
}

#[test]
fn aabb_encloses_camera_not_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    // Huge box_ that fully encloses the camera
    let aabb: Aabb<f32> = Aabb::new(Vector3::new(-500.0, -500.0, -500.0), Vector3::new(500.0, 500.0, 500.0));
    assert!(!(cam.is_aabb_culled_by_frustum(&aabb)));
}

#[test]
fn aabb_exactly_at_near_plane_not_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    // Box starting exactly at the near plane distance
    let aabb: Aabb<f32> = Aabb::new(Vector3::new(0.01, -1.0, -1.0), Vector3::new(10.0, 1.0, 1.0));
    assert!(!(cam.is_aabb_culled_by_frustum(&aabb)));
}

#[test]
fn aabb_exactly_at_far_plane_not_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    // Box ending exactly at the far plane distance
    let aabb: Aabb<f32> = Aabb::new(Vector3::new(990.0, -1.0, -1.0), Vector3::new(1000.0, 1.0, 1.0));
    assert!(!(cam.is_aabb_culled_by_frustum(&aabb)));
}

#[test]
fn aabb_narrow_fov_culled_at_edge() {
    // Narrow FOV — box_ that would be visible at 90 is culled at 30
    let fov = projection::FieldOfView::from_degrees(30.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let aabb: Aabb<f32> = Aabb::new(Vector3::new(100.0, 200.0, -1.0), Vector3::new(110.0, 210.0, 1.0));
    assert!(cam.is_aabb_culled_by_frustum(&aabb));
}

#[test]
fn aabb_wide_fov_not_culled_at_edge() {
    // Wide FOV — same box_ is visible
    let fov = projection::FieldOfView::from_degrees(120.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let aabb: Aabb<f32> = Aabb::new(Vector3::new(100.0, 200.0, -1.0), Vector3::new(110.0, 210.0, 1.0));
    assert!(!(cam.is_aabb_culled_by_frustum(&aabb)));
}

#[test]
fn aabb_camera_off_origin() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(500.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    // Box in front of shifted camera
    let in_front: Aabb<f32> = Aabb::new(Vector3::new(600.0, -1.0, -1.0), Vector3::new(700.0, 1.0, 1.0));
    assert!(!(cam.is_aabb_culled_by_frustum(&in_front)));

    // Box behind shifted camera
    let behind: Aabb<f32> = Aabb::new(Vector3::new(0.0, -1.0, -1.0), Vector3::new(100.0, 1.0, 1.0));
    assert!(cam.is_aabb_culled_by_frustum(&behind));
}

#[test]
fn aabb_short_far_plane_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    // Very short far plane
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        50.0,
    );

    // Box at distance 100 — beyond the 50-unit far plane
    let aabb: Aabb<f32> = Aabb::new(Vector3::new(90.0, -1.0, -1.0), Vector3::new(110.0, 1.0, 1.0));
    assert!(cam.is_aabb_culled_by_frustum(&aabb));

    // Box at distance 30 — within range
    let near_box: Aabb<f32> = Aabb::new(Vector3::new(25.0, -1.0, -1.0), Vector3::new(35.0, 1.0, 1.0));
    assert!(!(cam.is_aabb_culled_by_frustum(&near_box)));
}

#[test]
fn aabb_point_sized_inside_not_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    // Degenerate zero-volume AABB (a point) inside the frustum
    let aabb: Aabb<f32> = Aabb::new(Vector3::new(100.0, 0.0, 0.0), Vector3::new(100.0, 0.0, 0.0));
    assert!(!(cam.is_aabb_culled_by_frustum(&aabb)));
}

#[test]
fn aabb_opengl_engine_inside_not_culled() {
    // OpenGL: COLUMN_MAJOR, NEGATIVE_ONE_TO_ONE, inverted_z, forward = -Z
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = opengl::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    // Box in front of camera (OpenGL: -Z forward)
    let aabb: Aabb<f32> = Aabb::new(Vector3::new(-1.0, -1.0, -110.0), Vector3::new(1.0, 1.0, -90.0));
    assert!(!(cam.is_aabb_culled_by_frustum(&aabb)));
}

#[test]
fn aabb_opengl_engine_behind_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = opengl::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    // Box behind (OpenGL: +Z is behind the camera)
    let aabb: Aabb<f32> = Aabb::new(Vector3::new(-1.0, -1.0, 100.0), Vector3::new(1.0, 1.0, 200.0));
    assert!(cam.is_aabb_culled_by_frustum(&aabb));
}

#[test]
fn aabb_opengl_engine_beyond_far_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = opengl::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    // Box beyond far plane along -Z
    let aabb: Aabb<f32> = Aabb::new(Vector3::new(-1.0, -1.0, -2000.0), Vector3::new(1.0, 1.0, -1500.0));
    assert!(cam.is_aabb_culled_by_frustum(&aabb));
}

#[test]
fn aabb_opengl_engine_side_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = opengl::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    // Box far to the right (OpenGL: +X right)
    let aabb: Aabb<f32> = Aabb::new(Vector3::new(4000.0, -1.0, -110.0), Vector3::new(5000.0, 1.0, -90.0));
    assert!(cam.is_aabb_culled_by_frustum(&aabb));
}

#[test]
fn aabb_unity_engine_beyond_far_culled() {
    let fov = projection::FieldOfView::from_degrees(60.0);
    let cam = unity::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1280.0, 720.0),
        fov,
        0.03,
        500.0,
    );

    // Box beyond 500-unit far plane (Unity: +Z forward)
    let aabb: Aabb<f32> = Aabb::new(Vector3::new(-1.0, -1.0, 600.0), Vector3::new(1.0, 1.0, 700.0));
    assert!(cam.is_aabb_culled_by_frustum(&aabb));
}

#[test]
fn aabb_unity_engine_side_culled() {
    let fov = projection::FieldOfView::from_degrees(60.0);
    let cam = unity::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1280.0, 720.0),
        fov,
        0.03,
        1000.0,
    );

    // Box far above (Unity: +Y up)
    let aabb: Aabb<f32> = Aabb::new(Vector3::new(-1.0, 5000.0, 50.0), Vector3::new(1.0, 6000.0, 100.0));
    assert!(cam.is_aabb_culled_by_frustum(&aabb));
}

#[test]
fn aabb_unity_engine_straddles_near_not_culled() {
    let fov = projection::FieldOfView::from_degrees(60.0);
    let cam = unity::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1280.0, 720.0),
        fov,
        0.03,
        1000.0,
    );

    // Box straddles near plane (Unity: +Z forward)
    let aabb: Aabb<f32> = Aabb::new(Vector3::new(-1.0, -1.0, -5.0), Vector3::new(1.0, 1.0, 5.0));
    assert!(!(cam.is_aabb_culled_by_frustum(&aabb)));
}

#[test]
fn world_to_view_coordinates_translated_source_camera() {
    let k_eps: f32 = 1e-4;
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(10.0, 20.0, 30.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let view_coordinates = cam.world_to_view_coordinates(Vector3::new(15.0, 12.0, 37.0));

    expect_near!(view_coordinates.x, 8.0, k_eps);
    expect_near!(view_coordinates.y, 7.0, k_eps);
    expect_near!(view_coordinates.z, 5.0, k_eps);
}

#[test]
fn world_to_view_coordinates_rotated_source_camera() {
    let k_eps: f32 = 1e-4;
    let fov = projection::FieldOfView::from_degrees(90.0);
    let angles: source::ViewAngles = source::ViewAngles::new(
        source::PitchAngle::from_degrees(0.0),
        source::YawAngle::from_degrees(90.0),
        source::RollAngle::from_degrees(0.0),
    );
    let cam =
        source::Camera::new(Vector3::new(10.0, 20.0, 30.0), angles, ViewPort::new(1920.0, 1080.0), fov, 0.01, 1000.0);

    let view_coordinates = cam.world_to_view_coordinates(Vector3::new(14.0, 26.0, 38.0));

    expect_near!(view_coordinates.x, 4.0, k_eps);
    expect_near!(view_coordinates.y, 8.0, k_eps);
    expect_near!(view_coordinates.z, 6.0, k_eps);
}

#[test]
fn world_to_view_coordinates_column_major_opengl_camera() {
    let k_eps: f32 = 1e-4;
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = opengl::Camera::new(
        Vector3::new(10.0, 20.0, 30.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let view_coordinates = cam.world_to_view_coordinates(Vector3::new(14.0, 26.0, 22.0));

    expect_near!(view_coordinates.x, 4.0, k_eps);
    expect_near!(view_coordinates.y, 6.0, k_eps);
    expect_near!(view_coordinates.z, -8.0, k_eps);
}

#[test]
fn calc_view_angles_from_view_matrix_looking_forward() {
    let k_eps: f32 = 1e-4;
    let fov = projection::FieldOfView::from_degrees(90.0);
    let angles: source::ViewAngles = source::ViewAngles::new(
        source::PitchAngle::from_degrees(0.0),
        source::YawAngle::from_degrees(0.0),
        source::RollAngle::from_degrees(0.0),
    );
    let cam =
        source::Camera::new(Vector3::new(0.0, 0.0, 0.0), angles, ViewPort::new(1920.0, 1080.0), fov, 0.01, 1000.0);

    let result = source::Camera::calc_view_angles_from_view_matrix(&cam.get_view_matrix());

    expect_near!(result.pitch.as_degrees(), 0.0, k_eps);
    expect_near!(result.yaw.as_degrees(), 0.0, k_eps);
}

#[test]
fn calc_view_angles_from_view_matrix_positive_pitch_and_yaw() {
    let k_eps: f32 = 1e-4;
    let fov = projection::FieldOfView::from_degrees(90.0);
    let angles: source::ViewAngles = source::ViewAngles::new(
        source::PitchAngle::from_degrees(30.0),
        source::YawAngle::from_degrees(45.0),
        source::RollAngle::from_degrees(0.0),
    );
    let cam =
        source::Camera::new(Vector3::new(0.0, 0.0, 0.0), angles, ViewPort::new(1920.0, 1080.0), fov, 0.01, 1000.0);

    let result = source::Camera::calc_view_angles_from_view_matrix(&cam.get_view_matrix());

    expect_near!(result.pitch.as_degrees(), 30.0, k_eps);
    expect_near!(result.yaw.as_degrees(), 45.0, k_eps);
}

#[test]
fn calc_view_angles_from_view_matrix_negative_pitch_and_yaw() {
    let k_eps: f32 = 1e-4;
    let fov = projection::FieldOfView::from_degrees(90.0);
    let angles: source::ViewAngles = source::ViewAngles::new(
        source::PitchAngle::from_degrees(-45.0),
        source::YawAngle::from_degrees(-90.0),
        source::RollAngle::from_degrees(0.0),
    );
    let cam =
        source::Camera::new(Vector3::new(0.0, 0.0, 0.0), angles, ViewPort::new(1920.0, 1080.0), fov, 0.01, 1000.0);

    let result = source::Camera::calc_view_angles_from_view_matrix(&cam.get_view_matrix());

    expect_near!(result.pitch.as_degrees(), -45.0, k_eps);
    expect_near!(result.yaw.as_degrees(), -90.0, k_eps);
}

#[test]
fn calc_view_angles_from_view_matrix_off_origin_camera_ignored() {
    // The forward vector from the view matrix does not depend on camera origin,
    // so the same angles should be recovered regardless of position.
    let k_eps: f32 = 1e-4;
    let fov = projection::FieldOfView::from_degrees(90.0);
    let angles: source::ViewAngles = source::ViewAngles::new(
        source::PitchAngle::from_degrees(20.0),
        source::YawAngle::from_degrees(60.0),
        source::RollAngle::from_degrees(0.0),
    );
    let cam = source::Camera::new(
        Vector3::new(100.0, 200.0, -50.0),
        angles,
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let result = source::Camera::calc_view_angles_from_view_matrix(&cam.get_view_matrix());

    expect_near!(result.pitch.as_degrees(), 20.0, k_eps);
    expect_near!(result.yaw.as_degrees(), 60.0, k_eps);
}

#[test]
fn calc_view_angles_from_view_matrix_recovers_roll() {
    // Roll is not in the forward row, but the up row carries it.
    let fov = projection::FieldOfView::from_degrees(90.0);
    let angles: source::ViewAngles = source::ViewAngles::new(
        source::PitchAngle::from_degrees(10.0),
        source::YawAngle::from_degrees(30.0),
        source::RollAngle::from_degrees(15.0),
    );
    let cam =
        source::Camera::new(Vector3::new(0.0, 0.0, 0.0), angles, ViewPort::new(1920.0, 1080.0), fov, 0.01, 1000.0);

    let result = source::Camera::calc_view_angles_from_view_matrix(&cam.get_view_matrix());

    expect_near!(result.roll.as_degrees(), 15.0, 1e-3);
}

#[test]
fn calc_origin_from_view_matrix_at_origin() {
    let k_eps: f32 = 1e-4;
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let origin = source::Camera::calc_origin_from_view_matrix(&cam.get_view_matrix());

    expect_near!(origin.x, 0.0, k_eps);
    expect_near!(origin.y, 0.0, k_eps);
    expect_near!(origin.z, 0.0, k_eps);
}

#[test]
fn calc_origin_from_view_matrix_arbitrary_position() {
    let k_eps: f32 = 1e-3;
    let fov = projection::FieldOfView::from_degrees(90.0);
    let angles: source::ViewAngles = source::ViewAngles::new(
        source::PitchAngle::from_degrees(0.0),
        source::YawAngle::from_degrees(0.0),
        source::RollAngle::from_degrees(0.0),
    );
    let cam = source::Camera::new(
        Vector3::new(100.0, 200.0, -50.0),
        angles,
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let origin = source::Camera::calc_origin_from_view_matrix(&cam.get_view_matrix());

    expect_near!(origin.x, 100.0, k_eps);
    expect_near!(origin.y, 200.0, k_eps);
    expect_near!(origin.z, -50.0, k_eps);
}

#[test]
fn calc_origin_from_view_matrix_with_rotation() {
    // Origin recovery must work even when the camera is rotated.
    let k_eps: f32 = 1e-3;
    let fov = projection::FieldOfView::from_degrees(90.0);
    let angles: source::ViewAngles = source::ViewAngles::new(
        source::PitchAngle::from_degrees(30.0),
        source::YawAngle::from_degrees(45.0),
        source::RollAngle::from_degrees(0.0),
    );
    let cam = source::Camera::new(
        Vector3::new(300.0, -100.0, 75.0),
        angles,
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let origin = source::Camera::calc_origin_from_view_matrix(&cam.get_view_matrix());

    expect_near!(origin.x, 300.0, k_eps);
    expect_near!(origin.y, -100.0, k_eps);
    expect_near!(origin.z, 75.0, k_eps);
}

#[test]
fn calc_origin_from_view_matrix_independent_of_angles() {
    // Same position, different orientations — should always recover the same origin.
    let k_eps: f32 = 1e-3;
    let fov = projection::FieldOfView::from_degrees(90.0);
    let expected_origin: Vector3<f32> = Vector3::new(50.0, 50.0, 50.0);

    let angles_a: source::ViewAngles = source::ViewAngles::new(
        source::PitchAngle::from_degrees(0.0),
        source::YawAngle::from_degrees(0.0),
        source::RollAngle::from_degrees(0.0),
    );
    let angles_b: source::ViewAngles = source::ViewAngles::new(
        source::PitchAngle::from_degrees(-60.0),
        source::YawAngle::from_degrees(135.0),
        source::RollAngle::from_degrees(0.0),
    );

    let cam_a = source::Camera::new(expected_origin, angles_a, ViewPort::new(1920.0, 1080.0), fov, 0.01, 1000.0);
    let cam_b = source::Camera::new(expected_origin, angles_b, ViewPort::new(1920.0, 1080.0), fov, 0.01, 1000.0);

    let origin_a = source::Camera::calc_origin_from_view_matrix(&cam_a.get_view_matrix());
    let origin_b = source::Camera::calc_origin_from_view_matrix(&cam_b.get_view_matrix());

    expect_near!(origin_a.x, expected_origin.x, k_eps);
    expect_near!(origin_a.y, expected_origin.y, k_eps);
    expect_near!(origin_a.z, expected_origin.z, k_eps);

    expect_near!(origin_b.x, expected_origin.x, k_eps);
    expect_near!(origin_b.y, expected_origin.y, k_eps);
    expect_near!(origin_b.z, expected_origin.z, k_eps);
}

#[test]
fn unity_calc_view_angles_from_view_matrix_looking_forward() {
    let k_eps: f32 = 1e-4;
    let fov = projection::FieldOfView::from_degrees(60.0);
    let angles: unity::ViewAngles = unity::ViewAngles::new(
        unity::PitchAngle::from_degrees(0.0),
        unity::YawAngle::from_degrees(0.0),
        unity::RollAngle::from_degrees(0.0),
    );
    let cam = unity::Camera::new(Vector3::new(0.0, 0.0, 0.0), angles, ViewPort::new(1280.0, 720.0), fov, 0.03, 1000.0);

    let result = unity::Camera::calc_view_angles_from_view_matrix(&cam.get_view_matrix());

    expect_near!(result.pitch.as_degrees(), 0.0, k_eps);
    expect_near!(result.yaw.as_degrees(), 0.0, k_eps);
}

#[test]
fn unity_calc_view_angles_from_view_matrix_positive_pitch_and_yaw() {
    let k_eps: f32 = 1e-4;
    let fov = projection::FieldOfView::from_degrees(60.0);
    let angles: unity::ViewAngles = unity::ViewAngles::new(
        unity::PitchAngle::from_degrees(30.0),
        unity::YawAngle::from_degrees(45.0),
        unity::RollAngle::from_degrees(0.0),
    );
    let cam = unity::Camera::new(Vector3::new(0.0, 0.0, 0.0), angles, ViewPort::new(1280.0, 720.0), fov, 0.03, 1000.0);

    let result = unity::Camera::calc_view_angles_from_view_matrix(&cam.get_view_matrix());

    expect_near!(result.pitch.as_degrees(), 30.0, k_eps);
    expect_near!(result.yaw.as_degrees(), 45.0, k_eps);
}

#[test]
fn unity_calc_view_angles_from_view_matrix_negative_pitch_and_yaw() {
    let k_eps: f32 = 1e-4;
    let fov = projection::FieldOfView::from_degrees(60.0);
    let angles: unity::ViewAngles = unity::ViewAngles::new(
        unity::PitchAngle::from_degrees(-45.0),
        unity::YawAngle::from_degrees(-90.0),
        unity::RollAngle::from_degrees(0.0),
    );
    let cam = unity::Camera::new(Vector3::new(0.0, 0.0, 0.0), angles, ViewPort::new(1280.0, 720.0), fov, 0.03, 1000.0);

    let result = unity::Camera::calc_view_angles_from_view_matrix(&cam.get_view_matrix());

    expect_near!(result.pitch.as_degrees(), -45.0, k_eps);
    expect_near!(result.yaw.as_degrees(), -90.0, k_eps);
}

#[test]
fn unity_calc_origin_from_view_matrix_at_origin() {
    let k_eps: f32 = 1e-4;
    let fov = projection::FieldOfView::from_degrees(60.0);
    let cam = unity::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1280.0, 720.0),
        fov,
        0.03,
        1000.0,
    );

    let origin = unity::Camera::calc_origin_from_view_matrix(&cam.get_view_matrix());

    expect_near!(origin.x, 0.0, k_eps);
    expect_near!(origin.y, 0.0, k_eps);
    expect_near!(origin.z, 0.0, k_eps);
}

#[test]
fn unity_calc_origin_from_view_matrix_arbitrary_position() {
    let k_eps: f32 = 1e-3;
    let fov = projection::FieldOfView::from_degrees(60.0);
    let angles: unity::ViewAngles = unity::ViewAngles::new(
        unity::PitchAngle::from_degrees(0.0),
        unity::YawAngle::from_degrees(0.0),
        unity::RollAngle::from_degrees(0.0),
    );
    let cam =
        unity::Camera::new(Vector3::new(100.0, 200.0, -50.0), angles, ViewPort::new(1280.0, 720.0), fov, 0.03, 1000.0);

    let origin = unity::Camera::calc_origin_from_view_matrix(&cam.get_view_matrix());

    expect_near!(origin.x, 100.0, k_eps);
    expect_near!(origin.y, 200.0, k_eps);
    expect_near!(origin.z, -50.0, k_eps);
}

#[test]
fn unity_calc_origin_from_view_matrix_with_rotation() {
    let k_eps: f32 = 1e-3;
    let fov = projection::FieldOfView::from_degrees(60.0);
    let angles: unity::ViewAngles = unity::ViewAngles::new(
        unity::PitchAngle::from_degrees(30.0),
        unity::YawAngle::from_degrees(45.0),
        unity::RollAngle::from_degrees(0.0),
    );
    let cam =
        unity::Camera::new(Vector3::new(300.0, -100.0, 75.0), angles, ViewPort::new(1280.0, 720.0), fov, 0.03, 1000.0);

    let origin = unity::Camera::calc_origin_from_view_matrix(&cam.get_view_matrix());

    expect_near!(origin.x, 300.0, k_eps);
    expect_near!(origin.y, -100.0, k_eps);
    expect_near!(origin.z, 75.0, k_eps);
}

#[test]
fn source_engine_zero_angles_basis_vectors() {
    let k_eps: f32 = 1e-5;
    let cam = source::Camera::new(
        Default::default(),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        projection::FieldOfView::from_degrees(90.0),
        0.01,
        1000.0,
    );
    let fwd = cam.get_abs_forward();
    let right = cam.get_abs_right();
    let up = cam.get_abs_up();

    expect_near!(fwd.x, source::K_ABS_FORWARD.x, k_eps);
    expect_near!(fwd.y, source::K_ABS_FORWARD.y, k_eps);
    expect_near!(fwd.z, source::K_ABS_FORWARD.z, k_eps);

    expect_near!(right.x, source::K_ABS_RIGHT.x, k_eps);
    expect_near!(right.y, source::K_ABS_RIGHT.y, k_eps);
    expect_near!(right.z, source::K_ABS_RIGHT.z, k_eps);

    expect_near!(up.x, source::K_ABS_UP.x, k_eps);
    expect_near!(up.y, source::K_ABS_UP.y, k_eps);
    expect_near!(up.z, source::K_ABS_UP.z, k_eps);
}

#[test]
fn unity_engine_zero_angles_basis_vectors() {
    let k_eps: f32 = 1e-5;
    let cam = unity::Camera::new(
        Default::default(),
        Default::default(),
        ViewPort::new(1280.0, 720.0),
        projection::FieldOfView::from_degrees(60.0),
        0.03,
        1000.0,
    );
    let fwd = cam.get_abs_forward();
    let right = cam.get_abs_right();
    let up = cam.get_abs_up();

    expect_near!(fwd.x, unity::K_ABS_FORWARD.x, k_eps);
    expect_near!(fwd.y, unity::K_ABS_FORWARD.y, k_eps);
    expect_near!(fwd.z, unity::K_ABS_FORWARD.z, k_eps);

    expect_near!(right.x, unity::K_ABS_RIGHT.x, k_eps);
    expect_near!(right.y, unity::K_ABS_RIGHT.y, k_eps);
    expect_near!(right.z, unity::K_ABS_RIGHT.z, k_eps);

    expect_near!(up.x, unity::K_ABS_UP.x, k_eps);
    expect_near!(up.y, unity::K_ABS_UP.y, k_eps);
    expect_near!(up.z, unity::K_ABS_UP.z, k_eps);
}

#[test]
fn opengl_engine_zero_angles_basis_vectors() {
    let k_eps: f32 = 1e-5;
    let cam = opengl::Camera::new(
        Default::default(),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        projection::FieldOfView::from_degrees(90.0),
        0.01,
        1000.0,
    );
    let fwd = cam.get_abs_forward();
    let right = cam.get_abs_right();
    let up = cam.get_abs_up();

    expect_near!(fwd.x, opengl::K_ABS_FORWARD.x, k_eps);
    expect_near!(fwd.y, opengl::K_ABS_FORWARD.y, k_eps);
    expect_near!(fwd.z, opengl::K_ABS_FORWARD.z, k_eps);

    expect_near!(right.x, opengl::K_ABS_RIGHT.x, k_eps);
    expect_near!(right.y, opengl::K_ABS_RIGHT.y, k_eps);
    expect_near!(right.z, opengl::K_ABS_RIGHT.z, k_eps);

    expect_near!(up.x, opengl::K_ABS_UP.x, k_eps);
    expect_near!(up.y, opengl::K_ABS_UP.y, k_eps);
    expect_near!(up.z, opengl::K_ABS_UP.z, k_eps);
}

#[test]
fn unreal_engine_zero_angles_basis_vectors() {
    let k_eps: f32 = 1e-5;
    let cam = unreal::Camera::new(
        Default::default(),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        projection::FieldOfView::from_degrees(90.0),
        0.01,
        1000.0,
    );
    let fwd = cam.get_abs_forward();
    let right = cam.get_abs_right();
    let up = cam.get_abs_up();

    expect_near!(fwd.x, unreal::K_ABS_FORWARD.x, k_eps);
    expect_near!(fwd.y, unreal::K_ABS_FORWARD.y, k_eps);
    expect_near!(fwd.z, unreal::K_ABS_FORWARD.z, k_eps);

    expect_near!(right.x, unreal::K_ABS_RIGHT.x, k_eps);
    expect_near!(right.y, unreal::K_ABS_RIGHT.y, k_eps);
    expect_near!(right.z, unreal::K_ABS_RIGHT.z, k_eps);

    expect_near!(up.x, unreal::K_ABS_UP.x, k_eps);
    expect_near!(up.y, unreal::K_ABS_UP.y, k_eps);
    expect_near!(up.z, unreal::K_ABS_UP.z, k_eps);
}

#[test]
fn frostbite_engine_zero_angles_basis_vectors() {
    let k_eps: f32 = 1e-5;
    let cam = frostbite::Camera::new(
        Default::default(),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        projection::FieldOfView::from_degrees(90.0),
        0.01,
        1000.0,
    );
    let fwd = cam.get_abs_forward();
    let right = cam.get_abs_right();
    let up = cam.get_abs_up();

    expect_near!(fwd.x, frostbite::K_ABS_FORWARD.x, k_eps);
    expect_near!(fwd.y, frostbite::K_ABS_FORWARD.y, k_eps);
    expect_near!(fwd.z, frostbite::K_ABS_FORWARD.z, k_eps);

    expect_near!(right.x, frostbite::K_ABS_RIGHT.x, k_eps);
    expect_near!(right.y, frostbite::K_ABS_RIGHT.y, k_eps);
    expect_near!(right.z, frostbite::K_ABS_RIGHT.z, k_eps);

    expect_near!(up.x, frostbite::K_ABS_UP.x, k_eps);
    expect_near!(up.y, frostbite::K_ABS_UP.y, k_eps);
    expect_near!(up.z, frostbite::K_ABS_UP.z, k_eps);
}

#[test]
fn cry_engine_zero_angles_basis_vectors() {
    let k_eps: f32 = 1e-5;
    let cam = cry::Camera::new(
        Default::default(),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        projection::FieldOfView::from_degrees(90.0),
        0.01,
        1000.0,
    );
    let fwd = cam.get_abs_forward();
    let right = cam.get_abs_right();
    let up = cam.get_abs_up();

    expect_near!(fwd.x, cry::K_ABS_FORWARD.x, k_eps);
    expect_near!(fwd.y, cry::K_ABS_FORWARD.y, k_eps);
    expect_near!(fwd.z, cry::K_ABS_FORWARD.z, k_eps);

    expect_near!(right.x, cry::K_ABS_RIGHT.x, k_eps);
    expect_near!(right.y, cry::K_ABS_RIGHT.y, k_eps);
    expect_near!(right.z, cry::K_ABS_RIGHT.z, k_eps);

    expect_near!(up.x, cry::K_ABS_UP.x, k_eps);
    expect_near!(up.y, cry::K_ABS_UP.y, k_eps);
    expect_near!(up.z, cry::K_ABS_UP.z, k_eps);
}

#[test]
fn iw_engine_zero_angles_basis_vectors() {
    let k_eps: f32 = 1e-5;
    let cam = iw::Camera::new(
        Default::default(),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        projection::FieldOfView::from_degrees(90.0),
        0.01,
        1000.0,
    );
    let fwd = cam.get_abs_forward();
    let right = cam.get_abs_right();
    let up = cam.get_abs_up();

    expect_near!(fwd.x, iw::K_ABS_FORWARD.x, k_eps);
    expect_near!(fwd.y, iw::K_ABS_FORWARD.y, k_eps);
    expect_near!(fwd.z, iw::K_ABS_FORWARD.z, k_eps);

    expect_near!(right.x, iw::K_ABS_RIGHT.x, k_eps);
    expect_near!(right.y, iw::K_ABS_RIGHT.y, k_eps);
    expect_near!(right.z, iw::K_ABS_RIGHT.z, k_eps);

    expect_near!(up.x, iw::K_ABS_UP.x, k_eps);
    expect_near!(up.y, iw::K_ABS_UP.y, k_eps);
    expect_near!(up.z, iw::K_ABS_UP.z, k_eps);
}

#[test]
fn extract_projection_params_fov_round_trip() {
    // Unity uses a plain vertical FOV, so this exercises the default formula. Engines with other FOV conventions are
    // covered by ExtractProjectionParams_FovRoundTripAllEngines.
    let k_eps: f32 = 1e-4;
    let fov = projection::FieldOfView::from_degrees(75.0);
    let cam =
        unity::Camera::new(Default::default(), Default::default(), ViewPort::new(1280.0, 720.0), fov, 0.03, 1000.0);

    let params = unity::Camera::extract_projection_params(&cam.get_projection_matrix());

    expect_near!(params.fov.as_degrees(), 75.0, k_eps);
}

#[test]
fn extract_projection_params_aspect_ratio_round_trip() {
    let k_eps: f32 = 1e-4;
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam =
        source::Camera::new(Default::default(), Default::default(), ViewPort::new(1920.0, 1080.0), fov, 0.01, 1000.0);

    let params = source::Camera::extract_projection_params(&cam.get_projection_matrix());

    expect_near!(params.aspect_ratio, 1920.0 / 1080.0, k_eps);
}

#[test]
fn extract_projection_params_unity_engine() {
    let k_eps: f32 = 1e-4;
    let fov = projection::FieldOfView::from_degrees(60.0);
    let cam =
        unity::Camera::new(Default::default(), Default::default(), ViewPort::new(1280.0, 720.0), fov, 0.03, 1000.0);

    let params = unity::Camera::extract_projection_params(&cam.get_projection_matrix());

    expect_near!(params.fov.as_degrees(), 60.0, k_eps);
    expect_near!(params.aspect_ratio, 1280.0 / 720.0, k_eps);
}

#[test]
fn extract_projection_params_fov_round_trip_all_engines() {
    for fov in [60.0, 90.0, 110.0] {
        expect_projection_params_round_trip::<source::CameraTrait>(fov);
        expect_projection_params_round_trip::<iw::CameraTrait>(fov);
        expect_projection_params_round_trip::<unreal::CameraTrait>(fov);
        expect_projection_params_round_trip::<serious::CameraTrait>(fov);
        expect_projection_params_round_trip::<unity::CameraTrait>(fov);
        expect_projection_params_round_trip::<opengl::CameraTrait>(fov);
        expect_projection_params_round_trip::<frostbite::CameraTrait>(fov);
        expect_projection_params_round_trip::<rage::CameraTrait>(fov);
        expect_projection_params_round_trip::<cry::CameraTrait>(fov);
    }
}

#[test]
fn calc_view_angles_from_view_matrix_round_trip_source() {
    expect_view_angles_round_trip_grid::<source::CameraTrait>();
}

#[test]
fn calc_view_angles_from_view_matrix_round_trip_iw() {
    expect_view_angles_round_trip_grid::<iw::CameraTrait>();
}

#[test]
fn calc_view_angles_from_view_matrix_round_trip_unreal() {
    expect_view_angles_round_trip_grid::<unreal::CameraTrait>();
}

#[test]
fn calc_view_angles_from_view_matrix_round_trip_serious() {
    expect_view_angles_round_trip_grid::<serious::CameraTrait>();
}

#[test]
fn calc_view_angles_from_view_matrix_round_trip_unity() {
    expect_view_angles_round_trip_grid::<unity::CameraTrait>();
}

#[test]
fn calc_view_angles_from_view_matrix_round_trip_opengl() {
    expect_view_angles_round_trip_grid::<opengl::CameraTrait>();
}

#[test]
fn calc_view_angles_from_view_matrix_round_trip_frostbite() {
    expect_view_angles_round_trip_grid::<frostbite::CameraTrait>();
}

#[test]
fn calc_view_angles_from_view_matrix_round_trip_rage() {
    expect_view_angles_round_trip_grid::<rage::CameraTrait>();
}

#[test]
fn calc_view_angles_from_view_matrix_round_trip_cry() {
    expect_view_angles_round_trip_grid::<cry::CameraTrait>();
}

#[test]
fn roll_turns_around_the_camera_forward_axis_all_engines() {
    expect_roll_keeps_forward::<source::CameraTrait>();
    expect_roll_keeps_forward::<iw::CameraTrait>();
    expect_roll_keeps_forward::<unreal::CameraTrait>();
    expect_roll_keeps_forward::<serious::CameraTrait>();
    expect_roll_keeps_forward::<unity::CameraTrait>();
    expect_roll_keeps_forward::<opengl::CameraTrait>();
    expect_roll_keeps_forward::<frostbite::CameraTrait>();
    expect_roll_keeps_forward::<rage::CameraTrait>();
    expect_roll_keeps_forward::<cry::CameraTrait>();
}

#[test]
fn calc_view_angles_from_view_matrix_fallback_recovers_roll_around_forward() {
    // Without extract_rotation_angles() the camera falls back to look-at plus a roll solved around forward, which is
    // exact for engines like Source that roll around the camera's own forward axis.
    expect_view_angles_round_trip_grid::<WithoutRotationExtraction<source::CameraTrait>>();
}

#[test]
fn calc_view_angles_from_view_matrix_fallback_keeps_forward_for_world_axis_roll() {
    // A custom trait that rolls around a world axis (roll applied last, Rz(roll) * Ry(yaw) * Rx(pitch)): rolling also
    // moves forward, so no roll fits on top of the look-at angles. The fallback leaves roll at 0 and still points the
    // camera the same way.
    type FallbackCamera = Camera<WorldAxisRollTrait>;

    let cam = FallbackCamera::new(
        Default::default(),
        unity::ViewAngles::new(
            unity::PitchAngle::from_degrees(25.0),
            unity::YawAngle::from_degrees(-40.0),
            unity::RollAngle::from_degrees(30.0),
        ),
        ViewPort::new(1280.0, 1024.0),
        projection::FieldOfView::from_degrees(90.0),
        0.1,
        1000.0,
    );
    let recovered = FallbackCamera::calc_view_angles_from_view_matrix(&cam.get_view_matrix());
    let rebuilt = FallbackCamera::new(
        Default::default(),
        recovered,
        ViewPort::new(1280.0, 1024.0),
        projection::FieldOfView::from_degrees(90.0),
        0.1,
        1000.0,
    );

    expect_float_eq!(recovered.roll.as_degrees(), 0.0);
    expect_near!(rebuilt.get_forward().dot(cam.get_forward()), 1.0, 1e-4);
}

#[test]
fn accessors_get_fov_near_far_origin() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let origin: Vector3<f32> = Vector3::new(10.0, 20.0, 30.0);
    let cam = source::Camera::new(origin, Default::default(), ViewPort::new(1920.0, 1080.0), fov, 0.1, 500.0);

    expect_near!(cam.get_field_of_view().as_degrees(), 90.0, 1e-4);
    expect_float_eq!(cam.get_near_plane(), 0.1);
    expect_float_eq!(cam.get_far_plane(), 500.0);
    expect_float_eq!(cam.get_origin().x, 10.0);
    expect_float_eq!(cam.get_origin().y, 20.0);
    expect_float_eq!(cam.get_origin().z, 30.0);
}

#[test]
fn set_field_of_view_invalidates_projection() {
    let fov_a = projection::FieldOfView::from_degrees(90.0);
    let fov_b = projection::FieldOfView::from_degrees(45.0);
    let mut cam =
        source::Camera::new(Default::default(), Default::default(), ViewPort::new(1920.0, 1080.0), fov_a, 0.01, 1000.0);

    let proj_before = cam.get_projection_matrix();
    cam.set_field_of_view(fov_b);
    let proj_after = cam.get_projection_matrix();

    assert_ne!(proj_before.at(0, 0), proj_after.at(0, 0));
    expect_near!(cam.get_field_of_view().as_degrees(), 45.0, 1e-4);
}

#[test]
fn set_near_plane_invalidates_projection() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let mut cam =
        source::Camera::new(Default::default(), Default::default(), ViewPort::new(1920.0, 1080.0), fov, 0.01, 1000.0);

    let proj_before = cam.get_projection_matrix();
    cam.set_near_plane(1.0);
    let proj_after = cam.get_projection_matrix();

    expect_float_eq!(cam.get_near_plane(), 1.0);
    assert_ne!(proj_before.at(2, 2), proj_after.at(2, 2));
}

#[test]
fn set_far_plane_invalidates_projection() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let mut cam =
        source::Camera::new(Default::default(), Default::default(), ViewPort::new(1920.0, 1080.0), fov, 0.01, 1000.0);

    let proj_before = cam.get_projection_matrix();
    cam.set_far_plane(100.0);
    let proj_after = cam.get_projection_matrix();

    expect_float_eq!(cam.get_far_plane(), 100.0);
    assert_ne!(proj_before.at(2, 2), proj_after.at(2, 2));
}

#[test]
fn set_origin_invalidates_view_matrix() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let mut cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    // Target is off to the side — stays at the same world position while camera
    // moves laterally, so the projected X must change.
    let screen_before = cam.world_to_screen(Vector3::new(500.0, 100.0, 0.0));
    cam.set_origin(Vector3::new(0.0, 100.0, 0.0)); // lateral shift
    let screen_after = cam.world_to_screen(Vector3::new(500.0, 100.0, 0.0));

    assert!(screen_before.is_ok());
    assert!(screen_after.is_ok());
    assert_ne!(screen_before.unwrap().x, screen_after.unwrap().x);
    expect_float_eq!(cam.get_origin().y, 100.0);
}

#[test]
fn set_view_port_invalidates_projection() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let mut cam =
        source::Camera::new(Default::default(), Default::default(), ViewPort::new(1920.0, 1080.0), fov, 0.01, 1000.0);

    let proj_before = cam.get_projection_matrix();
    // 1280x800 is 8:5, different aspect ratio from 1920x1080 (16:9)
    cam.set_view_port(ViewPort::new(1280.0, 800.0));
    let proj_after = cam.get_projection_matrix();

    assert_ne!(proj_before.at(0, 0), proj_after.at(0, 0));
}

#[test]
fn set_view_angles_invalidates_view_matrix() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let mut cam =
        source::Camera::new(Default::default(), Default::default(), ViewPort::new(1920.0, 1080.0), fov, 0.01, 1000.0);

    let view_before = cam.get_view_matrix();
    let rotated: source::ViewAngles = source::ViewAngles::new(
        source::PitchAngle::from_degrees(30.0),
        source::YawAngle::from_degrees(45.0),
        source::RollAngle::from_degrees(0.0),
    );
    cam.set_view_angles(rotated);
    let view_after = cam.get_view_matrix();

    assert_ne!(view_before.at(0, 0), view_after.at(0, 0));
}

#[test]
fn calc_look_at_angles_forward_target() {
    // Source engine: +X is forward. Camera at origin, target on +X axis.
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let angles = cam.calc_look_at_angles(Vector3::new(100.0, 0.0, 0.0));

    expect_near!(angles.pitch.as_degrees(), 0.0, 1e-4);
    expect_near!(angles.yaw.as_degrees(), 0.0, 1e-4);
}

#[test]
fn look_at_forward_vector_points_at_target() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let mut cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    cam.look_at(Vector3::new(200.0, 0.0, 0.0));
    let fwd = cam.get_abs_forward();

    // After pointing at +X target the forward vector should be ~(1,0,0)
    expect_near!(fwd.x, 1.0, 1e-4);
    expect_near!(fwd.y, 0.0, 1e-4);
    expect_near!(fwd.z, 0.0, 1e-4);
}

#[test]
fn source_engine_look_at_random_world_points_project_to_screen_center() {
    verify_random_look_at_targets_project_to_screen_center::<source::CameraTrait>(
        Vector3::new(10.0, -20.0, 30.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        0.01,
        5000.0,
        101,
    );
}

#[test]
fn unity_engine_look_at_random_world_points_project_to_screen_center() {
    verify_random_look_at_targets_project_to_screen_center::<unity::CameraTrait>(
        Vector3::new(-15.0, 25.0, -35.0),
        unity::ViewAngles::default(),
        ViewPort::new(1280.0, 720.0),
        0.03,
        5000.0,
        102,
    );
}

#[test]
fn unreal_engine_look_at_random_world_points_project_to_screen_center() {
    verify_random_look_at_targets_project_to_screen_center::<unreal::CameraTrait>(
        Vector3::new(100.0, -50.0, 25.0),
        unreal::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        0.01,
        5000.0,
        103,
    );
}

#[test]
fn frostbite_engine_look_at_random_world_points_project_to_screen_center() {
    verify_random_look_at_targets_project_to_screen_center::<frostbite::CameraTrait>(
        Vector3::new(-4.0, 2.0, 8.0),
        frostbite::ViewAngles::default(),
        ViewPort::new(1600.0, 900.0),
        0.01,
        5000.0,
        104,
    );
}

#[test]
fn iw_engine_look_at_random_world_points_project_to_screen_center() {
    verify_random_look_at_targets_project_to_screen_center::<iw::CameraTrait>(
        Vector3::new(40.0, -60.0, 20.0),
        iw::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        0.01,
        5000.0,
        105,
    );
}

#[test]
fn opengl_engine_look_at_random_world_points_project_to_screen_center() {
    verify_random_look_at_targets_project_to_screen_center::<opengl::CameraTrait>(
        Vector3::new(-30.0, 70.0, 15.0),
        opengl::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        0.01,
        5000.0,
        106,
    );
}

#[test]
fn cry_engine_look_at_random_world_points_project_to_screen_center() {
    verify_random_look_at_targets_project_to_screen_center::<cry::CameraTrait>(
        Vector3::new(5.0, 15.0, -25.0),
        cry::ViewAngles::default(),
        ViewPort::new(1280.0, 720.0),
        0.01,
        5000.0,
        107,
    );
}

#[test]
fn rage_engine_look_at_random_world_points_project_to_screen_center() {
    verify_random_look_at_targets_project_to_screen_center::<rage::CameraTrait>(
        Vector3::new(-5.0, 45.0, 10.0),
        rage::ViewAngles::default(),
        ViewPort::new(1280.0, 720.0),
        0.01,
        5000.0,
        108,
    );
}

#[test]
fn triangle_inside_frustum_not_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    // Small triangle directly in front (Source: +X forward)
    let tri: Triangle<Vector3<f32>> =
        Triangle::new(Vector3::new(100.0, 0.0, 1.0), Vector3::new(100.0, 1.0, -1.0), Vector3::new(100.0, -1.0, -1.0));
    assert!(!(cam.is_culled_by_frustum(&tri)));
}

#[test]
fn triangle_behind_camera_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    // Triangle entirely behind the camera (-X)
    let tri: Triangle<Vector3<f32>> = Triangle::new(
        Vector3::new(-100.0, 0.0, 1.0),
        Vector3::new(-100.0, 1.0, -1.0),
        Vector3::new(-100.0, -1.0, -1.0),
    );
    assert!(cam.is_culled_by_frustum(&tri));
}

#[test]
fn triangle_beyond_far_plane_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    // Triangle beyond the 1000-unit far plane
    let tri: Triangle<Vector3<f32>> = Triangle::new(
        Vector3::new(2000.0, 0.0, 1.0),
        Vector3::new(2000.0, 1.0, -1.0),
        Vector3::new(2000.0, -1.0, -1.0),
    );
    assert!(cam.is_culled_by_frustum(&tri));
}

#[test]
fn triangle_far_to_side_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    // Triangle far outside the side frustum planes
    let tri: Triangle<Vector3<f32>> = Triangle::new(
        Vector3::new(100.0, 5000.0, 0.0),
        Vector3::new(100.0, 5001.0, 1.0),
        Vector3::new(100.0, 5001.0, -1.0),
    );
    assert!(cam.is_culled_by_frustum(&tri));
}

#[test]
fn triangle_straddling_frustum_not_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    // Large triangle with vertices on both sides of the frustum — should not be culled
    let tri: Triangle<Vector3<f32>> = Triangle::new(
        Vector3::new(100.0, 0.0, 0.0),
        Vector3::new(100.0, 5000.0, 0.0),
        Vector3::new(100.0, 0.0, 5000.0),
    );
    assert!(!(cam.is_culled_by_frustum(&tri)));
}

#[test]
fn ndc_screen_round_trip_through_public_helpers() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let ndc: Vector3<f32> = Vector3::new(0.25, -0.5, 0.75);
    let top_left = cam.ndc_to_screen_position_from_top_left_corner(ndc);
    let bottom_left = cam.ndc_to_screen_position_from_bottom_left_corner(ndc);

    expect_near!(top_left.x, 1200.0, 1e-3);
    expect_near!(top_left.y, 810.0, 1e-3);
    expect_near!(bottom_left.y, 270.0, 1e-3);

    let back_top_left = cam.screen_to_ndc(top_left, ScreenStart::TopLeftCorner);
    let back_bottom_left = cam.screen_to_ndc(bottom_left, ScreenStart::BottomLeftCorner);
    expect_near!(back_top_left.x, ndc.x, 1e-5);
    expect_near!(back_top_left.y, ndc.y, 1e-5);
    expect_near!(back_bottom_left.x, ndc.x, 1e-5);
    expect_near!(back_bottom_left.y, ndc.y, 1e-5);
}

#[test]
fn world_to_view_coordinates_matches_matrix_multiply() {
    let mut cam = source::Camera::new(
        Vector3::new(10.0, -5.0, 3.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        projection::FieldOfView::from_degrees(90.0),
        0.01,
        1000.0,
    );
    cam.look_at(Vector3::new(100.0, 40.0, -20.0));

    let world: Vector3<f32> = Vector3::new(57.0, 12.0, -3.0);
    let expected = cam.get_view_matrix() * Mat::<4, 1, f32, RowMajor>::column_from_vector(world);
    let actual = cam.world_to_view_coordinates(world);

    expect_near!(actual.x, expected.at(0, 0), 1e-4);
    expect_near!(actual.y, expected.at(1, 0), 1e-4);
    expect_near!(actual.z, expected.at(2, 0), 1e-4);
}
