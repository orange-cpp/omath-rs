use omath::utility::PePatternScanner;

#[test]
fn missing_file_returns_null() {
    let res = PePatternScanner::scan_for_pattern_in_file(
        "/non/existent/file.exe",
        "55 8B EC",
        PePatternScanner::DEFAULT_SECTION_NAME,
    );

    assert_eq!(res, None);
}
