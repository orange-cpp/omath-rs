use super::constants::*;
use super::formulas;
use crate::engines::Clamped90;
use crate::{RowMajor, Vector3};

engine_camera! {
    f32, RowMajor, Clamped90, ZeroToOne,
    look_at(direction) => ViewAngles::new(
        -PitchAngle::from_asin(direction.y),
        YawAngle::from_atan2(direction.x, direction.z),
        RollAngle::from_radians(0.0),
    );
}
