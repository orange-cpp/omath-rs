//! Select targets from any iterator; ties preserve the first eligible target.
use crate::{
    Vector3,
    projection::{Camera, CameraEngine},
    scalar::Scalar,
};

pub fn get_closest_target_by_distance<T>(
    targets: impl IntoIterator<Item = T>,
    origin: Vector3<f32>,
    mut position: impl FnMut(&T) -> Vector3<f32>,
    mut filter: impl FnMut(&T) -> bool,
) -> Option<T> {
    let mut best = None;
    let mut distance = f32::INFINITY;
    for target in targets {
        if !filter(&target) {
            continue;
        }
        let current = origin.distance_to(position(&target));
        if best.is_none() || current < distance {
            distance = current;
            best = Some(target);
        }
    }
    best
}
pub fn get_closest_target_by_fov<E: CameraEngine, T>(
    targets: impl IntoIterator<Item = T>,
    camera: &Camera<E>,
    mut position: impl FnMut(&T) -> Vector3<E::Scalar>,
    mut filter: impl FnMut(&T) -> bool,
) -> Option<T> {
    let angles = camera.get_view_angles();
    let mut best = None;
    let mut distance = f64::INFINITY;
    for target in targets {
        if !filter(&target) {
            continue;
        }
        let look = camera.calc_look_at_angles(position(&target));
        // Compare pitch/yaw using the shortest angular distance across the wrap boundary.
        let delta = |a: E::Scalar, b: E::Scalar| {
            let d = (a.to_f64() - b.to_f64()).rem_euclid(360.0);
            d.min(360.0 - d)
        };
        let current = delta(angles.pitch.as_degrees(), look.pitch.as_degrees())
            .hypot(delta(angles.yaw.as_degrees(), look.yaw.as_degrees()));
        if current.is_finite() && (best.is_none() || current < distance) {
            distance = current;
            best = Some(target);
        }
    }
    best
}
