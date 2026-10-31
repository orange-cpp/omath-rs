use super::{Mesh, MeshEngine};
use crate::linear_algebra::Vector3;

/// A [`Mesh`] with the fixed-size buffers of [`create_box`].
pub type BoxMesh<E> = Mesh<
    E,
    Vector3<<E as MeshEngine>::Scalar>,
    [Vector3<<E as MeshEngine>::Scalar>; 8],
    [Vector3<u32>; 12],
>;

/// Builds a box around the `bottom` to `top` line. `ratio` is its height over half its side length, and the C++ default
/// is `4`.
pub fn create_box<E: MeshEngine>(
    top: Vector3<E::Scalar>,
    bottom: Vector3<E::Scalar>,
    dir_forward: Vector3<E::Scalar>,
    dir_right: Vector3<E::Scalar>,
    ratio: E::Scalar,
) -> BoxMesh<E> {
    let height = top.distance_to(bottom);
    let side_size = height / ratio;

    // corner layout (0-3 bottom, 4-7 top)
    let p = [
        bottom + (dir_forward + dir_right) * side_size, // front-right-bottom
        bottom + (dir_forward - dir_right) * side_size, // front-left-bottom
        bottom + (-dir_forward + dir_right) * side_size, // back-right-bottom
        bottom + (-dir_forward - dir_right) * side_size, // back-left-bottom
        top + (dir_forward + dir_right) * side_size,    // front-right-top
        top + (dir_forward - dir_right) * side_size,    // front-left-top
        top + (-dir_forward + dir_right) * side_size,   // back-right-top
        top + (-dir_forward - dir_right) * side_size,   // back-left-top
    ];

    let poly = [
        // bottom face (+Y up, so wound clockwise when viewed from above)
        Vector3::new(0, 2, 3),
        Vector3::new(0, 3, 1),
        // top face
        Vector3::new(4, 7, 6),
        Vector3::new(4, 5, 7),
        // front face
        Vector3::new(0, 5, 1),
        Vector3::new(0, 4, 5),
        // right face
        Vector3::new(0, 6, 2),
        Vector3::new(0, 4, 6),
        // back face
        Vector3::new(2, 7, 3),
        Vector3::new(2, 6, 7),
        // left face
        Vector3::new(1, 7, 5),
        Vector3::new(1, 3, 7),
    ];

    Mesh::new(p, poly)
}
