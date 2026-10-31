//! Configurable GObjects layouts from UE 2.5 through UE 5.
use crate::rev_eng::{MemoryRead, read_pointer, shift_address};
use std::io;
#[derive(Debug)]
pub enum ObjectArrayError {
    NullPointer,
    IndexOutOfRange,
    NullObject,
    InvalidLayout,
    Memory(io::Error),
}
impl From<io::Error> for ObjectArrayError {
    fn from(e: io::Error) -> Self {
        Self::Memory(e)
    }
}
impl std::fmt::Display for ObjectArrayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for ObjectArrayError {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectArrayKind {
    Flat,
    Chunked,
}
#[derive(Clone, Copy, Debug)]
pub struct ObjectArrayLayout {
    pub kind: ObjectArrayKind,
    pub objects_offset: isize,
    pub count_offset: isize,
    pub elements_per_chunk: usize,
    pub indirect_chunk_table: bool,
    pub item_stride: usize,
    pub item_object_offset: isize,
    pub pointer_size: usize,
}
impl Default for ObjectArrayLayout {
    fn default() -> Self {
        Self {
            kind: ObjectArrayKind::Chunked,
            objects_offset: 0x10,
            count_offset: 0x24,
            elements_per_chunk: 65536,
            indirect_chunk_table: true,
            item_stride: 0x18,
            item_object_offset: 0,
            pointer_size: 8,
        }
    }
}
impl ObjectArrayLayout {
    pub fn ue2_5() -> Self {
        Self {
            kind: ObjectArrayKind::Flat,
            objects_offset: 0,
            count_offset: 4,
            item_stride: 4,
            pointer_size: 4,
            ..Self::default()
        }
    }
    pub fn ue3() -> Self {
        Self::ue2_5()
    }
    pub fn ue4_fixed() -> Self {
        Self {
            kind: ObjectArrayKind::Flat,
            count_offset: 0x1c,
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
pub fn get_object_count(
    memory: &impl MemoryRead,
    g_objects: usize,
    layout: &ObjectArrayLayout,
) -> Result<u32, ObjectArrayError> {
    if g_objects == 0 {
        return Err(ObjectArrayError::NullPointer);
    }
    Ok(memory.read_memory(shift_address(g_objects, layout.count_offset))?)
}
pub fn get_object_by_index(
    memory: &impl MemoryRead,
    g_objects: usize,
    index: u32,
    layout: &ObjectArrayLayout,
) -> Result<usize, ObjectArrayError> {
    if index >= get_object_count(memory, g_objects, layout)? {
        return Err(ObjectArrayError::IndexOutOfRange);
    }
    if layout.elements_per_chunk == 0 || layout.item_stride == 0 {
        return Err(ObjectArrayError::InvalidLayout);
    }
    let add = |base: usize, index: usize, stride: usize| {
        index
            .checked_mul(stride)
            .and_then(|n| base.checked_add(n))
            .ok_or(ObjectArrayError::InvalidLayout)
    };
    let address = shift_address(g_objects, layout.objects_offset);
    let index = index as usize;
    let element = match layout.kind {
        ObjectArrayKind::Flat => {
            let objects = read_pointer(memory, address, layout.pointer_size)?;
            if objects == 0 {
                return Err(ObjectArrayError::NullPointer);
            }
            add(objects, index, layout.item_stride)?
        }
        ObjectArrayKind::Chunked => {
            let chunks = if layout.indirect_chunk_table {
                read_pointer(memory, address, layout.pointer_size)?
            } else {
                address
            };
            if chunks == 0 {
                return Err(ObjectArrayError::NullPointer);
            }
            let chunk = read_pointer(
                memory,
                add(
                    chunks,
                    index / layout.elements_per_chunk,
                    layout.pointer_size,
                )?,
                layout.pointer_size,
            )?;
            if chunk == 0 {
                return Err(ObjectArrayError::NullPointer);
            }
            add(chunk, index % layout.elements_per_chunk, layout.item_stride)?
        }
    };
    let object = read_pointer(
        memory,
        shift_address(element, layout.item_object_offset),
        layout.pointer_size,
    )?;
    if object == 0 {
        return Err(ObjectArrayError::NullObject);
    }
    Ok(object)
}
