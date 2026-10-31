//! Not a port of a C++ file: truncated and corrupted images must give `None`, never a panic, and
//! the layouts and C++ bugs the ported tests leave out.

mod image_support;

use image_support::*;
use omath::utility::{ElfPatternScanner, MachOPatternScanner, PePatternScanner};

const CODE: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];

fn assert_every_truncation_is_none(image: &[u8], scan: impl Fn(&[u8]) -> bool) {
    assert!(scan(image), "the untruncated image must scan");

    for size in 0..image.len() {
        assert!(
            !scan(&image[..size]),
            "image truncated to {size} bytes was accepted"
        );
    }
}

fn pe_scan(image: &[u8]) -> bool {
    PePatternScanner::scan_for_pattern_in_memory_file(image, "DE AD", ".text").is_some()
}

fn elf_scan(image: &[u8]) -> bool {
    ElfPatternScanner::scan_for_pattern_in_memory_file(image, "DE AD", ".text").is_some()
}

fn macho_scan(image: &[u8]) -> bool {
    MachOPatternScanner::scan_for_pattern_in_memory_file(image, "DE AD", "__text").is_some()
}

#[test]
fn truncated_pe_returns_none() {
    assert_every_truncation_is_none(&build_minimal_pe(&CODE), pe_scan);
}

#[test]
fn truncated_pe32_returns_none() {
    let mut image = build_minimal_pe(&CODE);
    w16(&mut image, PE_OH_OFF, 0x10B);

    assert_every_truncation_is_none(&image, pe_scan);
}

#[test]
fn truncated_elf64_returns_none() {
    assert_every_truncation_is_none(&make_elf64_with_text_section(&CODE), elf_scan);
}

#[test]
fn truncated_elf32_returns_none() {
    assert_every_truncation_is_none(&make_elf32_with_text_section(&CODE), elf_scan);
}

#[test]
fn truncated_macho64_returns_none() {
    assert_every_truncation_is_none(&build_minimal_macho64(&CODE), macho_scan);
}

#[test]
fn truncated_macho32_returns_none() {
    assert_every_truncation_is_none(&build_minimal_macho32(&CODE), macho_scan);
}

#[test]
fn truncated_file_returns_none() {
    let pe = build_minimal_pe(&CODE);
    let elf = make_elf64_with_text_section(&CODE);
    let macho = build_minimal_macho64(&CODE);

    let pe_file = TempFile::create(&pe[..pe.len() - 1]);
    let elf_file = TempFile::create(&elf[..elf.len() - 1]);
    let macho_file = TempFile::create(&macho[..macho.len() - 1]);

    assert_eq!(
        PePatternScanner::scan_for_pattern_in_file(pe_file.path(), "DE AD", ".text"),
        None
    );
    assert_eq!(
        ElfPatternScanner::scan_for_pattern_in_file(elf_file.path(), "DE AD", ".text"),
        None
    );
    assert_eq!(
        MachOPatternScanner::scan_for_pattern_in_file(macho_file.path(), "DE AD", "__text"),
        None
    );
}

#[test]
fn corrupted_bytes_do_not_panic() {
    type Scan = fn(&[u8]) -> bool;

    let images: [(Vec<u8>, Scan); 5] = [
        (build_minimal_pe(&CODE), pe_scan),
        (make_elf64_with_text_section(&CODE), elf_scan),
        (make_elf32_with_text_section(&CODE), elf_scan),
        (build_minimal_macho64(&CODE), macho_scan),
        (build_minimal_macho32(&CODE), macho_scan),
    ];

    for (image, scan) in images {
        for index in 0..image.len() {
            for value in [0x00, 0x01, 0x7F, 0x80, 0xFF] {
                let mut corrupted = image.to_vec();
                corrupted[index] = value;

                scan(&corrupted);
            }
        }
    }
}

#[test]
fn pe_e_lfanew_past_end_returns_none() {
    let mut image = build_minimal_pe(&CODE);
    w32(&mut image, 0x3C, u32::MAX);

    assert!(!pe_scan(&image));
}

// C++ file scans zero-fill what is missing, so "00" would match past the end of the file
#[test]
fn pe_section_past_end_of_file_returns_none() {
    let mut image = build_minimal_pe(&CODE);
    w32(&mut image, PE_SH_OFF + 16, 0x1000); // size_raw_data
    let f = TempFile::create(&image);

    assert_eq!(
        PePatternScanner::scan_for_pattern_in_file(f.path(), "00", ".text"),
        None
    );
}

#[test]
fn pe_image_base_wraps_at_image_width() {
    let mut image = build_minimal_pe(&CODE);
    w64(&mut image, PE_OH_OFF + 24, u64::MAX);

    let result =
        PePatternScanner::scan_for_pattern_in_memory_file(&image, "DE AD", ".text").unwrap();
    assert_eq!(result.virtual_base_addr, 0xFFF);

    let mut image = build_minimal_pe(&CODE);
    w16(&mut image, PE_OH_OFF, 0x10B);
    w32(&mut image, PE_OH_OFF + 28, 0xFFFF_F800);

    let result =
        PePatternScanner::scan_for_pattern_in_memory_file(&image, "DE AD", ".text").unwrap();
    assert_eq!(result.virtual_base_addr, 0x800);
}

