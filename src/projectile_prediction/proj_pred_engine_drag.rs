use super::{
    AimAngles, AimSolution, Arc, Launcher, PredEngine, ProjPredEngineInterface, Projectile, ProjectileFlight, Target,
};
use crate::Vector3;
use crate::engines::source;
use crate::scalar::{Real, Scalar, Signed};
use crate::trigonometry::angles;
use std::marker::PhantomData;

/// Aim solver for rounds that lose speed to the air. The closed-form engines pick their launch pitch from a parabola,
/// and there is no closed form once drag is involved: the pipe a parabola promises at 1000 units arrives about 150
/// short. This engine finds the pitch by flying the round through [`ProjectileFlight`], so whatever it returns is a
/// shot that flight lands.
///
/// It answers [`ProjPredEngineInterface`] and fills [`AimSolution`] the same way
/// [`ProjPredEngineLegacy`](super::ProjPredEngineLegacy) does, so the two are interchangeable to everything downstream.
///
/// It is not only for drag. A round with no gravity flies a straight line, which stepping follows exactly at any step,
/// and this engine finds its answer by closing in on it rather than by hoping a scan step lands inside the tolerance,
/// so it cannot come back empty for a target that closes faster than the scan allowed for. What it is wrong for is a
/// round with gravity and no drag in a game that moves such rounds on a true parabola: the stepped fall here drops a
/// little further than that (see [`ProjectileFlight`]), and the closed-form engines are exact.
///
/// Built for [`Arc::High`] it solves for the lob instead: the steeper of the two pitches that reach the target, which
/// drops onto it from above and takes longer to get there. Such a shot is refused, like any other, when its flight
/// would outlast `maximum_simulation_time`, and a lob to anything near takes a long time.
#[derive(Clone, Copy, Debug)]
pub struct ProjPredEngineDrag<E: PredEngine = source::PredEngineTrait> {
    gravity_constant: E::Scalar,
    simulation_time_step: E::Scalar,
    maximum_simulation_time: E::Scalar,
    distance_tolerance: E::Scalar,
    arc: Arc,
    engine: PhantomData<E>,
}

#[derive(Clone, Copy)]
struct Pass<T> {
    /// How far above the target point the round is as it goes by it, negative when it is below.
    height_error: T,
    time: T,
}

#[derive(Clone, Copy)]
struct PitchSolution<T> {
    view_pitch: T,
    time: T,
}

struct ParabolaPitches<T> {
    low: T,
    high: T,
}

/// What stays fixed through one solve, and what carries over from one shot of it to the next.
struct Search<'a, T> {
    projectile: &'a Projectile<T>,
    launcher: &'a Launcher<T>,
    target: &'a Target<T>,
    time_limit: T,
    arc: Arc,
    yaw: T,
    predicted: Vector3<T>,
    /// How far above the parabola's pitch the last pass ended up. Drag asks for about the same extra loft from one pass
    /// to the next, which makes it a good guess at where the next search will end.
    loft: Option<T>,
}

/// The pitch search for one target point.
struct PitchSearch<'a, T> {
    projectile: &'a Projectile<T>,
    launcher: &'a Launcher<T>,
    yaw: T,
    heading: Vector3<T>,
    target_point: Vector3<T>,
    time_limit: T,
}

/// Which way to turn the pitch to bring the round up at the target: steeper on the low arc, and the other way about on
/// the high arc, where a steeper lob comes down sooner.
fn climb<T: Real>(arc: Arc) -> T {
    match arc {
        Arc::Low => T::ONE,
        Arc::High => -T::ONE,
    }
}

impl<E: PredEngine> ProjPredEngineDrag<E> {
    /// `simulation_time_step` is the physics step of the game being predicted (Source: its tick interval), not an
    /// accuracy setting. See [`ProjectileFlight`]. Solves for [`Arc::Low`], unless [`with_arc`](Self::with_arc) says
    /// otherwise.
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

