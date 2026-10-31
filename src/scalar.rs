//! Numeric traits that replace the C++ `std::is_arithmetic` / `std::is_floating_point` constraints.

use std::fmt::{Debug, Display};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, Sub, SubAssign};

/// Any primitive number usable as a vector or matrix element.
pub trait Scalar:
    Copy
    + Debug
    + Display
    + Default
    + PartialEq
    + PartialOrd
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + AddAssign
    + SubAssign
    + MulAssign
    + DivAssign
{
    const ZERO: Self;
    const ONE: Self;

    /// Same semantics as `as`: saturating for float to int, nearest for int to float.
    fn from_f64(value: f64) -> Self;
    fn to_f64(self) -> f64;

    /// Bit pattern used for hashing. `-0.0` and `0.0` hash identically because they compare equal.
    fn hash_bits(self) -> u64;

    /// Same semantics as `std::clamp`: NaN passes through.
    #[inline]
    fn clamped(self, min: Self, max: Self) -> Self {
        if self < min {
            min
        } else if max < self {
            max
        } else {
            self
        }
    }
}

/// A signed [`Scalar`].
pub trait Signed: Scalar + Neg<Output = Self> {
    #[inline]
    fn abs_value(self) -> Self {
        if self < Self::ZERO { -self } else { self }
    }
}

macro_rules! impl_int {
    ($($t:ty),* $(,)?) => {$(
        impl Scalar for $t {
            const ZERO: Self = 0;
            const ONE: Self = 1;
            #[inline]
            fn from_f64(value: f64) -> Self { value as $t }
            #[inline]
            fn to_f64(self) -> f64 { self as f64 }
            #[inline]
            fn hash_bits(self) -> u64 { self as u64 }
        }
    )*};
}
impl_int!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

macro_rules! impl_signed {
    ($($t:ty),* $(,)?) => {$( impl Signed for $t {} )*};
}
impl_signed!(i8, i16, i32, i64, isize, f32, f64);

/// A floating point [`Scalar`].
pub trait Real: Signed + Rem<Output = Self> {
    const PI: Self;
    const EPSILON: Self;
    const MAX: Self;
    const MIN_POSITIVE: Self;
    const INFINITY: Self;

    fn sqrt(self) -> Self;
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn tan(self) -> Self;
    fn asin(self) -> Self;
    fn acos(self) -> Self;
    fn atan(self) -> Self;
    fn atan2(self, x: Self) -> Self;
    fn hypot(self, other: Self) -> Self;
    fn floor(self) -> Self;
    fn ceil(self) -> Self;
    fn powi(self, exponent: i32) -> Self;
    fn exp(self) -> Self;
    fn ln(self) -> Self;
    fn is_nan(self) -> bool;
    fn is_finite(self) -> bool;
    fn min_of(self, other: Self) -> Self;
    fn max_of(self, other: Self) -> Self;
}

macro_rules! impl_real {
    ($t:ident, $bits:ty) => {
        impl Scalar for $t {
            const ZERO: Self = 0.0;
            const ONE: Self = 1.0;
            #[inline]
            fn from_f64(value: f64) -> Self {
                value as $t
            }
            #[inline]
            fn to_f64(self) -> f64 {
                self as f64
            }
            #[inline]
            fn hash_bits(self) -> u64 {
                (self + 0.0).to_bits() as u64
            }
        }

        impl Real for $t {
            const PI: Self = std::$t::consts::PI;
            const EPSILON: Self = $t::EPSILON;
            const MAX: Self = $t::MAX;
            const MIN_POSITIVE: Self = $t::MIN_POSITIVE;
            const INFINITY: Self = $t::INFINITY;

            #[inline]
            fn sqrt(self) -> Self {
                $t::sqrt(self)
            }
            #[inline]
            fn sin(self) -> Self {
                $t::sin(self)
            }
            #[inline]
            fn cos(self) -> Self {
                $t::cos(self)
            }
            #[inline]
            fn tan(self) -> Self {
                $t::tan(self)
            }
            #[inline]
            fn asin(self) -> Self {
                $t::asin(self)
            }
            #[inline]
            fn acos(self) -> Self {
                $t::acos(self)
            }
            #[inline]
            fn atan(self) -> Self {
                $t::atan(self)
            }
            #[inline]
            fn atan2(self, x: Self) -> Self {
                $t::atan2(self, x)
            }
            #[inline]
            fn hypot(self, other: Self) -> Self {
                $t::hypot(self, other)
            }
            #[inline]
            fn floor(self) -> Self {
                $t::floor(self)
            }
            #[inline]
            fn ceil(self) -> Self {
                $t::ceil(self)
            }
            #[inline]
            fn powi(self, exponent: i32) -> Self {
                $t::powi(self, exponent)
            }
            #[inline]
            fn exp(self) -> Self {
                $t::exp(self)
            }
            #[inline]
            fn ln(self) -> Self {
                $t::ln(self)
            }
            #[inline]
            fn is_nan(self) -> bool {
                $t::is_nan(self)
            }
            #[inline]
            fn is_finite(self) -> bool {
                $t::is_finite(self)
            }
            #[inline]
            fn min_of(self, other: Self) -> Self {
                $t::min(self, other)
            }
            #[inline]
            fn max_of(self, other: Self) -> Self {
                $t::max(self, other)
            }
        }
    };
}
impl_real!(f32, u32);
impl_real!(f64, u64);
