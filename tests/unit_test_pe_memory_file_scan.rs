mod image_support;

use image_support::{TempFile, make_fake_pe_file};
use omath::consteval_pattern;
use omath::utility::PePatternScanner;

const DEFAULT_SECTION: &str = PePatternScanner::DEFAULT_SECTION_NAME;

#[test]
fn finds_pattern() {
    let code = [0x90, 0x01, 0x02, 0x03, 0x04];
    let buf = make_fake_pe_file(0x1000, 0x400, code.len() as u32, &code);

    let result =
        PePatternScanner::scan_for_pattern_in_memory_file(&buf, "90 01 02", DEFAULT_SECTION)
            .unwrap();

    assert_eq!(result.target_offset, 0);
    assert_eq!(result.raw_base_addr, 0x400);
}

#[test]
fn finds_pattern_with_wildcard() {
    let code = [0xDE, 0xAD, 0xBE, 0xEF];
    let buf = make_fake_pe_file(0x2000, 0x600, code.len() as u32, &code);

    let result =
        PePatternScanner::scan_for_pattern_in_memory_file(&buf, "DE ?? BE EF", DEFAULT_SECTION)
            .unwrap();

    assert_eq!(result.target_offset, 0);
}

#[test]
fn consteval_finds_pattern_with_wildcard() {
    let code = [0x00, 0xDE, 0xAD, 0xBE, 0xEF];
    let buf = make_fake_pe_file(0x2000, 0x600, code.len() as u32, &code);

    let result = PePatternScanner::scan_for_parsed_pattern_in_memory_file(
        &buf,
        &consteval_pattern!("DE ?? BE EF"),
        DEFAULT_SECTION,
    )
    .unwrap();

    assert_eq!(result.target_offset, 1);
}

#[test]
fn pattern_not_found_returns_nullopt() {
    let code = [0x01, 0x02, 0x03];
    let buf = make_fake_pe_file(0x1000, 0x400, code.len() as u32, &code);

    assert_eq!(
        PePatternScanner::scan_for_pattern_in_memory_file(&buf, "AA BB CC", DEFAULT_SECTION),
        None
    );
}

#[test]
fn invalid_data_returns_nullopt() {
    let garbage = [0xFFu8; 128];

    assert_eq!(
        PePatternScanner::scan_for_pattern_in_memory_file(&garbage, "FF FF", DEFAULT_SECTION),
        None
    );
}

#[test]
fn empty_data_returns_nullopt() {
    assert_eq!(
        PePatternScanner::scan_for_pattern_in_memory_file(&[], "FF", DEFAULT_SECTION),
        None
    );
}

#[test]
fn raw_addr_differs_from_virtual_address() {
    // ptr_raw_data = 0x600, virtual_address = 0x3000: different intentionally
    let code = [0xCA, 0xFE, 0xBA, 0xBE];
    let buf = make_fake_pe_file(0x3000, 0x600, code.len() as u32, &code);

    let result =
        PePatternScanner::scan_for_pattern_in_memory_file(&buf, "CA FE BA BE", DEFAULT_SECTION)
            .unwrap();

    // raw_base_addr should be ptr_raw_data, not virtual_address
    assert_eq!(result.raw_base_addr, 0x600);
    // virtual_base_addr = virtual_address + image_base (image_base = 0)
    assert_eq!(result.virtual_base_addr, 0x3000);
}

#[test]
fn consteval_file_scan_finds_pattern() {
    let code = [0x48, 0x89, 0xE5, 0xDE, 0xAD];
    let buf = make_fake_pe_file(0x2000, 0x600, code.len() as u32, &code);
    let file = TempFile::create(&buf);

    let result = PePatternScanner::scan_for_parsed_pattern_in_file(
        file.path(),
        &consteval_pattern!("48 ?? E5"),
        DEFAULT_SECTION,
    )
    .unwrap();

    assert_eq!(result.target_offset, 0);
}
