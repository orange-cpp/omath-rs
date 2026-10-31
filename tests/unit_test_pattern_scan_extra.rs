use omath::consteval_pattern;
use omath::utility::PatternScanner;
use std::collections::VecDeque;

#[test]
fn iterator_scan_found() {
    let buf = [0xDE, 0xAD, 0xBE, 0xEF, 0x00];
    assert_eq!(
        PatternScanner::scan_for_pattern(&buf, "DE AD BE EF"),
        Some(0)
    );
}

#[test]
fn consteval_iterator_scan() {
    let buf = [0x00, 0xDE, 0xAD, 0xBE, 0xEF, 0x11];
    assert_eq!(
        PatternScanner::scan_for_parsed_pattern(&buf, &consteval_pattern!("DE ?? BE EF")),
        Some(1)
    );
}

#[test]
fn iterator_scan_not_found() {
    let buf = [0x00, 0x11, 0x22];
    assert_eq!(PatternScanner::scan_for_pattern(&buf, "FF EE DD"), None);
}

#[test]
fn parse_invalid_pattern() {
    // Invalid hex token should cause the public scan to report no match
    let buf = [0x00, 0x11];
    assert_eq!(PatternScanner::scan_for_pattern(&buf, "GG HH"), None);
}

#[test]
fn finds_match_at_every_offset() {
    const BUFFER_SIZE: usize = 100;
    const SIGNATURE: [u8; 4] = [0xDE, 0xAD, 0x11, 0xEF];

    for offset in 0..=BUFFER_SIZE - SIGNATURE.len() {
        let mut buf = vec![0u8; BUFFER_SIZE];
        buf[offset..offset + SIGNATURE.len()].copy_from_slice(&SIGNATURE);

        assert_eq!(
            PatternScanner::scan_for_pattern(&buf, "DE AD ? EF"),
            Some(offset)
        );
    }
}

#[test]
fn partial_matches_are_skipped() {
    // Both ends of the pattern are everywhere in the buffer, only its middle tells the match apart
    let mut buf = vec![0xAAu8; 200];
    buf[150] = 0xBB;

    assert_eq!(
        PatternScanner::scan_for_pattern(&buf, "AA AA BB AA"),
        Some(148)
    );
}

#[test]
fn same_first_and_last_byte() {
    // The buffer is a run of the value the pattern starts and ends with, only its middle tells the match apart
    let mut buf = vec![0xDEu8; 160];
    buf[66] = 0xAD;

    assert_eq!(
        PatternScanner::scan_for_pattern(&buf, "DE ? AD ? DE"),
        Some(64)
    );
}

#[test]
fn returns_first_of_several_matches() {
    let mut buf = vec![0u8; 128];
    for offset in [70, 72, 90] {
        buf[offset] = 0xDE;
        buf[offset + 1] = 0xAD;
    }

    assert_eq!(PatternScanner::scan_for_pattern(&buf, "DE AD"), Some(70));
}

#[test]
fn surrounding_wildcards_need_room() {
    let mut buf = vec![0u8; 64];
    // One byte before it instead of two
    buf[1] = 0xDE;
    buf[2] = 0xAD;
    // No byte after it
    buf[62] = 0xDE;
    buf[63] = 0xAD;

    assert_eq!(PatternScanner::scan_for_pattern(&buf, "? ? DE AD ?"), None);

    buf[40] = 0xDE;
    buf[41] = 0xAD;

    assert_eq!(
        PatternScanner::scan_for_pattern(&buf, "? ? DE AD ?"),
        Some(38)
    );
}

#[test]
fn only_wildcards() {
    let buf = [0u8; 8];

    assert_eq!(PatternScanner::scan_for_pattern(&buf, "? ?? ?"), Some(0));
    assert_eq!(PatternScanner::scan_for_pattern(&buf[..2], "? ?? ?"), None);
}

#[test]
fn non_contiguous_range() {
    // The scanner takes a slice, so a range that is not contiguous has to be made so first
    let mut buf = VecDeque::from(vec![0u8; 100]);
    buf.rotate_left(30);
    buf[77] = 0xDE;
    buf[79] = 0xEF;

    assert_eq!(
        PatternScanner::scan_for_pattern(buf.make_contiguous(), "DE ? EF"),
        Some(77)
    );
}

// Not in the C++ file

#[test]
fn empty_pattern_matches_start_of_non_empty_range() {
    assert_eq!(PatternScanner::scan_for_pattern(&[0x00], ""), Some(0));
}

#[test]
fn empty_pattern_does_not_match_empty_range() {
    assert_eq!(PatternScanner::scan_for_pattern(&[], ""), None);
    assert_eq!(PatternScanner::scan_for_pattern(&[], "AA"), None);
}

#[test]
fn pattern_as_long_as_range_matches() {
    assert_eq!(
        PatternScanner::scan_for_pattern(&[0xAA, 0xBB], "AA ?"),
        Some(0)
    );
}
