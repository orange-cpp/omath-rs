//! Interactive counterparts of the GLFW/barycentric/projection/HUD examples.
//! The window and compatibility OpenGL renderer are implemented in Rust.
#[cfg(all(windows, feature = "hooks", feature = "imgui"))]
#[path = "support/desktop.rs"]
mod desktop;
#[cfg(all(windows, feature = "hooks", feature = "imgui"))]
fn main() -> Result<(), std::boxed::Box<dyn std::error::Error>> {
    desktop::run()
}
#[cfg(not(all(windows, feature = "hooks", feature = "imgui")))]
fn main() {
    println!(
        "The native showcase requires Windows and --features hooks,imgui. Portable command-line examples: hud, barycentric, projection_matrix."
    );
}
