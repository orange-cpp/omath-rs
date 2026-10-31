use crate::linear_algebra::{Mat, Triangle, Vector3};
use crate::primitives::{Aabb, HasPosition, Mesh, MeshEngine, Obb};
use crate::scalar::{Real, Scalar};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ray<T = f32> {
    pub start: Vector3<T>,
    pub end: Vector3<T>,
    pub infinite_length: bool,
}

impl<T: Scalar> Ray<T> {
    #[inline]
    pub fn direction_vector(&self) -> Vector3<T> {
        self.end - self.start
    }
}

impl<T: Real> Ray<T> {
    #[inline]
    pub fn direction_vector_normalized(&self) -> Vector3<T> {
        self.direction_vector().normalized()
    }
}

/// Anything a ray can hit, in place of the C++ `get_ray_hit_point` overloads.
pub trait RayTraceable {
    type Scalar: Real;

    /// The hit point, or `ray.end` if there is none.
    fn get_ray_hit_point(&self, ray: &Ray<Self::Scalar>) -> Vector3<Self::Scalar>;
}

/// The hit point on `target`, or `ray.end` if there is none.
#[inline]
pub fn get_ray_hit_point<G: RayTraceable>(ray: &Ray<G::Scalar>, target: &G) -> Vector3<G::Scalar> {
    target.get_ray_hit_point(ray)
}

pub fn can_trace_line<T: Real>(ray: &Ray<T>, triangle: &Triangle<Vector3<T>>) -> bool {
    get_ray_hit_point(ray, triangle) == ray.end
}

impl<T: Real> RayTraceable for Triangle<Vector3<T>> {
    type Scalar = T;

    /// Möller–Trumbore. The epsilon is the `f32` one whatever `T` is, as in C++.
    ///
    /// A hit past `ray.end` is a miss even with `infinite_length`, unlike in the slab tests. Kept as is.
    fn get_ray_hit_point(&self, ray: &Ray<T>) -> Vector3<T> {
        let k_epsilon = T::from_f64(f32::EPSILON as f64);

        let side_a = self.side_a_vector();
        let side_b = self.side_b_vector();

        let ray_dir = ray.direction_vector();

        let p = ray_dir.cross(side_b);
        let det = side_a.dot(p);

        if det.abs_value() < k_epsilon {
            return ray.end;
        }

        let inv_det = T::ONE / det;
        let t = ray.start - self.vertex2;
        let u = t.dot(p) * inv_det;

        if (u < T::ZERO && u.abs_value() > k_epsilon)
            || (u > T::ONE && (u - T::ONE).abs_value() > k_epsilon)
        {
            return ray.end;
        }

        let q = t.cross(side_a);
        let v = ray_dir.dot(q) * inv_det;

        if (v < T::ZERO && v.abs_value() > k_epsilon)
            || (u + v > T::ONE && (u + v - T::ONE).abs_value() > k_epsilon)
        {
            return ray.end;
        }

        let t_hit = side_b.dot(q) * inv_det;

        if t_hit <= k_epsilon || t_hit > T::ONE - k_epsilon {
            return ray.end;
        }

        ray.start + ray_dir * t_hit
    }
}

/// One axis of a slab test: the ray's direction and start along it, and the box's bounds.
struct Slab<T> {
    d: T,
    origin: T,
    min: T,
    max: T,
}

fn slab_hit<T: Real>(ray: &Ray<T>, slabs: [Slab<T>; 3]) -> Vector3<T> {
    let mut t_min = -T::INFINITY;
    let mut t_max = T::INFINITY;

    for Slab {
        d,
        origin,
        min,
        max,
    } in slabs
    {
        if d.abs_value() < T::EPSILON {
            if origin >= min && origin <= max {
                continue;
            }
            return ray.end;
        }

        let inv = T::ONE / d;
        let mut t0 = (min - origin) * inv;
        let mut t1 = (max - origin) * inv;
        if t0 > t1 {
            std::mem::swap(&mut t0, &mut t1);
        }

        t_min = t_min.max_of(t0);
        t_max = t_max.min_of(t1);
        if t_min > t_max {
            return ray.end;
        }
    }

    // The entry point if it is in front of the start, otherwise 0: the ray started inside.
    let t_hit = T::ZERO.max_of(t_min);

    if t_max < T::ZERO {
        return ray.end; // box entirely behind the start
    }

    if !ray.infinite_length && t_hit > T::ONE {
        return ray.end; // box beyond the ray's end
    }

    ray.start + ray.direction_vector() * t_hit
}

