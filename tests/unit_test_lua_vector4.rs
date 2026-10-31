#![cfg(feature = "lua")]

mod lua_support;

const SCRIPT: &str = "vec4_tests.lua";

#[test]
fn constructor_default() {
    lua_support::check(SCRIPT, "Vec4_Constructor_default");
}

#[test]
fn constructor_xyzw() {
    lua_support::check(SCRIPT, "Vec4_Constructor_xyzw");
}

#[test]
fn field_mutation() {
    lua_support::check(SCRIPT, "Vec4_Field_mutation");
}

#[test]
fn addition() {
    lua_support::check(SCRIPT, "Vec4_Addition");
}

#[test]
fn subtraction() {
    lua_support::check(SCRIPT, "Vec4_Subtraction");
}

#[test]
fn unary_minus() {
    lua_support::check(SCRIPT, "Vec4_UnaryMinus");
}

#[test]
fn multiplication_scalar() {
    lua_support::check(SCRIPT, "Vec4_Multiplication_scalar");
}

#[test]
fn multiplication_scalar_reversed() {
    lua_support::check(SCRIPT, "Vec4_Multiplication_scalar_reversed");
}

#[test]
fn multiplication_vec() {
    lua_support::check(SCRIPT, "Vec4_Multiplication_vec");
}

#[test]
fn division_scalar() {
    lua_support::check(SCRIPT, "Vec4_Division_scalar");
}

#[test]
fn division_vec() {
    lua_support::check(SCRIPT, "Vec4_Division_vec");
}

#[test]
fn equal_to_true() {
    lua_support::check(SCRIPT, "Vec4_EqualTo_true");
}

#[test]
fn equal_to_false() {
    lua_support::check(SCRIPT, "Vec4_EqualTo_false");
}

#[test]
fn less_than() {
    lua_support::check(SCRIPT, "Vec4_LessThan");
}

#[test]
fn less_than_or_equal() {
    lua_support::check(SCRIPT, "Vec4_LessThanOrEqual");
}

#[test]
fn to_string() {
    lua_support::check(SCRIPT, "Vec4_ToString");
}

#[test]
fn length() {
    lua_support::check(SCRIPT, "Vec4_Length");
}

#[test]
fn length_sqr() {
    lua_support::check(SCRIPT, "Vec4_LengthSqr");
}

#[test]
fn dot() {
    lua_support::check(SCRIPT, "Vec4_Dot");
}

#[test]
fn dot_perpendicular() {
    lua_support::check(SCRIPT, "Vec4_Dot_perpendicular");
}

#[test]
fn sum() {
    lua_support::check(SCRIPT, "Vec4_Sum");
}

#[test]
fn abs() {
    lua_support::check(SCRIPT, "Vec4_Abs");
}

#[test]
fn clamp() {
    lua_support::check(SCRIPT, "Vec4_Clamp");
}
