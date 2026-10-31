use crate::scalar::{Real, Scalar, Signed};
use crate::trigonometry::AngleBetween;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// Failure of `angle_between`: one of the vectors has zero length.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vector3Error {
    ImpossibleBetweenAngle,
}

impl fmt::Display for Vector3Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("cannot compute the angle to or from a zero-length vector")
    }
}

impl std::error::Error for Vector3Error {}

/// Operations shared by [`Vector2`], [`Vector3`] and [`Vector4`], for code generic over the dimension.
pub trait Vector:
    Copy
    + PartialEq
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<<Self as Vector>::Scalar, Output = Self>
    + Div<<Self as Vector>::Scalar, Output = Self>
{
    type Scalar: Scalar;

    fn dot(self, other: Self) -> Self::Scalar;
    fn length_sqr(self) -> Self::Scalar;
    fn distance_to_sqr(self, other: Self) -> Self::Scalar;

    #[inline]
    fn length(self) -> Self::Scalar
    where
        Self::Scalar: Real,
    {
        self.length_sqr().sqrt()
    }

    #[inline]
    fn distance_to(self, other: Self) -> Self::Scalar
    where
        Self::Scalar: Real,
    {
        (self - other).length()
    }
}

macro_rules! vector_type {
    ($(#[$doc:meta])* $name:ident, $len:literal, $($field:ident : $idx:tt),+) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, Default, PartialEq)]
        #[repr(C)]
        pub struct $name<T> {
            $(pub $field: T,)+
        }

        impl<T> $name<T> {
            #[inline]
            pub const fn new($($field: T),+) -> Self {
                Self { $($field),+ }
            }
        }

        impl<T: Scalar> $name<T> {
            /// Every component set to `value`.
            #[inline]
            pub fn splat(value: T) -> Self {
                Self { $($field: value),+ }
            }

            /// Component-wise numeric cast, with `as` semantics.
            ///
            /// Goes through `f64`, so integers above 2^53 lose precision.
            #[inline]
            pub fn cast<U: Scalar>(self) -> $name<U> {
                $name { $($field: U::from_f64(self.$field.to_f64())),+ }
            }

            #[inline]
            pub fn dot(self, other: Self) -> T {
                let mut sum = T::ZERO;
                $(sum += self.$field * other.$field;)+
                sum
            }

            #[inline]
            pub fn length_sqr(self) -> T {
                self.dot(self)
            }

            #[inline]
            pub fn distance_to_sqr(self, other: Self) -> T {
                (self - other).length_sqr()
            }

            /// Sum of the elements.
            #[inline]
            pub fn sum(self) -> T {
                let mut sum = T::ZERO;
                $(sum += self.$field;)+
                sum
            }

            #[inline]
            pub fn point_to_same_direction(self, other: Self) -> bool {
                self.dot(other) > T::ZERO
            }

            /// Orders by length. Comparing squared lengths gives the same order without the square root.
            ///
            /// This is a method rather than `PartialOrd`, which would have to agree with component-wise `==`.
            #[inline]
            pub fn cmp_length(self, other: Self) -> Option<Ordering> {
                self.length_sqr().partial_cmp(&other.length_sqr())
            }

            #[inline]
            pub fn as_array(self) -> [T; $len] {
                [$(self.$field),+]
            }

            #[inline]
            pub fn as_tuple(self) -> ($(vector_type!(@ty T, $field),)+) {
                ($(self.$field,)+)
            }

            /// Clamps every component into `[min, max]`.
            #[inline]
            #[must_use]
            pub fn clamped(self, min: T, max: T) -> Self {
                Self { $($field: self.$field.clamped(min, max)),+ }
            }
        }

        impl<T: Signed> $name<T> {
            #[inline]
            #[must_use]
            pub fn abs(self) -> Self {
                Self { $($field: self.$field.abs_value()),+ }
            }
        }

        impl<T: Real> $name<T> {
            pub fn angle_between(self, other: Self) -> Result<AngleBetween<T>, Vector3Error> {
                let bottom = self.length() * other.length();

                if bottom == T::ZERO {
                    return Err(Vector3Error::ImpossibleBetweenAngle);
                }
                Ok(AngleBetween::from_radians((self.dot(other) / bottom).acos()))
            }

            /// Uses `epsilon` degrees of tolerance. The C++ default is `0.0001`.
            pub fn is_perpendicular(self, other: Self, epsilon: T) -> bool {
                self.angle_between(other)
                    .is_ok_and(|angle| (angle.as_degrees() - T::from_f64(90.0)).abs_value() <= epsilon)
            }
        }

        impl<T: Scalar> Vector for $name<T> {
            type Scalar = T;

            #[inline]
            fn dot(self, other: Self) -> T {
                $name::dot(self, other)
            }

            #[inline]
            fn length_sqr(self) -> T {
                $name::length_sqr(self)
            }

            #[inline]
            fn distance_to_sqr(self, other: Self) -> T {
                $name::distance_to_sqr(self, other)
            }

            #[inline]
            fn length(self) -> T
            where
                T: Real,
            {
                $name::length(self)
            }

            #[inline]
            fn distance_to(self, other: Self) -> T
            where
                T: Real,
            {
                $name::distance_to(self, other)
            }
        }

        impl<T: Scalar> Add for $name<T> {
            type Output = Self;
            #[inline]
            fn add(self, other: Self) -> Self {
                Self { $($field: self.$field + other.$field),+ }
            }
        }

        impl<T: Scalar> Sub for $name<T> {
            type Output = Self;
            #[inline]
            fn sub(self, other: Self) -> Self {
                Self { $($field: self.$field - other.$field),+ }
            }
        }

        impl<T: Scalar> Mul for $name<T> {
            type Output = Self;
            #[inline]
            fn mul(self, other: Self) -> Self {
                Self { $($field: self.$field * other.$field),+ }
            }
        }

        impl<T: Scalar> Div for $name<T> {
            type Output = Self;
            #[inline]
            fn div(self, other: Self) -> Self {
                Self { $($field: self.$field / other.$field),+ }
            }
        }

        impl<T: Scalar> Add<T> for $name<T> {
            type Output = Self;
            #[inline]
            fn add(self, value: T) -> Self {
                Self { $($field: self.$field + value),+ }
            }
        }

        impl<T: Scalar> Sub<T> for $name<T> {
            type Output = Self;
            #[inline]
            fn sub(self, value: T) -> Self {
                Self { $($field: self.$field - value),+ }
            }
        }

        impl<T: Scalar> Mul<T> for $name<T> {
            type Output = Self;
            #[inline]
            fn mul(self, value: T) -> Self {
                Self { $($field: self.$field * value),+ }
            }
        }

        impl<T: Scalar> Div<T> for $name<T> {
            type Output = Self;
            #[inline]
            fn div(self, value: T) -> Self {
                Self { $($field: self.$field / value),+ }
            }
        }

        impl<T: Scalar> AddAssign for $name<T> {
            #[inline]
            fn add_assign(&mut self, other: Self) {
                *self = *self + other;
            }
        }

        impl<T: Scalar> SubAssign for $name<T> {
            #[inline]
            fn sub_assign(&mut self, other: Self) {
                *self = *self - other;
            }
        }

        impl<T: Scalar> MulAssign for $name<T> {
            #[inline]
            fn mul_assign(&mut self, other: Self) {
                *self = *self * other;
            }
        }

        impl<T: Scalar> DivAssign for $name<T> {
            #[inline]
            fn div_assign(&mut self, other: Self) {
                *self = *self / other;
            }
        }

        impl<T: Scalar> AddAssign<T> for $name<T> {
            #[inline]
            fn add_assign(&mut self, value: T) {
                *self = *self + value;
            }
        }

        impl<T: Scalar> SubAssign<T> for $name<T> {
            #[inline]
            fn sub_assign(&mut self, value: T) {
                *self = *self - value;
            }
        }

        impl<T: Scalar> MulAssign<T> for $name<T> {
            #[inline]
            fn mul_assign(&mut self, value: T) {
                *self = *self * value;
            }
        }

        impl<T: Scalar> DivAssign<T> for $name<T> {
            #[inline]
            fn div_assign(&mut self, value: T) {
                *self = *self / value;
            }
        }

        impl<T: Scalar + Neg<Output = T>> Neg for $name<T> {
            type Output = Self;
            #[inline]
            fn neg(self) -> Self {
                Self { $($field: -self.$field),+ }
            }
        }

        impl<T: Scalar> Hash for $name<T> {
            fn hash<H: Hasher>(&self, state: &mut H) {
                $(self.$field.hash_bits().hash(state);)+
            }
        }

        impl<T: fmt::Display> fmt::Display for $name<T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let mut separator = "";
                f.write_str("[")?;
                $(
                    write!(f, "{separator}{}", self.$field)?;
                    separator = ", ";
                )+
                let _ = separator;
                f.write_str("]")
            }
        }

        impl<T> From<[T; $len]> for $name<T> {
            #[inline]
            fn from(array: [T; $len]) -> Self {
                let [$($field),+] = array;
                Self { $($field),+ }
            }
        }

        impl<T> From<$name<T>> for [T; $len] {
            #[inline]
            fn from(vector: $name<T>) -> Self {
                [$(vector.$field),+]
            }
        }

        impl<T: Copy> From<&$name<T>> for ($(vector_type!(@ty T, $field),)+) {
            #[inline]
            fn from(vector: &$name<T>) -> Self {
                ($(vector.$field,)+)
            }
        }
    };
    (@ty $t:ident, $field:ident) => { $t };
}

