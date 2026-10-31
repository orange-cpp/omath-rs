mod image_support;

use image_support::{MACHO64_HDR_SIZE, MACHO64_SECT_SIZE, MACHO64_SEG_SIZE, TempFile, make_macho64_with_text_section};
use omath::consteval_pattern;
use omath::utility::MachOPatternScanner;

const DEFAULT_SECTION: &str = MachOPatternScanner::DEFAULT_SECTION_NAME;

#[test]
fn finds_pattern() {
    let buf = make_macho64_with_text_section(&[0x55, 0x48, 0x89, 0xE5, 0xC3]);

    let result = MachOPatternScanner::scan_for_pattern_in_memory_file(&buf, "55 48 89 E5", DEFAULT_SECTION).unwrap();

    assert_eq!(result.target_offset, 0);
}

#[test]
fn finds_pattern_with_wildcard() {
    let buf = make_macho64_with_text_section(&[0xDE, 0xAD, 0xBE, 0xEF]);

    let result = MachOPatternScanner::scan_for_pattern_in_memory_file(&buf, "DE ?? BE EF", DEFAULT_SECTION).unwrap();

    assert_eq!(result.target_offset, 0);
}

#[test]
fn consteval_finds_pattern_with_wildcard() {
    let buf = make_macho64_with_text_section(&[0x00, 0xDE, 0xAD, 0xBE, 0xEF]);

    let result = MachOPatternScanner::scan_for_parsed_pattern_in_memory_file(
        &buf,
        &consteval_pattern!("DE ?? BE EF"),
        DEFAULT_SECTION,
    )
    .unwrap();

    assert_eq!(result.target_offset, 1);
}

#[test]
fn pattern_not_found_returns_nullopt() {
    let buf = make_macho64_with_text_section(&[0x01, 0x02, 0x03]);

    assert_eq!(MachOPatternScanner::scan_for_pattern_in_memory_file(&buf, "AA BB CC", DEFAULT_SECTION), None);
}

#[test]
fn invalid_data_returns_nullopt() {
    let garbage = [0xFFu8; 64];

    assert_eq!(MachOPatternScanner::scan_for_pattern_in_memory_file(&garbage, "FF FF", DEFAULT_SECTION), None);
}

#[test]
fn empty_data_returns_nullopt() {
    assert_eq!(MachOPatternScanner::scan_for_pattern_in_memory_file(&[], "FF", DEFAULT_SECTION), None);
}

#[test]
fn raw_addr_and_virtual_addr_correct() {
    let buf = make_macho64_with_text_section(&[0xCA, 0xFE, 0xBA, 0xBE]);

    const EXPECTED_RAW_OFF: usize = MACHO64_HDR_SIZE + MACHO64_SEG_SIZE + MACHO64_SECT_SIZE;
    let result = MachOPatternScanner::scan_for_pattern_in_memory_file(&buf, "CA FE BA BE", DEFAULT_SECTION).unwrap();

    assert_eq!(result.raw_base_addr, EXPECTED_RAW_OFF as u64);
    assert_eq!(result.virtual_base_addr, 0x1000);
}

#[test]
fn consteval_file_scan_finds_pattern() {
    let buf = make_macho64_with_text_section(&[0x48, 0x89, 0xE5, 0xDE, 0xAD]);
    let file = TempFile::create(&buf);

    let result = MachOPatternScanner::scan_for_parsed_pattern_in_file(
        file.path(),
        &consteval_pattern!("48 ?? E5"),
        DEFAULT_SECTION,
    )
    .unwrap();

    assert_eq!(result.target_offset, 0);
}
