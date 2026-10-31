mod image_support;

use image_support::{TempFile, make_elf64_with_text_section};
use omath::consteval_pattern;
use omath::utility::ElfPatternScanner;

#[test]
fn finds_pattern() {
    let buf = make_elf64_with_text_section(&[0x55, 0x48, 0x89, 0xE5, 0xC3]);

    let result =
        ElfPatternScanner::scan_for_pattern_in_memory_file(&buf, "55 48 89 E5", ".text").unwrap();

    assert_eq!(result.target_offset, 0);
}

#[test]
fn finds_pattern_with_wildcard() {
    let buf = make_elf64_with_text_section(&[0xDE, 0xAD, 0xBE, 0xEF, 0x00]);

    let result =
        ElfPatternScanner::scan_for_pattern_in_memory_file(&buf, "DE ?? BE EF", ".text").unwrap();

    assert_eq!(result.target_offset, 0);
}

#[test]
fn consteval_finds_pattern_with_wildcard() {
    let buf = make_elf64_with_text_section(&[0x00, 0xDE, 0xAD, 0xBE, 0xEF]);

    let result = ElfPatternScanner::scan_for_parsed_pattern_in_memory_file(
        &buf,
        &consteval_pattern!("DE ?? BE EF"),
        ".text",
    )
    .unwrap();

    assert_eq!(result.target_offset, 1);
}

#[test]
fn pattern_not_found_returns_nullopt() {
    let buf = make_elf64_with_text_section(&[0x01, 0x02, 0x03, 0x04]);

    assert_eq!(
        ElfPatternScanner::scan_for_pattern_in_memory_file(&buf, "AA BB CC", ".text"),
        None
    );
}

#[test]
fn invalid_data_returns_nullopt() {
    let garbage = [0xFFu8; 64];

    assert_eq!(
        ElfPatternScanner::scan_for_pattern_in_memory_file(&garbage, "FF FF", ".text"),
        None
    );
}

#[test]
fn empty_data_returns_nullopt() {
    assert_eq!(
        ElfPatternScanner::scan_for_pattern_in_memory_file(&[], "FF", ".text"),
        None
    );
}

#[test]
fn missing_section_returns_nullopt() {
    let buf = make_elf64_with_text_section(&[0x90, 0x90]);

    assert_eq!(
        ElfPatternScanner::scan_for_pattern_in_memory_file(&buf, "90 90", ".nonexistent"),
        None
    );
}

#[test]
fn matches_file_scan() {
    // Write our synthetic ELF to a temp file and verify memory scan == file scan
    let buf = make_elf64_with_text_section(&[0x48, 0x89, 0xE5, 0xDE, 0xAD, 0xBE, 0xEF, 0x00]);
    let file = TempFile::create(&buf);

    let file_result =
        ElfPatternScanner::scan_for_pattern_in_file(file.path(), "48 89 E5 DE AD", ".text")
            .unwrap();
    let mem_result =
        ElfPatternScanner::scan_for_pattern_in_memory_file(&buf, "48 89 E5 DE AD", ".text")
            .unwrap();

    assert_eq!(file_result.virtual_base_addr, mem_result.virtual_base_addr);
    assert_eq!(file_result.raw_base_addr, mem_result.raw_base_addr);
    assert_eq!(file_result.target_offset, mem_result.target_offset);
}

#[test]
fn consteval_file_scan_finds_pattern() {
    let buf = make_elf64_with_text_section(&[0x48, 0x89, 0xE5, 0xDE, 0xAD]);
    let file = TempFile::create(&buf);

    let result = ElfPatternScanner::scan_for_parsed_pattern_in_file(
        file.path(),
        &consteval_pattern!("48 ?? E5"),
        ".text",
    )
    .unwrap();

    assert_eq!(result.target_offset, 0);
}