vector_type!(
    /// A 2D vector.
    Vector2, 2, x: 0, y: 1
);
vector_type!(
    /// A 3D vector.
    Vector3, 3, x: 0, y: 1, z: 2
);
vector_type!(
    /// A 4D vector.
    Vector4, 4, x: 0, y: 1, z: 2, w: 3
);

impl<T: Real> Vector2<T> {
    #[inline]
    pub fn length(self) -> T {
        self.x.hypot(self.y)
    }

    #[inline]
    pub fn distance_to(self, other: Self) -> T {
        self.distance_to_sqr(other).sqrt()
    }

    /// The unit vector, or the vector unchanged if its length is not positive.
    #[inline]
    #[must_use]
    pub fn normalized(self) -> Self {
        let length = self.length();
        if length > T::ZERO {
            self / length
        } else {
            self
        }
    }
}

impl<T: Real> Vector3<T> {
    #[inline]
    pub fn length(self) -> T {
        self.x.hypot(self.y).hypot(self.z)
    }

    #[inline]
    pub fn distance_to(self, other: Self) -> T {
        (self - other).length()
    }

    /// The unit vector, or the zero vector unchanged.
    #[inline]
    #[must_use]
    pub fn normalized(self) -> Self {
        let length = self.length();
        if length != T::ZERO {
            self / length
        } else {
            self
        }
    }

