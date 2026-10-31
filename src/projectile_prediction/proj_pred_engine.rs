use super::{Launcher, Projectile, Target};
use crate::Vector3;
use crate::scalar::Real;

/// View angles in degrees. Pitch is positive upwards whatever the engine's own sign convention is.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AimAngles<T> {
    pub pitch: T,
    pub yaw: T,
}

/// Which of the two pitches that put a round on a point an engine solves for. A round with gravity gets there either
/// flat and quick, or lobbed steeply and dropping onto it from above; the flat one is what an engine aims with unless
/// told otherwise. A round without gravity has one straight line and the choice does not apply.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Arc {
    #[default]
    Low,
    High,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AimSolution<T> {
    /// View angles to set on the eye. The round leaves along them, raised by
    /// [`Launcher::launch_pitch_offset`] if the weapon has one, so the launch pitch is
    /// `angles.pitch + launcher.launch_pitch_offset`.
    pub angles: AimAngles<T>,
    /// A point on the eye's aim ray, as far away as the predicted target. Any point on that ray projects to the same
    /// pixel, so this is where a crosshair or marker belongs on screen.
    pub aim_point: Vector3<T>,
    pub predicted_target_position: Vector3<T>,
    pub time_of_flight: T,
}

/// What the compatibility wrappers of [`ProjPredEngineInterface`] solve with: the eye from
/// [`Projectile::origin`] and [`Projectile::launch_offset`] as a fixed world-space offset, which is how the engines
/// behaved before [`Launcher`] existed.
pub fn launcher_from_projectile<T: Real>(projectile: &Projectile<T>) -> Launcher<T> {
    Launcher { eye_origin: projectile.origin, world_offset: projectile.launch_offset, ..Launcher::default() }
}

/// A projectile prediction solver.
pub trait ProjPredEngineInterface<T: Real> {
    fn maybe_calculate_aim(
        &self,
        projectile: &Projectile<T>,
        launcher: &Launcher<T>,
        target: &Target<T>,
    ) -> Option<AimSolution<T>>;

    /// Compatibility wrapper over [`maybe_calculate_aim`](Self::maybe_calculate_aim), see
    /// [`launcher_from_projectile`]. Kept for one release; new code should call it with a [`Launcher`].
    fn maybe_calculate_aim_point(&self, projectile: &Projectile<T>, target: &Target<T>) -> Option<Vector3<T>> {
        let solution = self.maybe_calculate_aim(projectile, &launcher_from_projectile(projectile), target)?;

        Some(solution.aim_point)
    }

    /// Compatibility wrapper over [`maybe_calculate_aim`](Self::maybe_calculate_aim), see
    /// [`launcher_from_projectile`]. Kept for one release; new code should call it with a [`Launcher`].
    fn maybe_calculate_aim_angles(&self, projectile: &Projectile<T>, target: &Target<T>) -> Option<AimAngles<T>> {
        let solution = self.maybe_calculate_aim(projectile, &launcher_from_projectile(projectile), target)?;

        Some(solution.angles)
    }
}
