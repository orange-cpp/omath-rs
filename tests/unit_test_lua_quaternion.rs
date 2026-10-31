#![cfg(feature = "lua")]

mod lua_support;

const SCRIPT: &str = "quaternion_tests.lua";

#[test]
fn constructor_default() {
    lua_support::check(SCRIPT, "Quaternion_Constructor_default");
}

#[test]
fn constructor_xyzw() {
    lua_support::check(SCRIPT, "Quaternion_Constructor_xyzw");
}

#[test]
fn field_mutation() {
    lua_support::check(SCRIPT, "Quaternion_Field_mutation");
}

#[test]
fn from_axis_angle_zero_angle() {
    lua_support::check(SCRIPT, "Quaternion_FromAxisAngle_zero_angle");
}

#[test]
fn addition() {
    lua_support::check(SCRIPT, "Quaternion_Addition");
}

#[test]
fn multiplication_scalar() {
    lua_support::check(SCRIPT, "Quaternion_Multiplication_scalar");
}

#[test]
fn multiplication_scalar_reversed() {
    lua_support::check(SCRIPT, "Quaternion_Multiplication_scalar_reversed");
}

#[test]
fn multiplication_quaternion() {
    lua_support::check(SCRIPT, "Quaternion_Multiplication_quaternion");
}

#[test]
fn unary_minus() {
    lua_support::check(SCRIPT, "Quaternion_UnaryMinus");
}

#[test]
fn equal_to_true() {
    lua_support::check(SCRIPT, "Quaternion_EqualTo_true");
}

#[test]
fn equal_to_false() {
    lua_support::check(SCRIPT, "Quaternion_EqualTo_false");
}

#[test]
fn to_string() {
    lua_support::check(SCRIPT, "Quaternion_ToString");
}

#[test]
fn conjugate() {
    lua_support::check(SCRIPT, "Quaternion_Conjugate");
}

#[test]
fn dot() {
    lua_support::check(SCRIPT, "Quaternion_Dot");
}

#[test]
fn length() {
    lua_support::check(SCRIPT, "Quaternion_Length");
}

#[test]
fn length_sqr() {
    lua_support::check(SCRIPT, "Quaternion_LengthSqr");
}

#[test]
fn normalized() {
    lua_support::check(SCRIPT, "Quaternion_Normalized");
}

#[test]
fn inverse() {
    lua_support::check(SCRIPT, "Quaternion_Inverse");
}

#[test]
fn rotate() {
    lua_support::check(SCRIPT, "Quaternion_Rotate");
}

#[test]
fn to_rotation_matrix3() {
    lua_support::check(SCRIPT, "Quaternion_ToRotationMatrix3");
}

#[test]
fn to_rotation_matrix4() {
    lua_support::check(SCRIPT, "Quaternion_ToRotationMatrix4");
}

#[test]
fn as_table() {
    lua_support::check(SCRIPT, "Quaternion_AsTable");
}
