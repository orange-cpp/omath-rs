use crate::Vector3;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Target<T> {
    pub origin: Vector3<T>,
    pub velocity: Vector3<T>,
    pub is_airborne: bool,
}
