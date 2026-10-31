//! Algorithms built on top of the projection types.

pub mod radar;

pub use radar::world_to_radar;
pub mod targeting;
pub use targeting::{get_closest_target_by_distance, get_closest_target_by_fov};
