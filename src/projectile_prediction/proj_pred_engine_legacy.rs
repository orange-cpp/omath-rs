use super::{AimAngles, AimSolution, Arc, Launcher, PredEngine, ProjPredEngineInterface, Projectile, Target};
use crate::Vector3;
use crate::engines::source;
use crate::scalar::{Real, Scalar, Signed};
use crate::trigonometry::angles;
use std::marker::PhantomData;

/// Scans the flight time a step at a time and solves the launch pitch in closed form at each.
#[derive(Clone, Copy, Debug)]
pub struct ProjPredEngineLegacy<E: PredEngine = source::PredEngineTrait> {
    gravity_constant: E::Scalar,
    simulation_time_step: E::Scalar,
    maximum_simulation_time: E::Scalar,
    distance_tolerance: E::Scalar,
    arc: Arc,
    engine: PhantomData<E>,
}

struct Solution<T> {
    predicted_target_position: Vector3<T>,
    /// View angles: the launch pitch with the launcher's pitch offset already taken off.
    pitch: T,
    yaw: T,
    time: T,
}

/// Everything the per-step solve needs that does not depend on time. The scan runs up to
/// `maximum_simulation_time / simulation_time_step` steps, so these are worth computing once.
struct LaunchContext<'a, T> {
    launcher: &'a Launcher<T>,
    /// eye + world offset: the launch origin whenever the muzzle offset is zero
    fixed_origin: Vector3<T>,
    gravity: T,
    speed_sqr: T,
    speed_pow4: T,
}

impl<E: PredEngine> ProjPredEngineLegacy<E> {
    /// Solves for [`Arc::Low`], unless [`with_arc`](Self::with_arc) says otherwise.
    pub fn new(
        gravity_constant: E::Scalar,
        simulation_time_step: E::Scalar,
        maximum_simulation_time: E::Scalar,
        distance_tolerance: E::Scalar,
    ) -> Self {
        Self {
            gravity_constant,
            simulation_time_step,
            maximum_simulation_time,
            distance_tolerance,
            arc: Arc::Low,
            engine: PhantomData,
        }
    }

    #[must_use]
    pub fn with_arc(mut self, arc: Arc) -> Self {
        self.arc = arc;
        self
    }

    fn find_solution(
        &self,
        projectile: &Projectile<E::Scalar>,
        launcher: &Launcher<E::Scalar>,
        target: &Target<E::Scalar>,
    ) -> Option<Solution<E::Scalar>> {
        // A non-positive step or horizon has no scan to run.
        if !(self.simulation_time_step > E::Scalar::ZERO) || !(self.maximum_simulation_time > E::Scalar::ZERO) {
            return None;
        }

        let speed_sqr = projectile.launch_speed * projectile.launch_speed;
        let launch = LaunchContext {
            launcher,
            fixed_origin: launcher.eye_origin + launcher.world_offset,
            gravity: self.gravity_constant * projectile.gravity_scale,
            speed_sqr,
            speed_pow4: speed_sqr * speed_sqr,
        };

        // Two instantiations so the common fixed-muzzle case keeps a tight loop: no muzzle placement, and a pitch
        // solve small enough to stay inlined in the scan.
        if launcher.muzzle_offset.is_zero() {
            self.scan::<false>(&launch, projectile, target)
        } else {
            self.scan::<true>(&launch, projectile, target)
        }
    }

    /// With a view-relative muzzle offset the launch origin depends on the very angles being solved for, so the muzzle
    /// is first placed from the direct angles to the target and the solve is repeated once from where it ends up. One
    /// pass is enough: the offset is a few units against a target hundreds away, so the second correction is far below
    /// the step tolerance.
    ///
    /// `pitch` below is always the launch pitch, the direction the round leaves in. The muzzle turns with the view, not
    /// with the round, so every basis is built from the view pitch: launch pitch minus the launcher's offset.
    fn scan<const MUZZLE_ROTATES: bool>(
        &self,
        launch: &LaunchContext<'_, E::Scalar>,
        projectile: &Projectile<E::Scalar>,
        target: &Target<E::Scalar>,
    ) -> Option<Solution<E::Scalar>> {
        // time = step * index rather than time += step: repeated addition drifts (a 1 ms step over 50 s lands 16 steps
        // and 16 ms off), and the count below is exactly what the parameters say.
        let step_count = (self.maximum_simulation_time / self.simulation_time_step).ceil().to_f64() as usize;
        let launcher = launch.launcher;
        let pitch_offset = launcher.launch_pitch_offset;

        for step in 0..step_count {
            let time = self.simulation_time_step * E::Scalar::from_f64(step as f64);
            let predicted_target_position = E::predict_target_position(target, time, self.gravity_constant);

            let mut origin = launch.fixed_origin;
            if MUZZLE_ROTATES {
                origin = launcher.launch_origin(E::calc_view_basis(
                    E::calc_direct_pitch_angle(launcher.eye_origin, predicted_target_position) - pitch_offset,
                    E::calc_direct_yaw_angle(launcher.eye_origin, predicted_target_position),
                ));
            }

            let Some(mut pitch) =
                self.maybe_calculate_projectile_launch_pitch_angle(launch, origin, predicted_target_position)
            else {
                continue;
            };

            let mut yaw = E::calc_direct_yaw_angle(origin, predicted_target_position);

            if MUZZLE_ROTATES {
                origin = launcher.launch_origin(E::calc_view_basis(pitch - pitch_offset, yaw));

                let Some(refined_pitch) =
                    self.maybe_calculate_projectile_launch_pitch_angle(launch, origin, predicted_target_position)
                else {
                    continue;
                };

                pitch = refined_pitch;
                yaw = E::calc_direct_yaw_angle(origin, predicted_target_position);
            }

            if !self.is_projectile_reached_target(origin, predicted_target_position, projectile, pitch, yaw, time) {
                continue;
            }

            let view_pitch = pitch - pitch_offset;
            if pitch_offset != E::Scalar::ZERO && !Self::is_view_pitch_reachable(view_pitch, yaw) {
                continue;
            }

            return Some(Solution { predicted_target_position, pitch: view_pitch, yaw, time });
        }

        None
    }

