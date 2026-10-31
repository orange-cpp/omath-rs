use super::{Mat, Vector3};
use crate::scalar::{Real, Scalar};
use std::fmt;
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg};

/// A rotation quaternion. [`Default`] is the identity.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Quaternion<T> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

impl<T: Scalar> Default for Quaternion<T> {
    #[inline]
    fn default() -> Self {
        Self::new(T::ZERO, T::ZERO, T::ZERO, T::ONE)
    }
}

impl<T> Quaternion<T> {
    #[inline]
    pub const fn new(x: T, y: T, z: T, w: T) -> Self {
        Self { x, y, z, w }
    }
}

impl<T: Real> Quaternion<T> {
    /// From a normalized axis and an angle in radians.
    pub fn from_axis_angle(axis: Vector3<T>, angle_rad: T) -> Self {
        let half = angle_rad / T::from_f64(2.0);
        let s = half.sin();
        Self::new(axis.x * s, axis.y * s, axis.z * s, half.cos())
    }

    /// Negates the vector part.
    #[inline]
    #[must_use]
    pub fn conjugate(self) -> Self {
        Self::new(-self.x, -self.y, -self.z, self.w)
    }

    #[inline]
    pub fn dot(self, other: Self) -> T {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    #[inline]
    pub fn length_sqr(self) -> T {
        self.dot(self)
    }

    #[inline]
    pub fn length(self) -> T {
        self.length_sqr().sqrt()
    }

    #[inline]
    #[must_use]
    pub fn normalized(self) -> Self {
        let length = self.length();
        if length != T::ZERO { self * (T::ONE / length) } else { self }
    }

    /// `q* / |q|^2`. For unit quaternions this equals the conjugate.
    #[inline]
    #[must_use]
    pub fn inverse(self) -> Self {
        self.conjugate() * (T::ONE / self.length_sqr())
    }

    /// `v' = q * pure(v) * q^-1`, computed via Rodrigues' formula to avoid the full quaternion product.
    pub fn rotate(self, v: Vector3<T>) -> Vector3<T> {
        let two = T::from_f64(2.0);
        let q_vec = Vector3::new(self.x, self.y, self.z);
        let cross = q_vec.cross(v);
        v + cross * (two * self.w) + q_vec.cross(cross) * two
    }

    /// 3x3 rotation matrix from this unit quaternion.
    pub fn to_rotation_matrix3(self) -> Mat<3, 3, T> {
        let [r0, r1, r2] = self.rotation_rows();
        Mat::from_rows([r0, r1, r2])
    }

    /// 4x4 rotation matrix, with a homogeneous row and column.
    pub fn to_rotation_matrix4(self) -> Mat<4, 4, T> {
        let [r0, r1, r2] = self.rotation_rows();
        let (o, one) = (T::ZERO, T::ONE);
        Mat::from_rows([[r0[0], r0[1], r0[2], o], [r1[0], r1[1], r1[2], o], [r2[0], r2[1], r2[2], o], [o, o, o, one]])
    }

    fn rotation_rows(self) -> [[T; 3]; 3] {
        let Self { x, y, z, w } = self;
        let (xx, yy, zz) = (x * x, y * y, z * z);
        let (xy, xz, yz) = (x * y, x * z, y * z);
        let (wx, wy, wz) = (w * x, w * y, w * z);
        let (one, two) = (T::ONE, T::from_f64(2.0));

        [
            [one - two * (yy + zz), two * (xy - wz), two * (xz + wy)],
            [two * (xy + wz), one - two * (xx + zz), two * (yz - wx)],
            [two * (xz - wy), two * (yz + wx), one - two * (xx + yy)],
        ]
    }

    #[inline]
    pub fn as_array(self) -> [T; 4] {
        [self.x, self.y, self.z, self.w]
    }
}

impl<T: Real> Mul for Quaternion<T> {
    type Output = Self;

    /// Hamilton product.
    #[inline]
    fn mul(self, other: Self) -> Self {
        Self::new(
            self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
            self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
        )
    }
}

impl<T: Real> MulAssign for Quaternion<T> {
    #[inline]
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

impl<T: Real> Mul<T> for Quaternion<T> {
    type Output = Self;

    #[inline]
    fn mul(self, scalar: T) -> Self {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar, self.w * scalar)
    }
}

impl<T: Real> MulAssign<T> for Quaternion<T> {
    #[inline]
    fn mul_assign(&mut self, scalar: T) {
        *self = *self * scalar;
    }
}

impl<T: Real> Add for Quaternion<T> {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z, self.w + other.w)
    }
}

impl<T: Real> AddAssign for Quaternion<T> {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl<T: Real> Neg for Quaternion<T> {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z, -self.w)
    }
}

impl<T: fmt::Display> fmt::Display for Quaternion<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}, {}, {}]", self.x, self.y, self.z, self.w)
    }
}
