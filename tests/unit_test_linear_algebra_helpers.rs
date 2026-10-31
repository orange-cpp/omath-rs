use omath::{Triangle, Vector3, Vector4};

#[test]
fn vector3_no_inline_helpers_execute() {
    let a = Vector3::<f32>::new(1.0f32, 2.0f32, 3.0f32);
    let b = Vector3::<f32>::new(4.0f32, 5.0f32, 6.0f32);

    // Execute helpers that were made non-inlined
    let _ = a.length();
    let _ = a.angle_between(b);
    let _ = a.is_perpendicular(b, 0.0001f32);
    let _ = a.normalized();
}

#[test]
fn triangle_no_inline_helpers_execute() {
    let v1 = Vector3::<f32>::new(0.0f32, 0.0f32, 0.0f32);
    let v2 = Vector3::<f32>::new(3.0f32, 0.0f32, 0.0f32);
    let v3 = Vector3::<f32>::new(3.0f32, 4.0f32, 0.0f32);

    let t = Triangle::new(v1, v2, v3);

    let _ = t.calculate_normal();
    let _ = t.side_a_length();
    let _ = t.side_b_length();
    let _ = t.hypot();
    let _ = t.is_rectangular();
}

#[test]
fn vector4_no_inline_helpers_execute() {
    let v = Vector4::<f32>::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);

    let _ = v.length();
    let _ = v.sum();
    let _ = v.clamped(-10.0f32, 10.0f32);
}