    /// Where a round fired along `view_angles` is after `time` seconds. Between steps it is on the straight line
    /// joining them, which is how the game moves and draws it too.
    pub fn predict_projectile_position(
        &self,
        projectile: &Projectile<E::Scalar>,
        launcher: &Launcher<E::Scalar>,
        view_angles: AimAngles<E::Scalar>,
        time: E::Scalar,
    ) -> Vector3<E::Scalar> {
        let mut flight = self.flight(projectile, launcher, view_angles);

        let steps = E::Scalar::ZERO.max_of(time) / self.simulation_time_step;
        let whole_steps = steps.to_f64() as usize;

        for _ in 0..whole_steps {
            flight.step();
        }

        let before = flight.position();
        flight.step();

        before + (flight.position() - before) * (steps - E::Scalar::from_f64(whole_steps as f64))
    }

    fn flight(
        &self,
        projectile: &Projectile<E::Scalar>,
        launcher: &Launcher<E::Scalar>,
        view_angles: AimAngles<E::Scalar>,
    ) -> ProjectileFlight<E> {
        ProjectileFlight::new(projectile, launcher, view_angles, self.gravity_constant, self.simulation_time_step)
    }

    /// One shot at the target as it will be at `at`: the view pitch that lands there, and when the round does. The
    /// search is always anchored to what a parabola would need for that same point, on the arc being solved for.
    /// Carrying the last pass's pitch over instead looks cheaper and is a trap: a step in time can move the target by
    /// tens of degrees, and a search started on the wrong side of the arc's peak, where turning the pitch the way it
    /// expects moves the round the other way, walks itself into the limit.
    fn shoot_at(&self, search: &mut Search<'_, E::Scalar>, at: E::Scalar) -> Option<PitchSolution<E::Scalar>> {
        let Search { projectile, launcher, target, time_limit, arc, .. } = *search;

        search.predicted = E::predict_target_position(target, at, self.gravity_constant);
        let predicted = search.predicted;

        let parabola = self.maybe_calculate_parabola_launch_pitches(projectile, launcher.eye_origin, predicted)?;

        let mut start_pitch = match arc {
            Arc::Low => parabola.low,
            Arc::High => parabola.high,
        } - launcher.launch_pitch_offset;

        // The high arc lies between the two parabola pitches: drag needs more loft than the low one and stands less
        // than the high one. And no lob steeper than the one whose parabola takes the whole time limit to cover the
        // distance gets there in time, since drag only ever makes a flight later, so the search need not start above
        // that either.
        let floor_pitch = parabola.low - launcher.launch_pitch_offset;
        if arc == Arc::High {
            let cosine =
                E::calc_vector_2d_distance(predicted - launcher.eye_origin) / (projectile.launch_speed * time_limit);
            if cosine < E::Scalar::ONE {
                start_pitch =
                    start_pitch.min_of(angles::radians_to_degrees(cosine.acos()) - launcher.launch_pitch_offset);
            }
        }

        // The muzzle sits off to one side and swings with the yaw, so the yaw that points the muzzle's own flight plane
        // at the target is found from where the muzzle ends up
        for _ in 0..2 {
            search.yaw = E::calc_direct_yaw_angle(
                launcher.launch_origin(E::calc_view_basis(start_pitch, search.yaw)),
                predicted,
            );
        }

        // A shade past the last loft, on the side that has the round pass above the target, so that the pair of passes
        // closes on the answer straight away
        let hint = search.loft.map(|loft| {
            start_pitch + loft * E::Scalar::from_f64(1.05) + E::Scalar::from_f64(0.1) * climb::<E::Scalar>(arc)
        });

        // A level view looks along the horizontal, whichever axes the engine keeps that in
        let pitch_search = PitchSearch {
            projectile,
            launcher,
            yaw: search.yaw,
            heading: E::calc_view_basis(E::Scalar::ZERO, search.yaw).forward,
            target_point: predicted,
            time_limit,
        };

        let shot = self.solve_view_pitch(&pitch_search, start_pitch, floor_pitch, hint, arc);
        if let Some(shot) = shot {
            search.loft = Some(shot.view_pitch - start_pitch);
        }

        shot
    }

