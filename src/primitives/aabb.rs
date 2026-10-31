use crate::linear_algebra::Vector3;
use crate::scalar::Scalar;
use std::marker::PhantomData;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpAxis {
    X,
    Y,
    Z,
}

/// Picks an [`Aabb`]'s up axis at compile time, in place of the C++ `UpAxis` non-type parameter.
pub trait UpAxisKind: Copy + std::fmt::Debug + Default + PartialEq + 'static {
    const AXIS: UpAxis;
}

macro_rules! up_axis_kind {
    ($name:ident, $axis:ident) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        pub struct $name;

        impl UpAxisKind for $name {
            const AXIS: UpAxis = UpAxis::$axis;
        }
    };
}
up_axis_kind!(UpX, X);
up_axis_kind!(UpY, Y);
up_axis_kind!(UpZ, Z);

/// An axis-aligned bounding box.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Aabb<T, U = UpY> {
    pub min: Vector3<T>,
    pub max: Vector3<T>,
    up_axis: PhantomData<U>,
}

impl<T, U> Aabb<T, U> {
    #[inline]
    pub const fn new(min: Vector3<T>, max: Vector3<T>) -> Self {
        Self { min, max, up_axis: PhantomData }
    }
}

impl<T: Scalar, U: UpAxisKind> Aabb<T, U> {
    #[inline]
    pub const fn get_up_axis() -> UpAxis {
        U::AXIS
    }

    #[inline]
    pub fn center(&self) -> Vector3<T> {
        (self.min + self.max) / T::from_f64(2.0)
    }

    #[inline]
    pub fn extents(&self) -> Vector3<T> {
        (self.max - self.min) / T::from_f64(2.0)
    }

    /// The center of the face furthest along the up axis.
    pub fn top(&self) -> Vector3<T> {
        self.center_with_up(self.max)
    }

    /// The center of the face furthest against the up axis.
    pub fn bottom(&self) -> Vector3<T> {
        self.center_with_up(self.min)
    }

    fn center_with_up(&self, corner: Vector3<T>) -> Vector3<T> {
        let center = self.center();
        match U::AXIS {
            UpAxis::X => Vector3::new(corner.x, center.y, center.z),
            UpAxis::Y => Vector3::new(center.x, corner.y, center.z),
            UpAxis::Z => Vector3::new(center.x, center.y, corner.z),
        }
    }

    pub fn vertices(&self) -> [Vector3<T>; 8] {
        let (min, max) = (self.min, self.max);
        [
            Vector3::new(min.x, min.y, min.z),
            Vector3::new(max.x, min.y, min.z),
            Vector3::new(min.x, max.y, min.z),
            Vector3::new(max.x, max.y, min.z),
            Vector3::new(min.x, min.y, max.z),
            Vector3::new(max.x, min.y, max.z),
            Vector3::new(min.x, max.y, max.z),
            Vector3::new(max.x, max.y, max.z),
        ]
    }

    pub fn is_collide(&self, other: &Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }
}
