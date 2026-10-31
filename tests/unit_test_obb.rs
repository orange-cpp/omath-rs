#[macro_use]
mod projection_support;

use omath::Vector3;
use omath::engines::{opengl, source, unity};
use omath::primitives::{Aabb, Obb};
use omath::projection::{self, ViewPort};

type ObbF = Obb<f32>;
type ObbD = Obb<f64>;
type Vec3F = Vector3<f32>;
type Vec3D = Vector3<f64>;

const fn axis_aligned_obb(center: Vec3F, half_extents: Vec3F) -> ObbF {
    ObbF {
        center,
        axis_x: Vec3F::new(1.0, 0.0, 0.0),
        axis_y: Vec3F::new(0.0, 1.0, 0.0),
        axis_z: Vec3F::new(0.0, 0.0, 1.0),
        half_extents,
    }
}

fn rotated_around_z(center: Vec3F, half_extents: Vec3F, radians: f32) -> ObbF {
    let (s, c) = radians.sin_cos();
    ObbF {
        center,
        axis_x: Vec3F::new(c, s, 0.0),
        axis_y: Vec3F::new(-s, c, 0.0),
        axis_z: Vec3F::new(0.0, 0.0, 1.0),
        half_extents,
    }
}

fn rotated_around_y(center: Vec3F, half_extents: Vec3F, radians: f32) -> ObbF {
    let (s, c) = radians.sin_cos();
    ObbF {
        center,
        axis_x: Vec3F::new(c, 0.0, -s),
        axis_y: Vec3F::new(0.0, 1.0, 0.0),
        axis_z: Vec3F::new(s, 0.0, c),
        half_extents,
    }
}

fn expect_vec_near(actual: Vec3F, expected: Vec3F, epsilon: f32) {
    expect_near!(actual.x, expected.x, epsilon);
    expect_near!(actual.y, expected.y, epsilon);
    expect_near!(actual.z, expected.z, epsilon);
}

#[test]
fn vertices_of_axis_aligned_unit_box() {
    let box_ = axis_aligned_obb(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0));
    let v = box_.vertices();

    assert_eq!(v[0], Vec3F::new(-1.0, -1.0, -1.0));
    assert_eq!(v[1], Vec3F::new(1.0, -1.0, -1.0));
    assert_eq!(v[2], Vec3F::new(-1.0, 1.0, -1.0));
    assert_eq!(v[3], Vec3F::new(1.0, 1.0, -1.0));
    assert_eq!(v[4], Vec3F::new(-1.0, -1.0, 1.0));
    assert_eq!(v[5], Vec3F::new(1.0, -1.0, 1.0));
    assert_eq!(v[6], Vec3F::new(-1.0, 1.0, 1.0));
    assert_eq!(v[7], Vec3F::new(1.0, 1.0, 1.0));
}

#[test]
fn vertices_of_translated_box() {
    let box_ = axis_aligned_obb(Vector3::new(10.0, 20.0, 30.0), Vector3::new(1.0, 2.0, 3.0));
    let v = box_.vertices();

    assert_eq!(v[0], Vec3F::new(9.0, 18.0, 27.0));
    assert_eq!(v[7], Vec3F::new(11.0, 22.0, 33.0));
}

#[test]
fn vertices_of_rotated_box() {
    let pi = std::f32::consts::PI;
    let box_ = rotated_around_z(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0), pi / 2.0);
    let v = box_.vertices();

    // After 90° rotation around Z, local +X maps to world +Y, local +Y maps to world -X.
    // The eight vertices are still the same eight points (a cube is symmetric), but their
    // ordering changes. Check that the corner set as a whole is still |coord| == 1.0
    for corner in v {
        expect_near!((corner.x).abs(), 1.0, 1e-5);
        expect_near!((corner.y).abs(), 1.0, 1e-5);
        expect_near!((corner.z).abs(), 1.0, 1e-5);
    }
}

#[test]
fn vertices_of_translated_non_uniform_rotated_box() {
    let box_ = rotated_around_z(Vector3::new(2.0, 3.0, 4.0), Vector3::new(2.0, 1.0, 0.5), std::f32::consts::PI / 2.0);
    let v = box_.vertices();

    expect_vec_near(v[0], Vector3::new(3.0, 1.0, 3.5), 1e-5);
    expect_vec_near(v[1], Vector3::new(3.0, 5.0, 3.5), 1e-5);
    expect_vec_near(v[2], Vector3::new(1.0, 1.0, 3.5), 1e-5);
    expect_vec_near(v[3], Vector3::new(1.0, 5.0, 3.5), 1e-5);
    expect_vec_near(v[4], Vector3::new(3.0, 1.0, 4.5), 1e-5);
    expect_vec_near(v[5], Vector3::new(3.0, 5.0, 4.5), 1e-5);
    expect_vec_near(v[6], Vector3::new(1.0, 1.0, 4.5), 1e-5);
    expect_vec_near(v[7], Vector3::new(1.0, 5.0, 4.5), 1e-5);
}