    /// Flies the round from this view pitch until it has covered the horizontal distance to the target point, and
    /// reports the height it passes at and when. The muzzle is placed from the same angles, so the way it swings with
    /// the view is part of the answer rather than an error in it. Empty when the round never gets that far.
    fn fly_past(&self, search: &PitchSearch<'_, E::Scalar>, view_pitch: E::Scalar) -> Option<Pass<E::Scalar>> {
        let mut flight =
            self.flight(search.projectile, search.launcher, AimAngles { pitch: view_pitch, yaw: search.yaw });

        let to_target = search.target_point - flight.origin();
        let distance = to_target.dot(search.heading);

        if !(distance > E::Scalar::ZERO) {
            return None;
        }

        let height = E::get_vector_height_coordinate(to_target);

        let mut previous_travel = E::Scalar::ZERO;
        let mut previous_height = E::Scalar::ZERO;

        while flight.time() < search.time_limit {
            let previous_time = flight.time();
            flight.step();

            let flown = flight.position() - flight.origin();
            let travel = flown.dot(search.heading);
            let flown_height = E::get_vector_height_coordinate(flown);

            if travel >= distance {
                // The round moves in a straight line within a step, so the crossing is linear too
                let fraction = (distance - previous_travel) / (travel - previous_travel);

                return Some(Pass {
                    height_error: previous_height + (flown_height - previous_height) * fraction - height,
                    time: previous_time + self.simulation_time_step * fraction,
                });
            }

            previous_travel = travel;
            previous_height = flown_height;
        }

        None
    }

    /// The two pitches a round without drag would need, the low one being the formula
    /// [`ProjPredEngineLegacy`](super::ProjPredEngineLegacy) solves with. With drag the real answers lie between them,
    /// each a little in from its own side, which makes them the places to start looking from. Empty when even a
    /// parabola falls short, and drag only ever shortens the reach.
    fn maybe_calculate_parabola_launch_pitches(
        &self,
        projectile: &Projectile<E::Scalar>,
        origin: Vector3<E::Scalar>,
        target_point: Vector3<E::Scalar>,
    ) -> Option<ParabolaPitches<E::Scalar>> {
        let gravity = self.gravity_constant * projectile.gravity_scale;
        let delta = target_point - origin;
        let distance2d = E::calc_vector_2d_distance(delta);

        if gravity == E::Scalar::ZERO || distance2d == E::Scalar::ZERO {
            let direct = E::calc_direct_pitch_angle(origin, target_point);
            return Some(ParabolaPitches { low: direct, high: direct });
        }

        let speed_sqr = projectile.launch_speed * projectile.launch_speed;
        let inner = gravity * distance2d * distance2d
            + E::Scalar::from_f64(2.0) * E::get_vector_height_coordinate(delta) * speed_sqr;
        let discriminant = speed_sqr * speed_sqr - gravity * inner;

        if discriminant < E::Scalar::ZERO {
            return None;
        }

        let root = discriminant.sqrt();

        Some(ParabolaPitches {
            low: angles::radians_to_degrees((inner / (distance2d * (speed_sqr + root))).atan()),
            high: angles::radians_to_degrees(((speed_sqr + root) / (gravity * distance2d)).atan()),
        })
    }

