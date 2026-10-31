#![cfg(feature = "lua")]

mod lua_support;

const SCRIPT: &str = "color_tests.lua";

#[test]
fn constructor_float() {
    lua_support::check(SCRIPT, "Color_Constructor_float");
}

#[test]
fn constructor_default() {
    lua_support::check(SCRIPT, "Color_Constructor_default");
}

#[test]
fn constructor_clamping() {
    lua_support::check(SCRIPT, "Color_Constructor_clamping");
}

#[test]
fn from_rgba() {
    lua_support::check(SCRIPT, "Color_from_rgba");
}

#[test]
fn from_hsv_components() {
    lua_support::check(SCRIPT, "Color_from_hsv_components");
}

#[test]
fn from_hsv_struct() {
    lua_support::check(SCRIPT, "Color_from_hsv_struct");
}

#[test]
fn red() {
    lua_support::check(SCRIPT, "Color_red");
}

#[test]
fn green() {
    lua_support::check(SCRIPT, "Color_green");
}

#[test]
fn blue() {
    lua_support::check(SCRIPT, "Color_blue");
}

#[test]
fn to_hsv() {
    lua_support::check(SCRIPT, "Color_to_hsv");
}

#[test]
fn set_hue() {
    lua_support::check(SCRIPT, "Color_set_hue");
}

#[test]
fn set_saturation() {
    lua_support::check(SCRIPT, "Color_set_saturation");
}

#[test]
fn set_value() {
    lua_support::check(SCRIPT, "Color_set_value");
}

#[test]
fn blend() {
    lua_support::check(SCRIPT, "Color_blend");
}

#[test]
fn blend_clamped_ratio() {
    lua_support::check(SCRIPT, "Color_blend_clamped_ratio");
}

#[test]
fn to_string() {
    lua_support::check(SCRIPT, "Color_to_string");
}

#[test]
fn hsv_fields() {
    lua_support::check(SCRIPT, "Hsv_fields");
}
