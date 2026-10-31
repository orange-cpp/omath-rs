//! Serious Engine: right-handed, Y-up, camera looks down -Z (same axes as OpenGL). One world unit is one meter.
//!
//! `ANGLE3D` (heading, pitch, banking): positive heading turns left, positive pitch looks up.

mod camera;
mod collider;
mod constants;
mod formulas;
mod mesh;
mod pred_engine_trait;

pub use camera::{Camera, CameraTrait};
pub use collider::MeshCollider;
pub use constants::*;
pub use formulas::*;
pub use mesh::{BoxMesh, Mesh, MeshTrait, PlaneMesh};
pub use pred_engine_trait::PredEngineTrait;