    /// A pitch offset can put the view pitch that produces the launch outside what the engine lets a player look at
    /// (Source stops at +-89 degrees). `calc_view_basis` clamps the way the engine does, so a forward vector that no
    /// longer has the requested pitch means nobody can set this shot up. Without an offset the launch pitch is the view
    /// pitch, and the reach check already flies it through the same clamp.
    fn is_view_pitch_reachable(view_pitch: E::Scalar, yaw: E::Scalar) -> bool {
        let tolerance_degrees = E::Scalar::from_f64(0.1);

        let forward = E::calc_view_basis(view_pitch, yaw).forward;
        let reachable_pitch = E::calc_direct_pitch_angle(Vector3::default(), forward);

        (reachable_pitch - view_pitch).abs_value() <= tolerance_degrees
    }

    /// `theta = atan((v^2 -+ sqrt(v^4 - g * (g * x^2 + 2 * y * v^2))) / (g * x))`, minus the root for the low arc and
    /// plus it for the high one, from
    /// <https://stackoverflow.com/questions/54917375/how-to-calculate-the-angle-to-shoot-a-bullet-in-order-to-hit-a-moving-target>
    fn maybe_calculate_projectile_launch_pitch_angle(
        &self,
        launch: &LaunchContext<'_, E::Scalar>,
        launch_origin: Vector3<E::Scalar>,
        target_position: Vector3<E::Scalar>,
    ) -> Option<E::Scalar> {
        if launch.gravity == E::Scalar::ZERO {
            return Some(E::calc_direct_pitch_angle(launch_origin, target_position));
        }

        let delta = target_position - launch_origin;

        let distance2d = E::calc_vector_2d_distance(delta);
        let distance2d_sqr = distance2d * distance2d;

        let root = launch.speed_pow4
            - launch.gravity
                * (launch.gravity * distance2d_sqr
                    + E::Scalar::from_f64(2.0) * E::get_vector_height_coordinate(delta) * launch.speed_sqr);

        if root < E::Scalar::ZERO {
            return None;
        }

        // Straight up or down: the formula divides by g x. The direct angle is the only launch direction anyway.
        if distance2d == E::Scalar::ZERO {
            return Some(E::calc_direct_pitch_angle(launch_origin, target_position));
        }

        let root = if self.arc == Arc::High { -root.sqrt() } else { root.sqrt() };
        let angle = ((launch.speed_sqr - root) / (launch.gravity * distance2d)).atan();

        Some(angles::radians_to_degrees(angle))
    }

    fn is_projectile_reached_target(
        &self,
        launch_origin: Vector3<E::Scalar>,
        target_position: Vector3<E::Scalar>,
        projectile: &Projectile<E::Scalar>,
        pitch: E::Scalar,
        yaw: E::Scalar,
        time: E::Scalar,
    ) -> bool {
        let projectile_position =
            E::predict_projectile_position(launch_origin, projectile, pitch, yaw, time, self.gravity_constant);

        // Squared compare keeps the per step square root out of the scan
        projectile_position.distance_to_sqr(target_position) <= self.distance_tolerance * self.distance_tolerance
    }
}

impl<E: PredEngine> ProjPredEngineInterface<E::Scalar> for ProjPredEngineLegacy<E> {
    fn maybe_calculate_aim(
        &self,
        projectile: &Projectile<E::Scalar>,
        launcher: &Launcher<E::Scalar>,
        target: &Target<E::Scalar>,
    ) -> Option<AimSolution<E::Scalar>> {
        let solution = self.find_solution(projectile, launcher, target)?;

        // Every point on the eye's aim ray lands on the same pixel; the target's distance keeps the point at a sensible
        // depth for anything that uses it in 3D.
        let forward = E::calc_view_basis(solution.pitch, solution.yaw).forward;
        let distance = launcher.eye_origin.distance_to(solution.predicted_target_position);

        Some(AimSolution {
            angles: AimAngles { pitch: solution.pitch, yaw: solution.yaw },
            aim_point: launcher.eye_origin + forward * distance,
            predicted_target_position: solution.predicted_target_position,
            time_of_flight: solution.time,
        })
    }
}
