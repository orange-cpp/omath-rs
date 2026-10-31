#![cfg(feature = "lua")]

mod lua_support;

const SCRIPT: &str = "vec3_tests.lua";

#[test]
fn constructor_default() {
    lua_support::check(SCRIPT, "Vec3_Constructor_default");
}

#[test]
fn constructor_xyz() {
    lua_support::check(SCRIPT, "Vec3_Constructor_xyz");
}

#[test]
fn field_mutation() {
    lua_support::check(SCRIPT, "Vec3_Field_mutation");
}

#[test]
fn addition() {
    lua_support::check(SCRIPT, "Vec3_Addition");
}

#[test]
fn subtraction() {
    lua_support::check(SCRIPT, "Vec3_Subtraction");
}

#[test]
fn unary_minus() {
    lua_support::check(SCRIPT, "Vec3_UnaryMinus");
}

#[test]
fn multiplication_scalar() {
    lua_support::check(SCRIPT, "Vec3_Multiplication_scalar");
}

#[test]
fn multiplication_scalar_reversed() {
    lua_support::check(SCRIPT, "Vec3_Multiplication_scalar_reversed");
}

#[test]
fn multiplication_vec() {
    lua_support::check(SCRIPT, "Vec3_Multiplication_vec");
}

#[test]
fn division_scalar() {
    lua_support::check(SCRIPT, "Vec3_Division_scalar");
}

#[test]
fn division_vec() {
    lua_support::check(SCRIPT, "Vec3_Division_vec");
}

#[test]
fn equal_to_true() {
    lua_support::check(SCRIPT, "Vec3_EqualTo_true");
}

#[test]
fn equal_to_false() {
    lua_support::check(SCRIPT, "Vec3_EqualTo_false");
}

#[test]
fn less_than() {
    lua_support::check(SCRIPT, "Vec3_LessThan");
}

#[test]
fn less_than_or_equal() {
    lua_support::check(SCRIPT, "Vec3_LessThanOrEqual");
}

#[test]
fn to_string() {
    lua_support::check(SCRIPT, "Vec3_ToString");
}

#[test]
fn length() {
    lua_support::check(SCRIPT, "Vec3_Length");
}

#[test]
fn length2d() {
    lua_support::check(SCRIPT, "Vec3_Length2d");
}

#[test]
fn length_sqr() {
    lua_support::check(SCRIPT, "Vec3_LengthSqr");
}

#[test]
fn normalized() {
    lua_support::check(SCRIPT, "Vec3_Normalized");
}

#[test]
fn dot_perpendicular() {
    lua_support::check(SCRIPT, "Vec3_Dot_perpendicular");
}

#[test]
fn dot_parallel() {
    lua_support::check(SCRIPT, "Vec3_Dot_parallel");
}

#[test]
fn cross() {
    lua_support::check(SCRIPT, "Vec3_Cross");
}

#[test]
fn distance_to() {
    lua_support::check(SCRIPT, "Vec3_DistanceTo");
}

#[test]
fn distance_to_sqr() {
    lua_support::check(SCRIPT, "Vec3_DistanceToSqr");
}

#[test]
fn sum() {
    lua_support::check(SCRIPT, "Vec3_Sum");
}

#[test]
fn sum2d() {
    lua_support::check(SCRIPT, "Vec3_Sum2d");
}

#[test]
fn abs() {
    lua_support::check(SCRIPT, "Vec3_Abs");
}

#[test]
fn point_to_same_direction_true() {
    lua_support::check(SCRIPT, "Vec3_PointToSameDirection_true");
}

#[test]
fn point_to_same_direction_false() {
    lua_support::check(SCRIPT, "Vec3_PointToSameDirection_false");
}

#[test]
fn is_perpendicular_true() {
    lua_support::check(SCRIPT, "Vec3_IsPerpendicular_true");
}

#[test]
fn is_perpendicular_false() {
    lua_support::check(SCRIPT, "Vec3_IsPerpendicular_false");
}

#[test]
fn angle_between_90deg() {
    lua_support::check(SCRIPT, "Vec3_AngleBetween_90deg");
}

#[test]
fn angle_between_zero_vector_error() {
    lua_support::check(SCRIPT, "Vec3_AngleBetween_zero_vector_error");
}

#[test]
fn as_table() {
    lua_support::check(SCRIPT, "Vec3_AsTable");
}
