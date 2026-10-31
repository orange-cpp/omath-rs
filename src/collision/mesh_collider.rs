use super::Collider;
use crate::linear_algebra::Vector3;
use crate::primitives::{HasPosition, Mesh, MeshEngine, Vertex};
use std::fmt;

/// A [`Mesh`] as a [`Collider`]. Only the vertex buffer matters to it, so the element buffer can stay empty.
pub struct MeshCollider<
    E: MeshEngine,
    V = Vertex<Vector3<<E as MeshEngine>::Scalar>>,
    Vbo = Vec<V>,
    Ebo = Vec<Vector3<u32>>,
> {
    mesh: Mesh<E, V, Vbo, Ebo>,
}

impl<E: MeshEngine, V, Vbo: Clone, Ebo: Clone> Clone for MeshCollider<E, V, Vbo, Ebo> {
    fn clone(&self) -> Self {
        Self {
            mesh: self.mesh.clone(),
        }
    }
}

impl<E: MeshEngine, V, Vbo: fmt::Debug, Ebo: fmt::Debug> fmt::Debug
    for MeshCollider<E, V, Vbo, Ebo>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MeshCollider")
            .field("mesh", &self.mesh)
            .finish()
    }
}

impl<E: MeshEngine, V, Vbo, Ebo> MeshCollider<E, V, Vbo, Ebo> {
    #[inline]
    pub fn new(mesh: Mesh<E, V, Vbo, Ebo>) -> Self {
        Self { mesh }
    }

    #[inline]
    pub fn get_mesh(&self) -> &Mesh<E, V, Vbo, Ebo> {
        &self.mesh
    }

    #[inline]
    pub fn get_mesh_mut(&mut self) -> &mut Mesh<E, V, Vbo, Ebo> {
        &mut self.mesh
    }
}

impl<E: MeshEngine, V: HasPosition<E::Scalar>, Vbo: AsRef<[V]>, Ebo> MeshCollider<E, V, Vbo, Ebo> {
    /// The first vertex furthest along the world-space `direction`, as `std::ranges::max_element` picks it. Panics on
    /// an empty vertex buffer.
    pub fn find_furthest_vertex(&self, direction: Vector3<E::Scalar>) -> &V {
        // `world(v) · d = v · Mᵀ·d`, with the translation constant across vertices. Moving the direction into local
        // space once is far cheaper than a 4x4 multiply per vertex.
        let m = self.mesh.get_to_world_matrix();
        let d_local = Vector3::new(
            m.at(0, 0) * direction.x + m.at(1, 0) * direction.y + m.at(2, 0) * direction.z,
            m.at(0, 1) * direction.x + m.at(1, 1) * direction.y + m.at(2, 1) * direction.z,
            m.at(0, 2) * direction.x + m.at(1, 2) * direction.y + m.at(2, 2) * direction.z,
        );

        let (first, rest) = self
            .mesh
            .vertex_buffer
            .as_ref()
            .split_first()
            .expect("the vertex buffer is empty");
        let mut furthest = first;
        for vertex in rest {
            if furthest.position().dot(d_local) < vertex.position().dot(d_local) {
                furthest = vertex;
            }
        }
        furthest
    }
}

impl<E: MeshEngine, V: HasPosition<E::Scalar>, Vbo: AsRef<[V]>, Ebo> Collider
    for MeshCollider<E, V, Vbo, Ebo>
{
    type Scalar = E::Scalar;

    fn find_abs_furthest_vertex_position(
        &self,
        direction: Vector3<E::Scalar>,
    ) -> Vector3<E::Scalar> {
        self.mesh
            .vertex_position_to_world_space(self.find_furthest_vertex(direction).position())
    }

    #[inline]
    fn get_origin(&self) -> Vector3<E::Scalar> {
        self.mesh.get_origin()
    }

    #[inline]
    fn set_origin(&mut self, new_origin: Vector3<E::Scalar>) {
        self.mesh.set_origin(new_origin);
    }
}
