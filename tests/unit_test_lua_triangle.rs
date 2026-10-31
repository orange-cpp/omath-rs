#![cfg(feature = "lua")]

mod lua_support;

const SCRIPT: &str = "triangle_tests.lua";

#[test]
fn constructor_default() {
    lua_support::check(SCRIPT, "Triangle_Constructor_default");
}

#[test]
fn constructor_vertices() {
    lua_support::check(SCRIPT, "Triangle_Constructor_vertices");
}

#[test]
fn vertex_mutation() {
    lua_support::check(SCRIPT, "Triangle_Vertex_mutation");
}

#[test]
fn side_a_length() {
    lua_support::check(SCRIPT, "Triangle_SideALength");
}

#[test]
fn side_b_length() {
    lua_support::check(SCRIPT, "Triangle_SideBLength");
}

#[test]
fn hypot() {
    lua_support::check(SCRIPT, "Triangle_Hypot");
}

#[test]
fn side_a_vector() {
    lua_support::check(SCRIPT, "Triangle_SideAVector");
}

#[test]
fn side_b_vector() {
    lua_support::check(SCRIPT, "Triangle_SideBVector");
}

#[test]
fn is_rectangular_true() {
    lua_support::check(SCRIPT, "Triangle_IsRectangular_true");
}

#[test]
fn is_rectangular_false() {
    lua_support::check(SCRIPT, "Triangle_IsRectangular_false");
}

#[test]
fn mid_point() {
    lua_support::check(SCRIPT, "Triangle_MidPoint");
}

#[test]
fn calculate_normal() {
    lua_support::check(SCRIPT, "Triangle_CalculateNormal");
}

#[test]
fn to_string() {
    lua_support::check(SCRIPT, "Triangle_ToString");
}
