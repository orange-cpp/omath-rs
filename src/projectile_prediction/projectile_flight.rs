use super::{AimAngles, Launcher, PredEngine, Projectile};
use crate::Vector3;
use crate::engines::source;
use crate::scalar::{Real, Scalar, Signed};
use crate::trigonometry::angles;

/// One round in flight, advanced a physics step at a time. Drag has no closed form, so a round that has it can only be
/// followed the way the game's physics moves it: every step adds gravity, then scales the whole velocity down by the
/// drag for that step, then moves, in that order and without sub-steps. That is what Source's VPhysics does to a
/// grenade, and stepping at its tick interval follows the real thing to within a couple of units over a two second
/// flight, where a parabola is hundreds out by then.
///
/// A round without drag is a parabola, which [`PredEngine::predict_projectile_position`] reads off directly.
#[derive(Clone, Copy, Debug)]
pub struct ProjectileFlight<E: PredEngine = source::PredEngineTrait> {
    projectile: Projectile<E::Scalar>,
    gravity: E::Scalar,
    time_step: E::Scalar,

    origin: Vector3<E::Scalar>,
    position: Vector3<E::Scalar>,
    velocity: Vector3<E::Scalar>,

    // The round's own axes. It is spawned facing along the view and nothing in flight turns it, so for a round that
    // does not tumble these stay where they were at launch.
    forward: Vector3<E::Scalar>,
    up: Vector3<E::Scalar>,

    world_up: Vector3<E::Scalar>,

    step_count: usize,
}

impl<E: PredEngine> ProjectileFlight<E> {
    /// `view_angles` are the shooter's, pitch positive upwards: the muzzle is placed from them and the round leaves
    /// `launcher.launch_pitch_offset` above them. `gravity_constant` is the world's, scaled here by the round's own
    /// [`Projectile::gravity_scale`]. `time_step` is the physics step of the game being predicted, not an accuracy
    /// setting: the arc only matches when it is stepped the way the game steps it.
    pub fn new(
        projectile: &Projectile<E::Scalar>,
        launcher: &Launcher<E::Scalar>,
        view_angles: AimAngles<E::Scalar>,
        gravity_constant: E::Scalar,
        time_step: E::Scalar,
    ) -> Self {
        let basis = E::calc_view_basis(view_angles.pitch, view_angles.yaw);
        let origin = launcher.launch_origin(basis);

        // forward * a + up * b in the view frame, written back out of the speed and offset they are kept as
        let offset = angles::degrees_to_radians(launcher.launch_pitch_offset);

        let mut flight = Self {
            projectile: *projectile,
            gravity: gravity_constant * projectile.gravity_scale,
            time_step,
            origin,
            position: origin,
            velocity: basis.forward * (projectile.launch_speed * offset.cos())
                + basis.up * (projectile.launch_speed * offset.sin()),
            forward: basis.forward,
            up: basis.up,
            // A level view's up is the world's, whichever axis the engine keeps it on
            world_up: E::calc_view_basis(E::Scalar::ZERO, E::Scalar::ZERO).up,
            step_count: 0,
        };

        flight.clamp_speed();
        flight
    }

    pub fn step(&mut self) {
        self.velocity = self.velocity - self.world_up * (self.gravity * self.time_step);

        let rate = self.projectile.drag * self.velocity.length()
            + self.projectile.drag_forward * self.velocity.dot(self.forward).abs_value()
            + self.projectile.drag_up * self.velocity.dot(self.up).abs_value();

        // A step cannot take more than the whole velocity
        self.velocity = self.velocity * (E::Scalar::ONE - E::Scalar::ONE.min_of(rate * self.time_step));

        self.clamp_speed();

        self.position = self.position + self.velocity * self.time_step;
        self.step_count += 1;
    }

    /// Where the round left from: the muzzle, not the eye.
    pub fn origin(&self) -> Vector3<E::Scalar> {
        self.origin
    }

    pub fn position(&self) -> Vector3<E::Scalar> {
        self.position
    }

    pub fn velocity(&self) -> Vector3<E::Scalar> {
        self.velocity
    }

    /// Step times count rather than a running sum, which drifts.
    pub fn time(&self) -> E::Scalar {
        self.time_step * E::Scalar::from_f64(self.step_count as f64)
    }

    fn clamp_speed(&mut self) {
        let max_speed = self.projectile.max_speed;

        if !(max_speed > E::Scalar::ZERO) {
            return;
        }

        let speed = self.velocity.length();

        if speed > max_speed {
            self.velocity = self.velocity * (max_speed / speed);
        }
    }
}
