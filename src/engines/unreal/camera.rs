use super::constants::*;
use super::formulas;
use crate::engines::Clamped90;
use crate::projection::FieldOfView;
use crate::{RowMajor, Vector3};

engine_camera! {
    f64, RowMajor, Clamped90, ZeroToOne,
    look_at(direction) => ViewAngles::new(
        PitchAngle::from_asin(direction.z),
        YawAngle::from_atan2(direction.y, direction.x),
        RollAngle::from_radians(0.0),
    );

    /// The FOV is horizontal: `m[0,0] == 1 / tan(fov / 2)`.
    fn calc_field_of_view(projection_matrix: &Mat4X4) -> FieldOfView {
        FieldOfView::from_radians((2.0 * (1.0 / projection_matrix.at(0, 0)).atan()) as f32)
    }
}
