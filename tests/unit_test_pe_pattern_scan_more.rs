mod image_support;

use image_support::{TempFile, build_minimal_pe_with_section, w32};
use omath::utility::PePatternScanner;

#[test]
fn invalid_dos_header() {
    let mut data = vec![0u8; 128];
    data[0] = b'N';
    data[1] = b'Z';
    let f = TempFile::create(&data);

    assert_eq!(
        PePatternScanner::scan_for_pattern_in_file(f.path(), "55 8B EC", ".text"),
        None
    );
}

#[test]
fn invalid_nt_signature() {
    const E_LFANEW: usize = 0x80;

    let mut data = vec![0u8; 256];
    data[0] = b'M';
    data[1] = b'Z';
    w32(&mut data, 0x3C, E_LFANEW as u32);
    data[E_LFANEW..E_LFANEW + 4].copy_from_slice(b"XYZW");
    let f = TempFile::create(&data);

    assert_eq!(
        PePatternScanner::scan_for_pattern_in_file(f.path(), "55 8B EC", ".text"),
        None
    );
}

#[test]
fn section_not_found() {
    // Minimal PE with a .data section; scanning for .text should fail
    let buf = build_minimal_pe_with_section(&[0x00, 0x01, 0x02, 0x03], b".data\0\0\0");
    let f = TempFile::create(&buf);

    assert_eq!(
        PePatternScanner::scan_for_pattern_in_file(f.path(), "00 01", ".text"),
        None
    );
}
