//! Unreal Engine: Z-up, forward along +X, in double precision. One unit is one centimeter.

mod camera;
mod constants;
mod formulas;
mod mesh;
mod pred_engine_trait;

pub use camera::{Camera, CameraTrait};
pub use constants::*;
pub use formulas::*;
pub use mesh::{BoxMesh, Mesh, MeshTrait, PlaneMesh};
pub use pred_engine_trait::PredEngineTrait;
pub mod actor_name;
pub mod object_array;
pub use actor_name::{ActorNameError, NameCharKind, NameLayout, NamePoolKind, get_actor_name};
pub use object_array::{
    ObjectArrayError, ObjectArrayKind, ObjectArrayLayout, get_object_by_index, get_object_count,
};
