//! Build-specific FName layouts. Addresses refer to the supplied memory backend.
use crate::rev_eng::{MemoryRead, read_pointer, shift_address};
use std::io;
#[derive(Debug)]
pub enum ActorNameError {
    NullPointer,
    InvalidNameIndex,
    EmptyName,
    InvalidLayout,
    Memory(io::Error),
}
impl From<io::Error> for ActorNameError {
    fn from(e: io::Error) -> Self {
        Self::Memory(e)
    }
}
impl std::fmt::Display for ActorNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for ActorNameError {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NamePoolKind {
    PointerArray,
    ChunkedArray,
    BlockPool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NameCharKind {
    Ansi,
    Wide,
    PerEntryFlag,
}
#[derive(Clone, Copy, Debug)]
pub struct NameLayout {
    pub object_name_offset: isize,
    pub name_number_offset: Option<isize>,
    pub pool_kind: NamePoolKind,
    pub pool_data_offset: isize,
    pub pointer_size: usize,
    pub elements_per_chunk: usize,
    pub block_offset_bits: u32,
    pub block_entry_stride: u32,
    pub entry_flags_offset: isize,
    pub entry_text_offset: isize,
    pub char_kind: NameCharKind,
    pub header_length_shift: u32,
    pub max_name_length: usize,
}
impl Default for NameLayout {
    fn default() -> Self {
        Self {
            object_name_offset: 0x18,
            name_number_offset: Some(4),
            pool_kind: NamePoolKind::BlockPool,
            pool_data_offset: 0x10,
            pointer_size: 8,
            elements_per_chunk: 16384,
            block_offset_bits: 16,
            block_entry_stride: 2,
            entry_flags_offset: 0,
            entry_text_offset: 2,
            char_kind: NameCharKind::PerEntryFlag,
            header_length_shift: 6,
            max_name_length: 1024,
        }
    }
}
impl NameLayout {
    pub fn ue2_5() -> Self {
        Self {
            object_name_offset: 0x24,
            name_number_offset: None,
            pool_kind: NamePoolKind::PointerArray,
            pool_data_offset: 0,
            pointer_size: 4,
            entry_text_offset: 0xc,
            char_kind: NameCharKind::Wide,
            ..Self::default()
        }
    }
    pub fn ue3() -> Self {
        Self {
            object_name_offset: 0x28,
            pool_kind: NamePoolKind::PointerArray,
            pool_data_offset: 0,
            pointer_size: 4,
            entry_flags_offset: 8,
            entry_text_offset: 0x10,
            ..Self::default()
        }
    }
    pub fn ue4_legacy() -> Self {
        Self {
            pool_kind: NamePoolKind::ChunkedArray,
            pool_data_offset: 0,
            entry_text_offset: 0x10,
            ..Self::default()
        }
    }
    pub fn ue4() -> Self {
        Self::default()
    }
    pub fn ue5() -> Self {
        Self::ue4()
    }
}
pub fn get_actor_name(
    memory: &impl MemoryRead,
    actor: usize,
    g_names: usize,
    layout: &NameLayout,
) -> Result<String, ActorNameError> {
    if actor == 0 || g_names == 0 {
        return Err(ActorNameError::NullPointer);
    }
    if layout.elements_per_chunk == 0
        || layout.block_offset_bits >= 32
        || layout.header_length_shift >= 16
        || layout.block_entry_stride == 0
    {
        return Err(ActorNameError::InvalidLayout);
    }
    let add = |base: usize, index: usize, stride: usize| {
        index
            .checked_mul(stride)
            .and_then(|n| base.checked_add(n))
            .ok_or(ActorNameError::InvalidLayout)
    };
    let name_address = shift_address(actor, layout.object_name_offset);
    let index: u32 = memory.read_memory(name_address)?;
    let pool = shift_address(g_names, layout.pool_data_offset);
    let entry = match layout.pool_kind {
        NamePoolKind::PointerArray => {
            let entries = read_pointer(memory, pool, layout.pointer_size)?;
            if entries == 0 {
                return Err(ActorNameError::InvalidNameIndex);
            }
            read_pointer(
                memory,
                add(entries, index as usize, layout.pointer_size)?,
                layout.pointer_size,
            )?
        }
        NamePoolKind::ChunkedArray => {
            let chunk = read_pointer(
                memory,
                add(
                    pool,
                    index as usize / layout.elements_per_chunk,
                    layout.pointer_size,
                )?,
                layout.pointer_size,
            )?;
            if chunk == 0 {
                return Err(ActorNameError::InvalidNameIndex);
            }
            read_pointer(
                memory,
                add(
                    chunk,
                    index as usize % layout.elements_per_chunk,
                    layout.pointer_size,
                )?,
                layout.pointer_size,
            )?
        }
        NamePoolKind::BlockPool => {
            let block = read_pointer(
                memory,
                add(
                    pool,
                    (index >> layout.block_offset_bits) as usize,
                    layout.pointer_size,
                )?,
                layout.pointer_size,
            )?;
            if block == 0 {
                return Err(ActorNameError::InvalidNameIndex);
            }
            add(
                block,
                (index & ((1 << layout.block_offset_bits) - 1)) as usize,
                layout.block_entry_stride as usize,
            )?
        }
    };
    if entry == 0 {
        return Err(ActorNameError::InvalidNameIndex);
    }
    let flags = if layout.char_kind == NameCharKind::PerEntryFlag
        || layout.pool_kind == NamePoolKind::BlockPool
    {
        memory.read_memory::<u16>(shift_address(entry, layout.entry_flags_offset))?
    } else {
        0
    };
    let wide = layout.char_kind == NameCharKind::Wide
        || (layout.char_kind == NameCharKind::PerEntryFlag && flags & 1 != 0);
    let length = if layout.pool_kind == NamePoolKind::BlockPool {
        ((flags >> layout.header_length_shift) as usize).min(layout.max_name_length)
    } else {
        layout.max_name_length
    };
    let text = shift_address(entry, layout.entry_text_offset);
    let mut units = Vec::new();
    for i in 0..length {
        let unit = if wide {
            memory.read_memory::<u16>(add(text, i, 2)?)?
        } else {
            u16::from(memory.read_memory::<u8>(add(text, i, 1)?)?)
        };
        if unit == 0 {
            break;
        }
        units.push(unit);
    }
    // Decode UTF-16 surrogate pairs rather than producing invalid UTF-8 as the C++ helper does.
    let mut name = String::from_utf16_lossy(&units);
    if name.is_empty() {
        return Err(ActorNameError::EmptyName);
    }
    if let Some(offset) = layout.name_number_offset {
        let number: u32 = memory.read_memory(shift_address(name_address, offset))?;
        if number != 0 {
            name.push('_');
            name.push_str(&(number - 1).to_string());
        }
    }
    Ok(name)
}
