//! Memory access with explicit ownership and fallible reads, including cross-bitness pointers.
use std::{
    io,
    mem::{MaybeUninit, size_of},
};

/// Plain values whose entire representation is initialized and accepts every bit pattern.
///
/// # Safety
/// Implementors must be `Copy`, contain no padding or pointers/references with validity
/// requirements, and permit every possible byte representation.
pub unsafe trait MemoryValue: Copy + 'static {}
macro_rules! values { ($($t:ty),*) => { $(unsafe impl MemoryValue for $t {})* }; }
values!(
    u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize, f32, f64
);
unsafe impl<T: MemoryValue, const N: usize> MemoryValue for [T; N] {}

pub trait MemoryRead {
    /// Must fill the entire destination or return an error.
    fn read_bytes(&self, address: usize, destination: &mut [u8]) -> io::Result<()>;
    fn read_memory<T: MemoryValue>(&self, address: usize) -> io::Result<T> {
        let mut value = MaybeUninit::<T>::zeroed();
        // MemoryValue permits every byte pattern and has no uninitialized padding.
        let bytes =
            unsafe { std::slice::from_raw_parts_mut(value.as_mut_ptr().cast(), size_of::<T>()) };
        self.read_bytes(address, bytes)?;
        Ok(unsafe { value.assume_init() })
    }
}
pub trait MemoryWrite: MemoryRead {
    fn write_bytes(&self, address: usize, source: &[u8]) -> io::Result<()>;
    fn write_memory<T: MemoryValue>(&self, address: usize, value: &T) -> io::Result<()> {
        let bytes =
            unsafe { std::slice::from_raw_parts((value as *const T).cast(), size_of::<T>()) };
        self.write_bytes(address, bytes)
    }
}
pub use MemoryRead as MemoryReadTrait;
pub const fn shift_address(address: usize, offset: isize) -> usize {
    address.wrapping_add_signed(offset)
}
pub fn read_pointer(
    memory: &impl MemoryRead,
    address: usize,
    pointer_size: usize,
) -> io::Result<usize> {
    match pointer_size {
        4 => Ok(memory.read_memory::<u32>(address)? as usize),
        8 => usize::try_from(memory.read_memory::<u64>(address)?)
            .map_err(|_| io::Error::other("target pointer exceeds host address space")),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "pointer size must be 4 or 8",
        )),
    }
}

/// A borrowed byte image at a chosen virtual address; useful for dumps and deterministic tests.
pub struct MemoryView<'a> {
    pub base: usize,
    pub bytes: &'a [u8],
}
impl MemoryRead for MemoryView<'_> {
    fn read_bytes(&self, address: usize, destination: &mut [u8]) -> io::Result<()> {
        let offset = address
            .checked_sub(self.base)
            .ok_or_else(|| io::Error::other("address below image"))?;
        let end = offset
            .checked_add(destination.len())
            .ok_or_else(|| io::Error::other("address overflow"))?;
        destination.copy_from_slice(self.bytes.get(offset..end).ok_or_else(|| {
            io::Error::new(io::ErrorKind::UnexpectedEof, "address outside image")
        })?);
        Ok(())
    }
}

