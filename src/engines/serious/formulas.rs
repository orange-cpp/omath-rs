use super::constants::*;
use crate::{NdcDepthRange, Vector3};

/// Heading (Y) * pitch (X) * banking (Z), entry for entry the same as `MakeRotationMatrix()` in Serious Engine's
/// `Sources/Engine/Math/Geometry.cpp`. Banking acts around the camera's own forward axis.
pub fn rotation_matrix(angles: ViewAngles) -> Mat4X4 {
    Mat4X4::rotation_yxz(angles.yaw, angles.pitch, angles.roll)
}

engine_formulas!(f32);

/// At pitch +/-90 heading and banking turn around the same axis; banking is then pinned to 0.
pub fn extract_rotation_angles(mat: &Mat4X4) -> ViewAngles {
    let angles = mat.extract_rotation_yxz();
    ViewAngles::new(
        PitchAngle::from_degrees(angles.x),
        YawAngle::from_degrees(angles.y),
        RollAngle::from_degrees(angles.z),
    )
}

/// The rows are right, up and back (-forward), the same layout the engine keeps its view matrix in.
pub fn calc_view_matrix(angles: ViewAngles, cam_origin: Vector3<f32>) -> Mat4X4 {
    // Build the rotation once, instead of once per basis vector.
    let rotation = rotation_matrix(angles);

    Mat4X4::camera_view(
        -rotation.rotate_vector(K_ABS_FORWARD),
        rotation.rotate_vector(K_ABS_RIGHT),
        rotation.rotate_vector(K_ABS_UP),
        cam_origin,
    )
}

/// The engine's FOV is horizontal (`ppr_FOVWidth` in `Sources/Engine/Math/Projection_Perspective.cpp`): the X scale is
/// `1 / tan(fov / 2)` and the Y scale follows from the aspect ratio.
///
/// The C++ default for `ndc_depth_range` is [`K_DEFAULT_NDC_DEPTH_RANGE`].
pub fn calc_perspective_projection_matrix(
    field_of_view: f32,
    aspect_ratio: f32,
    near_plane: f32,
    far_plane: f32,
    ndc_depth_range: NdcDepthRange,
) -> Mat4X4 {
    Mat4X4::perspective_right_handed_horizontal_fov(field_of_view, aspect_ratio, near_plane, far_plane, ndc_depth_range)
}

engine_units!(metric);
