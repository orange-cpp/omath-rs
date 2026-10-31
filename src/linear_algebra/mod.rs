//! Vectors, matrices, quaternions and triangles.

mod mat;
mod quaternion;
mod triangle;
mod vector;

pub use mat::{ColumnMajor, Mat, MatSize, MatStoreType, NdcDepthRange, RowMajor, StoreOrder};
pub use quaternion::Quaternion;
pub use triangle::Triangle;
pub use vector::{Vector, Vector2, Vector3, Vector3Error, Vector4};
