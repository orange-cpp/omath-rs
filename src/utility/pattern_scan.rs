//! IDA-style byte pattern parsing and searching.

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PatternScanError {
    InvalidPatternString,
}

impl fmt::Display for PatternScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidPatternString => "pattern string is invalid",
        })
    }
}

impl std::error::Error for PatternScanError {}

const fn is_space(character: u8) -> bool {
    matches!(character, b' ' | b'\t' | b'\n' | b'\r')
}

const fn hex_value(character: u8) -> Option<u8> {
    match character {
        b'0'..=b'9' => Some(character - b'0'),
        b'A'..=b'F' => Some(character - b'A' + 10),
        b'a'..=b'f' => Some(character - b'a' + 10),
        _ => None,
    }
}

/// Same semantics as `std::from_chars` in base 16: reads the hex digits it can and ignores the rest.
fn parse_hex_prefix(token: &[u8]) -> Option<u8> {
    let high = hex_value(*token.first()?)?;

    Some(match token.get(1).copied().and_then(hex_value) {
        Some(low) => high << 4 | low,
        None => high,
    })
}

pub struct PatternScanner;

impl PatternScanner {
    /// Offset of the first match of `pattern` in `range`.
    ///
    /// `None` when nothing matches, and when `pattern` is invalid: use [`Self::parse_pattern`] to
    /// tell the two apart.
    pub fn scan_for_pattern(range: &[u8], pattern: &str) -> Option<usize> {
        Self::scan_for_parsed_pattern(range, &Self::parse_pattern(pattern).ok()?)
    }

    /// Offset of the first match of `pattern` in `range`.
    ///
    /// An empty pattern matches at offset 0 of a non-empty range. It does not match an empty range:
    /// C++ reports a match at the end of a range the same way as no match.
    pub fn scan_for_parsed_pattern(range: &[u8], pattern: &[Option<u8>]) -> Option<usize> {
        if range.is_empty() || pattern.len() > range.len() {
            return None;
        }
        if pattern.is_empty() {
            return Some(0);
        }

        range.windows(pattern.len()).position(|window| {
            window.iter().zip(pattern).all(|(actual, expected)| expected.is_none_or(|byte| byte == *actual))
        })
    }

    /// Parses space separated tokens, where `?` and `??` are wildcards and the rest are hex bytes.
    ///
    /// Same leniency as C++: only `' '` separates tokens, tokens of three or more bytes are skipped,
    /// a one digit token is a byte, and a token only has to start with a hex digit (`"1G"` is `0x01`).
    /// A token that does not is [`PatternScanError::InvalidPatternString`].
    pub fn parse_pattern(pattern_string: &str) -> Result<Vec<Option<u8>>, PatternScanError> {
        let mut pattern = Vec::new();

        for byte_str in pattern_string.as_bytes().split(|&character| character == b' ') {
            if byte_str.is_empty() || byte_str.len() >= 3 {
                continue;
            }
            if byte_str == b"?" || byte_str == b"??" {
                pattern.push(None);
                continue;
            }

            pattern.push(Some(parse_hex_prefix(byte_str).ok_or(PatternScanError::InvalidPatternString)?));
        }

        Ok(pattern)
    }

    /// Number of tokens separated by whitespace, the array size [`Self::parse_pattern_const`] needs.
    pub const fn signature_size(pattern: &str) -> usize {
        let pattern = pattern.as_bytes();
        let mut count = 0;
        let mut in_token = false;
        let mut index = 0;

        while index < pattern.len() {
            if is_space(pattern[index]) {
                in_token = false;
            } else if !in_token {
                count += 1;
                in_token = true;
            }
            index += 1;
        }

        count
    }

    /// Strict parser behind [`consteval_pattern!`](crate::consteval_pattern). Unlike
    /// [`Self::parse_pattern`], any whitespace separates tokens and every token must be `?`, `??` or
    /// exactly two hex digits. `N` must be [`Self::signature_size`] of `pattern`.
    pub const fn parse_pattern_const<const N: usize>(pattern: &str) -> Result<[Option<u8>; N], PatternScanError> {
        let pattern = pattern.as_bytes();
        let mut result = [None; N];
        let mut out = 0;
        let mut index = 0;

        while index < pattern.len() {
            while index < pattern.len() && is_space(pattern[index]) {
                index += 1;
            }

            let token_start = index;

            while index < pattern.len() && !is_space(pattern[index]) {
                index += 1;
            }

            let token_size = index - token_start;

            if token_size == 0 {
                continue;
            }
            if out == N {
                return Err(PatternScanError::InvalidPatternString);
            }

            result[out] = if token_size <= 2 && pattern[token_start] == b'?' {
                if token_size == 2 && pattern[token_start + 1] != b'?' {
                    return Err(PatternScanError::InvalidPatternString);
                }
                None
            } else if token_size != 2 {
                return Err(PatternScanError::InvalidPatternString);
            } else {
                match (hex_value(pattern[token_start]), hex_value(pattern[token_start + 1])) {
                    (Some(high), Some(low)) => Some(high << 4 | low),
                    _ => return Err(PatternScanError::InvalidPatternString),
                }
            };
            out += 1;
        }

        if out != N {
            return Err(PatternScanError::InvalidPatternString);
        }

        Ok(result)
    }
}

/// Parses a pattern literal at compile time into a `[Option<u8>; N]`; an invalid pattern does not
/// compile. See [`PatternScanner::parse_pattern_const`] for the accepted syntax.
#[macro_export]
macro_rules! consteval_pattern {
    ($pattern:expr) => {{
        const PATTERN: &str = $pattern;
        const SIZE: usize = $crate::utility::PatternScanner::signature_size(PATTERN);
        const PARSED: [Option<u8>; SIZE] = match $crate::utility::PatternScanner::parse_pattern_const::<SIZE>(PATTERN) {
            Ok(parsed) => parsed,
            Err(_) => panic!("invalid pattern"),
        };
        PARSED
    }};
}
