//! Pattern scanning in Mach-O images held in a file or a byte slice.

use super::byte_reader::{read_bytes, read_u32, read_u32_as_u64, read_u64, until_nul};
use super::pattern_scan::PatternScanner;
use super::section_scan_result::{ExtractedSection, SectionScanResult, impl_section_scanner};

const MH_MAGIC_32: u32 = 0xFEED_FACE;
const MH_MAGIC_64: u32 = 0xFEED_FACF;
const MH_CIGAM_32: u32 = 0xCEFA_EDFE;
const MH_CIGAM_64: u32 = 0xCFFA_EDFE;

const LC_SEGMENT: u32 = 0x1;
const LC_SEGMENT_64: u32 = 0x19;

const NCMDS_OFFSET: usize = 16;
const LOAD_COMMAND_SIZE: usize = 8;
const CMDSIZE_OFFSET: usize = 4;
const SECTION_NAME_SIZE: usize = 16;
const SECTION_ADDR_OFFSET: usize = 32;

/// Where the 32-bit and 64-bit formats keep the fields the scanner reads.
struct MachOLayout {
    header_size: usize,
    segment_cmd: u32,
    segment_size: usize,
    segment_nsects: usize,
    section_size: usize,
    section_size_field: usize,
    section_offset_field: usize,
    /// Reads a field that is as wide as an address in this format.
    read_word: fn(&[u8], usize) -> Option<u64>,
}

const MACHO32_LAYOUT: MachOLayout = MachOLayout {
    header_size: 28,
    segment_cmd: LC_SEGMENT,
    segment_size: 56,
    segment_nsects: 48,
    section_size: 68,
    section_size_field: 36,
    section_offset_field: 40,
    read_word: read_u32_as_u64,
};

const MACHO64_LAYOUT: MachOLayout = MachOLayout {
    header_size: 32,
    segment_cmd: LC_SEGMENT_64,
    segment_size: 72,
    segment_nsects: 64,
    section_size: 80,
    section_size_field: 40,
    section_offset_field: 48,
    read_word: read_u64,
};

fn extract_section_with_layout<'a>(
    data: &'a [u8],
    section_name: &str,
    layout: &MachOLayout,
) -> Option<ExtractedSection<'a>> {
    if data.len() < layout.header_size {
        return None;
    }

    let ncmds = read_u32(data, NCMDS_OFFSET)?;
    let mut cmd_offset = layout.header_size;

    for _ in 0..ncmds {
        let cmd = read_u32(data, cmd_offset)?;
        let cmdsize = usize::try_from(read_u32(data, cmd_offset + CMDSIZE_OFFSET)?).ok()?;

        // Would not move on to the next command, spinning in place for up to 2^32 iterations
        if cmdsize < LOAD_COMMAND_SIZE {
            return None;
        }

        if cmd == layout.segment_cmd {
            read_bytes(data, cmd_offset, layout.segment_size)?;

            let nsects = read_u32(data, cmd_offset + layout.segment_nsects)?;
            let mut section_offset = cmd_offset + layout.segment_size;

            for _ in 0..nsects {
                let section = read_bytes(data, section_offset, layout.section_size)?;

                if until_nul(&section[..SECTION_NAME_SIZE]) == section_name.as_bytes() {
                    let raw_off = read_u32(section, layout.section_offset_field)?;
                    let size = (layout.read_word)(section, layout.section_size_field)?;

                    return Some(ExtractedSection {
                        virtual_base_addr: (layout.read_word)(section, SECTION_ADDR_OFFSET)?,
                        raw_base_addr: u64::from(raw_off),
                        data: read_bytes(
                            data,
                            usize::try_from(raw_off).ok()?,
                            usize::try_from(size).ok()?,
                        )?,
                    });
                }

                section_offset += layout.section_size;
            }
        }

        cmd_offset = cmd_offset.checked_add(cmdsize)?;
    }

    None
}

fn extract_section<'a>(data: &'a [u8], section_name: &str) -> Option<ExtractedSection<'a>> {
    match read_u32(data, 0)? {
        MH_MAGIC_64 | MH_CIGAM_64 => {
            extract_section_with_layout(data, section_name, &MACHO64_LAYOUT)
        }
        MH_MAGIC_32 | MH_CIGAM_32 => {
            extract_section_with_layout(data, section_name, &MACHO32_LAYOUT)
        }
        _ => None,
    }
}

pub struct MachOPatternScanner;

impl_section_scanner!(MachOPatternScanner, "__text");
