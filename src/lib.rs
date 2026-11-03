//! Rust port of [omath](https://github.com/orange-cpp/omath): math, physics, navigation,
//! HUD, process utilities, Unreal walkers and optional Lua, ImGui and graphics hooks.

pub mod algorithm;
pub mod collision;
pub mod engines;
pub mod hashing;
#[cfg(all(
    feature = "hooks",
    any(windows, target_os = "linux"),
    any(target_arch = "x86", target_arch = "x86_64")
))]
pub mod hooks;
pub mod hud;
pub mod linear_algebra;
#[cfg(feature = "lua")]
pub mod lua;
pub mod pathfinding;
pub mod primitives;
pub mod projectile_prediction;
pub mod projection;
pub mod rev_eng;
pub mod scalar;
pub mod trigonometry;
pub mod utility;

pub use linear_algebra::{
    ColumnMajor, Mat, MatSize, MatStoreType, NdcDepthRange, Quaternion, RowMajor, StoreOrder,
    Triangle, Vector, Vector2, Vector3, Vector3Error, Vector4,
};
pub use scalar::{Real, Scalar, Signed};
pub use trigonometry::{
    Angle, AngleBetween, AnglePolicy, AngleRange, Clamped0To180, Normalized0To360, Trig, ViewAngles,
};
pub use utility::{Color, Hsv};

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
