use crate::Vector3;
use crate::scalar::Scalar;

/// Physical properties of the round. Where it is fired from is described by [`Launcher`](super::Launcher).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Projectile<T> {
    /// Read only by the compatibility wrappers of [`ProjPredEngineInterface`](super::ProjPredEngineInterface): this is
    /// the eye. New code leaves it default and passes a [`Launcher`](super::Launcher) instead.
    pub origin: Vector3<T>,
    /// A fixed world-space spawn offset, with the same caveat as `origin`.
    pub launch_offset: Vector3<T>,

    pub launch_speed: T,
    pub gravity_scale: T,

    /// Air drag, in 1 / distance. Only [`ProjPredEngineDrag`](super::ProjPredEngineDrag) and
    /// [`ProjectileFlight`](super::ProjectileFlight) fly a round with it: the closed-form engines assume a parabola and
    /// ignore these. Each simulation step takes `dt * (drag * |v| + drag_forward * |v . F| + drag_up * |v . U|)` off
    /// the velocity, with F and U the round's own forward and up axes, which are those of the view it was fired from.
    ///
    /// `drag` is for a round that tumbles, showing the air every side in turn, which averages out to one number.
    pub drag: T,
    /// With `drag_up`, for a round that holds its attitude, where end-on and side-on drag differ and the arc bending
    /// away from the nose moves the round from one to the other. There is no third axis because a round fired without
    /// roll never moves along its own right.
    pub drag_forward: T,
    pub drag_up: T,

    /// Speed the physics engine caps the round at, on launch and again after every step (Source's VPhysics stops at
    /// 2000 units/s). Zero means there is no cap. Read by the same two types as the drag.
    pub max_speed: T,
}

impl<T: Scalar> Projectile<T> {
    pub fn has_drag(&self) -> bool {
        self.drag > T::ZERO || self.drag_forward > T::ZERO || self.drag_up > T::ZERO
    }
}
