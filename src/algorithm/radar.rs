use crate::linear_algebra::{Vector2, Vector3};
use crate::projection::{Camera, CameraEngine};
use crate::scalar::Scalar;
use crate::trigonometry::Angle;

/// Maps a world position to a radar offset around the camera, with `+y` pointing behind it: straight ahead comes out at
/// `(0, -distance * scale)`.
///
/// `calc_distance` replaces the Euclidean distance from the camera, for example to ignore height.
pub type DistanceFunction<'a, T> = dyn Fn(Vector3<T>, Vector3<T>) -> T + 'a;

pub fn world_to_radar<E: CameraEngine>(
    camera: &Camera<E>,
    position: Vector3<E::Scalar>,
    scale: E::Scalar,
    calc_distance: Option<&DistanceFunction<'_, E::Scalar>>,
) -> Vector2<f32> {
    let quarter_turn = Angle::<E::Scalar, E::YawRange>::from_degrees(E::Scalar::from_f64(90.0));
    let current_yaw = camera.get_view_angles().yaw;
    let look_at_yaw = camera.calc_look_at_angles(position).yaw;

    // Engines disagree on which way yaw turns. The C++ keeps this in a function-local `static`, which fixes it to the
    // first camera seen; it depends on the engine and on the current angles, so it is computed on each call.
    let right_yaw = current_yaw
        - camera
            .calc_look_at_angles(camera.get_origin() + camera.get_abs_right())
            .yaw
        - quarter_turn;
    let sign = if right_yaw.cos() < E::Scalar::ZERO {
        -1.0_f32
    } else {
        1.0
    };

    let yaw = current_yaw - look_at_yaw - quarter_turn;

    let distance = match calc_distance {
        Some(calc_distance) => calc_distance(camera.get_origin(), position),
        None => camera.get_origin().distance_to(position),
    };

    Vector2::new(yaw.cos().to_f64() as f32 * sign, yaw.sin().to_f64() as f32)
        * (distance * scale).to_f64() as f32
}
