#[macro_use]
mod projection_support;

use omath::Vector3;
use omath::primitives::{Aabb, UpAxis, UpZ};

type AABB = Aabb<f32>;
type AABBZ = Aabb<f32, UpZ>;
type Vec3 = Vector3<f32>;

#[test]
fn center_of_symmetric_box() {
    let box_: AABB = Aabb::new(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0));
    let c = box_.center();
    expect_float_eq!(c.x, 0.0);
    expect_float_eq!(c.y, 0.0);
    expect_float_eq!(c.z, 0.0);
}

#[test]
fn center_of_offset_box() {
    let box_: AABB = Aabb::new(Vector3::new(1.0, 2.0, 3.0), Vector3::new(3.0, 6.0, 7.0));
    let c = box_.center();
    expect_float_eq!(c.x, 2.0);
    expect_float_eq!(c.y, 4.0);
    expect_float_eq!(c.z, 5.0);
}

#[test]
fn center_of_degenerate_box() {
    let box_: AABB = Aabb::new(Vector3::new(5.0, 5.0, 5.0), Vector3::new(5.0, 5.0, 5.0));
    let c = box_.center();
    expect_float_eq!(c.x, 5.0);
    expect_float_eq!(c.y, 5.0);
    expect_float_eq!(c.z, 5.0);
}

#[test]
fn extents_of_symmetric_box() {
    let box_: AABB = Aabb::new(Vector3::new(-2.0, -3.0, -4.0), Vector3::new(2.0, 3.0, 4.0));
    let e = box_.extents();
    expect_float_eq!(e.x, 2.0);
    expect_float_eq!(e.y, 3.0);
    expect_float_eq!(e.z, 4.0);
}

#[test]
fn extents_of_unit_box() {
    let box_: AABB = Aabb::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(2.0, 2.0, 2.0));
    let e = box_.extents();
    expect_float_eq!(e.x, 1.0);
    expect_float_eq!(e.y, 1.0);
    expect_float_eq!(e.z, 1.0);
}

#[test]
fn extents_of_degenerate_box() {
    let box_: AABB = Aabb::new(Vector3::new(3.0, 3.0, 3.0), Vector3::new(3.0, 3.0, 3.0));
    let e = box_.extents();
    expect_float_eq!(e.x, 0.0);
    expect_float_eq!(e.y, 0.0);
    expect_float_eq!(e.z, 0.0);
}

#[test]
fn top_y_up_symmetric_box() {
    let box_: AABB = Aabb::new(Vector3::new(-1.0, -2.0, -3.0), Vector3::new(1.0, 2.0, 3.0));
    let t = box_.top();
    expect_float_eq!(t.x, 0.0);
    expect_float_eq!(t.y, 2.0);
    expect_float_eq!(t.z, 0.0);
}

#[test]
fn top_y_up_offset_box() {
    let box_: AABB = Aabb::new(Vector3::new(1.0, 4.0, 2.0), Vector3::new(3.0, 10.0, 6.0));
    let t = box_.top();
    expect_float_eq!(t.x, 2.0);
    expect_float_eq!(t.y, 10.0);
    expect_float_eq!(t.z, 4.0);
}

#[test]
fn top_z_up_symmetric_box() {
    let box_: AABBZ = Aabb::new(Vector3::new(-1.0, -2.0, -3.0), Vector3::new(1.0, 2.0, 3.0));
    let t = box_.top();
    expect_float_eq!(t.x, 0.0);
    expect_float_eq!(t.y, 0.0);
    expect_float_eq!(t.z, 3.0);
}

#[test]
fn top_z_up_offset_box() {
    let box_: AABBZ = Aabb::new(Vector3::new(1.0, 4.0, 2.0), Vector3::new(3.0, 10.0, 6.0));
    let t = box_.top();
    expect_float_eq!(t.x, 2.0);
    expect_float_eq!(t.y, 7.0);
    expect_float_eq!(t.z, 6.0);
}

#[test]
fn top_default_is_y_up() {
    let box_: AABB = Aabb::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(2.0, 4.0, 6.0));
    assert_eq!(AABB::get_up_axis(), UpAxis::Y);
    expect_float_eq!(box_.top().y, 4.0);
}

#[test]
fn bottom_y_up_symmetric_box() {
    let box_: AABB = Aabb::new(Vector3::new(-1.0, -2.0, -3.0), Vector3::new(1.0, 2.0, 3.0));
    let b = box_.bottom();
    expect_float_eq!(b.x, 0.0);
    expect_float_eq!(b.y, -2.0);
    expect_float_eq!(b.z, 0.0);
}

#[test]
fn bottom_y_up_offset_box() {
    let box_: AABB = Aabb::new(Vector3::new(1.0, 4.0, 2.0), Vector3::new(3.0, 10.0, 6.0));
    let b = box_.bottom();
    expect_float_eq!(b.x, 2.0);
    expect_float_eq!(b.y, 4.0);
    expect_float_eq!(b.z, 4.0);
}

#[test]
fn bottom_z_up_symmetric_box() {
    let box_: AABBZ = Aabb::new(Vector3::new(-1.0, -2.0, -3.0), Vector3::new(1.0, 2.0, 3.0));
    let b = box_.bottom();
    expect_float_eq!(b.x, 0.0);
    expect_float_eq!(b.y, 0.0);
    expect_float_eq!(b.z, -3.0);
}