    /// Finds the view pitch on the engine's arc that flies the round through the target point. It walks away from
    /// `start_pitch` in whichever direction the first miss says to, until the round passes on the other side of the
    /// target, then closes in on the crossing between the two.
    ///
    /// `start_pitch` has to be on the same arc as the answer, which the parabola's pitch for that arc always is,
    /// because everything the search concludes it concludes relative to that pass. `hint_pitch` is a guess at the
    /// answer and is
    /// only ever used to pair with it: if the round passes on the other side of the target from there the walk is
    /// skipped, and if it does not the hint is dropped. A wrong hint costs one flight and can never turn a shot down.
    /// `floor_pitch` is the low arc's parabola pitch, below which a high arc cannot lie.
    fn solve_view_pitch(
        &self,
        search: &PitchSearch<'_, E::Scalar>,
        start_pitch: E::Scalar,
        floor_pitch: E::Scalar,
        hint_pitch: Option<E::Scalar>,
        arc: Arc,
    ) -> Option<PitchSolution<E::Scalar>> {
        const MAX_REFINEMENTS: usize = 24;

        let pitch_limit = E::Scalar::from_f64(90.0);
        let first_step = E::Scalar::from_f64(0.5);
        let max_step = E::Scalar::from_f64(4.0);
        let climb = climb::<E::Scalar>(arc);

        // How close to the target's height a pass has to come before the pitch counts as found
        let height_tolerance = self.distance_tolerance / E::Scalar::from_f64(100.0);
        let is_on_target = |pass: &Pass<E::Scalar>| pass.height_error.abs_value() <= height_tolerance;
        let is_below = |pass: &Pass<E::Scalar>| pass.height_error < E::Scalar::ZERO;

        let lowest = match arc {
            Arc::Low => -pitch_limit,
            Arc::High => (-pitch_limit).max_of(floor_pitch),
        };

        let mut near_pitch = start_pitch.clamped(lowest, pitch_limit);
        let mut near = self.fly_past(search, near_pitch);

        if arc == Arc::High {
            // The parabola's high arc to a near target is all but vertical, and with drag on top of that such a lob
            // takes longer than any flight is allowed. Come down until one gets there. If the first that does is
            // already over the target, the crossing lies steeper still, among the flights too long to count.
            let started_at = near_pitch;
            while near.is_none() && near_pitch > lowest {
                near_pitch = (near_pitch - max_step).max_of(lowest);
                near = self.fly_past(search, near_pitch);
            }
            if let Some(pass) = near {
                if near_pitch != started_at && pass.height_error > height_tolerance {
                    return None;
                }
            }
        }

        let mut near = near?;
        if is_on_target(&near) {
            return Some(PitchSolution { view_pitch: near_pitch, time: near.time });
        }

        let direction = if is_below(&near) { climb } else { -climb };

        let mut crossing = None;

        if let Some(hint_pitch) = hint_pitch {
            let far_pitch = hint_pitch.clamped(lowest, pitch_limit);

            if let Some(far) = self.fly_past(search, far_pitch) {
                if is_on_target(&far) {
                    return Some(PitchSolution { view_pitch: far_pitch, time: far.time });
                }
                if is_below(&far) != is_below(&near) {
                    crossing = Some((far_pitch, far));
                }
            }
        }

        let mut step = first_step;

        let (far_pitch, far) = loop {
            if let Some(crossing) = crossing {
                break crossing;
            }

            let at_the_limit =
                if direction > E::Scalar::ZERO { near_pitch >= pitch_limit } else { near_pitch <= lowest };
            if at_the_limit {
                return None;
            }

            let far_pitch = (near_pitch + direction * step).clamped(lowest, pitch_limit);

            // A lob this steep that still cannot cover the distance in time is over the top of what the weapon can
            // reach, or on the high arc past what the time limit allows
            let far = self.fly_past(search, far_pitch)?;

            if is_on_target(&far) {
                return Some(PitchSolution { view_pitch: far_pitch, time: far.time });
            }

            if is_below(&far) != is_below(&near) {
                crossing = Some((far_pitch, far));
                continue;
            }

            // Turning the pitch towards the peak has to bring the round up at the target. Once it stops doing that the
            // arc has gone over its peak without ever getting there: out of reach.
            if direction == climb && far.height_error <= near.height_error {
                return None;
            }

            near_pitch = far_pitch;
            near = far;
            step = (step * E::Scalar::from_f64(2.0)).min_of(max_step);
        };

        // Regula falsi with the Illinois correction, between the pass below and the pass above
        let mut below_pitch = near_pitch;
        let mut below_error = near.height_error;
        let mut above_pitch = far_pitch;
        let mut above_error = far.height_error;

        if below_error > E::Scalar::ZERO {
            std::mem::swap(&mut below_pitch, &mut above_pitch);
            std::mem::swap(&mut below_error, &mut above_error);
        }

        let mut best = PitchSolution { view_pitch: below_pitch, time: near.time };
        let mut last_side = 0;

        for _ in 0..MAX_REFINEMENTS {
            let pitch = (below_pitch * above_error - above_pitch * below_error) / (above_error - below_error);
            let pass = self.fly_past(search, pitch)?;

            best = PitchSolution { view_pitch: pitch, time: pass.time };

            if is_on_target(&pass) || (above_pitch - below_pitch).abs_value() < E::Scalar::from_f64(1e-4) {
                break;
            }

            if is_below(&pass) {
                below_pitch = pitch;
                below_error = pass.height_error;
                if last_side == -1 {
                    above_error /= E::Scalar::from_f64(2.0);
                }
                last_side = -1;
            } else {
                above_pitch = pitch;
                above_error = pass.height_error;
                if last_side == 1 {
                    below_error /= E::Scalar::from_f64(2.0);
                }
                last_side = 1;
            }
        }

        Some(best)
    }