#[test]
fn pe32_image_base_is_read_from_its_own_offset() {
    let mut image = build_minimal_pe(&CODE);
    w16(&mut image, PE_OH_OFF, 0x10B);
    w32(&mut image, PE_OH_OFF + 28, 0x0040_0000);

    let result =
        PePatternScanner::scan_for_pattern_in_memory_file(&image, "DE AD", ".text").unwrap();

    assert_eq!(result.virtual_base_addr, 0x0040_1000);
}

// C++ builds the name from a NUL terminated string, so it reads on into virtual_size
#[test]
fn pe_section_name_of_eight_characters() {
    let image = build_minimal_pe_with_section(&CODE, b".textbss");

    assert!(
        PePatternScanner::scan_for_pattern_in_memory_file(&image, "DE AD", ".textbss").is_some()
    );
    assert_eq!(
        PePatternScanner::scan_for_pattern_in_memory_file(&image, "DE AD", ".text"),
        None
    );
}

#[test]
fn elf_section_table_offset_overflow_returns_none() {
    let mut image = make_elf64_with_text_section(&CODE);
    w64(&mut image, 40, u64::MAX - 8);

    assert!(!elf_scan(&image));
}

#[test]
fn elf_string_table_range_overflow_returns_none() {
    let mut image = make_elf64_with_text_section(&CODE);
    let shstrtab_hdr = ELF_SHDR_TABLE_OFF + 2 * ELF_SHDR_SIZE;
    w64(&mut image, shstrtab_hdr + 24, u64::MAX);

    assert!(!elf_scan(&image));
}

#[test]
fn elf_section_range_overflow_returns_none() {
    let mut image = make_elf64_with_text_section(&CODE);
    let text_hdr = ELF_SHDR_TABLE_OFF + ELF_SHDR_SIZE;
    w64(&mut image, text_hdr + 24, u64::MAX); // sh_offset
    w64(&mut image, text_hdr + 32, 2); // sh_size, wraps to a valid range in C++

    assert!(!elf_scan(&image));
}

// C++ reads the name on past the end of the string table
#[test]
fn elf_section_name_is_bounded_by_string_table() {
    let mut image = make_elf64_with_text_section(&CODE);
    let shstrtab_hdr = ELF_SHDR_TABLE_OFF + 2 * ELF_SHDR_SIZE;
    w64(&mut image, shstrtab_hdr + 32, 6); // ends right after ".text", with no terminator

    assert!(elf_scan(&image));
}

#[test]
fn elf_section_name_offset_past_table_is_skipped() {
    let mut image = make_elf64_with_text_section(&CODE);
    let text_hdr = ELF_SHDR_TABLE_OFF + ELF_SHDR_SIZE;
    w32(&mut image, text_hdr, u32::MAX);

    assert!(!elf_scan(&image));
}

#[test]
fn elf32_finds_pattern() {
    let image = make_elf32_with_text_section(&CODE);

    let result =
        ElfPatternScanner::scan_for_pattern_in_memory_file(&image, "BE EF", ".text").unwrap();

    assert_eq!(result.target_offset, 2);
    assert_eq!(result.raw_base_addr, 0x34);
    assert_eq!(result.virtual_base_addr, 0x8000);
}

#[test]
fn empty_section_does_not_match_empty_pattern() {
    let image = build_minimal_pe(&[]);

    assert_eq!(
        PePatternScanner::scan_for_pattern_in_memory_file(&image, "", ".text"),
        None
    );
}

#[test]
fn macho_zero_cmdsize_returns_none() {
    // With ncmds this large, a loop that stays on the same command would take minutes
    for cmd in [LC_SEGMENT_64, 0x2] {
        let mut image = build_minimal_macho64(&CODE);
        w32(&mut image, 16, u32::MAX); // ncmds
        w32(&mut image, MACHO64_HDR_SIZE, cmd);
        w32(&mut image, MACHO64_HDR_SIZE + 4, 0); // cmdsize
        w32(&mut image, MACHO64_HDR_SIZE + 64, 0); // nsects

        assert!(!macho_scan(&image));
    }
}

#[test]
fn macho_huge_nsects_returns_none() {
    let mut image = build_minimal_macho64(&CODE);
    w32(&mut image, MACHO64_HDR_SIZE + 64, u32::MAX);

    assert_eq!(
        MachOPatternScanner::scan_for_pattern_in_memory_file(&image, "DE AD", "__nonexistent"),
        None
    );
}

#[test]
fn macho_section_range_overflow_returns_none() {
    let mut image = build_minimal_macho64(&CODE);
    let section = MACHO64_HDR_SIZE + MACHO64_SEG_SIZE;
    w64(&mut image, section + 40, u64::MAX); // size

    assert!(!macho_scan(&image));
}

