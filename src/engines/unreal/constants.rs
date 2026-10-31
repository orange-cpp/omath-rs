use crate::engines::Clamped90;
use crate::{NdcDepthRange, RowMajor, Vector3};

pub const K_ABS_UP: Vector3<f64> = Vector3::new(0.0, 0.0, 1.0);
pub const K_ABS_RIGHT: Vector3<f64> = Vector3::new(0.0, 1.0, 0.0);
pub const K_ABS_FORWARD: Vector3<f64> = Vector3::new(1.0, 0.0, 0.0);

/// What the C++ `calc_perspective_projection_matrix` defaults to. Not always what the engine's [`Camera`](super::Camera)
/// uses, which is `CameraEngine::DEPTH_RANGE`.
pub const K_DEFAULT_NDC_DEPTH_RANGE: NdcDepthRange = NdcDepthRange::NegativeOneToOne;

engine_types!(f64, RowMajor, Clamped90);
