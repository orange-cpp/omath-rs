use crate::linear_algebra::Vector3;
use crate::scalar::Real;

/// A convex shape that GJK and EPA can query. Replaces the C++ `ColliderInterface`.
///
/// A shape without a meaningful origin returns zero from [`get_origin`](Self::get_origin): GJK then starts along +X,
/// which is what C++ does for a collider that has no `get_origin` at all.
pub trait Collider {
    type Scalar: Real;

    /// The world-space vertex furthest along `direction`.
    fn find_abs_furthest_vertex_position(
        &self,
        direction: Vector3<Self::Scalar>,
    ) -> Vector3<Self::Scalar>;

    fn get_origin(&self) -> Vector3<Self::Scalar>;
    fn set_origin(&mut self, new_origin: Vector3<Self::Scalar>);
}
