use super::{Vector, Vector3};
use crate::scalar::{Real, Scalar, Signed};

/// A triangle, with the right-angle corner conventionally at `vertex2`:
///
/// ```text
///  v1
///  |\
///  | \
/// a|  \ hypot
///  |   \
/// v2 --- v3
///     b
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Triangle<V> {
    pub vertex1: V,
    pub vertex2: V,
    pub vertex3: V,
}

impl<V> Triangle<V> {
    #[inline]
    pub const fn new(vertex1: V, vertex2: V, vertex3: V) -> Self {
        Self {
            vertex1,
            vertex2,
            vertex3,
        }
    }
}

impl<V: Vector> Triangle<V>
where
    V::Scalar: Real,
{
    #[inline]
    pub fn side_a_length(&self) -> V::Scalar {
        self.vertex1.distance_to(self.vertex2)
    }

    #[inline]
    pub fn side_b_length(&self) -> V::Scalar {
        self.vertex3.distance_to(self.vertex2)
    }

    #[inline]
    pub fn hypot(&self) -> V::Scalar {
        self.vertex1.distance_to(self.vertex3)
    }

    pub fn is_rectangular(&self) -> bool {
        let side_a = self.side_a_length();
        let side_b = self.side_b_length();
        let hypot = self.hypot();

        (side_a * side_a + side_b * side_b - hypot * hypot).abs_value()
            <= V::Scalar::from_f64(0.0001f32 as f64)
    }
}

impl<V: Vector> Triangle<V> {
    #[inline]
    pub fn side_a_vector(&self) -> V {
        self.vertex1 - self.vertex2
    }

    #[inline]
    pub fn side_b_vector(&self) -> V {
        self.vertex3 - self.vertex2
    }

    #[inline]
    pub fn mid_point(&self) -> V {
        (self.vertex1 + self.vertex2 + self.vertex3) / V::Scalar::from_f64(3.0)
    }
}

impl<T: Real> Triangle<Vector3<T>> {
    #[inline]
    pub fn calculate_normal(&self) -> Vector3<T> {
        self.side_b_vector()
            .cross(self.side_a_vector())
            .normalized()
    }
}
