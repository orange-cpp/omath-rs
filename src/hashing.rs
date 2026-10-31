//! FNV-1a, CRC-32 and Base64.

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Base64Error {
    InvalidLength,
    InvalidCharacter,
    InvalidPadding,
}

impl fmt::Display for Base64Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidLength => "base64 input length is not a multiple of 4",
            Self::InvalidCharacter => "base64 input contains an invalid character",
            Self::InvalidPadding => "base64 input has invalid padding",
        })
    }
}

impl std::error::Error for Base64Error {}

pub const fn fnv1a(value: &[u8]) -> u64 {
    let mut hash = 14695981039346656037u64;
    let mut index = 0;

    while index < value.len() {
        hash ^= value[index] as u64;
        hash = hash.wrapping_mul(1099511628211);
        index += 1;
    }

    hash
}

pub const fn crc32(value: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    let mut index = 0;

    while index < value.len() {
        crc ^= value[index] as u32;

        let mut bit = 0;
        while bit < 8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
            bit += 1;
        }
        index += 1;
    }

    crc ^ 0xFFFF_FFFF
}

pub fn base64_encode(value: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::with_capacity(value.len().div_ceil(3) * 4);

    for bytes in value.chunks(3) {
        let first = bytes[0] as u32;
        let second = bytes.get(1).copied().unwrap_or(0) as u32;
        let third = bytes.get(2).copied().unwrap_or(0) as u32;
        let chunk = first << 16 | second << 8 | third;
        let symbol = |shift: u32| ALPHABET[((chunk >> shift) & 0x3F) as usize] as char;

        result.push(symbol(18));
        result.push(symbol(12));
        result.push(if bytes.len() > 1 { symbol(6) } else { '=' });
        result.push(if bytes.len() > 2 { symbol(0) } else { '=' });
    }

    result
}

fn decode_character(character: u8) -> Option<u32> {
    Some(match character {
        b'A'..=b'Z' => character - b'A',
        b'a'..=b'z' => character - b'a' + 26,
        b'0'..=b'9' => character - b'0' + 52,
        b'+' => 62,
        b'/' => 63,
        _ => return None,
    } as u32)
}

pub fn base64_decode(value: &str) -> Result<Vec<u8>, Base64Error> {
    let value = value.as_bytes();

    if value.len() % 4 != 0 {
        return Err(Base64Error::InvalidLength);
    }

    let mut result = Vec::with_capacity(value.len() / 4 * 3);
    let chunk_count = value.len() / 4;

    for (index, chunk) in value.chunks_exact(4).enumerate() {
        let is_last_chunk = index + 1 == chunk_count;
        let decode_or_padding = |character: u8| match character {
            b'=' => Ok(None),
            _ => decode_character(character).map(Some).ok_or(Base64Error::InvalidCharacter),
        };

        let first = decode_character(chunk[0]).ok_or(Base64Error::InvalidCharacter)?;
        let second = decode_character(chunk[1]).ok_or(Base64Error::InvalidCharacter)?;
        let third = decode_or_padding(chunk[2])?;
        let fourth = decode_or_padding(chunk[3])?;

        match (third, fourth) {
            (None, fourth) if fourth.is_some() || !is_last_chunk || second & 0x0F != 0 => {
                return Err(Base64Error::InvalidPadding);
            }
            (Some(third), None) if !is_last_chunk || third & 0x03 != 0 => {
                return Err(Base64Error::InvalidPadding);
            }
            _ => {}
        }

        let bits = first << 18 | second << 12 | third.unwrap_or(0) << 6 | fourth.unwrap_or(0);

        result.push((bits >> 16) as u8);
        if third.is_some() {
            result.push((bits >> 8) as u8);
        }
        if fourth.is_some() {
            result.push(bits as u8);
        }
    }

    Ok(result)
}