#[test]
fn bottom_z_up_offset_box() {
    let box_: AABBZ = Aabb::new(Vector3::new(1.0, 4.0, 2.0), Vector3::new(3.0, 10.0, 6.0));
    let b = box_.bottom();
    expect_float_eq!(b.x, 2.0);
    expect_float_eq!(b.y, 7.0);
    expect_float_eq!(b.z, 2.0);
}

#[test]
fn bottom_default_is_y_up() {
    let box_: AABB = Aabb::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(2.0, 4.0, 6.0));
    assert_eq!(AABB::get_up_axis(), UpAxis::Y);
    expect_float_eq!(box_.bottom().y, 0.0);
}

#[test]
fn top_and_bottom_are_symmetric() {
    let box_: AABB = Aabb::new(Vector3::new(-1.0, -2.0, -3.0), Vector3::new(1.0, 2.0, 3.0));
    let z_up_box: AABBZ = Aabb::new(Vector3::new(-1.0, -2.0, -3.0), Vector3::new(1.0, 2.0, 3.0));
    expect_float_eq!(box_.top().y, -box_.bottom().y);
    expect_float_eq!(z_up_box.top().z, -z_up_box.bottom().z);
}

#[test]
fn vertices_returns_all_corners() {
    let box_: AABB = Aabb::new(Vector3::new(1.0, 2.0, 3.0), Vector3::new(4.0, 6.0, 8.0));
    let v = box_.vertices();

    assert_eq!(v[0], Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(v[1], Vec3::new(4.0, 2.0, 3.0));
    assert_eq!(v[2], Vec3::new(1.0, 6.0, 3.0));
    assert_eq!(v[3], Vec3::new(4.0, 6.0, 3.0));
    assert_eq!(v[4], Vec3::new(1.0, 2.0, 8.0));
    assert_eq!(v[5], Vec3::new(4.0, 2.0, 8.0));
    assert_eq!(v[6], Vec3::new(1.0, 6.0, 8.0));
    assert_eq!(v[7], Vec3::new(4.0, 6.0, 8.0));
}

#[test]
fn overlapping_boxes_collide() {
    let a: AABB = Aabb::new(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0));
    let b: AABB = Aabb::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(2.0, 2.0, 2.0));
    assert!(a.is_collide(&b));
    assert!(b.is_collide(&a));
}

#[test]
fn separated_boxes_do_not_collide() {
    let a: AABB = Aabb::new(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0));
    let b: AABB = Aabb::new(Vector3::new(2.0, 2.0, 2.0), Vector3::new(4.0, 4.0, 4.0));
    assert!(!(a.is_collide(&b)));
    assert!(!(b.is_collide(&a)));
}

#[test]
fn touching_faces_collide() {
    let a: AABB = Aabb::new(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0));
    let b: AABB = Aabb::new(Vector3::new(1.0, -1.0, -1.0), Vector3::new(3.0, 1.0, 1.0));
    assert!(a.is_collide(&b));
    assert!(b.is_collide(&a));
}

#[test]
fn contained_box_collides() {
    let outer: AABB = Aabb::new(Vector3::new(-3.0, -3.0, -3.0), Vector3::new(3.0, 3.0, 3.0));
    let inner: AABB = Aabb::new(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0));
    assert!(outer.is_collide(&inner));
    assert!(inner.is_collide(&outer));
}

#[test]
fn separated_on_x_axis_do_not_collide() {
    let a: AABB = Aabb::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0));
    let b: AABB = Aabb::new(Vector3::new(2.0, 0.0, 0.0), Vector3::new(3.0, 1.0, 1.0));
    assert!(!(a.is_collide(&b)));
}

#[test]
fn separated_on_y_axis_do_not_collide() {
    let a: AABB = Aabb::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0));
    let b: AABB = Aabb::new(Vector3::new(0.0, 2.0, 0.0), Vector3::new(1.0, 3.0, 1.0));
    assert!(!(a.is_collide(&b)));
}

#[test]
fn separated_on_z_axis_do_not_collide() {
    let a: AABB = Aabb::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0));
    let b: AABB = Aabb::new(Vector3::new(0.0, 0.0, 2.0), Vector3::new(1.0, 1.0, 3.0));
    assert!(!(a.is_collide(&b)));
}

#[test]
fn identical_boxes_collide() {
    let a: AABB = Aabb::new(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0));
    assert!(a.is_collide(&a));
}

#[test]
fn degenerate_point_box_collides_when_inside_other() {
    let box_: AABB = Aabb::new(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0));
    let point: AABB = Aabb::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0));
    assert!(box_.is_collide(&point));
    assert!(point.is_collide(&box_));
}

#[test]
fn degenerate_point_box_does_not_collide_when_outside() {
    let box_: AABB = Aabb::new(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0));
    let point: AABB = Aabb::new(Vector3::new(5.0, 0.0, 0.0), Vector3::new(5.0, 0.0, 0.0));
    assert!(!(box_.is_collide(&point)));
    assert!(!(point.is_collide(&box_)));
}
