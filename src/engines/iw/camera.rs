use super::constants::*;
use super::formulas;
use crate::engines::Clamped89;
use crate::projection::FieldOfView;
use crate::trigonometry::angles;
use crate::{RowMajor, Vector3};

engine_camera! {
    f32, RowMajor, Clamped89, ZeroToOne,
    look_at(direction) => ViewAngles::new(
        -PitchAngle::from_asin(direction.z),
        YawAngle::from_atan2(direction.y, direction.x),
        RollAngle::from_radians(0.0),
    );

    /// The FOV is horizontal at a 4:3 reference aspect ratio (see `calc_perspective_projection_matrix`), so recover the
    /// vertical FOV from `m[1,1]` and widen it back to 4:3.
    fn calc_field_of_view(projection_matrix: &Mat4X4) -> FieldOfView {
        let vertical_fov = angles::radians_to_degrees(2.0 * (1.0 / projection_matrix.at(1, 1)).atan());
        FieldOfView::from_degrees(angles::vertical_fov_to_horizontal(vertical_fov, K_FOV_REFERENCE_ASPECT_RATIO))
    }
}
