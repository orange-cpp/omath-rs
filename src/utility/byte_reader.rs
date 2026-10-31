//! Bounds-checked little-endian reads of untrusted file images.

/// `None` when `offset + size` overflows or runs past the end of `data`.
pub(super) fn read_bytes(data: &[u8], offset: usize, size: usize) -> Option<&[u8]> {
    data.get(offset..offset.checked_add(size)?)
}

fn read_array<const N: usize>(data: &[u8], offset: usize) -> Option<[u8; N]> {
    read_bytes(data, offset, N)?.try_into().ok()
}

pub(super) fn read_u16(data: &[u8], offset: usize) -> Option<u16> {
    read_array(data, offset).map(u16::from_le_bytes)
}

pub(super) fn read_u32(data: &[u8], offset: usize) -> Option<u32> {
    read_array(data, offset).map(u32::from_le_bytes)
}

pub(super) fn read_u64(data: &[u8], offset: usize) -> Option<u64> {
    read_array(data, offset).map(u64::from_le_bytes)
}

/// [`read_u32`] widened, for formats whose 32-bit and 64-bit layouts share one parser.
pub(super) fn read_u32_as_u64(data: &[u8], offset: usize) -> Option<u64> {
    read_u32(data, offset).map(u64::from)
}

/// The part of a name field before its first NUL, or all of it when it has none.
pub(super) fn until_nul(name: &[u8]) -> &[u8] {
    name.split(|&byte| byte == 0).next().unwrap_or(name)
}
