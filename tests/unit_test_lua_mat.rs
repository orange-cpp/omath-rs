#![cfg(feature = "lua")]

mod lua_support;

const SCRIPT: &str = "mat_tests.lua";

#[test]
fn constructor_default() {
    lua_support::check(SCRIPT, "Mat4_Constructor_default");
}

#[test]
fn from_rows() {
    lua_support::check(SCRIPT, "Mat4_FromRows");
}

#[test]
fn set_at() {
    lua_support::check(SCRIPT, "Mat4_SetAt");
}

#[test]
fn set_and_clear() {
    lua_support::check(SCRIPT, "Mat4_SetAndClear");
}

#[test]
fn multiplication_matrix() {
    lua_support::check(SCRIPT, "Mat4_Multiplication_matrix");
}

#[test]
fn multiplication_scalar() {
    lua_support::check(SCRIPT, "Mat4_Multiplication_scalar");
}

#[test]
fn multiplication_scalar_reversed() {
    lua_support::check(SCRIPT, "Mat4_Multiplication_scalar_reversed");
}

#[test]
fn division_scalar() {
    lua_support::check(SCRIPT, "Mat4_Division_scalar");
}

#[test]
fn transposed() {
    lua_support::check(SCRIPT, "Mat4_Transposed");
}

#[test]
fn determinant() {
    lua_support::check(SCRIPT, "Mat4_Determinant");
}

#[test]
fn inverted_success() {
    lua_support::check(SCRIPT, "Mat4_Inverted_success");
}

#[test]
fn inverted_singular() {
    lua_support::check(SCRIPT, "Mat4_Inverted_singular");
}

#[test]
fn to_screen_mat() {
    lua_support::check(SCRIPT, "Mat4_ToScreenMat");
}

#[test]
fn translation() {
    lua_support::check(SCRIPT, "Mat4_Translation");
}

#[test]
fn scale() {
    lua_support::check(SCRIPT, "Mat4_Scale");
}

#[test]
fn perspective() {
    lua_support::check(SCRIPT, "Mat4_Perspective");
}

#[test]
fn as_table() {
    lua_support::check(SCRIPT, "Mat4_AsTable");
}

#[test]
fn to_string() {
    lua_support::check(SCRIPT, "Mat4_ToString");
}

#[test]
fn mat3_from_rows() {
    lua_support::check(SCRIPT, "Mat3_FromRows");
}

#[test]
fn mat4_column_major_from_rows() {
    lua_support::check(SCRIPT, "Mat4ColumnMajor_FromRows");
}

#[test]
fn mat4_column_major_to_screen_mat() {
    lua_support::check(SCRIPT, "Mat4ColumnMajor_ToScreenMat");
}

#[test]
fn mat4_column_major_translation() {
    lua_support::check(SCRIPT, "Mat4ColumnMajor_Translation");
}

#[test]
fn mat4d_from_rows() {
    lua_support::check(SCRIPT, "Mat4d_FromRows");
}
