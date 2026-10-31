use super::Aabb;
use crate::linear_algebra::{Mat, StoreOrder, Triangle, Vector2, Vector3};
use crate::scalar::{Real, Scalar};
use std::cell::Cell;
use std::fmt;
use std::marker::PhantomData;

/// What a [`Mesh`] needs from an engine. Replaces the C++ `MeshTypeTrait` and the matrix template parameters.
pub trait MeshEngine: 'static {
    type Scalar: Real;
    type Order: StoreOrder;
    type RotationAngles: Copy + Default + fmt::Debug;

    fn rotation_matrix(rotation: &Self::RotationAngles) -> Mat<4, 4, Self::Scalar, Self::Order>;
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vertex<V = Vector3<f32>, U = Vector2<f32>> {
    pub position: V,
    pub normal: V,
    pub uv: U,
}

/// Anything a vertex buffer can hold: either a [`Vertex`], or a bare position.
pub trait HasPosition<T> {
    fn position(&self) -> Vector3<T>;
}

impl<T: Copy, U> HasPosition<T> for Vertex<Vector3<T>, U> {
    #[inline]
    fn position(&self) -> Vector3<T> {
        self.position
    }
}

impl<T: Copy> HasPosition<T> for Vector3<T> {
    #[inline]
    fn position(&self) -> Vector3<T> {
        *self
    }
}

/// A triangle mesh with a transform.
///
/// The vertices share the engine's scalar type: the C++ `unreal_engine` aliases mix `float` vertices with `double`
/// matrices, which only compiles for as long as no world-space method is instantiated.
///
/// `Vbo` and `Ebo` are any contiguous storage, so `Vec` or a fixed-size array. The transform is cached on first use and
/// dropped by the setters; the cache sits in a [`Cell`] so that the getters can take `&self`.
pub struct Mesh<
    E: MeshEngine,
    V = Vertex<Vector3<<E as MeshEngine>::Scalar>>,
    Vbo = Vec<V>,
    Ebo = Vec<Vector3<u32>>,
> {
    pub vertex_buffer: Vbo,
    pub element_buffer_object: Ebo,

    origin: Vector3<E::Scalar>,
    scale: Vector3<E::Scalar>,
    rotation_angles: E::RotationAngles,
    to_world_matrix: Cell<Option<Mat<4, 4, E::Scalar, E::Order>>>,
    vertex: PhantomData<V>,
}

impl<E: MeshEngine, V, Vbo: Clone, Ebo: Clone> Clone for Mesh<E, V, Vbo, Ebo> {
    fn clone(&self) -> Self {
        Self {
            vertex_buffer: self.vertex_buffer.clone(),
            element_buffer_object: self.element_buffer_object.clone(),
            origin: self.origin,
            scale: self.scale,
            rotation_angles: self.rotation_angles,
            to_world_matrix: self.to_world_matrix.clone(),
            vertex: PhantomData,
        }
    }
}

impl<E: MeshEngine, V, Vbo: fmt::Debug, Ebo: fmt::Debug> fmt::Debug for Mesh<E, V, Vbo, Ebo> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Mesh")
            .field("vertex_buffer", &self.vertex_buffer)
            .field("element_buffer_object", &self.element_buffer_object)
            .field("origin", &self.origin)
            .field("scale", &self.scale)
            .field("rotation_angles", &self.rotation_angles)
            .finish_non_exhaustive()
    }
}

impl<E: MeshEngine, V, Vbo, Ebo> Mesh<E, V, Vbo, Ebo> {
    /// A mesh at the world origin, unrotated, with a scale of 1.
    pub fn new(vertex_buffer: Vbo, element_buffer_object: Ebo) -> Self {
        Self::with_scale(
            vertex_buffer,
            element_buffer_object,
            Vector3::splat(E::Scalar::ONE),
        )
    }

