use crate::linear_algebra::Vector3;
use crate::scalar::{Real, Scalar};
use std::ops::{Index, IndexMut};

/// Up to four points, newest first, that GJK grows towards the origin.
#[derive(Clone, Copy, Debug, Default)]
pub struct Simplex<T = f32> {
    points: [Vector3<T>; 4],
    size: usize,
}

impl<T> Simplex<T> {
    pub const CAPACITY: usize = 4;

    #[inline]
    pub const fn size(&self) -> usize {
        self.size
    }

    #[inline]
    pub const fn empty(&self) -> bool {
        self.size == 0
    }

    #[inline]
    pub const fn data(&self) -> &[Vector3<T>; 4] {
        &self.points
    }

    /// The live points, so `size()` of them.
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, Vector3<T>> {
        self.points[..self.size].iter()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.size = 0;
    }
}

impl<T: Scalar> Simplex<T> {
    /// Replaces the contents, in place of the C++ `operator=(initializer_list)`. Panics on more than four points.
    pub fn assign(&mut self, points: &[Vector3<T>]) {
        assert!(
            points.len() <= Self::CAPACITY,
            "Simplex can have at most 4 points"
        );
        self.points[..points.len()].copy_from_slice(points);
        self.size = points.len();
    }

    /// Drops the oldest point when full.
    pub fn push_front(&mut self, point: Vector3<T>) {
        let limit = self.size.min(Self::CAPACITY - 1);
        self.points.copy_within(..limit, 1);
        self.points[0] = point;
        if self.size < Self::CAPACITY {
            self.size += 1;
        }
    }

    #[inline]
    pub fn front(&self) -> Vector3<T> {
        self.points[0]
    }

    /// Panics on an empty simplex.
    #[inline]
    pub fn back(&self) -> Vector3<T> {
        self.points[self.size - 1]
    }
}

impl<T> Index<usize> for Simplex<T> {
    type Output = Vector3<T>;

    #[inline]
    fn index(&self, i: usize) -> &Vector3<T> {
        &self.points[i]
    }
}

impl<T> IndexMut<usize> for Simplex<T> {
    #[inline]
    fn index_mut(&mut self, i: usize) -> &mut Vector3<T> {
        &mut self.points[i]
    }
}

impl<T: Real> Simplex<T> {
    /// GJK step: updates the simplex and the next search direction. Returns true iff the origin lies inside the
    /// tetrahedron.
    pub fn handle(&mut self, direction: &mut Vector3<T>) -> bool {
        match self.size {
            1 => self.handle_point(direction),
            2 => self.handle_line(direction),
            3 => self.handle_triangle(direction),
            4 => self.handle_tetrahedron(direction),
            _ => false,
        }
    }

    fn handle_point(&mut self, direction: &mut Vector3<T>) -> bool {
        *direction = -self.points[0];
        false
    }

    fn near_zero(v: Vector3<T>) -> bool {
        const EPS: f32 = 1e-7;
        v.dot(v) <= T::from_f64((EPS * EPS) as f64)
    }

    /// C++ is undefined here when `v` is too short for any cross product to clear `near_zero`. Falls back to +X, like
    /// the same helper in EPA.
    fn any_perp(v: Vector3<T>) -> Vector3<T> {
        let (zero, one) = (T::ZERO, T::ONE);
        let axes = [
            Vector3::new(one, zero, zero),
            Vector3::new(zero, one, zero),
            Vector3::new(zero, zero, one),
        ];

        axes.into_iter()
            .map(|axis| v.cross(axis))
            .find(|d| !Self::near_zero(*d))
            .unwrap_or(axes[0])
    }

    fn handle_line(&mut self, direction: &mut Vector3<T>) -> bool {
        let [a, b, ..] = self.points;

        let ab = b - a;
        let ao = -a;

        if ab.point_to_same_direction(ao) {
            // Needed to validly handle a collision of colliders placed at the same origin.
            let n = ab.cross(ao);
            *direction = if Self::near_zero(n) {
                Self::any_perp(ab)
            } else {
                n.cross(ab)
            };
            return false;
        }
        self.assign(&[a]);
        *direction = ao;
        false
    }

    fn handle_triangle(&mut self, direction: &mut Vector3<T>) -> bool {
        let [a, b, c, _] = self.points;

        let ab = b - a;
        let ac = c - a;
        let ao = -a;

        let abc = ab.cross(ac);

        // Region AC
        if abc.cross(ac).point_to_same_direction(ao) {
            if ac.point_to_same_direction(ao) {
                self.assign(&[a, c]);
                *direction = ac.cross(ao).cross(ac);
                return false;
            }
            self.assign(&[a, b]);
            return self.handle_line(direction);
        }

        // Region AB
        if ab.cross(abc).point_to_same_direction(ao) {
            self.assign(&[a, b]);
            return self.handle_line(direction);
        }

        // Above or below the triangle
        if abc.point_to_same_direction(ao) {
            *direction = abc;
        } else {
            self.assign(&[a, c, b]); // flip winding
            *direction = -abc;
        }
        false
    }

    fn handle_tetrahedron(&mut self, direction: &mut Vector3<T>) -> bool {
        let [a, b, c, d] = self.points;

        let ab = b - a;
        let ac = c - a;
        let ad = d - a;
        let ao = -a;

        let abc = ab.cross(ac);
        let acd = ac.cross(ad);
        let adb = ad.cross(ab);

        if abc.point_to_same_direction(ao) {
            self.assign(&[a, b, c]);
            return self.handle_triangle(direction);
        }
        if acd.point_to_same_direction(ao) {
            self.assign(&[a, c, d]);
            return self.handle_triangle(direction);
        }
        if adb.point_to_same_direction(ao) {
            self.assign(&[a, d, b]);
            return self.handle_triangle(direction);
        }
        // Origin inside the tetrahedron
        true
    }
}
