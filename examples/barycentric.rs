use omath::{Triangle, Vector3};
fn main() {
    let triangle = Triangle::new(
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
    );
    for i in 0..=10 {
        let u = i as f32 / 10.0;
        let v = (1.0 - u) * 0.5;
        let w = 1.0 - u - v;
        let point = triangle.vertex1 * u + triangle.vertex2 * v + triangle.vertex3 * w;
        println!("weights ({u}, {v}, {w}) -> {point:?}");
    }
}
