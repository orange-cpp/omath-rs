use super::constants::*;
use super::formulas::{forward_vector, rotation_matrix};
use crate::Vector3;
use crate::trigonometry::angles;

engine_pred_trait! {
    f64, height: z, ground: (x, y),
    view_pitch(pitch) => pitch;
    direct_pitch(origin, view_to) => {
        let direction = (view_to - origin).normalized();

        angles::radians_to_degrees(direction.z.asin())
    };
    direct_yaw(origin, view_to) => {
        let direction = (view_to - origin).normalized();

        angles::radians_to_degrees(direction.y.atan2(direction.x))
    };
}
