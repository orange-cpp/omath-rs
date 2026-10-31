use super::{Angle, AngleRange};
use crate::linear_algebra::Vector3;
use crate::scalar::Real;

/// Pitch, yaw and roll, each with its own range.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ViewAngles<T: Real, P: AngleRange, Y: AngleRange, R: AngleRange> {
    pub pitch: Angle<T, P>,
    pub yaw: Angle<T, Y>,
    pub roll: Angle<T, R>,
}

impl<T: Real, P: AngleRange, Y: AngleRange, R: AngleRange> ViewAngles<T, P, Y, R> {
    #[inline]
    pub fn new(pitch: Angle<T, P>, yaw: Angle<T, Y>, roll: Angle<T, R>) -> Self {
        Self { pitch, yaw, roll }
    }

    /// `(pitch, yaw, roll)` in degrees.
    #[inline]
    pub fn as_vector3(&self) -> Vector3<T> {
        Vector3::new(
            self.pitch.as_degrees(),
            self.yaw.as_degrees(),
            self.roll.as_degrees(),
        )
    }
}
