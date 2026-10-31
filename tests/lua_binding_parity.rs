#![cfg(feature = "lua")]

//! Behaviour of the C++ bindings that its own Lua tests do not reach, and the scripts no C++ test runs.

mod lua_support;

use mlua::Function;

const SCRIPT: &str = "binding_parity_tests.lua";

macro_rules! parity_tests {
    ($($test:ident => $function:literal),+ $(,)?) => {$(
        #[test]
        fn $test() {
            lua_support::check(SCRIPT, $function);
        }
    )+};
}

parity_tests! {
    mat_errors_are_plain_strings => "Mat_errors_are_plain_strings",
    mat_static_and_instance_access => "Mat_static_and_instance_access",
    mat_set_at_returns_a_copy => "Mat_set_at_returns_a_copy",
    mat_index_rounds_like_sol2 => "Mat_index_rounds_like_sol2",
    mat_helpers_only_on_mat4_kinds => "Mat_helpers_only_on_mat4_kinds",
    mat_to_string_format => "Mat_to_string_format",
    vector_arity_and_overloads => "Vector_arity_and_overloads",
    vector_surface_is_exact => "Vector_surface_is_exact",
    vector_mixed_types_are_not_equal => "Vector_mixed_types_are_not_equal",
    vec3_narrows_like_a_cast => "Vec3_narrows_like_a_cast",
    vec3_as_array_is_one_based => "Vec3_as_array_is_one_based",
    vec3_angle_between_returns_two_values => "Vec3_angle_between_returns_two_values",
    vec4_clamp_mutates_and_returns_a_copy => "Vec4_clamp_mutates_and_returns_a_copy",
    float_strings_match_std_format => "Float_strings_match_std_format",
    color_extras => "Color_extras",
    identity_equality_where_cpp_has_no_operator => "Identity_equality_where_cpp_has_no_operator",
    angle_ordering_comes_from_the_spaceship_operator => "Angle_ordering_comes_from_the_spaceship_operator",
    angle_cot => "Angle_cot",
    up_axis_is_a_read_only_enum => "UpAxis_is_a_read_only_enum",
    aabb_values_are_copies => "Aabb_values_are_copies",
    aabb_axis_defaults_when_not_a_number => "Aabb_axis_defaults_when_not_a_number",
    ray_third_argument_must_be_boolean => "Ray_third_argument_must_be_boolean",
    convex_collider_errors_and_ties => "ConvexCollider_errors_and_ties",
    line_tracer_rejects_other_targets => "LineTracer_rejects_other_targets",
    projection_error_texts => "Projection_error_texts",
    scanners_have_no_module_surface => "Scanners_have_no_module_surface",
    aliases_share_the_type_objects => "Aliases_share_the_type_objects",
    unreal_pitch_clamps_to_the_engines_own_range => "Unreal_pitch_clamps_to_the_engines_own_range",
}

#[test]
fn every_engine_projects() {
    let lua = lua_support::load(SCRIPT);
    let check: Function = lua.globals().get("Every_engine_projects").unwrap();

    for (engine, mat_type) in [
        ("opengl", "Mat4ColumnMajor"),
        ("frostbite", "Mat4"),
        ("iw", "Mat4"),
        ("source", "Mat4"),
        ("rage", "Mat4"),
        ("serious", "Mat4"),
        ("unity", "Mat4"),
        ("unreal", "Mat4d"),
        ("cry", "Mat4"),
    ] {
        check.call::<()>((engine, mat_type)).unwrap_or_else(|error| panic!("{engine}: {error}"));
    }
}

/// A one-section PE image with `90 01 02 03 04` in `.text`, at raw offset and RVA 0x200 and 0x1000.
fn fake_pe() -> Vec<u8> {
    const E_LFANEW: usize = 0x80;
    const OPTIONAL_HEADER: usize = E_LFANEW + 4 + 20;
    const SECTION: usize = OPTIONAL_HEADER + 0xF0;
    const CODE: usize = 0x200;

    let mut image = vec![0u8; 0x300];
    image[..2].copy_from_slice(&0x5A4Du16.to_le_bytes());
    image[0x3C..0x40].copy_from_slice(&(E_LFANEW as u32).to_le_bytes());
    image[E_LFANEW..E_LFANEW + 4].copy_from_slice(&0x4550u32.to_le_bytes());
    image[E_LFANEW + 6..E_LFANEW + 8].copy_from_slice(&1u16.to_le_bytes());
    image[E_LFANEW + 20..E_LFANEW + 22].copy_from_slice(&0xF0u16.to_le_bytes());
    image[OPTIONAL_HEADER..OPTIONAL_HEADER + 2].copy_from_slice(&0x020Bu16.to_le_bytes());
    image[OPTIONAL_HEADER + 24..OPTIONAL_HEADER + 32].copy_from_slice(&0x200u64.to_le_bytes());
    image[SECTION..SECTION + 5].copy_from_slice(b".text");
    image[SECTION + 12..SECTION + 16].copy_from_slice(&0x1000u32.to_le_bytes());
    image[SECTION + 16..SECTION + 20].copy_from_slice(&5u32.to_le_bytes());
    image[SECTION + 20..SECTION + 24].copy_from_slice(&(CODE as u32).to_le_bytes());
    image[CODE..CODE + 5].copy_from_slice(&[0x90, 0x01, 0x02, 0x03, 0x04]);
    image
}

#[test]
fn scanner_scan_in_file() {
    let path = std::env::temp_dir().join(format!("omath_lua_scan_{}.bin", std::process::id()));
    std::fs::write(&path, fake_pe()).unwrap();

    let lua = lua_support::load(SCRIPT);
    let check: Function = lua.globals().get("Scanner_scan_in_file").unwrap();
    let result = check.call::<()>(path.to_str().unwrap());

    std::fs::remove_file(&path).unwrap();
    result.unwrap();
}

/// The example scripts next to the test scripts, which print but assert nothing, still run.
#[test]
fn example_scripts_run() {
    for script in ["vector2_test.lua", "vector3_test.lua", "vector4_test.lua"] {
        lua_support::load(script);
    }
}
