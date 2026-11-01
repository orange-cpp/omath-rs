//! KF1 1065 process discovery, reflected camera readout and filtered object dump.
#[path = "support/kf1/mod.rs"]
mod kf1;
#[cfg(all(windows, feature = "hooks"))]
fn main() -> Result<(), std::boxed::Box<dyn std::error::Error>> {
    use kf1::reader::*;
    use std::collections::HashMap;
    let target = platform::attach()?;
    let filter = std::env::args().nth(1).unwrap_or_default();
    let classes = target.find_reflection_classes();
    let player = target.find_local_player(&classes);
    println!(
        "GObjects {:#x}, GNames {:#x}, {} slots",
        target.g_objects,
        target.g_names,
        target.live_object_count()
    );
    if player.valid() {
        let camera = target.read_camera(&player);
        println!(
            "controller {:#x}, pawn {:#x}, position {:?}, angles {:?}, FOV {:.2}",
            player.controller, player.pawn, camera.position, camera.angles, camera.fov
        );
        let forward = omath::engines::unreal::forward_vector(camera.angles);
        println!("forward {forward:?}");
        if player.fov_offset.is_none() {
            for name in target.list_properties(
                target.read_ptr(player.controller, OBJECT_CLASS_OFFSET),
                classes.property,
                40,
            ) {
                println!("{name}");
            }
        }
    } else {
        println!("no local player; waiting for a level");
    }
    let mut names = HashMap::new();
    let mut count = 0;
    for index in 0..target.live_object_count() {
        if let Some(object) = target.object_at(index) {
            let name = target.object_name(object);
            let class = target.read_ptr(object, OBJECT_CLASS_OFFSET);
            let class_name = names
                .entry(class)
                .or_insert_with(|| target.object_name(class));
            if !filter.is_empty() && !name.contains(&filter) && !class_name.contains(&filter) {
                continue;
            }
            println!("[{index:6}] {object:#010x} {class_name:<32} {name}");
            count += 1;
        }
    }
    println!("printed {count} objects");
    Ok(())
}
#[cfg(not(all(windows, feature = "hooks")))]
fn main() {
    println!(
        "KF1 process discovery requires Windows and --features hooks. Memory/layout helpers remain portable."
    );
}
