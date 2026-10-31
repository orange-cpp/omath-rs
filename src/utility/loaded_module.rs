//! Snapshots of live module sections. Match addresses are actual process addresses, including ASLR.
use super::{
    PatternScanner,
    byte_reader::{read_u16, read_u32, read_u64, until_nul},
};
use crate::rev_eng::{MemoryRead, ProcessMemory};
use std::{io, path::PathBuf};

#[derive(Clone, Debug)]
pub struct LoadedModule {
    pub base: usize,
    pub path: PathBuf,
    pub slide: isize,
}
impl LoadedModule {
    /// An empty name selects the main executable. Other names match the full path or filename.
    pub fn find(name: &str) -> io::Result<Option<Self>> {
        Ok(Self::list()?.into_iter().find(|module| {
            if name.is_empty() {
                std::env::current_exe()
                    .ok()
                    .is_some_and(|path| path == module.path)
            } else {
                module.path.to_string_lossy().contains(name)
                    || module
                        .path
                        .file_name()
                        .is_some_and(|file| file.to_string_lossy().eq_ignore_ascii_case(name))
            }
        }))
    }
    pub fn list() -> io::Result<Vec<Self>> {
        list_modules()
    }
}

fn read(memory: &impl MemoryRead, address: usize, size: usize) -> Option<Vec<u8>> {
    // Allocate fallibly: malformed image headers must not force an abort.
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(size).ok()?;
    bytes.resize(size, 0);
    memory.read_bytes(address, &mut bytes).ok()?;
    Some(bytes)
}
pub(super) fn scan_pe(
    memory: &impl MemoryRead,
    base: usize,
    pattern: &[Option<u8>],
    section_name: &str,
) -> Option<usize> {
    let dos = read(memory, base, 64)?;
    if read_u16(&dos, 0)? != 0x5a4d {
        return None;
    }
    let nt = base.checked_add(read_u32(&dos, 0x3c)? as usize)?;
    let header = read(memory, nt, 24)?;
    if read_u32(&header, 0)? != 0x4550 {
        return None;
    }
    let count = read_u16(&header, 6)? as usize;
    let table = nt
        .checked_add(24)?
        .checked_add(read_u16(&header, 20)? as usize)?;
    for i in 0..count {
        let section = read(memory, table.checked_add(i.checked_mul(40)?)?, 40)?;
        if until_nul(&section[..8]) != section_name.as_bytes() {
            continue;
        }
        let address = base.checked_add(read_u32(&section, 12)? as usize)?;
        // VirtualSize, rather than SizeOfRawData: loaded sections include zero-filled tails.
        let size = read_u32(&section, 8)? as usize;
        return address.checked_add(PatternScanner::scan_for_parsed_pattern(
            &read(memory, address, size)?,
            pattern,
        )?);
    }
    None
}
pub(super) fn scan_macho(
    memory: &impl MemoryRead,
    base: usize,
    slide: isize,
    pattern: &[Option<u8>],
    section_name: &str,
) -> Option<usize> {
    let header = read(memory, base, 28)?;
    let wide = match read_u32(&header, 0)? {
        0xfeedfacf => true,
        0xfeedface => false,
        _ => return None,
    };
    let ncmds = read_u32(&header, 16)?;
    let end = base
        .checked_add(if wide { 32 } else { 28 })?
        .checked_add(read_u32(&header, 20)? as usize)?;
    let mut command = base.checked_add(if wide { 32 } else { 28 })?;
    for _ in 0..ncmds {
        if command.checked_add(8)? > end {
            return None;
        }
        let head = read(memory, command, 8)?;
        let size = read_u32(&head, 4)? as usize;
        if size < 8 || command.checked_add(size)? > end {
            return None;
        }
        if read_u32(&head, 0)? == if wide { 0x19 } else { 1 } {
            let bytes = read(memory, command, size)?;
            let segment_size = if wide { 72 } else { 56 };
            let count = read_u32(&bytes, if wide { 64 } else { 48 })? as usize;
            let stride = if wide { 80 } else { 68 };
            for i in 0..count {
                let offset = segment_size + i.checked_mul(stride)?;
                let section = bytes.get(offset..offset.checked_add(stride)?)?;
                if until_nul(&section[..16]) != section_name.as_bytes() {
                    continue;
                }
                let virtual_addr = if wide {
                    read_u64(section, 32)?
                } else {
                    read_u32(section, 32)? as u64
                };
                let size = if wide {
                    read_u64(section, 40)?
                } else {
                    read_u32(section, 36)? as u64
                };
                let address = usize::try_from(virtual_addr)
                    .ok()?
                    .checked_add_signed(slide)?;
                return address.checked_add(PatternScanner::scan_for_parsed_pattern(
                    &read(memory, address, usize::try_from(size).ok()?)?,
                    pattern,
                )?);
            }
        }
        command = command.checked_add(size)?;
    }
    None
}