/// Reads/writes a process through the operating system. The handle is owned on Windows.
pub struct ProcessMemory {
    #[cfg(windows)]
    handle: windows_sys::Win32::Foundation::HANDLE,
    #[cfg(unix)]
    pid: u32,
}
#[cfg(windows)]
unsafe impl Send for ProcessMemory {}
#[cfg(windows)]
unsafe impl Sync for ProcessMemory {}
impl ProcessMemory {
    pub fn current() -> io::Result<Self> {
        Self::open(std::process::id(), false)
    }
    pub fn open(pid: u32, writable: bool) -> io::Result<Self> {
        #[cfg(windows)]
        {
            use windows_sys::Win32::System::Threading::*;
            let access = PROCESS_VM_READ
                | PROCESS_QUERY_INFORMATION
                | if writable {
                    PROCESS_VM_WRITE | PROCESS_VM_OPERATION
                } else {
                    0
                };
            let handle = unsafe { OpenProcess(access, 0, pid) };
            if handle.is_null() {
                return Err(io::Error::last_os_error());
            }
            Ok(Self { handle })
        }
        #[cfg(unix)]
        {
            let _ = writable;
            Ok(Self { pid })
        }
    }
}
impl MemoryRead for ProcessMemory {
    fn read_bytes(&self, address: usize, destination: &mut [u8]) -> io::Result<()> {
        if destination.is_empty() {
            return Ok(());
        }
        #[cfg(windows)]
        {
            let mut count = 0;
            let ok = unsafe {
                windows_sys::Win32::System::Diagnostics::Debug::ReadProcessMemory(
                    self.handle,
                    address as _,
                    destination.as_mut_ptr().cast(),
                    destination.len(),
                    &mut count,
                )
            };
            if ok == 0 {
                return Err(io::Error::last_os_error());
            }
            if count == destination.len() {
                return Ok(());
            }
        }
        #[cfg(target_os = "linux")]
        {
            let local = libc::iovec {
                iov_base: destination.as_mut_ptr().cast(),
                iov_len: destination.len(),
            };
            let remote = libc::iovec {
                iov_base: address as _,
                iov_len: destination.len(),
            };
            let count = unsafe { libc::process_vm_readv(self.pid as _, &local, 1, &remote, 1, 0) };
            if count < 0 {
                return Err(io::Error::last_os_error());
            }
            if count as usize == destination.len() {
                return Ok(());
            }
        }
        #[cfg(target_os = "macos")]
        {
            return macos::read(self.pid, address, destination);
        }
        #[cfg(not(target_os = "macos"))]
        Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "partial process memory read",
        ))
    }
}
impl MemoryWrite for ProcessMemory {
    fn write_bytes(&self, address: usize, source: &[u8]) -> io::Result<()> {
        if source.is_empty() {
            return Ok(());
        }
        #[cfg(windows)]
        {
            let mut count = 0;
            let ok = unsafe {
                windows_sys::Win32::System::Diagnostics::Debug::WriteProcessMemory(
                    self.handle,
                    address as _,
                    source.as_ptr().cast(),
                    source.len(),
                    &mut count,
                )
            };
            if ok == 0 {
                return Err(io::Error::last_os_error());
            }
            if count == source.len() {
                return Ok(());
            }
        }
        #[cfg(target_os = "linux")]
        {
            let local = libc::iovec {
                iov_base: source.as_ptr() as _,
                iov_len: source.len(),
            };
            let remote = libc::iovec {
                iov_base: address as _,
                iov_len: source.len(),
            };
            let count = unsafe { libc::process_vm_writev(self.pid as _, &local, 1, &remote, 1, 0) };
            if count < 0 {
                return Err(io::Error::last_os_error());
            }
            if count as usize == source.len() {
                return Ok(());
            }
        }
        #[cfg(target_os = "macos")]
        {
            return macos::write(self.pid, address, source);
        }
        #[cfg(not(target_os = "macos"))]
        Err(io::Error::new(
            io::ErrorKind::WriteZero,
            "partial process memory write",
        ))
    }
}
#[cfg(windows)]
impl Drop for ProcessMemory {
    fn drop(&mut self) {
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(self.handle);
        }
    }
}
pub type InternalMemoryTrait = ProcessMemory;

pub struct ExternalReverseEngineeredObject<'a, M> {
    pub address: usize,
    pub memory: &'a M,
}
impl<'a, M: MemoryRead> ExternalReverseEngineeredObject<'a, M> {
    pub const fn new(address: usize, memory: &'a M) -> Self {
        Self { address, memory }
    }
    pub fn get_by_offset<T: MemoryValue>(&self, offset: isize) -> io::Result<T> {
        self.memory.read_memory(shift_address(self.address, offset))
    }
}
impl<M: MemoryWrite> ExternalReverseEngineeredObject<'_, M> {
    pub fn set_by_offset<T: MemoryValue>(&self, offset: isize, value: &T) -> io::Result<()> {
        self.memory
            .write_memory(shift_address(self.address, offset), value)
    }
}

