#![cfg(feature = "lua")]

mod lua_support;

const SCRIPT: &str = "collision_tests.lua";

#[test]
fn aabb_constructor_and_fields() {
    lua_support::check(SCRIPT, "Collision_Aabb_constructor_and_fields");
}

#[test]
fn aabb_center_extents() {
    lua_support::check(SCRIPT, "Collision_Aabb_center_extents");
}

#[test]
fn aabb_top_bottom_default_axis() {
    lua_support::check(SCRIPT, "Collision_Aabb_top_bottom_default_axis");
}

#[test]
fn aabb_top_bottom_explicit_axis() {
    lua_support::check(SCRIPT, "Collision_Aabb_top_bottom_explicit_axis");
}

#[test]
fn aabb_is_collide() {
    lua_support::check(SCRIPT, "Collision_Aabb_is_collide");
}

#[test]
fn aabb_as_table() {
    lua_support::check(SCRIPT, "Collision_Aabb_as_table");
}

#[test]
fn obb_constructor_and_vertices() {
    lua_support::check(SCRIPT, "Collision_Obb_constructor_and_vertices");
}

#[test]
fn ray_constructor_and_direction() {
    lua_support::check(SCRIPT, "Collision_Ray_constructor_and_direction");
}

#[test]
fn line_tracer_triangle_hit() {
    lua_support::check(SCRIPT, "Collision_LineTracer_triangle_hit");
}

#[test]
fn line_tracer_triangle_miss() {
    lua_support::check(SCRIPT, "Collision_LineTracer_triangle_miss");
}

#[test]
fn line_tracer_aabb_hit() {
    lua_support::check(SCRIPT, "Collision_LineTracer_aabb_hit");
}

#[test]
fn line_tracer_aabb_miss() {
    lua_support::check(SCRIPT, "Collision_LineTracer_aabb_miss");
}

#[test]
fn line_tracer_obb_hit() {
    lua_support::check(SCRIPT, "Collision_LineTracer_obb_hit");
}

#[test]
fn convex_collider_vertices_support() {
    lua_support::check(SCRIPT, "Collision_ConvexCollider_vertices_and_support");
}

#[test]
fn gjk_support_vertex() {
    lua_support::check(SCRIPT, "Collision_Gjk_support_vertex");
}

#[test]
fn gjk_collide_true() {
    lua_support::check(SCRIPT, "Collision_Gjk_collide_true");
}

#[test]
fn gjk_collide_false() {
    lua_support::check(SCRIPT, "Collision_Gjk_collide_false");
}

#[test]
fn epa_solve_hit() {
    lua_support::check(SCRIPT, "Collision_Epa_solve_hit");
}

#[test]
fn epa_solve_miss() {
    lua_support::check(SCRIPT, "Collision_Epa_solve_miss");
}