impl<T: Real, U> RayTraceable for Aabb<T, U> {
    type Scalar = T;

    fn get_ray_hit_point(&self, ray: &Ray<T>) -> Vector3<T> {
        let dir = ray.direction_vector();

        slab_hit(
            ray,
            [
                Slab {
                    d: dir.x,
                    origin: ray.start.x,
                    min: self.min.x,
                    max: self.max.x,
                },
                Slab {
                    d: dir.y,
                    origin: ray.start.y,
                    min: self.min.y,
                    max: self.max.y,
                },
                Slab {
                    d: dir.z,
                    origin: ray.start.z,
                    min: self.min.z,
                    max: self.max.z,
                },
            ],
        )
    }
}

impl<T: Real> RayTraceable for Obb<T> {
    type Scalar = T;

    /// Runs the slab test in the box's local frame. The axes are orthonormal, so getting there only takes dot products,
    /// and the ray parameter is invariant under a rigid transform.
    fn get_ray_hit_point(&self, ray: &Ray<T>) -> Vector3<T> {
        let offset = ray.start - self.center;
        let dir = ray.direction_vector();

        let slab = |axis: Vector3<T>, half: T| Slab {
            d: dir.dot(axis),
            origin: offset.dot(axis),
            min: -half,
            max: half,
        };

        slab_hit(
            ray,
            [
                slab(self.axis_x, self.half_extents.x),
                slab(self.axis_y, self.half_extents.y),
                slab(self.axis_z, self.half_extents.z),
            ],
        )
    }
}

fn to_local_space<E: MeshEngine>(
    inverse: &Mat<4, 4, E::Scalar, E::Order>,
    point: Vector3<E::Scalar>,
) -> Vector3<E::Scalar> {
    let row = |i| {
        inverse.at(i, 0) * point.x
            + inverse.at(i, 1) * point.y
            + inverse.at(i, 2) * point.z
            + inverse.at(i, 3)
    };
    Vector3::new(row(0), row(1), row(2))
}

impl<E, V, Vbo, Ebo> RayTraceable for Mesh<E, V, Vbo, Ebo>
where
    E: MeshEngine,
    V: HasPosition<E::Scalar>,
    Vbo: AsRef<[V]>,
    Ebo: AsRef<[Vector3<u32>]>,
{
    type Scalar = E::Scalar;

    /// Traces in the mesh's local frame, which costs two matrix multiplies for the whole mesh rather than three per
    /// face. An affine transform preserves the ray parameter, so the nearest local hit is the nearest world hit.
    fn get_ray_hit_point(&self, ray: &Ray<E::Scalar>) -> Vector3<E::Scalar> {
        // A degenerate world matrix means a zero scale, which collapses the mesh to nothing to hit.
        let Some(inverse) = self.get_to_world_matrix().inverted() else {
            return ray.end;
        };

        let local_ray = Ray {
            start: to_local_space::<E>(&inverse, ray.start),
            end: to_local_space::<E>(&inverse, ray.end),
            infinite_length: ray.infinite_length,
        };

        let mut best_local_hit = local_ray.end;
        let mut best_distance_sqr = local_ray.start.distance_to_sqr(local_ray.end);

        for &face in self.element_buffer_object.as_ref() {
            let local_hit = get_ray_hit_point(&local_ray, &self.make_face_in_local_space(face));
            let distance_sqr = local_hit.distance_to_sqr(local_ray.start);

            if distance_sqr < best_distance_sqr {
                best_local_hit = local_hit;
                best_distance_sqr = distance_sqr;
            }
        }

        if best_local_hit == local_ray.end {
            return ray.end;
        }

        // From the original ray rather than through the world matrix, to carry no round-trip error from the inverse.
        let local_dir = local_ray.direction_vector();
        let t_hit = (best_local_hit - local_ray.start).dot(local_dir) / local_dir.dot(local_dir);

        ray.start + ray.direction_vector() * t_hit
    }
}
