use super::constants::*;
use crate::{NdcDepthRange, Vector3};

/// yaw * pitch * roll: roll turns around the camera's own forward axis.
pub fn rotation_matrix(angles: ViewAngles) -> Mat4X4 {
    Mat4X4::rotation_zxy(angles.yaw, angles.pitch, angles.roll)
}

engine_formulas!(f32);

pub fn extract_rotation_angles(mat: &Mat4X4) -> ViewAngles {
    let angles = mat.extract_rotation_zxy();
    ViewAngles::new(
        PitchAngle::from_degrees(angles.x),
        YawAngle::from_degrees(angles.z),
        RollAngle::from_degrees(angles.y),
    )
}

pub fn calc_view_matrix(angles: ViewAngles, cam_origin: Vector3<f32>) -> Mat4X4 {
    // Build the rotation once, instead of once per basis vector.
    let rotation = rotation_matrix(angles);

    Mat4X4::camera_view(
        rotation.rotate_vector(K_ABS_FORWARD),
        rotation.rotate_vector(K_ABS_RIGHT),
        rotation.rotate_vector(K_ABS_UP),
        cam_origin,
    )
}

/// The C++ default for `ndc_depth_range` is [`K_DEFAULT_NDC_DEPTH_RANGE`].
pub fn calc_perspective_projection_matrix(
    field_of_view: f32,
    aspect_ratio: f32,
    near_plane: f32,
    far_plane: f32,
    ndc_depth_range: NdcDepthRange,
) -> Mat4X4 {
    Mat4X4::perspective_left_handed_vertical_fov(field_of_view, aspect_ratio, near_plane, far_plane, ndc_depth_range)
}

engine_units!(rage);
