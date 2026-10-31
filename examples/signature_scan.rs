use omath::utility::{ElfPatternScanner, MachOPatternScanner, PatternScanner, PePatternScanner};
fn main() -> Result<(), std::boxed::Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 3 {
        println!("usage: cargo run --example signature_scan -- <image> <pattern> [section]");
        return Ok(());
    }
    let data = std::fs::read(&args[1])?;
    let section = args.get(3).map(String::as_str);
    let result = if data.starts_with(b"MZ") {
        PePatternScanner::scan_for_pattern_in_memory_file(
            &data,
            &args[2],
            section.unwrap_or(".text"),
        )
    } else if data.starts_with(b"\x7fELF") {
        ElfPatternScanner::scan_for_pattern_in_memory_file(
            &data,
            &args[2],
            section.unwrap_or(".text"),
        )
    } else {
        MachOPatternScanner::scan_for_pattern_in_memory_file(
            &data,
            &args[2],
            section.unwrap_or("__text"),
        )
    };
    println!("section match: {result:?}");
    println!(
        "whole-image byte offset: {:?}",
        PatternScanner::scan_for_pattern(&data, &args[2])
    );
    Ok(())
}
