#[macro_use]
mod projection_support;

use omath::engines::{cry, frostbite, iw, opengl, rage, serious, source, unity, unreal};
use omath::primitives::MeshEngine;
use omath::projection::{self, CameraEngine, FieldOfView, ViewPort};
use omath::{ColumnMajor, Mat, NdcDepthRange, RowMajor, Vector3};

use omath::projection::Camera;
use omath::scalar::Scalar;
use omath::{Real, StoreOrder};

fn expect_matrix_near<T: Real, S: StoreOrder>(a: &Mat<4, 4, T, S>, b: &Mat<4, 4, T, S>, eps: f32) {
    for r in 0..4 {
        for c in 0..4 {
            expect_near!(a.at(r, c).to_f64(), b.at(r, c).to_f64(), eps);
        }
    }
}

fn matrix_has_non_zero_element<T: Real, S: StoreOrder>(mat: &Mat<4, 4, T, S>) -> bool {
    mat.as_slice().iter().any(|&element| element != T::ZERO)
}

/// The C++ runs this in a `static_assert`, to show that the matrices can be built at compile time.
fn camera_can_calculate_matrices<E: CameraEngine>() -> bool {
    let camera = Camera::<E>::new(
        Default::default(),
        Default::default(),
        ViewPort::new(1280.0, 720.0),
        FieldOfView::from_degrees(90.0),
        E::Scalar::from_f64(0.01),
        E::Scalar::from_f64(1000.0),
    );

    matrix_has_non_zero_element(&camera.get_view_matrix())
        && matrix_has_non_zero_element(&camera.get_projection_matrix())
        && matrix_has_non_zero_element(&camera.get_view_projection_matrix())
}

#[test]
fn camera_can_calculate_matrices_all_engines() {
    assert!(camera_can_calculate_matrices::<source::CameraTrait>());
    assert!(camera_can_calculate_matrices::<iw::CameraTrait>());
    assert!(camera_can_calculate_matrices::<frostbite::CameraTrait>());
    assert!(camera_can_calculate_matrices::<opengl::CameraTrait>());
    assert!(camera_can_calculate_matrices::<unity::CameraTrait>());
    assert!(camera_can_calculate_matrices::<cry::CameraTrait>());
    assert!(camera_can_calculate_matrices::<rage::CameraTrait>());
    assert!(camera_can_calculate_matrices::<serious::CameraTrait>());
    assert!(camera_can_calculate_matrices::<unreal::CameraTrait>());
}

fn project_z_lh<T: Real, S: StoreOrder>(proj: &Mat<4, 4, T, S>, z: f32) -> f32 {
    let clip = *proj * Mat::<4, 1, T, S>::column_from_vector(Vector3::new(T::ZERO, T::ZERO, T::from_f64(f64::from(z))));
    (clip.at(2, 0) / clip.at(3, 0)).to_f64() as f32
}

#[test]
fn frostbite_pred_and_mesh_and_camera() {
    // MeshTrait simply forwards to rotation_matrix; ensure it compiles and returns something
    let va = frostbite::ViewAngles::default();
    let m1 = frostbite::MeshTrait::rotation_matrix(&va);
    let m2 = frostbite::rotation_matrix(va);
    expect_matrix_near(&m1, &m2, 1e-5);

    // CameraTrait look at should be callable
    let _angles = frostbite::CameraTrait::calc_look_at_angle(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 1.0));
    let proj = frostbite::CameraTrait::calc_projection_matrix(
        projection::FieldOfView::from_degrees(60.0),
        ViewPort::new(1280.0, 720.0),
        0.1,
        1000.0,
        NdcDepthRange::NegativeOneToOne,
    );
    let expected = frostbite::calc_perspective_projection_matrix(
        60.0,
        1280.0 / 720.0,
        0.1,
        1000.0,
        frostbite::K_DEFAULT_NDC_DEPTH_RANGE,
    );
    expect_matrix_near(&proj, &expected, 1e-5);

    let proj_zo = frostbite::CameraTrait::calc_projection_matrix(
        projection::FieldOfView::from_degrees(60.0),
        ViewPort::new(1280.0, 720.0),
        0.1,
        1000.0,
        NdcDepthRange::ZeroToOne,
    );
    let expected_zo =
        frostbite::calc_perspective_projection_matrix(60.0, 1280.0 / 720.0, 0.1, 1000.0, NdcDepthRange::ZeroToOne);
    expect_matrix_near(&proj_zo, &expected_zo, 1e-5);
    assert_ne!(proj, proj_zo);
}

