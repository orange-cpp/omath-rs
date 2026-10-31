#![cfg(feature = "lua")]

mod lua_support;

const SCRIPT: &str = "vec2_tests.lua";

#[test]
fn constructor_default() {
    lua_support::check(SCRIPT, "Vec2_Constructor_default");
}

#[test]
fn constructor_xy() {
    lua_support::check(SCRIPT, "Vec2_Constructor_xy");
}

#[test]
fn field_mutation() {
    lua_support::check(SCRIPT, "Vec2_Field_mutation");
}

#[test]
fn addition() {
    lua_support::check(SCRIPT, "Vec2_Addition");
}

#[test]
fn subtraction() {
    lua_support::check(SCRIPT, "Vec2_Subtraction");
}

#[test]
fn unary_minus() {
    lua_support::check(SCRIPT, "Vec2_UnaryMinus");
}

#[test]
fn multiplication_scalar() {
    lua_support::check(SCRIPT, "Vec2_Multiplication_scalar");
}

#[test]
fn multiplication_scalar_reversed() {
    lua_support::check(SCRIPT, "Vec2_Multiplication_scalar_reversed");
}

#[test]
fn division_scalar() {
    lua_support::check(SCRIPT, "Vec2_Division_scalar");
}

#[test]
fn equal_to_true() {
    lua_support::check(SCRIPT, "Vec2_EqualTo_true");
}

#[test]
fn equal_to_false() {
    lua_support::check(SCRIPT, "Vec2_EqualTo_false");
}

#[test]
fn less_than() {
    lua_support::check(SCRIPT, "Vec2_LessThan");
}

#[test]
fn less_than_or_equal() {
    lua_support::check(SCRIPT, "Vec2_LessThanOrEqual");
}

#[test]
fn to_string() {
    lua_support::check(SCRIPT, "Vec2_ToString");
}

#[test]
fn length() {
    lua_support::check(SCRIPT, "Vec2_Length");
}

#[test]
fn length_sqr() {
    lua_support::check(SCRIPT, "Vec2_LengthSqr");
}

#[test]
fn normalized() {
    lua_support::check(SCRIPT, "Vec2_Normalized");
}

#[test]
fn dot() {
    lua_support::check(SCRIPT, "Vec2_Dot");
}

#[test]
fn distance_to() {
    lua_support::check(SCRIPT, "Vec2_DistanceTo");
}

#[test]
fn distance_to_sqr() {
    lua_support::check(SCRIPT, "Vec2_DistanceToSqr");
}

#[test]
fn sum() {
    lua_support::check(SCRIPT, "Vec2_Sum");
}

#[test]
fn abs() {
    lua_support::check(SCRIPT, "Vec2_Abs");
}
