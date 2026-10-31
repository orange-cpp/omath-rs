use crate::Vector3;
use crate::scalar::Scalar;

/// Where the muzzle sits relative to the eye, in the shooter's own view frame. Engines spawn a projectile at
/// `eye + forward * a + right * b + up * c`, so its world-space position rotates with the view angles.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MuzzleOffset<T> {
    pub forward: T,
    pub right: T,
    pub up: T,
}

impl<T: Scalar> MuzzleOffset<T> {
    pub fn is_zero(&self) -> bool {
        self.forward == T::ZERO && self.right == T::ZERO && self.up == T::ZERO
    }
}

/// Forward, right and up of a view frame for a given pitch and yaw, expressed in the engine's world axes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ViewBasis<T> {
    pub forward: Vector3<T>,
    pub right: Vector3<T>,
    pub up: Vector3<T>,
}

/// Who fires: the eye the view angles apply to, and where the projectile leaves relative to it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Launcher<T> {
    pub eye_origin: Vector3<T>,
    pub muzzle_offset: MuzzleOffset<T>,

    /// Fixed world-space part of the spawn offset, for engines whose offset does not rotate with the view. The
    /// compatibility wrappers of [`ProjPredEngineInterface`](super::ProjPredEngineInterface) put the old
    /// [`Projectile::launch_offset`](super::Projectile::launch_offset) here.
    pub world_offset: Vector3<T>,

    /// Degrees by which the round leaves above the view direction (negative: below). Some weapons launch at
    /// `forward * a + up * b` in the view frame instead of straight along the crosshair, TF2's pipe and sticky
    /// launchers being the usual example with b = 200. Forward and up span the view's vertical plane, so that is
    /// exactly a pitch offset of `atan(b / a)` with [`Projectile::launch_speed`](super::Projectile::launch_speed) =
    /// `hypot(a, b)`: 1200 and 200 give 9.46 degrees at 1216.6 units/s. The engines solve the launch pitch and report
    /// the view pitch, which is the launch pitch minus this.
    pub launch_pitch_offset: T,
}

impl<T: Scalar> Launcher<T> {
    pub fn launch_origin(&self, basis: ViewBasis<T>) -> Vector3<T> {
        self.eye_origin
            + self.world_offset
            + basis.forward * self.muzzle_offset.forward
            + basis.right * self.muzzle_offset.right
            + basis.up * self.muzzle_offset.up
    }
}
