use crate::linear_algebra::Vector3;
use crate::scalar::Real;

/// Oriented bounding box: a rectangular cuboid defined by a center, three orthonormal local axes, and the half-size
/// along each of those axes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Obb<T> {
    pub center: Vector3<T>,
    pub axis_x: Vector3<T>,
    pub axis_y: Vector3<T>,
    pub axis_z: Vector3<T>,
    pub half_extents: Vector3<T>,
}

impl<T: Real> Obb<T> {
    pub fn vertices(&self) -> [Vector3<T>; 8] {
        let center = self.center;
        let ex = self.axis_x * self.half_extents.x;
        let ey = self.axis_y * self.half_extents.y;
        let ez = self.axis_z * self.half_extents.z;

        [
            center - ex - ey - ez,
            center + ex - ey - ez,
            center - ex + ey - ez,
            center + ex + ey - ez,
            center - ex - ey + ez,
            center + ex - ey + ez,
            center - ex + ey + ez,
            center + ex + ey + ez,
        ]
    }
}
