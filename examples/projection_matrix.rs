use omath::{
    Vector3,
    engines::source,
    projection::{FieldOfView, ViewPort},
};
fn main() {
    let camera = source::Camera::new(
        Vector3::default(),
        source::ViewAngles::default(),
        ViewPort::new(1920.0, 1080.0),
        FieldOfView::from_degrees(90.0),
        0.01,
        1000.0,
    );
    println!("{:?}", camera.get_projection_matrix());
    println!(
        "screen center: {:?}",
        camera.world_to_screen(Vector3::new(100.0, 0.0, 0.0))
    );
}
