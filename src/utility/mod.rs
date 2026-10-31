//! Colors and byte pattern scanning.

mod byte_reader;
mod color;
mod elf_pattern_scan;
mod macho_pattern_scan;
mod pattern_scan;
mod pe_pattern_scan;
mod section_scan_result;

pub use color::{Color, Hsv};
pub use elf_pattern_scan::ElfPatternScanner;
pub use macho_pattern_scan::MachOPatternScanner;
pub use pattern_scan::{PatternScanError, PatternScanner};
pub use pe_pattern_scan::PePatternScanner;
pub use section_scan_result::SectionScanResult;
pub mod loaded_module;
pub use loaded_module::LoadedModule;
