#![cfg(feature = "lua")]

mod lua_support;

const SCRIPT: &str = "source_engine_tests.lua";

#[test]
fn pitch_angle_from_degrees() {
    lua_support::check(SCRIPT, "Source_PitchAngle_from_degrees");
}

#[test]
fn pitch_angle_clamping_max() {
    lua_support::check(SCRIPT, "Source_PitchAngle_clamping_max");
}

#[test]
fn pitch_angle_clamping_min() {
    lua_support::check(SCRIPT, "Source_PitchAngle_clamping_min");
}

#[test]
fn pitch_angle_from_radians() {
    lua_support::check(SCRIPT, "Source_PitchAngle_from_radians");
}

#[test]
fn pitch_angle_as_radians() {
    lua_support::check(SCRIPT, "Source_PitchAngle_as_radians");
}

#[test]
fn pitch_angle_sin() {
    lua_support::check(SCRIPT, "Source_PitchAngle_sin");
}

#[test]
fn pitch_angle_cos() {
    lua_support::check(SCRIPT, "Source_PitchAngle_cos");
}

#[test]
fn pitch_angle_tan() {
    lua_support::check(SCRIPT, "Source_PitchAngle_tan");
}

#[test]
fn pitch_angle_addition() {
    lua_support::check(SCRIPT, "Source_PitchAngle_addition");
}

#[test]
fn pitch_angle_addition_clamped() {
    lua_support::check(SCRIPT, "Source_PitchAngle_addition_clamped");
}

#[test]
fn pitch_angle_subtraction() {
    lua_support::check(SCRIPT, "Source_PitchAngle_subtraction");
}

#[test]
fn pitch_angle_unary_minus() {
    lua_support::check(SCRIPT, "Source_PitchAngle_unary_minus");
}

#[test]
fn pitch_angle_equal_to() {
    lua_support::check(SCRIPT, "Source_PitchAngle_equal_to");
}

#[test]
fn pitch_angle_to_string() {
    lua_support::check(SCRIPT, "Source_PitchAngle_to_string");
}

#[test]
fn yaw_angle_from_degrees() {
    lua_support::check(SCRIPT, "Source_YawAngle_from_degrees");
}

#[test]
fn yaw_angle_normalization() {
    lua_support::check(SCRIPT, "Source_YawAngle_normalization");
}

#[test]
fn roll_angle_from_degrees() {
    lua_support::check(SCRIPT, "Source_RollAngle_from_degrees");
}

#[test]
fn field_of_view_from_degrees() {
    lua_support::check(SCRIPT, "Source_FieldOfView_from_degrees");
}

#[test]
fn field_of_view_clamping() {
    lua_support::check(SCRIPT, "Source_FieldOfView_clamping");
}

#[test]
fn view_angles_new() {
    lua_support::check(SCRIPT, "Source_ViewAngles_new");
}

#[test]
fn view_angles_field_mutation() {
    lua_support::check(SCRIPT, "Source_ViewAngles_field_mutation");
}

#[test]
fn camera_constructor() {
    lua_support::check(SCRIPT, "Source_Camera_constructor");
}

#[test]
fn camera_get_set_origin() {
    lua_support::check(SCRIPT, "Source_Camera_get_set_origin");
}

#[test]
fn camera_get_set_near_plane() {
    lua_support::check(SCRIPT, "Source_Camera_get_set_near_plane");
}

#[test]
fn camera_get_set_far_plane() {
    lua_support::check(SCRIPT, "Source_Camera_get_set_far_plane");
}

#[test]
fn camera_get_set_fov() {
    lua_support::check(SCRIPT, "Source_Camera_get_set_fov");
}

#[test]
fn camera_get_set_view_angles() {
    lua_support::check(SCRIPT, "Source_Camera_get_set_view_angles");
}

#[test]
fn camera_look_at() {
    lua_support::check(SCRIPT, "Source_Camera_look_at");
}

#[test]
fn camera_get_forward() {
    lua_support::check(SCRIPT, "Source_Camera_get_forward");
}

#[test]
fn camera_get_right() {
    lua_support::check(SCRIPT, "Source_Camera_get_right");
}

#[test]
fn camera_get_up() {
    lua_support::check(SCRIPT, "Source_Camera_get_up");
}

#[test]
fn camera_get_view_matrix() {
    lua_support::check(SCRIPT, "Source_Camera_get_view_matrix");
}

#[test]
fn camera_get_projection_matrix() {
    lua_support::check(SCRIPT, "Source_Camera_get_projection_matrix");
}

#[test]
fn camera_get_view_projection_matrix() {
    lua_support::check(SCRIPT, "Source_Camera_get_view_projection_matrix");
}

#[test]
fn camera_extract_projection_params() {
    lua_support::check(SCRIPT, "Source_Camera_extract_projection_params");
}

#[test]
fn camera_calc_view_angles_from_view_matrix() {
    lua_support::check(SCRIPT, "Source_Camera_calc_view_angles_from_view_matrix");
}

#[test]
fn camera_calc_origin_from_view_matrix() {
    lua_support::check(SCRIPT, "Source_Camera_calc_origin_from_view_matrix");
}

#[test]
fn camera_world_to_screen_success() {
    lua_support::check(SCRIPT, "Source_Camera_world_to_screen_success");
}

#[test]
fn camera_world_to_screen_error() {
    lua_support::check(SCRIPT, "Source_Camera_world_to_screen_error");
}

#[test]
fn camera_screen_to_world() {
    lua_support::check(SCRIPT, "Source_Camera_screen_to_world");
}
