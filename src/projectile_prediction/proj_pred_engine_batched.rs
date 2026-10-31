use super::{
    AimAngles, AimSolution, Launcher, PredEngine, ProjPredEngineInterface, Projectile, Target,
};
use crate::Vector3;
use crate::engines::source::PredEngineTrait as Trait;
use crate::trigonometry::angles;

/// How many candidate flight times are looked at together.
const LANES: usize = 8;

/// Source-style solver (z up, yaw about z) that scans candidate flight times eight at a time.
///
/// This is the C++ `ProjPredEngineAvx2`. It is plain Rust laid out so that LLVM can vectorize the scan, with no
/// intrinsics and no requirement on the CPU, which is why it does not carry that name. Unlike the C++ it is available
/// on every build, and a non-positive time step or horizon gives no solution instead of a scan that never ends.
#[derive(Clone, Copy, Debug)]
pub struct ProjPredEngineBatched {
    gravity_constant: f32,
    simulation_time_step: f32,
    maximum_simulation_time: f32,
}

struct Candidate {
    time: f32,
    target_position: Vector3<f32>,
    pitch: f32,
}

impl ProjPredEngineBatched {
    pub fn new(
        gravity_constant: f32,
        simulation_time_step: f32,
        maximum_simulation_time: f32,
    ) -> Self {
        Self {
            gravity_constant,
            simulation_time_step,
            maximum_simulation_time,
        }
    }

    /// First flight time at which the projectile can reach the predicted target from `launch_origin`.
    fn find_candidate(
        &self,
        launch_origin: Vector3<f32>,
        target: &Target<f32>,
        bullet_gravity: f32,
        launch_speed: f32,
    ) -> Option<Candidate> {
        let step = self.simulation_time_step;
        let target_gravity = if target.is_airborne {
            0.5 * self.gravity_constant
        } else {
            0.0
        };
        let launch_speed_sqr = launch_speed * launch_speed + 1e-3;

        // Speed the projectile would need to be at the predicted target exactly at each of eight candidate times.
        // Times where that is at most the launch speed get a scalar solve.
        for block in 0usize.. {
            let block_time = step + step * LANES as f32 * block as f32;
            if block_time > self.maximum_simulation_time {
                break;
            }

            let mut times = [0.0; LANES];
            let mut required_speed_sqr = [0.0; LANES];

            for (lane, (time, required)) in
                times.iter_mut().zip(&mut required_speed_sqr).enumerate()
            {
                *time = block_time + step * lane as f32;

                let times_sq = *time * *time;
                let delta_x = target.velocity.x * *time + target.origin.x - launch_origin.x;
                let delta_y = target.velocity.y * *time + target.origin.y - launch_origin.y;
                let delta_z = target.velocity.z * *time + target.origin.z
                    - target_gravity * times_sq
                    - launch_origin.z;

                let term = delta_z + 0.5 * (bullet_gravity * times_sq);

                // Avoid division by zero
                *required =
                    (delta_x * delta_x + delta_y * delta_y + term * term) / (times_sq + 1e-8);
            }

            let mut valid_mask = 0u8;
            for (lane, required) in required_speed_sqr.iter().enumerate() {
                valid_mask |= u8::from(*required <= launch_speed_sqr) << lane;
            }

            if valid_mask == 0 {
                continue;
            }

            for (lane, &candidate_time) in times.iter().enumerate() {
                if valid_mask & (1 << lane) == 0 || candidate_time > self.maximum_simulation_time {
                    continue;
                }

                // Fine search around the candidate time
                for offset in -2..=2 {
                    let fine_time = candidate_time + step * offset as f32;
                    if fine_time < 0.0 {
                        continue;
                    }

                    let target_position =
                        Trait::predict_target_position(target, fine_time, self.gravity_constant);

                    if let Some(pitch) = Self::calculate_pitch(
                        launch_origin,
                        target_position,
                        bullet_gravity,
                        launch_speed,
                        fine_time,
                    ) {
                        return Some(Candidate {
                            time: fine_time,
                            target_position,
                            pitch,
                        });
                    }
                }
            }
        }

        None
    }

