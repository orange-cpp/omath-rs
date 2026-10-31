use super::constants::*;
use super::formulas;
use crate::engines::Clamped90;
use crate::projection::CameraAxes;
use crate::{ColumnMajor, Vector3};

engine_camera! {
    f32, ColumnMajor, Clamped90, NegativeOneToOne,
    look_at(direction) => ViewAngles::new(
        PitchAngle::from_asin(direction.y),
        -YawAngle::from_atan2(direction.x, -direction.z),
        RollAngle::from_radians(0.0),
    );

    const AXES: CameraAxes = CameraAxes { inverted_forward: true, inverted_right: false };
}
