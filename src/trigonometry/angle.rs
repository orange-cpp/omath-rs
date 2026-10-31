use super::angles;
use crate::scalar::Real;
use std::cmp::Ordering;
use std::fmt;
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

/// What happens to a value outside an [`AngleRange`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnglePolicy {
    /// Wrap around, for cyclic angles such as yaw.
    Normalized,
    /// Saturate at the bounds, for limited angles such as pitch.
    Clamped,
}

/// Compile-time bounds of an [`Angle`], in degrees.
///
/// Stands in for the C++ `Angle<T, min, max, flags>` non-type parameters, because float const generics are not stable
/// in Rust. Use [`angle_range!`](crate::angle_range) to declare one.
pub trait AngleRange: Copy + fmt::Debug + 'static {
    const MIN: f64;
    const MAX: f64;
    const POLICY: AnglePolicy;
}

/// Declares a marker type implementing [`AngleRange`].
///
/// ```
/// omath::angle_range!(pub Pitch, -89.0, 89.0, Clamped);
/// let pitch = omath::Angle::<f32, Pitch>::from_degrees(120.0);
/// assert_eq!(pitch.as_degrees(), 89.0);
/// ```
#[macro_export]
macro_rules! angle_range {
    ($vis:vis $name:ident, $min:expr, $max:expr, $policy:ident) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq)]
        $vis struct $name;

        impl $crate::AngleRange for $name {
            const MIN: f64 = $min;
            const MAX: f64 = $max;
            const POLICY: $crate::AnglePolicy = $crate::AnglePolicy::$policy;
        }
    };
}

crate::angle_range!(pub Normalized0To360, 0.0, 360.0, Normalized);
crate::angle_range!(pub Clamped0To180, 0.0, 180.0, Clamped);

/// The angle returned by `angle_between` on the vector types.
pub type AngleBetween<T> = Angle<T, Clamped0To180>;

/// Anything rotation matrix builders can take a sine and cosine of.
pub trait Trig<T> {
    fn sin(&self) -> T;
    fn cos(&self) -> T;
}

/// An angle kept inside the range `R`. Stored in degrees.
pub struct Angle<T = f32, R = Normalized0To360> {
    degrees: T,
    range: PhantomData<R>,
}

impl<T: Real, R: AngleRange> Angle<T, R> {
    #[inline]
    fn fit(degrees: T) -> T {
        let min = T::from_f64(R::MIN);
        let max = T::from_f64(R::MAX);
        match R::POLICY {
            AnglePolicy::Normalized => angles::wrap_angle(degrees, min, max),
            AnglePolicy::Clamped => degrees.clamped(min, max),
        }
    }

    #[inline]
    pub fn from_degrees(degrees: T) -> Self {
        Self {
            degrees: Self::fit(degrees),
            range: PhantomData,
        }
    }

    #[inline]
    pub fn from_radians(radians: T) -> Self {
        Self::from_degrees(angles::radians_to_degrees(radians))
    }

    #[inline]
    pub fn from_asin(value: T) -> Self {
        Self::from_radians(value.asin())
    }

    #[inline]
    pub fn from_acos(value: T) -> Self {
        Self::from_radians(value.acos())
    }

    #[inline]
    pub fn from_atan(value: T) -> Self {
        Self::from_radians(value.atan())
    }

    #[inline]
    pub fn from_atan2(y: T, x: T) -> Self {
        Self::from_radians(y.atan2(x))
    }

    #[inline]
    pub fn as_degrees(self) -> T {
        self.degrees
    }

    #[inline]
    pub fn as_radians(self) -> T {
        angles::degrees_to_radians(self.degrees)
    }

    #[inline]
    pub fn sin(self) -> T {
        self.as_radians().sin()
    }

    #[inline]
    pub fn cos(self) -> T {
        self.as_radians().cos()
    }

    #[inline]
    pub fn tan(self) -> T {
        self.as_radians().tan()
    }

    #[inline]
    pub fn cot(self) -> T {
        self.cos() / self.sin()
    }
}

impl<T: Real, R: AngleRange> Trig<T> for Angle<T, R> {
    #[inline]
    fn sin(&self) -> T {
        Angle::sin(*self)
    }

    #[inline]
    fn cos(&self) -> T {
        Angle::cos(*self)
    }
}

impl<T: Copy, R> Clone for Angle<T, R> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Copy, R> Copy for Angle<T, R> {}

impl<T: Real, R: AngleRange> Default for Angle<T, R> {
    #[inline]
    fn default() -> Self {
        Self::from_degrees(T::ZERO)
    }
}

impl<T: PartialEq, R> PartialEq for Angle<T, R> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.degrees == other.degrees
    }
}

impl<T: PartialOrd, R> PartialOrd for Angle<T, R> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.degrees.partial_cmp(&other.degrees)
    }
}

impl<T: fmt::Debug, R> fmt::Debug for Angle<T, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Angle").field(&self.degrees).finish()
    }
}

impl<T: fmt::Display, R> fmt::Display for Angle<T, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}deg", self.degrees)
    }
}

impl<T: Real, R: AngleRange> Add for Angle<T, R> {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Self::from_degrees(Self::fit(self.degrees + other.degrees))
    }
}

impl<T: Real, R: AngleRange> Sub for Angle<T, R> {
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self {
        Self::from_degrees(self.degrees - other.degrees)
    }
}

impl<T: Real, R: AngleRange> Neg for Angle<T, R> {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        Self::from_degrees(-self.degrees)
    }
}

impl<T: Real, R: AngleRange> AddAssign for Angle<T, R> {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl<T: Real, R: AngleRange> SubAssign for Angle<T, R> {
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}
