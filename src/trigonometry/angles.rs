//! Free angle helpers.

use crate::scalar::Real;

#[inline]
pub fn radians_to_degrees<T: Real>(radians: T) -> T {
    radians * (T::from_f64(180.0) / T::PI)
}

#[inline]
pub fn degrees_to_radians<T: Real>(degrees: T) -> T {
    degrees * (T::PI / T::from_f64(180.0))
}

pub fn horizontal_fov_to_vertical<T: Real>(horizontal_fov: T, aspect: T) -> T {
    let two = T::from_f64(2.0);
    let half = degrees_to_radians(horizontal_fov) / two;
    radians_to_degrees(two * (half.tan() / aspect).atan())
}

pub fn vertical_fov_to_horizontal<T: Real>(vertical_fov: T, aspect: T) -> T {
    let two = T::from_f64(2.0);
    let half = degrees_to_radians(vertical_fov) / two;
    radians_to_degrees(two * (half.tan() * aspect).atan())
}

/// Wraps `angle` into `[min, max)`.
pub fn wrap_angle<T: Real>(angle: T, min: T, max: T) -> T {
    if angle < max && angle >= min {
        return angle;
    }

    let range = max - min;
    let mut wrapped = (angle - min) % range;

    if wrapped < T::ZERO {
        wrapped += range;
    }

    wrapped + min
}