#[test]
fn vertices_collapse_when_one_extent_is_zero() {
    let box_ = axis_aligned_obb(Vector3::new(1.0, 2.0, 3.0), Vector3::new(2.0, 0.0, 4.0));
    let v = box_.vertices();

    assert_eq!(v[0], v[2]);
    assert_eq!(v[1], v[3]);
    assert_eq!(v[4], v[6]);
    assert_eq!(v[5], v[7]);

    assert_eq!(v[0], Vec3F::new(-1.0, 2.0, -1.0));
    assert_eq!(v[1], Vec3F::new(3.0, 2.0, -1.0));
    assert_eq!(v[4], Vec3F::new(-1.0, 2.0, 7.0));
    assert_eq!(v[5], Vec3F::new(3.0, 2.0, 7.0));
}

#[test]
fn double_precision_instantiation() {
    let box_ = ObbD {
        center: Vec3D::new(0.0, 0.0, 0.0),
        axis_x: Vec3D::new(1.0, 0.0, 0.0),
        axis_y: Vec3D::new(0.0, 1.0, 0.0),
        axis_z: Vec3D::new(0.0, 0.0, 1.0),
        half_extents: Vec3D::new(2.0, 3.0, 4.0),
    };
    let v = box_.vertices();
    expect_double_eq!(v[0].x, -2.0);
    expect_double_eq!(v[0].y, -3.0);
    expect_double_eq!(v[0].z, -4.0);
    expect_double_eq!(v[7].x, 2.0);
    expect_double_eq!(v[7].y, 3.0);
    expect_double_eq!(v[7].z, 4.0);
}

#[test]
fn axis_aligned_in_front_not_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let obb = axis_aligned_obb(Vector3::new(100.0, 0.0, 0.0), Vector3::new(10.0, 1.0, 1.0));
    assert!(!(cam.is_obb_culled_by_frustum(&obb)));
}

#[test]
fn axis_aligned_behind_camera_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let obb = axis_aligned_obb(Vector3::new(-150.0, 0.0, 0.0), Vector3::new(50.0, 1.0, 1.0));
    assert!(cam.is_obb_culled_by_frustum(&obb));
}

#[test]
fn axis_aligned_beyond_far_plane_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let obb = axis_aligned_obb(Vector3::new(1750.0, 0.0, 0.0), Vector3::new(250.0, 1.0, 1.0));
    assert!(cam.is_obb_culled_by_frustum(&obb));
}

#[test]
fn axis_aligned_straddling_far_plane_not_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let obb = axis_aligned_obb(Vector3::new(1005.0, 0.0, 0.0), Vector3::new(10.0, 1.0, 1.0));
    assert!(!(cam.is_obb_culled_by_frustum(&obb)));
}

#[test]
fn axis_aligned_far_left_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let obb = axis_aligned_obb(Vector3::new(100.0, 4500.0, 0.0), Vector3::new(10.0, 500.0, 1.0));
    assert!(cam.is_obb_culled_by_frustum(&obb));
}

#[test]
fn axis_aligned_far_right_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let obb = axis_aligned_obb(Vector3::new(100.0, -4500.0, 0.0), Vector3::new(10.0, 500.0, 1.0));
    assert!(cam.is_obb_culled_by_frustum(&obb));
}

#[test]
fn axis_aligned_above_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let obb = axis_aligned_obb(Vector3::new(100.0, 0.0, 5500.0), Vector3::new(10.0, 1.0, 500.0));
    assert!(cam.is_obb_culled_by_frustum(&obb));
}

#[test]
fn axis_aligned_below_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let obb = axis_aligned_obb(Vector3::new(100.0, 0.0, -5500.0), Vector3::new(10.0, 1.0, 500.0));
    assert!(cam.is_obb_culled_by_frustum(&obb));
}

#[test]
fn matches_aabb_for_axis_aligned_box() {
    // For axis-aligned OBBs, the result must agree with is_aabb_culled_by_frustum.
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let cases = [
        (Vec3F::new(100.0, 0.0, 0.0), Vec3F::new(10.0, 1.0, 1.0)), // in front
        (Vec3F::new(-150.0, 0.0, 0.0), Vec3F::new(50.0, 1.0, 1.0)), // behind
        (Vec3F::new(1750.0, 0.0, 0.0), Vec3F::new(250.0, 1.0, 1.0)), // beyond far
        (Vec3F::new(100.0, 4500.0, 0.0), Vec3F::new(10.0, 500.0, 1.0)), // far left
        (Vec3F::new(0.0, 0.0, 0.0), Vec3F::new(500.0, 500.0, 500.0)), // encloses camera
        (Vec3F::new(275.0, 0.0, 0.0), Vec3F::new(225.0, 1.0, 1.0)), // straddles near
    ];

    for (center, half) in cases {
        let aabb: Aabb<f32> = Aabb::new(center - half, center + half);
        let obb = axis_aligned_obb(center, half);
        assert_eq!(cam.is_obb_culled_by_frustum(&obb), cam.is_aabb_culled_by_frustum(&aabb));
    }
}

