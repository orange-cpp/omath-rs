//! Pattern scanning in ELF images held in a file or a byte slice.

use super::byte_reader::{read_bytes, read_u16, read_u32, read_u32_as_u64, read_u64, until_nul};
use super::pattern_scan::PatternScanner;
use super::section_scan_result::{ExtractedSection, SectionScanResult, impl_section_scanner};

const ELF_SIGNATURE: &[u8] = b"\x7FELF";
const EI_NIDENT: usize = 16;
const EI_CLASS: usize = 4;
const ELFCLASS32: u8 = 1;
const ELFCLASS64: u8 = 2;

/// Where the two ELF classes keep the fields the scanner reads.
struct ElfLayout {
    file_header_size: usize,
    e_shoff: usize,
    e_shnum: usize,
    e_shstrndx: usize,
    section_header_size: usize,
    sh_addr: usize,
    sh_offset: usize,
    sh_size: usize,
    /// Reads a field that is as wide as an address in this class.
    read_word: fn(&[u8], usize) -> Option<u64>,
}

const ELF32_LAYOUT: ElfLayout = ElfLayout {
    file_header_size: 52,
    e_shoff: 32,
    e_shnum: 48,
    e_shstrndx: 50,
    section_header_size: 40,
    sh_addr: 12,
    sh_offset: 16,
    sh_size: 20,
    read_word: read_u32_as_u64,
};

const ELF64_LAYOUT: ElfLayout = ElfLayout {
    file_header_size: 64,
    e_shoff: 40,
    e_shnum: 60,
    e_shstrndx: 62,
    section_header_size: 64,
    sh_addr: 16,
    sh_offset: 24,
    sh_size: 32,
    read_word: read_u64,
};

fn extract_section_with_layout<'a>(
    data: &'a [u8],
    section_name: &str,
    layout: &ElfLayout,
) -> Option<ExtractedSection<'a>> {
    if data.len() < layout.file_header_size {
        return None;
    }

    let read_offset =
        |bytes: &[u8], offset: usize| usize::try_from((layout.read_word)(bytes, offset)?).ok();

    let shoff = read_offset(data, layout.e_shoff)?;
    let shnum = usize::from(read_u16(data, layout.e_shnum)?);
    let shstrndx = usize::from(read_u16(data, layout.e_shstrndx)?);

    let section_header = |index: usize| {
        let offset = index
            .checked_mul(layout.section_header_size)?
            .checked_add(shoff)?;

        read_bytes(data, offset, layout.section_header_size)
    };

    let shstrtab_hdr = section_header(shstrndx)?;
    let shstrtab = read_bytes(
        data,
        read_offset(shstrtab_hdr, layout.sh_offset)?,
        read_offset(shstrtab_hdr, layout.sh_size)?,
    )?;

    for index in 0..shnum {
        let section = section_header(index)?;
        let sh_name = usize::try_from(read_u32(section, 0)?).ok()?;

        if sh_name >= shstrtab.len() || until_nul(&shstrtab[sh_name..]) != section_name.as_bytes() {
            continue;
        }

        let raw_off = (layout.read_word)(section, layout.sh_offset)?;

        return Some(ExtractedSection {
            virtual_base_addr: (layout.read_word)(section, layout.sh_addr)?,
            raw_base_addr: raw_off,
            data: read_bytes(
                data,
                usize::try_from(raw_off).ok()?,
                read_offset(section, layout.sh_size)?,
            )?,
        });
    }

    None
}

pub(super) fn extract_section<'a>(
    data: &'a [u8],
    section_name: &str,
) -> Option<ExtractedSection<'a>> {
    if data.len() < EI_NIDENT || !data.starts_with(ELF_SIGNATURE) {
        return None;
    }

    match data[EI_CLASS] {
        ELFCLASS64 => extract_section_with_layout(data, section_name, &ELF64_LAYOUT),
        ELFCLASS32 => extract_section_with_layout(data, section_name, &ELF32_LAYOUT),
        _ => None,
    }
}

pub struct ElfPatternScanner;

impl_section_scanner!(ElfPatternScanner, ".text");