#[test]
fn source_both_depth_ranges() {
    let proj_no = source::CameraTrait::calc_projection_matrix(
        projection::FieldOfView::from_degrees(90.0),
        ViewPort::new(1920.0, 1080.0),
        0.1,
        1000.0,
        NdcDepthRange::NegativeOneToOne,
    );
    let expected_no =
        source::calc_perspective_projection_matrix(90.0, 1920.0 / 1080.0, 0.1, 1000.0, NdcDepthRange::NegativeOneToOne);
    expect_matrix_near(&proj_no, &expected_no, 1e-5);

    let proj_zo = source::CameraTrait::calc_projection_matrix(
        projection::FieldOfView::from_degrees(90.0),
        ViewPort::new(1920.0, 1080.0),
        0.1,
        1000.0,
        NdcDepthRange::ZeroToOne,
    );
    let expected_zo =
        source::calc_perspective_projection_matrix(90.0, 1920.0 / 1080.0, 0.1, 1000.0, NdcDepthRange::ZeroToOne);
    expect_matrix_near(&proj_zo, &expected_zo, 1e-5);

    assert_ne!(proj_no, proj_zo);
}

#[test]
fn cry_engine_both_depth_ranges() {
    let proj_no = cry::CameraTrait::calc_projection_matrix(
        projection::FieldOfView::from_degrees(90.0),
        ViewPort::new(1920.0, 1080.0),
        0.1,
        1000.0,
        NdcDepthRange::NegativeOneToOne,
    );
    let expected_no =
        cry::calc_perspective_projection_matrix(90.0, 1920.0 / 1080.0, 0.1, 1000.0, NdcDepthRange::NegativeOneToOne);
    expect_matrix_near(&proj_no, &expected_no, 1e-5);

    let proj_zo = cry::CameraTrait::calc_projection_matrix(
        projection::FieldOfView::from_degrees(90.0),
        ViewPort::new(1920.0, 1080.0),
        0.1,
        1000.0,
        NdcDepthRange::ZeroToOne,
    );
    let expected_zo =
        cry::calc_perspective_projection_matrix(90.0, 1920.0 / 1080.0, 0.1, 1000.0, NdcDepthRange::ZeroToOne);
    expect_matrix_near(&proj_zo, &expected_zo, 1e-5);

    assert_ne!(proj_no, proj_zo);
}

#[test]
fn source_zero_to_one_z_range() {
    // Source is left-handed
    let proj = source::calc_perspective_projection_matrix(90.0, 16.0 / 9.0, 0.1, 1000.0, NdcDepthRange::ZeroToOne);

    expect_near!(project_z_lh(&proj, 0.1), 0.0, 1e-4);
    expect_near!(project_z_lh(&proj, 1000.0), 1.0, 1e-4);
    assert!(project_z_lh(&proj, 500.0) > 0.0);
    assert!(project_z_lh(&proj, 500.0) < 1.0);
}

#[test]
fn iw_zero_to_one_z_range() {
    // IW is left-handed
    let proj = iw::calc_perspective_projection_matrix(90.0, 16.0 / 9.0, 0.1, 1000.0, NdcDepthRange::ZeroToOne);

    expect_near!(project_z_lh(&proj, 0.1), 0.0, 1e-4);
    expect_near!(project_z_lh(&proj, 1000.0), 1.0, 1e-4);
    assert!(project_z_lh(&proj, 500.0) > 0.0);
    assert!(project_z_lh(&proj, 500.0) < 1.0);
}

#[test]
fn opengl_zero_to_one_z_range() {
    // OpenGL is right-handed (negative z forward), column-major
    let proj = opengl::calc_perspective_projection_matrix(90.0, 16.0 / 9.0, 0.1, 1000.0, NdcDepthRange::ZeroToOne);

    // OpenGL engine uses column-major matrices, project manually
    let proj_z = |z: f32| {
        let clip = proj * Mat::<4, 1, f32, ColumnMajor>::column_from_vector(Vector3::new(0.0, 0.0, z));
        return clip.at(2, 0) / clip.at(3, 0);
    };

    expect_near!(proj_z(-0.1), 0.0, 1e-4);
    expect_near!(proj_z(-1000.0), 1.0, 1e-4);
    assert!(proj_z(-500.0) > 0.0);
    assert!(proj_z(-500.0) < 1.0);
}

