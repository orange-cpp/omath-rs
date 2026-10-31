mod common;

use common::*;
use omath::trigonometry::angles;

#[test]
fn radians_to_deg() {
    let rad = 67.0f32;

    expect_near(angles::radians_to_degrees(rad), 3838.82f32, 0.01f32);
}

#[test]
fn degrees_to_radians() {
    let degree = 90.0f32;

    expect_near(angles::degrees_to_radians(degree), 1.5708f32, 0.01f32);
}

#[test]
fn horizontal_fov_to_verical() {

    let h_fov: f32 = 90.0f32;
    let aspect_ration: f32 = 16.0f32 / 9.0f32;
    let vertical_fov = angles::horizontal_fov_to_vertical(h_fov, aspect_ration);

    expect_near(vertical_fov, 58.71f32, 0.01f32);

}

#[test]
fn vertical_to_horizontal() {
    let v_fov = 58.71f32;
    let aspect_ration = 16.0f32 / 9.0f32;
    let horizontal_fov = angles::vertical_fov_to_horizontal(v_fov, aspect_ration);

    expect_near(horizontal_fov, 89.99f32, 0.01f32);
}

#[test]
fn wrap_angle() {

    let wrapped: f32 = angles::wrap_angle(361.0f32, 0.0f32, 360.0f32);

    expect_near(wrapped, 1.0f32, 0.01f32);

}

#[test]
fn wrap_angle_negative_range() {

    let wrapped: f32 = angles::wrap_angle(-90.0f32, 0.0f32, 360.0f32);

    expect_near(wrapped, 270.0f32, 0.01f32);

}

#[test]
fn wrap_angle_maximum_maps_to_minimum() {

    let wrapped: f32 = angles::wrap_angle(360.0f32, 0.0f32, 360.0f32);

    expect_float_eq(wrapped, 0.0f32);

}
