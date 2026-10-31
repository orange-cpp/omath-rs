mod image_support;

use image_support::{TempFile, build_minimal_pe};
use omath::utility::PePatternScanner;

#[test]
fn scan_finds_pattern() {
    let f = TempFile::create(&build_minimal_pe(&[0x55, 0x8B, 0xEC, 0x90, 0x90]));

    assert!(PePatternScanner::scan_for_pattern_in_file(f.path(), "55 8B EC", ".text").is_some());
}

#[test]
fn scan_missing_pattern() {
    let f = TempFile::create(&build_minimal_pe(&[0x00, 0x01, 0x02, 0x03]));

    assert_eq!(
        PePatternScanner::scan_for_pattern_in_file(f.path(), "FF EE DD", ".text"),
        None
    );
}
