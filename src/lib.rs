//! Rust port of the general-purpose parts of [omath](https://github.com/orange-cpp/omath).

pub mod algorithm;
pub mod collision;
pub mod engines;
pub mod hashing;
pub mod linear_algebra;
#[cfg(feature = "lua")]
pub mod lua;
pub mod pathfinding;
pub mod primitives;
pub mod projectile_prediction;
pub mod projection;
pub mod scalar;
pub mod trigonometry;
pub mod utility;

pub use linear_algebra::{
    ColumnMajor, Mat, MatSize, MatStoreType, NdcDepthRange, Quaternion, RowMajor, StoreOrder, Triangle, Vector,
    Vector2, Vector3, Vector3Error, Vector4,
};
pub use scalar::{Real, Scalar, Signed};
pub use trigonometry::{
    Angle, AngleBetween, AnglePolicy, AngleRange, Clamped0To180, Normalized0To360, Trig, ViewAngles,
};
pub use utility::{Color, Hsv};

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
