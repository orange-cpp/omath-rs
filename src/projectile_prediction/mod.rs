//! Where to aim a projectile so that it meets a moving target.

mod launcher;
mod pred_engine;
mod proj_pred_engine;
mod proj_pred_engine_batched;
mod proj_pred_engine_drag;
mod proj_pred_engine_legacy;
mod projectile;
mod projectile_flight;
mod target;

pub use launcher::{Launcher, MuzzleOffset, ViewBasis};
pub use pred_engine::PredEngine;
pub use proj_pred_engine::{AimAngles, AimSolution, Arc, ProjPredEngineInterface, launcher_from_projectile};
pub use proj_pred_engine_batched::ProjPredEngineBatched;
pub use proj_pred_engine_drag::ProjPredEngineDrag;
pub use proj_pred_engine_legacy::ProjPredEngineLegacy;
pub use projectile::Projectile;
pub use projectile_flight::ProjectileFlight;
pub use target::Target;