    /// Length of the `x`, `y` part.
    #[inline]
    pub fn length_2d(self) -> T {
        self.xy().length()
    }
}

impl<T: Real> Vector4<T> {
    #[inline]
    pub fn length(self) -> T {
        self.x.hypot(self.y).hypot(self.z).hypot(self.w)
    }

    #[inline]
    pub fn distance_to(self, other: Self) -> T {
        (self - other).length()
    }

    /// The unit vector, or the zero vector unchanged.
    #[inline]
    #[must_use]
    pub fn normalized(self) -> Self {
        let length = self.length();
        if length != T::ZERO {
            self / length
        } else {
            self
        }
    }
}

impl<T: Scalar> Vector3<T> {
    #[inline]
    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    /// Drops `z`. The C++ `Vector3` inherits from `Vector2`; this is the explicit form of that upcast.
    #[inline]
    pub fn xy(self) -> Vector2<T> {
        Vector2::new(self.x, self.y)
    }

    /// Sum of `x` and `y`.
    #[inline]
    pub fn sum_2d(self) -> T {
        self.x + self.y
    }
}

impl<T: Scalar> Vector4<T> {
    /// Drops `w`.
    #[inline]
    pub fn xyz(self) -> Vector3<T> {
        Vector3::new(self.x, self.y, self.z)
    }
}

#[cfg(feature = "imgui")]
impl<T: Scalar> Vector2<T> {
    pub fn to_im_vec2(&self) -> imgui::sys::ImVec2 {
        imgui::sys::ImVec2 {
            x: self.x.to_f64() as f32,
            y: self.y.to_f64() as f32,
        }
    }
}
#[cfg(feature = "imgui")]
impl<T: Scalar> Vector4<T> {
    pub fn to_im_vec4(&self) -> imgui::sys::ImVec4 {
        imgui::sys::ImVec4 {
            x: self.x.to_f64() as f32,
            y: self.y.to_f64() as f32,
            z: self.z.to_f64() as f32,
            w: self.w.to_f64() as f32,
        }
    }
}
