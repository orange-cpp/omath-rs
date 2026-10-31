use omath::consteval_pattern;
use omath::utility::{PatternScanError, PatternScanner};

fn assert_ff_wild_wild_e9(pattern: &str) {
    let result = PatternScanner::parse_pattern(pattern).unwrap();

    assert_eq!(result[0], Some(0xFF));
    assert_eq!(result[1], None);
    assert_eq!(result[2], None);
    assert_eq!(result[3], Some(0xE9));
}

#[test]
fn read_test() {
    assert_ff_wild_wild_e9("FF ? ?? E9");
}

#[test]
fn corner_case_1() {
    assert_ff_wild_wild_e9("     FF ? ?? E9");
}

#[test]
fn corner_case_2() {
    assert_ff_wild_wild_e9("     FF ? ?? E9       ");
}

#[test]
fn corner_case_3() {
    assert_ff_wild_wild_e9("     FF ?       ??      E9       ");
}

#[test]
fn corner_case_4() {
    let result = PatternScanner::parse_pattern("X ? ?? E9 ");
    assert_eq!(result, Err(PatternScanError::InvalidPatternString));
}

#[test]
fn consteval_read_test() {
    const RESULT: [Option<u8>; 4] = consteval_pattern!("FF ? ?? E9");

    const _: () = assert!(RESULT.len() == 4);
    const _: () = assert!(matches!(RESULT[0], Some(0xFF)));
    const _: () = assert!(RESULT[1].is_none());
    const _: () = assert!(RESULT[2].is_none());
    const _: () = assert!(matches!(RESULT[3], Some(0xE9)));

    assert_eq!(RESULT[0], Some(0xFF));
    assert_eq!(RESULT[1], None);
    assert_eq!(RESULT[2], None);
    assert_eq!(RESULT[3], Some(0xE9));
}

#[test]
fn consteval_spacing_and_case() {
    const RESULT: [Option<u8>; 5] = consteval_pattern!(" \tde\nAD\r?? ? ef ");

    const _: () = assert!(RESULT.len() == 5);
    const _: () = assert!(matches!(RESULT[0], Some(0xDE)));
    const _: () = assert!(matches!(RESULT[1], Some(0xAD)));
    const _: () = assert!(RESULT[2].is_none());
    const _: () = assert!(RESULT[3].is_none());
    const _: () = assert!(matches!(RESULT[4], Some(0xEF)));

    assert_eq!(RESULT[0], Some(0xDE));
    assert_eq!(RESULT[1], Some(0xAD));
    assert_eq!(RESULT[2], None);
    assert_eq!(RESULT[3], None);
    assert_eq!(RESULT[4], Some(0xEF));
}

// Not in the C++ file: behaviours of the two parsers that its tests leave out

#[test]
fn lenient_parser_skips_long_tokens() {
    assert_eq!(
        PatternScanner::parse_pattern("FFF AA ???"),
        Ok(vec![Some(0xAA)])
    );
}

#[test]
fn lenient_parser_reads_hex_prefix_of_token() {
    assert_eq!(
        PatternScanner::parse_pattern("A 1G F?"),
        Ok(vec![Some(0x0A), Some(0x01), Some(0x0F)])
    );
    assert_eq!(
        PatternScanner::parse_pattern("G1"),
        Err(PatternScanError::InvalidPatternString)
    );
    assert_eq!(
        PatternScanner::parse_pattern("?A"),
        Err(PatternScanError::InvalidPatternString)
    );
    assert_eq!(
        PatternScanner::parse_pattern("+1"),
        Err(PatternScanError::InvalidPatternString)
    );
}

#[test]
fn lenient_parser_splits_on_space_only() {
    assert_eq!(
        PatternScanner::parse_pattern("\tF"),
        Err(PatternScanError::InvalidPatternString)
    );
    assert_eq!(PatternScanner::parse_pattern("AA\tBB"), Ok(vec![]));
}

#[test]
fn empty_pattern_parses_to_nothing() {
    assert_eq!(PatternScanner::parse_pattern(""), Ok(vec![]));
    assert_eq!(PatternScanner::parse_pattern("    "), Ok(vec![]));
}

#[test]
fn strict_parser_rejects_what_the_lenient_one_accepts() {
    assert_eq!(
        PatternScanner::parse_pattern_const::<1>("A"),
        Err(PatternScanError::InvalidPatternString)
    );
    assert_eq!(
        PatternScanner::parse_pattern_const::<1>("1G"),
        Err(PatternScanError::InvalidPatternString)
    );
    assert_eq!(
        PatternScanner::parse_pattern_const::<1>("FFF"),
        Err(PatternScanError::InvalidPatternString)
    );
    assert_eq!(
        PatternScanner::parse_pattern_const::<1>("?A"),
        Err(PatternScanError::InvalidPatternString)
    );
}

#[test]
fn strict_parser_rejects_wrong_array_size() {
    assert_eq!(
        PatternScanner::parse_pattern_const::<1>("AA BB"),
        Err(PatternScanError::InvalidPatternString)
    );
    assert_eq!(
        PatternScanner::parse_pattern_const::<3>("AA BB"),
        Err(PatternScanError::InvalidPatternString)
    );
}