#[test]
fn frostbite_zero_to_one_z_range() {
    let proj = frostbite::calc_perspective_projection_matrix(90.0, 16.0 / 9.0, 0.1, 1000.0, NdcDepthRange::ZeroToOne);

    expect_near!(project_z_lh(&proj, 0.1), 0.0, 1e-4);
    expect_near!(project_z_lh(&proj, 1000.0), 1.0, 1e-4);
    assert!(project_z_lh(&proj, 500.0) > 0.0);
    assert!(project_z_lh(&proj, 500.0) < 1.0);
}

#[test]
fn unity_zero_to_one_z_range() {
    // Unity is right-handed, row-major
    let proj = unity::calc_perspective_projection_matrix(90.0, 16.0 / 9.0, 0.1, 1000.0, NdcDepthRange::ZeroToOne);

    let proj_z = |z: f32| {
        let clip = proj * Mat::<4, 1, f32, RowMajor>::column_from_vector(Vector3::new(0.0, 0.0, z));
        return clip.at(2, 0) / clip.at(3, 0);
    };

    expect_near!(proj_z(-0.1), 0.0, 1e-4);
    expect_near!(proj_z(-1000.0), 1.0, 1e-4);
    assert!(proj_z(-500.0) > 0.0);
    assert!(proj_z(-500.0) < 1.0);
}

#[test]
fn unreal_zero_to_one_z_range() {
    let proj = unreal::calc_perspective_projection_matrix(90.0, 16.0 / 9.0, 0.1, 1000.0, NdcDepthRange::ZeroToOne);

    expect_near!(project_z_lh(&proj, 0.1), 0.0, 1e-4);
    expect_near!(project_z_lh(&proj, 1000.0), 1.0, 1e-4);
    assert!(project_z_lh(&proj, 500.0) > 0.0);
    assert!(project_z_lh(&proj, 500.0) < 1.0);
}

#[test]
fn cry_engine_zero_to_one_z_range() {
    let proj = cry::calc_perspective_projection_matrix(90.0, 16.0 / 9.0, 0.1, 1000.0, NdcDepthRange::ZeroToOne);

    expect_near!(project_z_lh(&proj, 0.1), 0.0, 1e-4);
    expect_near!(project_z_lh(&proj, 1000.0), 1.0, 1e-4);
    assert!(project_z_lh(&proj, 500.0) > 0.0);
    assert!(project_z_lh(&proj, 500.0) < 1.0);
}

#[test]
fn source_negative_one_to_one_z_range() {
    let proj =
        source::calc_perspective_projection_matrix(90.0, 16.0 / 9.0, 0.1, 1000.0, NdcDepthRange::NegativeOneToOne);

    expect_near!(project_z_lh(&proj, 0.1), -1.0, 1e-3);
    expect_near!(project_z_lh(&proj, 1000.0), 1.0, 1e-3);
}

#[test]
fn iw_negative_one_to_one_z_range() {
    let proj = iw::calc_perspective_projection_matrix(90.0, 16.0 / 9.0, 0.1, 1000.0, NdcDepthRange::NegativeOneToOne);

    expect_near!(project_z_lh(&proj, 0.1), -1.0, 1e-3);
    expect_near!(project_z_lh(&proj, 1000.0), 1.0, 1e-3);
}

#[test]
fn frostbite_negative_one_to_one_z_range() {
    let proj =
        frostbite::calc_perspective_projection_matrix(90.0, 16.0 / 9.0, 0.1, 1000.0, NdcDepthRange::NegativeOneToOne);

    expect_near!(project_z_lh(&proj, 0.1), -1.0, 1e-3);
    expect_near!(project_z_lh(&proj, 1000.0), 1.0, 1e-3);
}