    /// `calc_view_basis` clamps the way the engine does, so a forward vector that no longer has the requested pitch
    /// means nobody can set this shot up. Same check [`ProjPredEngineLegacy`](super::ProjPredEngineLegacy) makes.
    fn is_view_pitch_reachable(view_pitch: E::Scalar, yaw: E::Scalar) -> bool {
        let tolerance_degrees = E::Scalar::from_f64(0.1);

        let forward = E::calc_view_basis(view_pitch, yaw).forward;
        let reachable_pitch = E::calc_direct_pitch_angle(Vector3::default(), forward);

        (reachable_pitch - view_pitch).abs_value() <= tolerance_degrees
    }
}

impl<E: PredEngine> ProjPredEngineInterface<E::Scalar> for ProjPredEngineDrag<E> {
    fn maybe_calculate_aim(
        &self,
        projectile: &Projectile<E::Scalar>,
        launcher: &Launcher<E::Scalar>,
        target: &Target<E::Scalar>,
    ) -> Option<AimSolution<E::Scalar>> {
        const MAX_ITERATIONS: usize = 12;

        if !(self.simulation_time_step > E::Scalar::ZERO)
            || !(self.maximum_simulation_time > E::Scalar::ZERO)
            || !(projectile.launch_speed > E::Scalar::ZERO)
        {
            return None;
        }

        // Probe flights may run past the horizon, so that a shot right at it still has a pass on either side to close
        // in from: the pass above is at most four degrees steeper than the answer, which gets it there a tenth or so
        // later. Any further than that only makes a flight that was never going to arrive take longer to give up. The
        // horizon itself is enforced on the answer.
        let time_limit = self.maximum_simulation_time * E::Scalar::from_f64(1.25);

        // A round without gravity flies one straight line, and both arcs are it
        let arc = if self.gravity_constant * projectile.gravity_scale > E::Scalar::ZERO { self.arc } else { Arc::Low };

        // Where the target will be depends on how long the round takes, and how long the round takes depends on where
        // the target will be. The time that satisfies both is the one where the round, aimed at where the target is at
        // that time, takes exactly that long to get there: mismatch(time) = arrival - time = 0.
        //
        // Feeding each arrival back in as the next guess would find it only while the target is slow. That closes in by
        // the ratio of the target's speed along the line of fire to the round's every pass, which is a crawl for a
        // target blown away at two thirds of the round's speed and gets further out every pass for one blown towards
        // the shooter faster than the round flies. So the first step is that guess, and every one after it is a secant
        // step through the last two mismatches, which does not care.
        let mut time = launcher.eye_origin.distance_to(target.origin) / projectile.launch_speed;

        let mut search = Search {
            projectile,
            launcher,
            target,
            time_limit,
            arc,
            yaw: E::calc_direct_yaw_angle(launcher.eye_origin, target.origin),
            predicted: target.origin,
            loft: None,
        };

        let mut solved = self.shoot_at(&mut search, time);

        if solved.is_none() {
            // The first guess can fall on a time at which the target cannot be reached although it can be later:
            // somebody running in from out of range, a jumper coming down into it. A shot that fails has failed for
            // being too far, so the one other time worth a search is the one along the horizon that brings the target
            // nearest, and only if that is nearer than it was.
            //
            // For a target on the ground only later times count. The guess is how long a straight flight at launch
            // speed takes, and no round gets anywhere sooner than that, so a target that is only nearer earlier on is
            // one that is leaving, and earlier is when it cannot be caught. Looking there cost four more searches to
            // refuse a shot that the first one had already settled.
            //
            // Not so for one in the air. It is falling, and the lower it gets the sooner it is reached, however far off
            // that is, so how near it is says little about whether it can be hit and every time is looked at. Twice a
            // short cut here that reasoned from distance turned down shots at airborne targets that were there to be
            // made.
            const CANDIDATES: usize = 8;

            let mut nearest = launcher.eye_origin.distance_to(search.predicted);
            let mut nearest_time = None;

            for candidate in 1..=CANDIDATES {
                let at = self.maximum_simulation_time * E::Scalar::from_f64(candidate as f64)
                    / E::Scalar::from_f64(CANDIDATES as f64);
                if !target.is_airborne && at <= time {
                    continue;
                }

                let distance =
                    launcher.eye_origin.distance_to(E::predict_target_position(target, at, self.gravity_constant));

                if distance < nearest {
                    nearest = distance;
                    nearest_time = Some(at);
                }
            }

            time = nearest_time?;
            solved = self.shoot_at(&mut search, time);
        }

        let mut solved = solved?;
        let mut previous = None;

        for _ in 0..MAX_ITERATIONS {
            // The round gets to `predicted` at solved.time, by when the target has moved on to where it will be at that
            // time. Once those two are the same place, the shot is the answer.
            let settled = E::predict_target_position(target, solved.time, self.gravity_constant)
                .distance_to(search.predicted)
                <= self.distance_tolerance / E::Scalar::from_f64(64.0);

            let mismatch = solved.time - time;
            let mut next_time = solved.time;

            if let (false, Some((previous_time, previous_mismatch))) = (settled, previous) {
                if mismatch != previous_mismatch {
                    next_time = time - mismatch * (time - previous_time) / (mismatch - previous_mismatch);
                }
            }

            let previous_time = time;
            previous = Some((time, mismatch));
            time = next_time.clamped(E::Scalar::ZERO, time_limit);

            if settled {
                break;
            }

            let mut next = self.shoot_at(&mut search, time);

            // A step can land on a time at which the target is somewhere the round cannot be put at all, although the
            // answer is somewhere it can. Back off towards the last time that could be shot at.
            for _ in 0..2 {
                if next.is_some() {
                    break;
                }

                time = (time + previous_time) / E::Scalar::from_f64(2.0);
                next = self.shoot_at(&mut search, time);
            }

            solved = next?;
        }

        if time > self.maximum_simulation_time || !Self::is_view_pitch_reachable(solved.view_pitch, search.yaw) {
            return None;
        }

        // Whatever came out above has to put the round on the target, at the time claimed
        let predicted = E::predict_target_position(target, time, self.gravity_constant);

        let angles = AimAngles { pitch: solved.view_pitch, yaw: search.yaw };
        if self.predict_projectile_position(projectile, launcher, angles, time).distance_to(predicted)
            > self.distance_tolerance
        {
            return None;
        }

        // Every point on the eye's aim ray lands on the same pixel; the target's distance keeps the point at a sensible
        // depth for anything that uses it in 3D.
        let forward = E::calc_view_basis(angles.pitch, angles.yaw).forward;

        Some(AimSolution {
            angles,
            aim_point: launcher.eye_origin + forward * launcher.eye_origin.distance_to(predicted),
            predicted_target_position: predicted,
            time_of_flight: time,
        })
    }
}
