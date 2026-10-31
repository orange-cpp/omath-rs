use super::{Mesh, MeshEngine};
use crate::linear_algebra::Vector3;

/// A [`Mesh`] with the fixed-size buffers of [`create_plane`].
pub type PlaneMesh<E> = Mesh<
    E,
    Vector3<<E as MeshEngine>::Scalar>,
    [Vector3<<E as MeshEngine>::Scalar>; 4],
    [Vector3<u32>; 2],
>;

/// Builds a quad from the edge `vertex_a` to `vertex_b`, swept `size` along `direction`.
///
/// The first triangle's indices are kept as they are in C++, which makes it degenerate.
pub fn create_plane<E: MeshEngine>(
    vertex_a: Vector3<E::Scalar>,
    vertex_b: Vector3<E::Scalar>,
    direction: Vector3<E::Scalar>,
    size: E::Scalar,
) -> PlaneMesh<E> {
    let second_vertex_a = vertex_a + direction * size;
    let second_vertex_b = vertex_b + direction * size;

    let grid = [vertex_a, vertex_b, second_vertex_a, second_vertex_b];
    let poly = [Vector3::new(1, 1, 2), Vector3::new(0, 1, 3)];

    Mesh::new(grid, poly)
}
