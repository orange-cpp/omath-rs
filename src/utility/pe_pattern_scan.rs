//! Pattern scanning in PE images held in a file or a byte slice.

use super::byte_reader::{read_bytes, read_u16, read_u32, read_u64, until_nul};
use super::pattern_scan::PatternScanner;
use super::section_scan_result::{ExtractedSection, SectionScanResult, impl_section_scanner};

const DOS_HDR_MAGIC: u16 = 0x5A4D;
const NT_HDR_MAGIC: u32 = 0x4550;
const OPT_HDR32_MAGIC: u16 = 0x010B;
const OPT_HDR64_MAGIC: u16 = 0x020B;

const DOS_HEADER_SIZE: usize = 64;
const E_LFANEW_OFFSET: usize = 0x3C;

const SIGNATURE_SIZE: usize = 4;
const FILE_HEADER_SIZE: usize = 20;
const NUM_SECTIONS_OFFSET: usize = 2;
const SIZE_OPTIONAL_HEADER_OFFSET: usize = 16;

const OPTIONAL_HEADER_X86_SIZE: usize = 224;
const OPTIONAL_HEADER_X64_SIZE: usize = 240;
const IMAGE_BASE_X86_OFFSET: usize = 28;
const IMAGE_BASE_X64_OFFSET: usize = 24;

const SECTION_HEADER_SIZE: usize = 40;
const SECTION_NAME_SIZE: usize = 8;
const VIRTUAL_ADDRESS_OFFSET: usize = 12;
const SIZE_RAW_DATA_OFFSET: usize = 16;
const PTR_RAW_DATA_OFFSET: usize = 20;

#[derive(Clone, Copy)]
enum ImageBase {
    X86(u32),
    X64(u64),
}

impl ImageBase {
    /// The sum wraps at the width of the image's own address space, as it does in C++.
    fn virtual_base_addr(self, virtual_address: u32) -> u64 {
        match self {
            Self::X86(image_base) => u64::from(virtual_address.wrapping_add(image_base)),
            Self::X64(image_base) => u64::from(virtual_address).wrapping_add(image_base),
        }
    }
}

fn extract_section<'a>(data: &'a [u8], section_name: &str) -> Option<ExtractedSection<'a>> {
    if data.len() < DOS_HEADER_SIZE || read_u16(data, 0)? != DOS_HDR_MAGIC {
        return None;
    }

    let nt_off = usize::try_from(read_u32(data, E_LFANEW_OFFSET)?).ok()?;
    let file_header_off = nt_off.checked_add(SIGNATURE_SIZE)?;
    let optional_header_off = file_header_off.checked_add(FILE_HEADER_SIZE)?;

    read_bytes(data, optional_header_off, OPTIONAL_HEADER_X86_SIZE)?;

    let image_base = match read_u16(data, optional_header_off)? {
        OPT_HDR32_MAGIC => ImageBase::X86(read_u32(data, optional_header_off + IMAGE_BASE_X86_OFFSET)?),
        OPT_HDR64_MAGIC => {
            read_bytes(data, optional_header_off, OPTIONAL_HEADER_X64_SIZE)?;
            ImageBase::X64(read_u64(data, optional_header_off + IMAGE_BASE_X64_OFFSET)?)
        }
        _ => return None,
    };

    if read_u32(data, nt_off)? != NT_HDR_MAGIC {
        return None;
    }

    let num_sections = usize::from(read_u16(data, file_header_off + NUM_SECTIONS_OFFSET)?);
    let size_optional_header = usize::from(read_u16(data, file_header_off + SIZE_OPTIONAL_HEADER_OFFSET)?);
    let section_table_off = optional_header_off.checked_add(size_optional_header)?;

    for index in 0..num_sections {
        let section_off = index.checked_mul(SECTION_HEADER_SIZE)?.checked_add(section_table_off)?;
        let section = read_bytes(data, section_off, SECTION_HEADER_SIZE)?;

        if until_nul(&section[..SECTION_NAME_SIZE]) != section_name.as_bytes() {
            continue;
        }

        let raw_size = usize::try_from(read_u32(section, SIZE_RAW_DATA_OFFSET)?).ok()?;
        let raw_off = read_u32(section, PTR_RAW_DATA_OFFSET)?;

        return Some(ExtractedSection {
            virtual_base_addr: image_base.virtual_base_addr(read_u32(section, VIRTUAL_ADDRESS_OFFSET)?),
            raw_base_addr: u64::from(raw_off),
            data: read_bytes(data, usize::try_from(raw_off).ok()?, raw_size)?,
        });
    }

    None
}

pub struct PePatternScanner;

impl_section_scanner!(PePatternScanner, ".text");
