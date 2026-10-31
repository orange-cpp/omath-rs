use super::constants::*;
use super::formulas::{forward_vector, rotation_matrix};
use crate::Vector3;
use crate::trigonometry::angles;

engine_pred_trait! {
    f32, height: y, ground: (x, z),
    view_pitch(pitch) => pitch;
    direct_pitch(origin, view_to) => {
        let direction = (view_to - origin).normalized();

        angles::radians_to_degrees(direction.y.asin())
    };
    direct_yaw(origin, view_to) => {
        let direction = (view_to - origin).normalized();

        angles::radians_to_degrees(-direction.x.atan2(-direction.z))
    };
}
