//! Result of a scan inside a named section of an executable image.

use super::pattern_scan::PatternScanner;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SectionScanResult {
    pub virtual_base_addr: u64,
    pub raw_base_addr: u64,
    /// Offset of the match from the start of the section.
    pub target_offset: usize,
}

pub(super) struct ExtractedSection<'a> {
    pub(super) virtual_base_addr: u64,
    pub(super) raw_base_addr: u64,
    pub(super) data: &'a [u8],
}

impl ExtractedSection<'_> {
    pub(super) fn scan(&self, pattern: &[Option<u8>]) -> Option<SectionScanResult> {
        let target_offset = PatternScanner::scan_for_parsed_pattern(self.data, pattern)?;

        Some(SectionScanResult {
            virtual_base_addr: self.virtual_base_addr,
            raw_base_addr: self.raw_base_addr,
            target_offset,
        })
    }
}

/// Public scanning API of a file format. Expects an
/// `extract_section(&[u8], &str) -> Option<ExtractedSection>` function in the calling module.
///
/// File scans read the whole file and go through the same parser as byte slices do. Every function
/// returns `None` for an unreadable file, a malformed image, a missing section, an invalid pattern
/// or no match; use `PatternScanner::parse_pattern` to tell an invalid pattern apart.
macro_rules! impl_section_scanner {
    ($scanner:ident, $default_section_name:literal) => {
        impl $scanner {
            /// The section C++ scans when the caller does not name one.
            pub const DEFAULT_SECTION_NAME: &'static str = $default_section_name;

            pub fn scan_for_pattern_in_file(
                path_to_file: impl AsRef<std::path::Path>,
                pattern: &str,
                target_section_name: &str,
            ) -> Option<SectionScanResult> {
                let pattern = PatternScanner::parse_pattern(pattern).ok()?;

                Self::scan_for_parsed_pattern_in_file(path_to_file, &pattern, target_section_name)
            }

            /// Takes a pattern from [`PatternScanner::parse_pattern`] or [`consteval_pattern!`](crate::consteval_pattern).
            pub fn scan_for_parsed_pattern_in_file(
                path_to_file: impl AsRef<std::path::Path>,
                pattern: &[Option<u8>],
                target_section_name: &str,
            ) -> Option<SectionScanResult> {
                let file_data = std::fs::read(path_to_file).ok()?;

                Self::scan_for_parsed_pattern_in_memory_file(&file_data, pattern, target_section_name)
            }

            pub fn scan_for_pattern_in_memory_file(
                file_data: &[u8],
                pattern: &str,
                target_section_name: &str,
            ) -> Option<SectionScanResult> {
                let pattern = PatternScanner::parse_pattern(pattern).ok()?;

                Self::scan_for_parsed_pattern_in_memory_file(file_data, &pattern, target_section_name)
            }

            /// Takes a pattern from [`PatternScanner::parse_pattern`] or [`consteval_pattern!`](crate::consteval_pattern).
            pub fn scan_for_parsed_pattern_in_memory_file(
                file_data: &[u8],
                pattern: &[Option<u8>],
                target_section_name: &str,
            ) -> Option<SectionScanResult> {
                extract_section(file_data, target_section_name)?.scan(pattern)
            }
        }
    };
}

pub(super) use impl_section_scanner;