impl super::PePatternScanner {
    pub fn scan_for_pattern_in_mapped_image(
        memory: &impl MemoryRead,
        base: usize,
        pattern: &str,
        section: &str,
    ) -> Option<usize> {
        scan_pe(
            memory,
            base,
            &PatternScanner::parse_pattern(pattern).ok()?,
            section,
        )
    }
    pub fn scan_for_parsed_pattern_in_loaded_module(
        module: &LoadedModule,
        pattern: &[Option<u8>],
        section: &str,
    ) -> Option<usize> {
        scan_pe(
            &ProcessMemory::current().ok()?,
            module.base,
            pattern,
            section,
        )
    }
    pub fn scan_for_pattern_in_loaded_module(
        module: &LoadedModule,
        pattern: &str,
        section: &str,
    ) -> Option<usize> {
        Self::scan_for_parsed_pattern_in_loaded_module(
            module,
            &PatternScanner::parse_pattern(pattern).ok()?,
            section,
        )
    }
}
impl super::MachOPatternScanner {
    pub fn scan_for_pattern_in_mapped_image(
        memory: &impl MemoryRead,
        base: usize,
        slide: isize,
        pattern: &str,
        section: &str,
    ) -> Option<usize> {
        scan_macho(
            memory,
            base,
            slide,
            &PatternScanner::parse_pattern(pattern).ok()?,
            section,
        )
    }
    pub fn scan_for_parsed_pattern_in_loaded_module(
        module: &LoadedModule,
        pattern: &[Option<u8>],
        section: &str,
    ) -> Option<usize> {
        scan_macho(
            &ProcessMemory::current().ok()?,
            module.base,
            module.slide,
            pattern,
            section,
        )
    }
    pub fn scan_for_pattern_in_loaded_module(
        module: &LoadedModule,
        pattern: &str,
        section: &str,
    ) -> Option<usize> {
        Self::scan_for_parsed_pattern_in_loaded_module(
            module,
            &PatternScanner::parse_pattern(pattern).ok()?,
            section,
        )
    }
}
impl super::ElfPatternScanner {
    /// ELF section tables usually are not mapped. Read their metadata from disk, but scan live bytes.
    pub fn scan_for_parsed_pattern_in_loaded_module(
        module: &LoadedModule,
        pattern: &[Option<u8>],
        section: &str,
    ) -> Option<usize> {
        let image = std::fs::read(&module.path).ok()?;
        let section = super::elf_pattern_scan::extract_section(&image, section)?;
        let address = usize::try_from(section.virtual_base_addr)
            .ok()?
            .checked_add_signed(module.slide)?;
        address.checked_add(PatternScanner::scan_for_parsed_pattern(
            &read(&ProcessMemory::current().ok()?, address, section.data.len())?,
            pattern,
        )?)
    }
    pub fn scan_for_pattern_in_loaded_module(
        module: &LoadedModule,
        pattern: &str,
        section: &str,
    ) -> Option<usize> {
        Self::scan_for_parsed_pattern_in_loaded_module(
            module,
            &PatternScanner::parse_pattern(pattern).ok()?,
            section,
        )
    }
}