    pub fn with_scale(
        vertex_buffer: Vbo,
        element_buffer_object: Ebo,
        scale: Vector3<E::Scalar>,
    ) -> Self {
        Self {
            vertex_buffer,
            element_buffer_object,
            origin: Vector3::default(),
            scale,
            rotation_angles: E::RotationAngles::default(),
            to_world_matrix: Cell::new(None),
            vertex: PhantomData,
        }
    }

    pub fn set_origin(&mut self, new_origin: Vector3<E::Scalar>) {
        self.origin = new_origin;
        self.to_world_matrix.set(None);
    }

    pub fn set_scale(&mut self, new_scale: Vector3<E::Scalar>) {
        self.scale = new_scale;
        self.to_world_matrix.set(None);
    }

    pub fn set_rotation(&mut self, new_rotation_angles: E::RotationAngles) {
        self.rotation_angles = new_rotation_angles;
        self.to_world_matrix.set(None);
    }

    #[inline]
    pub fn get_origin(&self) -> Vector3<E::Scalar> {
        self.origin
    }

    #[inline]
    pub fn get_scale(&self) -> Vector3<E::Scalar> {
        self.scale
    }

    #[inline]
    pub fn get_rotation_angles(&self) -> E::RotationAngles {
        self.rotation_angles
    }

    pub fn get_to_world_matrix(&self) -> Mat<4, 4, E::Scalar, E::Order> {
        if let Some(to_world) = self.to_world_matrix.get() {
            return to_world;
        }

        let to_world = Mat::translation(self.origin)
            * E::rotation_matrix(&self.rotation_angles)
            * Mat::scale(self.scale);
        self.to_world_matrix.set(Some(to_world));
        to_world
    }

    pub fn vertex_position_to_world_space(
        &self,
        vertex_position: Vector3<E::Scalar>,
    ) -> Vector3<E::Scalar> {
        let abs_vec = self.get_to_world_matrix()
            * Mat::<4, 1, E::Scalar, E::Order>::column_from_vector(vertex_position);

        Vector3::new(abs_vec.at(0, 0), abs_vec.at(1, 0), abs_vec.at(2, 0))
    }
}

impl<E: MeshEngine, V: HasPosition<E::Scalar>, Vbo: AsRef<[V]>, Ebo> Mesh<E, V, Vbo, Ebo> {
    /// Axis-aligned bounds in the mesh's own frame. No transform is involved, so this is a plain min/max sweep of the
    /// vertex buffer, and the result only changes when the vertex buffer does. Intended as a broadphase reject in front
    /// of a face-by-face trace: hoist it out of the query loop and slab-test against it, rather than paying the sweep
    /// per trace.
    pub fn local_bounds(&self) -> Aabb<E::Scalar> {
        let mut min = Vector3::splat(E::Scalar::MAX);
        let mut max = Vector3::splat(-E::Scalar::MAX);

        for vertex in self.vertex_buffer.as_ref() {
            let position = vertex.position();

            min.x = min.x.min_of(position.x);
            min.y = min.y.min_of(position.y);
            min.z = min.z.min_of(position.z);
            max.x = max.x.max_of(position.x);
            max.y = max.y.max_of(position.y);
            max.z = max.z.max_of(position.z);
        }

        Aabb::new(min, max)
    }

    /// `face` is one entry of the element buffer. Panics on an index outside the vertex buffer, as `std::vector::at`
    /// throws for it in C++.
    pub fn make_face_in_local_space(&self, face: Vector3<u32>) -> Triangle<Vector3<E::Scalar>> {
        let vertices = self.vertex_buffer.as_ref();
        Triangle::new(
            vertices[face.x as usize].position(),
            vertices[face.y as usize].position(),
            vertices[face.z as usize].position(),
        )
    }

    pub fn make_face_in_world_space(&self, face: Vector3<u32>) -> Triangle<Vector3<E::Scalar>> {
        let local = self.make_face_in_local_space(face);

        Triangle::new(
            self.vertex_position_to_world_space(local.vertex1),
            self.vertex_position_to_world_space(local.vertex2),
            self.vertex_position_to_world_space(local.vertex3),
        )
    }
}
