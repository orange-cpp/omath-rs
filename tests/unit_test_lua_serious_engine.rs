#![cfg(feature = "lua")]

mod lua_support;

const SCRIPT: &str = "serious_engine_tests.lua";

#[test]
fn pitch_angle_clamps_to_90() {
    lua_support::check(SCRIPT, "Serious_PitchAngle_clamps_to_90");
}

#[test]
fn yaw_angle_normalization() {
    lua_support::check(SCRIPT, "Serious_YawAngle_normalization");
}

#[test]
fn camera_default_looks_down_negative_z() {
    lua_support::check(SCRIPT, "Serious_Camera_default_looks_down_negative_z");
}

#[test]
fn camera_positive_heading_turns_left() {
    lua_support::check(SCRIPT, "Serious_Camera_positive_heading_turns_left");
}

#[test]
fn camera_horizontal_fov_reaches_the_screen_edge() {
    lua_support::check(SCRIPT, "Serious_Camera_horizontal_fov_reaches_the_screen_edge");
}

#[test]
fn camera_extract_projection_params() {
    lua_support::check(SCRIPT, "Serious_Camera_extract_projection_params");
}

#[test]
fn camera_calc_view_angles_from_view_matrix() {
    lua_support::check(SCRIPT, "Serious_Camera_calc_view_angles_from_view_matrix");
}

#[test]
fn camera_calc_origin_from_view_matrix() {
    lua_support::check(SCRIPT, "Serious_Camera_calc_origin_from_view_matrix");
}

#[test]
fn camera_world_to_screen_center() {
    lua_support::check(SCRIPT, "Serious_Camera_world_to_screen_center");
}

#[test]
fn camera_world_to_screen_behind_is_error() {
    lua_support::check(SCRIPT, "Serious_Camera_world_to_screen_behind_is_error");
}

#[test]
fn camera_look_at_centers_the_target() {
    lua_support::check(SCRIPT, "Serious_Camera_look_at_centers_the_target");
}

#[test]
fn camera_screen_to_world() {
    lua_support::check(SCRIPT, "Serious_Camera_screen_to_world");
}
