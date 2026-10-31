//! Collision detection: GJK, EPA, line tracing, and the mesh collider they run on.

mod collider_interface;
mod epa_algorithm;
mod gjk_algorithm;
mod line_tracer;
mod mesh_collider;
mod simplex;

pub use collider_interface::Collider;
pub use epa_algorithm::{EpaParams, EpaResult, epa_solve};
pub use gjk_algorithm::{GjkHitInfo, GjkSettings, find_support_vertex, is_collide, is_collide_with_simplex_info};
pub use line_tracer::{Ray, RayTraceable, can_trace_line, get_ray_hit_point};
pub use mesh_collider::MeshCollider;
pub use simplex::Simplex;
