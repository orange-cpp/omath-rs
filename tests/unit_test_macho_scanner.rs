mod image_support;

use image_support::{TempFile, build_minimal_macho32, build_minimal_macho64};
use omath::utility::MachOPatternScanner;

#[test]
fn scan_finds_pattern64() {
    let f = TempFile::create(&build_minimal_macho64(&[0x55, 0x48, 0x89, 0xE5, 0x90, 0x90]));

    let res = MachOPatternScanner::scan_for_pattern_in_file(f.path(), "55 48 89 E5", "__text").unwrap();

    assert_eq!(res.target_offset, 0);
}

#[test]
fn scan_finds_pattern32() {
    let f = TempFile::create(&build_minimal_macho32(&[0x55, 0x89, 0xE5, 0x90, 0x90]));

    let res = MachOPatternScanner::scan_for_pattern_in_file(f.path(), "55 89 E5", "__text").unwrap();

    assert_eq!(res.target_offset, 0);
}

#[test]
fn scan_missing_pattern() {
    let f = TempFile::create(&build_minimal_macho64(&[0x00, 0x01, 0x02, 0x03]));

    assert_eq!(MachOPatternScanner::scan_for_pattern_in_file(f.path(), "FF EE DD", "__text"), None);
}

#[test]
fn scan_pattern_at_offset() {
    let f = TempFile::create(&build_minimal_macho64(&[0x90, 0x90, 0x90, 0x55, 0x48, 0x89, 0xE5]));

    let res = MachOPatternScanner::scan_for_pattern_in_file(f.path(), "55 48 89 E5", "__text").unwrap();

    assert_eq!(res.target_offset, 3);
}

#[test]
fn scan_with_wildcard() {
    let f = TempFile::create(&build_minimal_macho64(&[0x55, 0x48, 0x89, 0xE5, 0x90]));

    assert!(MachOPatternScanner::scan_for_pattern_in_file(f.path(), "55 ? 89 E5", "__text").is_some());
}

#[test]
fn scan_non_existent_file() {
    assert_eq!(MachOPatternScanner::scan_for_pattern_in_file("/non/existent/file.bin", "55 48", "__text"), None);
}

#[test]
fn scan_invalid_file() {
    let f = TempFile::create(&[0x00, 0x01, 0x02, 0x03, 0x04, 0x05]);

    assert_eq!(MachOPatternScanner::scan_for_pattern_in_file(f.path(), "55 48", "__text"), None);
}

#[test]
fn scan_non_existent_section() {
    let f = TempFile::create(&build_minimal_macho64(&[0x55, 0x48, 0x89, 0xE5]));

    assert_eq!(MachOPatternScanner::scan_for_pattern_in_file(f.path(), "55 48", "__nonexistent"), None);
}
