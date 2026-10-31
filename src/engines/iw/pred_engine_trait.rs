use super::constants::*;
use super::formulas::{forward_vector, rotation_matrix};
use crate::Vector3;
use crate::trigonometry::angles;

engine_pred_trait! {
    f32, height: z, ground: (x, y),
    view_pitch(pitch) => -pitch;
    direct_pitch(origin, view_to) => {
        let distance = origin.distance_to(view_to);
        let delta = view_to - origin;

        angles::radians_to_degrees((delta.z / distance).asin())
    };
    direct_yaw(origin, view_to) => {
        let delta = view_to - origin;

        angles::radians_to_degrees(delta.y.atan2(delta.x))
    };
}
