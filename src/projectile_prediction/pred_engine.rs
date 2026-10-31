use super::{Projectile, Target, ViewBasis};
use crate::Vector3;
use crate::scalar::Real;

/// How a game engine moves projectiles and measures angles: the axes, the sign of the pitch and the unit of gravity.
///
/// All pitches here are positive upwards, whatever the engine's own convention is. Every engine module has an
/// implementation called `PredEngineTrait`.
pub trait PredEngine {
    type Scalar: Real;

    fn predict_projectile_position(
        launch_origin: Vector3<Self::Scalar>,
        projectile: &Projectile<Self::Scalar>,
        pitch: Self::Scalar,
        yaw: Self::Scalar,
        time: Self::Scalar,
        gravity: Self::Scalar,
    ) -> Vector3<Self::Scalar>;

    fn predict_target_position(
        target: &Target<Self::Scalar>,
        time: Self::Scalar,
        gravity: Self::Scalar,
    ) -> Vector3<Self::Scalar>;

    fn calc_vector_2d_distance(delta: Vector3<Self::Scalar>) -> Self::Scalar;

    fn get_vector_height_coordinate(vec: Vector3<Self::Scalar>) -> Self::Scalar;

    fn calc_view_basis(pitch: Self::Scalar, yaw: Self::Scalar) -> ViewBasis<Self::Scalar>;

    fn calc_direct_pitch_angle(
        origin: Vector3<Self::Scalar>,
        view_to: Vector3<Self::Scalar>,
    ) -> Self::Scalar;

    fn calc_direct_yaw_angle(
        origin: Vector3<Self::Scalar>,
        view_to: Vector3<Self::Scalar>,
    ) -> Self::Scalar;
}
