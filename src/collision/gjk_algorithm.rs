use super::{Collider, Simplex};
use crate::linear_algebra::Vector3;
use crate::scalar::Scalar;

#[derive(Clone, Copy, Debug, Default)]
pub struct GjkHitInfo<T = f32> {
    pub hit: bool,
    /// Valid only if `hit` is true, and then holds 4 points.
    pub simplex: Simplex<T>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GjkSettings {
    pub epsilon: f32,
    pub max_iterations: usize,
}

impl Default for GjkSettings {
    fn default() -> Self {
        Self {
            epsilon: 1e-6,
            max_iterations: 64,
        }
    }
}

pub fn find_support_vertex<C: Collider + ?Sized>(
    collider_a: &C,
    collider_b: &C,
    direction: Vector3<C::Scalar>,
) -> Vector3<C::Scalar> {
    collider_a.find_abs_furthest_vertex_position(direction)
        - collider_b.find_abs_furthest_vertex_position(-direction)
}

pub fn is_collide<C: Collider + ?Sized>(collider_a: &C, collider_b: &C) -> bool {
    is_collide_with_simplex_info(collider_a, collider_b, &GjkSettings::default()).hit
}

pub fn is_collide_with_simplex_info<C: Collider + ?Sized>(
    collider_a: &C,
    collider_b: &C,
    settings: &GjkSettings,
) -> GjkHitInfo<C::Scalar> {
    let epsilon = C::Scalar::from_f64(settings.epsilon as f64);
    let epsilon_sqr = C::Scalar::from_f64((settings.epsilon * settings.epsilon) as f64);

    // The centroid difference as initial direction greatly reduces iterations for separated shapes.
    let mut initial_dir = collider_b.get_origin() - collider_a.get_origin();
    if initial_dir.dot(initial_dir) < epsilon_sqr {
        initial_dir = Vector3::new(C::Scalar::ONE, C::Scalar::ZERO, C::Scalar::ZERO);
    }

    let mut support = find_support_vertex(collider_a, collider_b, initial_dir);

    let mut simplex = Simplex::default();
    simplex.push_front(support);

    let mut direction = -support;

    for _ in 0..settings.max_iterations {
        support = find_support_vertex(collider_a, collider_b, direction);

        if support.dot(direction) <= epsilon {
            return GjkHitInfo {
                hit: false,
                simplex,
            };
        }

        simplex.push_front(support);

        if simplex.handle(&mut direction) {
            return GjkHitInfo { hit: true, simplex };
        }
    }
    GjkHitInfo {
        hit: false,
        simplex,
    }
}