/// A local object address. Method calls require the caller to specify the exact ABI.
pub struct InternalReverseEngineeredObject {
    pub address: usize,
}
impl InternalReverseEngineeredObject {
    pub fn resolve_pattern(module_name: &str, pattern: &str) -> io::Result<usize> {
        let module = crate::utility::LoadedModule::find(module_name)?
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "module not found"))?;
        #[cfg(windows)]
        let address = crate::utility::PePatternScanner::scan_for_pattern_in_loaded_module(
            &module, pattern, ".text",
        );
        #[cfg(target_os = "linux")]
        let address = crate::utility::ElfPatternScanner::scan_for_pattern_in_loaded_module(
            &module, pattern, ".text",
        );
        #[cfg(target_os = "macos")]
        let address = crate::utility::MachOPatternScanner::scan_for_pattern_in_loaded_module(
            &module, pattern, "__text",
        );
        address.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "pattern not found"))
    }
    /// # Safety
    /// The resolved pattern must identify a function of type `F` and the closure must
    /// supply arguments valid for that function and object. No address is cached across module reloads.
    pub unsafe fn call_pattern_method<F: Copy, R>(
        &self,
        module: &str,
        pattern: &str,
        invoke: impl FnOnce(F, *mut std::ffi::c_void) -> R,
    ) -> io::Result<R> {
        let address = Self::resolve_pattern(module, pattern)?;
        Ok(unsafe { self.call_method(address, invoke) })
    }
    pub const fn new(address: usize) -> Self {
        Self { address }
    }
    pub fn get_by_offset<T: MemoryValue>(&self, offset: isize) -> io::Result<T> {
        ProcessMemory::current()?.read_memory(shift_address(self.address, offset))
    }
    pub fn set_by_offset<T: MemoryValue>(&self, offset: isize, value: &T) -> io::Result<()> {
        ProcessMemory::open(std::process::id(), true)?
            .write_memory(shift_address(self.address, offset), value)
    }
    pub fn virtual_method_address(&self, table_offset: isize, index: usize) -> io::Result<usize> {
        let memory = ProcessMemory::current()?;
        let table: usize = memory.read_memory(shift_address(self.address, table_offset))?;
        let slot = index
            .checked_mul(size_of::<usize>())
            .and_then(|offset| table.checked_add(offset))
            .ok_or_else(|| io::Error::other("vtable address overflow"))?;
        memory.read_memory(slot)
    }
    /// # Safety
    /// `F` must be a function pointer with the exact signature and ABI at `address`.
    /// The closure must supply a valid object and arguments for that method.
    pub unsafe fn call_method<F: Copy, R>(
        &self,
        address: usize,
        invoke: impl FnOnce(F, *mut std::ffi::c_void) -> R,
    ) -> R {
        assert_eq!(size_of::<F>(), size_of::<usize>());
        let function: F = unsafe { std::mem::transmute_copy(&address) };
        invoke(function, self.address as _)
    }
    /// # Safety
    /// The selected vtable slot must contain `F`; the object must support the call.
    pub unsafe fn call_virtual_method<F: Copy, R>(
        &self,
        table_offset: isize,
        index: usize,
        invoke: impl FnOnce(F, *mut std::ffi::c_void) -> R,
    ) -> io::Result<R> {
        let address = self.virtual_method_address(table_offset, index)?;
        let object = Self::new(shift_address(self.address, table_offset));
        Ok(unsafe { object.call_method(address, invoke) })
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    unsafe extern "C" {
        static mach_task_self_: u32;
        fn task_for_pid(task: u32, pid: i32, result: *mut u32) -> i32;
        fn mach_port_deallocate(task: u32, name: u32) -> i32;
        fn mach_vm_read_overwrite(
            task: u32,
            address: u64,
            size: u64,
            destination: u64,
            count: *mut u64,
        ) -> i32;
        fn mach_vm_write(task: u32, address: u64, data: usize, size: u32) -> i32;
    }
    struct Task(u32);
    impl Drop for Task {
        fn drop(&mut self) {
            unsafe {
                mach_port_deallocate(mach_task_self_, self.0);
            }
        }
    }
    fn task(pid: u32) -> io::Result<Task> {
        let mut result = 0;
        let status = unsafe { task_for_pid(mach_task_self_, pid as _, &mut result) };
        if status != 0 {
            return Err(io::Error::other(format!("task_for_pid: {status}")));
        }
        Ok(Task(result))
    }
    pub(super) fn read(pid: u32, address: usize, bytes: &mut [u8]) -> io::Result<()> {
        let task = task(pid)?;
        let mut count = 0;
        let status = unsafe {
            mach_vm_read_overwrite(
                task.0,
                address as _,
                bytes.len() as _,
                bytes.as_mut_ptr() as _,
                &mut count,
            )
        };
        if status != 0 || count != bytes.len() as u64 {
            return Err(io::Error::other(format!(
                "mach_vm_read_overwrite: {status}"
            )));
        }
        Ok(())
    }
    pub(super) fn write(pid: u32, address: usize, bytes: &[u8]) -> io::Result<()> {
        let task = task(pid)?;
        let length = u32::try_from(bytes.len()).map_err(io::Error::other)?;
        let status = unsafe { mach_vm_write(task.0, address as _, bytes.as_ptr() as _, length) };
        if status != 0 {
            return Err(io::Error::other(format!("mach_vm_write: {status}")));
        }
        Ok(())
    }
}