#[test]
fn rotation_can_pull_box_into_frustum() {
    // Tall thin column sitting just outside the +Y frustum boundary at X=50.0
    // Axis-aligned: every corner has Y≈100 at X≈50, all outside the +Y plane → culled.
    // Rotated 90° around world Y: the 50-unit extent now points along world +X, so the rod
    // sweeps forward to X≈100 where the +Y plane is far more permissive — front end inside,
    // box_ no longer fully outside → not culled.
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let center: Vec3F = Vec3F::new(50.0, 100.0, 0.0);
    let half: Vec3F = Vec3F::new(1.0, 1.0, 50.0);

    let axis_aligned = axis_aligned_obb(center, half);
    assert!(cam.is_obb_culled_by_frustum(&axis_aligned));

    let rotated = rotated_around_y(center, half, std::f32::consts::PI / 2.0);
    assert!(!(cam.is_obb_culled_by_frustum(&rotated)));
}

#[test]
fn rotation_can_push_box_out_of_frustum() {
    // Long forward-pointing rod whose front end pokes into the frustum near the +Y boundary.
    // Axis-aligned (long along X): the front end at X≈100 has Y=129 just inside the +Y plane,
    // so part of the rod is visible → not culled.
    // Rotated 90° around Z: the rod's long axis now points along world Y, so all corners
    // shift to Y∈[80,180] at X≈50 — every corner is outside the +Y plane → culled.
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let center: Vec3F = Vec3F::new(50.0, 130.0, 0.0);
    let half: Vec3F = Vec3F::new(50.0, 1.0, 1.0);

    let axis_aligned = axis_aligned_obb(center, half);
    assert!(!(cam.is_obb_culled_by_frustum(&axis_aligned)));

    let rotated = rotated_around_z(center, half, std::f32::consts::PI / 2.0);
    assert!(cam.is_obb_culled_by_frustum(&rotated));
}

#[test]
fn rotated_box_straddling_frustum_not_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    // Box centred in front, rotated 30° — clearly straddles into the frustum.
    let obb =
        rotated_around_z(Vector3::new(200.0, 0.0, 0.0), Vector3::new(50.0, 50.0, 50.0), std::f32::consts::PI / 6.0);
    assert!(!(cam.is_obb_culled_by_frustum(&obb)));
}

#[test]
fn opengl_engine_rotated_in_front_not_culled() {
    // OpenGL: -Z forward, COLUMN_MAJOR, NEGATIVE_ONE_TO_ONE
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = opengl::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let obb = rotated_around_z(Vector3::new(0.0, 0.0, -100.0), Vector3::new(5.0, 5.0, 5.0), std::f32::consts::PI / 4.0);
    assert!(!(cam.is_obb_culled_by_frustum(&obb)));
}

#[test]
fn opengl_engine_behind_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = opengl::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    let obb = axis_aligned_obb(Vector3::new(0.0, 0.0, 100.0), Vector3::new(5.0, 5.0, 5.0));
    assert!(cam.is_obb_culled_by_frustum(&obb));
}

#[test]
fn opengl_engine_straddling_far_plane_not_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = opengl::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        100.0,
    );

    let obb =
        rotated_around_z(Vector3::new(0.0, 0.0, -105.0), Vector3::new(5.0, 5.0, 10.0), std::f32::consts::PI / 4.0);
    assert!(!(cam.is_obb_culled_by_frustum(&obb)));
}

#[test]
fn unity_engine_beyond_far_culled() {
    let fov = projection::FieldOfView::from_degrees(60.0);
    let cam = unity::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1280.0, 720.0),
        fov,
        0.03,
        500.0,
    );

    let obb = rotated_around_z(Vector3::new(0.0, 0.0, 700.0), Vector3::new(5.0, 5.0, 5.0), std::f32::consts::PI / 4.0);
    assert!(cam.is_obb_culled_by_frustum(&obb));
}

#[test]
fn degenerate_zero_volume_inside_not_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    // Zero-extent OBB — collapses to a point, but still must not be culled if the centre is inside.
    let obb = axis_aligned_obb(Vector3::new(100.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0));
    assert!(!(cam.is_obb_culled_by_frustum(&obb)));
}

#[test]
fn enclosing_camera_not_culled() {
    let fov = projection::FieldOfView::from_degrees(90.0);
    let cam = source::Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        fov,
        0.01,
        1000.0,
    );

    // Huge rotated box_ that fully encloses the camera origin.
    let obb =
        rotated_around_z(Vector3::new(0.0, 0.0, 0.0), Vector3::new(500.0, 500.0, 500.0), std::f32::consts::PI / 5.0);
    assert!(!(cam.is_obb_culled_by_frustum(&obb)));
}