#[cfg(windows)]
fn list_modules() -> io::Result<Vec<LoadedModule>> {
    use windows_sys::Win32::System::{ProcessStatus::*, Threading::GetCurrentProcess};
    let process = unsafe { GetCurrentProcess() };
    let mut modules = vec![std::ptr::null_mut(); 128];
    loop {
        let mut required = 0;
        let size = u32::try_from(
            modules.len() * std::mem::size_of::<windows_sys::Win32::Foundation::HMODULE>(),
        )
        .map_err(io::Error::other)?;
        if unsafe { K32EnumProcessModules(process, modules.as_mut_ptr(), size, &mut required) } == 0
        {
            return Err(io::Error::last_os_error());
        }
        if required > size {
            modules.resize(
                required as usize / std::mem::size_of::<windows_sys::Win32::Foundation::HMODULE>()
                    + 16,
                std::ptr::null_mut(),
            );
            continue;
        }
        modules.truncate(
            required as usize / std::mem::size_of::<windows_sys::Win32::Foundation::HMODULE>(),
        );
        break;
    }
    let mut result = Vec::new();
    for module in modules {
        let mut name = vec![0_u16; 32768];
        let length =
            unsafe { K32GetModuleFileNameExW(process, module, name.as_mut_ptr(), name.len() as _) };
        if length == 0 {
            return Err(io::Error::last_os_error());
        }
        use std::os::windows::ffi::OsStringExt;
        result.push(LoadedModule {
            base: module as usize,
            slide: 0,
            path: std::ffi::OsString::from_wide(&name[..length as usize]).into(),
        });
    }
    Ok(result)
}
#[cfg(target_os = "linux")]
fn list_modules() -> io::Result<Vec<LoadedModule>> {
    unsafe extern "C" fn collect(
        info: *mut libc::dl_phdr_info,
        _: usize,
        data: *mut std::ffi::c_void,
    ) -> i32 {
        let info = unsafe { &*info };
        let result = unsafe { &mut *data.cast::<Vec<LoadedModule>>() };
        let path = if info.dlpi_name.is_null() || unsafe { *info.dlpi_name } == 0 {
            std::fs::read_link("/proc/self/exe").unwrap_or_default()
        } else {
            PathBuf::from(
                unsafe { std::ffi::CStr::from_ptr(info.dlpi_name) }
                    .to_string_lossy()
                    .into_owned(),
            )
        };
        let mut base = info.dlpi_addr as usize;
        for header in
            unsafe { std::slice::from_raw_parts(info.dlpi_phdr, info.dlpi_phnum as usize) }
        {
            if header.p_type == libc::PT_LOAD && header.p_offset == 0 {
                base += header.p_vaddr as usize;
                break;
            }
        }
        result.push(LoadedModule {
            base,
            slide: info.dlpi_addr as isize,
            path,
        });
        0
    }
    let mut result = Vec::new();
    unsafe {
        libc::dl_iterate_phdr(
            Some(collect),
            (&mut result as *mut Vec<LoadedModule>).cast(),
        );
    }
    Ok(result)
}
#[cfg(target_os = "macos")]
fn list_modules() -> io::Result<Vec<LoadedModule>> {
    unsafe extern "C" {
        fn _dyld_image_count() -> u32;
        fn _dyld_get_image_name(index: u32) -> *const std::ffi::c_char;
        fn _dyld_get_image_header(index: u32) -> *const std::ffi::c_void;
        fn _dyld_get_image_vmaddr_slide(index: u32) -> isize;
    }
    let mut result = Vec::new();
    for i in 0..unsafe { _dyld_image_count() } {
        let name = unsafe { _dyld_get_image_name(i) };
        if name.is_null() {
            continue;
        }
        result.push(LoadedModule {
            base: unsafe { _dyld_get_image_header(i) } as usize,
            slide: unsafe { _dyld_get_image_vmaddr_slide(i) },
            path: PathBuf::from(
                unsafe { std::ffi::CStr::from_ptr(name) }
                    .to_string_lossy()
                    .into_owned(),
            ),
        });
    }
    Ok(result)
}
