//! UE 2.5/Killing Floor 1 defaults; supply addresses from your own target build.
use omath::{engines::unreal::*, rev_eng::ProcessMemory};
fn main() -> Result<(), std::boxed::Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 4 {
        println!("usage: kf1_dumper <pid> <gobjects hex> <gnames hex>");
        return Ok(());
    }
    let memory = ProcessMemory::open(args[1].parse()?, false)?;
    let objects = usize::from_str_radix(args[2].trim_start_matches("0x"), 16)?;
    let names = usize::from_str_radix(args[3].trim_start_matches("0x"), 16)?;
    let layout = ObjectArrayLayout::ue2_5();
    for index in 0..get_object_count(&memory, objects, &layout)? {
        match get_object_by_index(&memory, objects, index, &layout) {
            Ok(object) => println!(
                "{index}: {object:#x} {}",
                get_actor_name(&memory, object, names, &NameLayout::ue2_5())?
            ),
            Err(ObjectArrayError::NullObject) => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}
