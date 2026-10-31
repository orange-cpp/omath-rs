//! The fixture of the C++ Lua tests: a fresh state with `omath` registered and a script loaded.

use mlua::{Function, Lua};
use std::path::Path;

pub fn load(script: &str) -> Lua {
    let lua = Lua::new();
    omath::lua::register_lib(&lua).unwrap();

    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/lua")
        .join(script);
    lua.load(path.as_path())
        .exec()
        .unwrap_or_else(|error| panic!("{script}: {error}"));

    lua
}

/// Runs a global function of `script`, which fails by raising an error.
pub fn check(script: &str, function: &str) {
    let lua = load(script);
    let function_value: Function = lua.globals().get(function).unwrap();

    function_value
        .call::<()>(())
        .unwrap_or_else(|error| panic!("{function}: {error}"));
}