#[test]
fn unreal_negative_one_to_one_z_range() {
    let proj =
        unreal::calc_perspective_projection_matrix(90.0, 16.0 / 9.0, 0.1, 1000.0, NdcDepthRange::NegativeOneToOne);

    expect_near!(project_z_lh(&proj, 0.1), -1.0, 1e-3);
    expect_near!(project_z_lh(&proj, 1000.0), 1.0, 1e-3);
}

#[test]
fn cry_engine_negative_one_to_one_z_range() {
    let proj = cry::calc_perspective_projection_matrix(90.0, 16.0 / 9.0, 0.1, 1000.0, NdcDepthRange::NegativeOneToOne);

    expect_near!(project_z_lh(&proj, 0.1), -1.0, 1e-3);
    expect_near!(project_z_lh(&proj, 1000.0), 1.0, 1e-3);
}

#[test]
fn opengl_negative_one_to_one_z_range() {
    let proj =
        opengl::calc_perspective_projection_matrix(90.0, 16.0 / 9.0, 0.1, 1000.0, NdcDepthRange::NegativeOneToOne);

    let proj_z = |z: f32| {
        let clip = proj * Mat::<4, 1, f32, ColumnMajor>::column_from_vector(Vector3::new(0.0, 0.0, z));
        return clip.at(2, 0) / clip.at(3, 0);
    };

    expect_near!(proj_z(-0.1), -1.0, 1e-3);
    expect_near!(proj_z(-1000.0), 1.0, 1e-3);
}

#[test]
fn unity_negative_one_to_one_z_range() {
    let proj =
        unity::calc_perspective_projection_matrix(90.0, 16.0 / 9.0, 0.1, 1000.0, NdcDepthRange::NegativeOneToOne);

    let proj_z = |z: f32| {
        let clip = proj * Mat::<4, 1, f32, RowMajor>::column_from_vector(Vector3::new(0.0, 0.0, z));
        return clip.at(2, 0) / clip.at(3, 0);
    };

    expect_near!(proj_z(-0.1), -1.0, 1e-3);
    expect_near!(proj_z(-1000.0), 1.0, 1e-3);
}

/// The mesh and camera half of the C++ `*_Pred_And_Mesh_And_Camera` tests. The prediction half goes with the pred engine
/// traits.
macro_rules! mesh_and_camera_test {
    ($name:ident, $engine:ident, $fov:expr, $width:expr, $height:expr) => {
        #[test]
        fn $name() {
            let va = $engine::ViewAngles::default();
            expect_matrix_near(&$engine::MeshTrait::rotation_matrix(&va), &$engine::rotation_matrix(va), 1e-5);

            let fov = projection::FieldOfView::from_degrees($fov);
            let view_port = ViewPort::new($width, $height);
            let aspect_ratio: f32 = $width / $height;
            let near = 0.1_f32.into();
            let far = 1000.0_f32.into();

            let proj = $engine::CameraTrait::calc_projection_matrix(
                fov,
                view_port,
                near,
                far,
                NdcDepthRange::NegativeOneToOne,
            );
            let expected = $engine::calc_perspective_projection_matrix(
                $fov.into(),
                aspect_ratio.into(),
                near,
                far,
                $engine::K_DEFAULT_NDC_DEPTH_RANGE,
            );
            expect_matrix_near(&proj, &expected, 1e-5);

            let proj_zo =
                $engine::CameraTrait::calc_projection_matrix(fov, view_port, near, far, NdcDepthRange::ZeroToOne);
            let expected_zo = $engine::calc_perspective_projection_matrix(
                $fov.into(),
                aspect_ratio.into(),
                near,
                far,
                NdcDepthRange::ZeroToOne,
            );
            expect_matrix_near(&proj_zo, &expected_zo, 1e-5);
            assert_ne!(proj, proj_zo);
        }
    };
}

mesh_and_camera_test!(iw_pred_and_mesh_and_camera, iw, 45.0_f32, 1920.0_f32, 1080.0_f32);
mesh_and_camera_test!(opengl_pred_and_mesh_and_camera, opengl, 60.0_f32, 1280.0_f32, 720.0_f32);
mesh_and_camera_test!(unity_pred_and_mesh_and_camera, unity, 60.0_f32, 1280.0_f32, 720.0_f32);
mesh_and_camera_test!(unreal_pred_and_mesh_and_camera, unreal, 60.0_f32, 1280.0_f32, 720.0_f32);
