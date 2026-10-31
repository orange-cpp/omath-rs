#![cfg(feature = "lua")]

mod lua_support;

const SCRIPT: &str = "pe_scanner_tests.lua";

#[test]
fn pattern_scanner_finds_exact_pattern() {
    lua_support::check(SCRIPT, "PatternScanner_FindsExactPattern");
}

#[test]
fn pattern_scanner_finds_pattern_at_offset() {
    lua_support::check(SCRIPT, "PatternScanner_FindsPatternAtNonZeroOffset");
}

#[test]
fn pattern_scanner_wildcard_matches() {
    lua_support::check(SCRIPT, "PatternScanner_WildcardMatches");
}

#[test]
fn pattern_scanner_returns_nil_when_not_found() {
    lua_support::check(SCRIPT, "PatternScanner_ReturnsNilWhenNotFound");
}

#[test]
fn pattern_scanner_returns_nil_for_empty_buffer() {
    lua_support::check(SCRIPT, "PatternScanner_ReturnsNilForEmptyBuffer");
}

#[test]
fn section_scan_result_type_is_registered() {
    lua_support::check(SCRIPT, "SectionScanResult_TypeIsRegistered");
}