// C++ file scans never move past a segment that has sections but not the wanted one
#[test]
fn macho_section_in_second_segment_is_found_in_file() {
    let image = make_macho64_with_two_segments();
    let f = TempFile::create(&image);

    let from_file =
        MachOPatternScanner::scan_for_pattern_in_file(f.path(), "DE AD", "__data").unwrap();
    let from_memory =
        MachOPatternScanner::scan_for_pattern_in_memory_file(&image, "DE AD", "__data").unwrap();

    assert_eq!(from_file, from_memory);
    assert_eq!(from_file.virtual_base_addr, 0x2000);
}

#[test]
fn macho_section_name_of_sixteen_characters() {
    let mut image = build_minimal_macho64(&CODE);
    let section = MACHO64_HDR_SIZE + MACHO64_SEG_SIZE;
    image[section..section + 16].copy_from_slice(b"__sixteen_chars_");

    assert!(
        MachOPatternScanner::scan_for_pattern_in_memory_file(&image, "DE AD", "__sixteen_chars_")
            .is_some()
    );
}

/// Minimal ELF32 file: header, .text at 0x34, "\0.text\0\0", then 3 section headers of 40 bytes.
fn make_elf32_with_text_section(code_bytes: &[u8]) -> Vec<u8> {
    const TEXT_OFF: usize = 0x34;
    const TEXT_SIZE: usize = 0x10;
    const SHSTRTAB_OFF: usize = TEXT_OFF + TEXT_SIZE;
    const SHSTRTAB_SIZE: usize = 8;
    const SHDR_TABLE_OFF: usize = SHSTRTAB_OFF + SHSTRTAB_SIZE;
    const SHDR_SIZE: usize = 40;

    let mut buf = vec![0u8; SHDR_TABLE_OFF + 3 * SHDR_SIZE];

    buf[..4].copy_from_slice(b"\x7FELF");
    buf[4] = 1; // ELFCLASS32
    buf[5] = 1; // ELFDATA2LSB
    w32(&mut buf, 32, SHDR_TABLE_OFF as u32); // e_shoff
    w16(&mut buf, 46, SHDR_SIZE as u16); // e_shentsize
    w16(&mut buf, 48, 3); // e_shnum
    w16(&mut buf, 50, 2); // e_shstrndx

    buf[TEXT_OFF..TEXT_OFF + code_bytes.len()].copy_from_slice(code_bytes);
    buf[SHSTRTAB_OFF + 1..SHSTRTAB_OFF + 6].copy_from_slice(b".text");

    let base = SHDR_TABLE_OFF + SHDR_SIZE;
    w32(&mut buf, base, 1); // sh_name
    w32(&mut buf, base + 12, 0x8000); // sh_addr
    w32(&mut buf, base + 16, TEXT_OFF as u32); // sh_offset
    w32(&mut buf, base + 20, TEXT_SIZE as u32); // sh_size

    let base = SHDR_TABLE_OFF + 2 * SHDR_SIZE;
    w32(&mut buf, base + 16, SHSTRTAB_OFF as u32); // sh_offset
    w32(&mut buf, base + 20, SHSTRTAB_SIZE as u32); // sh_size

    buf
}

/// Mach-O 64-bit file with `__text` in the first segment and `__data` in the second one.
fn make_macho64_with_two_segments() -> Vec<u8> {
    const CMD_SIZE: usize = MACHO64_SEG_SIZE + MACHO64_SECT_SIZE;
    const TEXT_RAW_OFF: usize = MACHO64_HDR_SIZE + 2 * CMD_SIZE;
    const DATA_RAW_OFF: usize = TEXT_RAW_OFF + 4;

    let mut buf = vec![0u8; DATA_RAW_OFF + 4];

    w32(&mut buf, 0, MH_MAGIC_64);
    w32(&mut buf, 16, 2); // ncmds
    w32(&mut buf, 20, 2 * CMD_SIZE as u32); // sizeofcmds

    for (index, (name, addr, raw_off)) in [
        (&b"__text"[..], 0x1000, TEXT_RAW_OFF),
        (b"__data", 0x2000, DATA_RAW_OFF),
    ]
    .into_iter()
    .enumerate()
    {
        let segment = MACHO64_HDR_SIZE + index * CMD_SIZE;
        w32(&mut buf, segment, LC_SEGMENT_64);
        w32(&mut buf, segment + 4, CMD_SIZE as u32);
        w32(&mut buf, segment + 64, 1); // nsects

        let section = segment + MACHO64_SEG_SIZE;
        buf[section..section + name.len()].copy_from_slice(name);
        w64(&mut buf, section + 32, addr);
        w64(&mut buf, section + 40, 4); // size
        w32(&mut buf, section + 48, raw_off as u32);
    }

    buf[DATA_RAW_OFF..].copy_from_slice(&CODE);
    buf
}