    fn calculate_pitch(
        launch_origin: Vector3<f32>,
        target_pos: Vector3<f32>,
        bullet_gravity: f32,
        launch_speed: f32,
        time: f32,
    ) -> Option<f32> {
        if time <= 0.0 {
            return None;
        }

        let delta = target_pos - launch_origin;
        let d_sqr = delta.x * delta.x + delta.y * delta.y;
        let h = delta.z;

        let term = h + 0.5 * bullet_gravity * time * time;
        let required_speed_sqr = (d_sqr + term * term) / (time * time);

        if required_speed_sqr > launch_speed * launch_speed + 1e-3 {
            return None;
        }

        if d_sqr == 0.0 {
            return Some(if term >= 0.0 { 90.0 } else { -90.0 });
        }

        Some(angles::radians_to_degrees((term / d_sqr.sqrt()).atan()))
    }
}

impl ProjPredEngineInterface<f32> for ProjPredEngineBatched {
    fn maybe_calculate_aim(
        &self,
        projectile: &Projectile<f32>,
        launcher: &Launcher<f32>,
        target: &Target<f32>,
    ) -> Option<AimSolution<f32>> {
        if self.simulation_time_step.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)
            || self.maximum_simulation_time.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)
        {
            return None;
        }

        let bullet_gravity = self.gravity_constant * projectile.gravity_scale;
        let launch_speed = projectile.launch_speed;

        // The scan needs one fixed launch origin. With a view-relative muzzle offset the muzzle is first placed from
        // the direct angles to the target, and the solve is refined once from the angles that come out.
        //
        // `pitch` is the launch pitch, the direction the round leaves in. The muzzle turns with the view, so every
        // basis is built from the view pitch: launch pitch minus the launcher's offset.
        let muzzle_rotates = !launcher.muzzle_offset.is_zero();
        let pitch_offset = launcher.launch_pitch_offset;
        let mut launch_origin = launcher.eye_origin + launcher.world_offset;

        if muzzle_rotates {
            launch_origin = launcher.launch_origin(Trait::calc_view_basis(
                Trait::calc_direct_pitch_angle(launcher.eye_origin, target.origin) - pitch_offset,
                Trait::calc_direct_yaw_angle(launcher.eye_origin, target.origin),
            ));
        }

        let candidate = self.find_candidate(launch_origin, target, bullet_gravity, launch_speed)?;

        let mut pitch = candidate.pitch;
        let mut yaw = Trait::calc_direct_yaw_angle(launch_origin, candidate.target_position);

        if muzzle_rotates {
            launch_origin =
                launcher.launch_origin(Trait::calc_view_basis(pitch - pitch_offset, yaw));

            pitch = Self::calculate_pitch(
                launch_origin,
                candidate.target_position,
                bullet_gravity,
                launch_speed,
                candidate.time,
            )?;
            yaw = Trait::calc_direct_yaw_angle(launch_origin, candidate.target_position);
        }

        let view_pitch = pitch - pitch_offset;
        let forward = Trait::calc_view_basis(view_pitch, yaw).forward;

        // With an offset the view pitch can land outside what the engine lets a player look at. calc_view_basis clamps
        // the way the engine does, so a forward vector without the requested pitch means the shot cannot be set up.
        // This engine only solves the first feasible time, so that is the end of it.
        if pitch_offset != 0.0
            && (Trait::calc_direct_pitch_angle(Vector3::default(), forward) - view_pitch).abs()
                > 0.1
        {
            return None;
        }

        let distance = launcher.eye_origin.distance_to(candidate.target_position);

        Some(AimSolution {
            angles: AimAngles {
                pitch: view_pitch,
                yaw,
            },
            aim_point: launcher.eye_origin + forward * distance,
            predicted_target_position: candidate.target_position,
            time_of_flight: candidate.time,
        })
    }
}
