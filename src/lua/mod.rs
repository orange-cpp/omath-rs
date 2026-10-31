//! Lua bindings, behind the `lua` feature.
//!
//! [`register_lib`] publishes the `omath` global with the names, metamethods and error behaviour of the C++ sol2
//! bindings. Types live under the same keys: `omath.Vec3`, `omath.primitives.Aabb`, `omath.source.Camera` and so on.
//! Nothing that touches a live process is bound: there is no `scan_in_module` and no HUD.

mod collision;
mod color;
mod engines;
mod mat;
mod pattern_scan;
mod quaternion;
mod support;
mod triangle;
mod vector;

use mlua::{Lua, Result};

/// Fills the `omath` global table, creating it if needed.
pub fn register_lib(lua: &Lua) -> Result<()> {
    support::install_helpers(lua)?;

    let omath = support::subtable(lua, &lua.globals(), "omath")?;

    vector::register(lua, &omath)?;
    mat::register(lua, &omath)?;
    quaternion::register(lua, &omath)?;
    color::register(lua, &omath)?;
    triangle::register(lua, &omath)?;
    collision::register_primitives(lua, &omath)?;
    collision::register_collision(lua, &omath)?;
    engines::register_shared_types(lua, &omath)?;
    engines::register_engines(lua, &omath)?;
    pattern_scan::register(lua, &omath)
}
