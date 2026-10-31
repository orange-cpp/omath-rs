mod image_support;

use image_support::{PE_DATA_OFF, TempFile, build_minimal_pe, w16, w32};
use omath::utility::PePatternScanner;

#[test]
fn file_x86_optional_header_scan_finds_pattern() {
    let f = TempFile::create(&build_minimal_pe(&[0xDE, 0xAD, 0xBE, 0xEF]));

    let res = PePatternScanner::scan_for_pattern_in_file(f.path(), "DE AD BE EF", ".text").unwrap();

    // C++ checks these are >= 0, which an unsigned value always is; the exact values are known here
    assert_eq!(res.virtual_base_addr, 0x1000);
    assert_eq!(res.raw_base_addr, PE_DATA_OFF as u64);
    assert_eq!(res.target_offset, 0);
}

#[test]
fn file_pattern_not_found_returns_null() {
    const E_LFANEW: usize = 0x80;
    const SIZE_OPTIONAL_HEADER: u16 = 0xE0;
    const FILE_HEADER_SIZE: usize = 20;
    const OFFSET_TO_SEGMENT_TABLE: usize = E_LFANEW + 4 + FILE_HEADER_SIZE + SIZE_OPTIONAL_HEADER as usize;

    let mut data = vec![0u8; 512];
    data[0] = b'M';
    data[1] = b'Z';
    w32(&mut data, 0x3C, E_LFANEW as u32);
    data[E_LFANEW] = b'P';
    data[E_LFANEW + 1] = b'E';

    w16(&mut data, E_LFANEW + 6, 1); // num_sections
    w16(&mut data, E_LFANEW + 4 + 16, SIZE_OPTIONAL_HEADER);
    w16(&mut data, E_LFANEW + 4 + FILE_HEADER_SIZE, 0x020B); // magic

    data[OFFSET_TO_SEGMENT_TABLE..OFFSET_TO_SEGMENT_TABLE + 8].copy_from_slice(b".text\0\0\0");
    w32(&mut data, OFFSET_TO_SEGMENT_TABLE + 8, 4); // virtual_size
    w32(&mut data, OFFSET_TO_SEGMENT_TABLE + 12, 0x1000); // virtual_address
    w32(&mut data, OFFSET_TO_SEGMENT_TABLE + 16, 4); // size_raw_data
    w32(&mut data, OFFSET_TO_SEGMENT_TABLE + 20, 0x200); // ptr_raw_data

    let f = TempFile::create(&data);

    assert_eq!(PePatternScanner::scan_for_pattern_in_file(f.path(), "AA BB CC", ".text"), None);
}

#[test]
fn pattern_at_start_found() {
    let f = TempFile::create(&build_minimal_pe(&[0x90, 0x01, 0x02, 0x03, 0x04]));

    assert!(PePatternScanner::scan_for_pattern_in_file(f.path(), "90 01 02", ".text").is_some());
}

#[test]
fn pattern_at_end_found() {
    let f = TempFile::create(&build_minimal_pe(&[0x00, 0x11, 0x22, 0x33, 0x44]));

    assert!(PePatternScanner::scan_for_pattern_in_file(f.path(), "22 33 44", ".text").is_some());
}

#[test]
fn wildcard_matches() {
    let f = TempFile::create(&build_minimal_pe(&[0xDE, 0xAD, 0xBE, 0xEF]));

    assert!(PePatternScanner::scan_for_pattern_in_file(f.path(), "DE ?? BE", ".text").is_some());
}

#[test]
fn pattern_longer_than_buffer() {
    let f = TempFile::create(&build_minimal_pe(&[0xAA, 0xBB]));

    assert_eq!(PePatternScanner::scan_for_pattern_in_file(f.path(), "AA BB CC", ".text"), None);
}

#[test]
fn invalid_pattern_parse() {
    let f = TempFile::create(&build_minimal_pe(&[0x01, 0x02, 0x03]));

    assert_eq!(PePatternScanner::scan_for_pattern_in_file(f.path(), "01 GG 03", ".text"), None);
}
