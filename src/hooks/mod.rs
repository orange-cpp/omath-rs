//! Optional x86/x64 inline hooks. Callbacks run before the original graphics call.
//!
//! Installation/removal require application-controlled quiescence: do not patch a function
//! while another thread is executing it or retain trampoline addresses after removal.
#[cfg(windows)]
pub type Dx9PresentCallback = Arc<
    dyn Fn(*mut c_void, *const c_void, *const c_void, *mut c_void, *const c_void) + Send + Sync,
>;
#[cfg(windows)]
pub type Dx9ResetCallback = Arc<dyn Fn(*mut c_void, *mut c_void) + Send + Sync>;
#[cfg(windows)]
pub type Dx9EndSceneCallback = Arc<dyn Fn(*mut c_void) + Send + Sync>;
#[cfg(windows)]
pub type PresentCallback = Arc<dyn Fn(*mut c_void, u32, u32) + Send + Sync>;
#[cfg(windows)]
pub type ResizeBuffersCallback = Arc<dyn Fn(*mut c_void, u32, u32, u32, i32, u32) + Send + Sync>;
#[cfg(windows)]
pub type ExecuteCommandListsCallback =
    Arc<dyn Fn(*mut c_void, u32, *const *mut c_void) + Send + Sync>;
#[cfg(windows)]
pub type OpenGlSwapBuffersCallback = Arc<dyn Fn(*mut c_void) + Send + Sync>;
#[cfg(target_os = "linux")]
pub type OpenGlSwapBuffersCallback = Arc<dyn Fn(*mut c_void, libc::c_ulong) + Send + Sync>;
use retour::RawDetour;
use std::{
    collections::BTreeMap,
    ffi::c_void,
    io,
    sync::{
        Arc, Mutex, OnceLock, RwLock,
        atomic::{AtomicUsize, Ordering},
    },
};
#[cfg(windows)]
mod discovery;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HookKind {
    Dx9,
    Dx11,
    Dx12,
    OpenGl,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum EventKind {
    Dx9Present,
    Dx9Reset,
    Dx9EndScene,
    Present,
    ResizeBuffers,
    ExecuteCommandLists,
    OpenGlSwapBuffers,
}
/// Raw API parameters retain their native widths. Pointer validity is controlled by the graphics API.
#[derive(Clone, Copy, Debug)]
pub struct HookEvent {
    pub kind: EventKind,
    pub object: *mut c_void,
    pub args: [usize; 5],
}
pub type Callback = Arc<dyn Fn(HookEvent) + Send + Sync>;
#[cfg(windows)]
pub type WndCallback = Arc<dyn Fn(usize, u32, usize, isize) -> Option<isize> + Send + Sync>;
#[derive(Default)]
struct HookState {
    groups: BTreeMap<HookKind, Vec<(usize, RawDetour)>>,
    libraries: Vec<libloading::Library>,
}
pub struct HooksManager {
    state: Mutex<HookState>,
    callbacks: RwLock<BTreeMap<EventKind, Callback>>,
    #[cfg(windows)]
    wnd_callback: RwLock<Option<WndCallback>>,
    #[cfg(windows)]
    window: Mutex<Option<(usize, isize)>>,
}
static ORIGINALS: [AtomicUsize; 11] = [const { AtomicUsize::new(0) }; 11];
impl HooksManager {
    pub fn get() -> &'static Self {
        static MANAGER: OnceLock<HooksManager> = OnceLock::new();
        MANAGER.get_or_init(|| Self {
            state: Mutex::new(HookState::default()),
            callbacks: RwLock::new(BTreeMap::new()),
            #[cfg(windows)]
            wnd_callback: RwLock::new(None),
            #[cfg(windows)]
            window: Mutex::new(None),
        })
    }
    pub fn set_callback(&self, kind: EventKind, callback: Option<Callback>) {
        let mut c = self.callbacks.write().unwrap_or_else(|e| e.into_inner());
        if let Some(callback) = callback {
            c.insert(kind, callback);
        } else {
            c.remove(&kind);
        }
    }
    fn dispatch(&self, event: HookEvent) {
        let callback = self
            .callbacks
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .get(&event.kind)
            .cloned();
        if let Some(callback) = callback {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| callback(event)));
        }
    }
    #[cfg(windows)]
    pub fn set_on_dx9_present(&self, callback: Option<Dx9PresentCallback>) {
        self.set_callback(
            EventKind::Dx9Present,
            callback.map(|cb| {
                Arc::new(move |e: HookEvent| {
                    cb(
                        e.object,
                        e.args[0] as _,
                        e.args[1] as _,
                        e.args[2] as _,
                        e.args[3] as _,
                    )
                }) as Callback
            }),
        );
    }
    #[cfg(windows)]
    pub fn set_on_dx9_reset(&self, callback: Option<Dx9ResetCallback>) {
        self.set_callback(
            EventKind::Dx9Reset,
            callback
                .map(|cb| Arc::new(move |e: HookEvent| cb(e.object, e.args[0] as _)) as Callback),
        );
    }
    #[cfg(windows)]
    pub fn set_on_dx9_end_scene(&self, callback: Option<Dx9EndSceneCallback>) {
        self.set_callback(
            EventKind::Dx9EndScene,
            callback.map(|cb| Arc::new(move |e: HookEvent| cb(e.object)) as Callback),
        );
    }
    #[cfg(windows)]
    pub fn set_on_present(&self, callback: Option<PresentCallback>) {
        self.set_callback(
            EventKind::Present,
            callback.map(|cb| {
                Arc::new(move |e: HookEvent| cb(e.object, e.args[0] as _, e.args[1] as _))
                    as Callback
            }),
        );
    }
    #[cfg(windows)]
    pub fn set_on_resize_buffers(&self, callback: Option<ResizeBuffersCallback>) {
        self.set_callback(
            EventKind::ResizeBuffers,
            callback.map(|cb| {
                Arc::new(move |e: HookEvent| {
                    cb(
                        e.object,
                        e.args[0] as _,
                        e.args[1] as _,
                        e.args[2] as _,
                        e.args[3] as _,
                        e.args[4] as _,
                    )
                }) as Callback
            }),
        );
    }
    #[cfg(windows)]
    pub fn set_on_execute_command_lists(&self, callback: Option<ExecuteCommandListsCallback>) {
        self.set_callback(
            EventKind::ExecuteCommandLists,
            callback.map(|cb| {
                Arc::new(move |e: HookEvent| cb(e.object, e.args[0] as _, e.args[1] as _))
                    as Callback
            }),
        );
    }
    /// # Safety
    /// All affected graphics functions and detours must be quiescent.
    pub unsafe fn unhook_dx9(&self) -> io::Result<()> {
        unsafe { self.unhook(HookKind::Dx9) }
    }
    /// # Safety
    /// All affected graphics functions and detours must be quiescent.
    pub unsafe fn unhook_dx11(&self) -> io::Result<()> {
        unsafe { self.unhook(HookKind::Dx11) }
    }
    /// # Safety
    /// All affected graphics functions and detours must be quiescent.
    pub unsafe fn unhook_dx12(&self) -> io::Result<()> {
        unsafe { self.unhook(HookKind::Dx12) }
    }
    /// # Safety
    /// All affected graphics functions and detours must be quiescent.
    pub unsafe fn unhook_opengl(&self) -> io::Result<()> {
        unsafe { self.unhook(HookKind::OpenGl) }
    }
    #[cfg(windows)]
    pub fn set_on_opengl_swap_buffers(&self, callback: Option<OpenGlSwapBuffersCallback>) {
        self.set_callback(
            EventKind::OpenGlSwapBuffers,
            callback.map(|cb| Arc::new(move |e: HookEvent| cb(e.object)) as Callback),
        );
    }
    #[cfg(target_os = "linux")]
    pub fn set_on_opengl_swap_buffers(&self, callback: Option<OpenGlSwapBuffersCallback>) {
        self.set_callback(
            EventKind::OpenGlSwapBuffers,
            callback.map(|cb| {
                Arc::new(move |e: HookEvent| cb(e.object, e.args[0] as libc::c_ulong)) as Callback
            }),
        );
    }
    pub fn is_hooked(&self, kind: HookKind) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .groups
            .contains_key(&kind)
    }
    /// # Safety
    /// Every target/detour pair must have the same ABI and signature. All participating
    /// threads must be outside these functions throughout installation and removal.
    unsafe fn install(&self, kind: HookKind, targets: &[(usize, usize, usize)]) -> io::Result<()> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.groups.contains_key(&kind) {
            return Ok(());
        }
        let mut hooks = Vec::new();
        for &(slot, target, detour) in targets {
            let hook =
                unsafe { RawDetour::new(target as _, detour as _) }.map_err(io::Error::other)?;
            ORIGINALS[slot].store(hook.trampoline() as *const () as usize, Ordering::Release);
            hooks.push((slot, hook));
        }
        for (_, hook) in &hooks {
            if let Err(error) = unsafe { hook.enable() } {
                for (_, hook) in &hooks {
                    let _ = unsafe { hook.disable() };
                }
                return Err(io::Error::other(error));
            }
        }
        state.groups.insert(kind, hooks);
        Ok(())
    }
    /// # Safety
    /// No thread may be running the affected detours or originals. Do not call from a callback.
    pub unsafe fn unhook(&self, kind: HookKind) -> io::Result<()> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(hooks) = state.groups.get(&kind) {
            for (_, hook) in hooks {
                unsafe { hook.disable() }.map_err(io::Error::other)?;
            }
        }
        if let Some(hooks) = state.groups.remove(&kind) {
            for (slot, _) in &hooks {
                ORIGINALS[*slot].store(0, Ordering::Release);
            }
            drop(hooks);
        }
        Ok(())
    }
    /// # Safety
    /// `device` must be a live IDirect3DDevice9 and all target functions must be quiescent.
    #[cfg(windows)]
    pub unsafe fn hook_dx9_device(&self, device: *mut c_void) -> io::Result<()> {
        unsafe {
            self.install(
                HookKind::Dx9,
                &[
                    (0, vtable(device, 17), dx9_present as *const () as usize),
                    (1, vtable(device, 16), dx9_reset as *const () as usize),
                    (2, vtable(device, 42), dx9_end_scene as *const () as usize),
                ],
            )
        }
    }
    /// # Safety
    /// `swap_chain` must be a live IDXGISwapChain. Graphics threads must be quiescent.
    #[cfg(windows)]
    pub unsafe fn hook_dx11_swap_chain(&self, swap_chain: *mut c_void) -> io::Result<()> {
        unsafe {
            self.install(
                HookKind::Dx11,
                &[
                    (3, vtable(swap_chain, 8), dx11_present as *const () as usize),
                    (4, vtable(swap_chain, 13), dx11_resize as *const () as usize),
                ],
            )
        }
    }
    /// # Safety
    /// The pointers must be a live IDXGISwapChain and ID3D12CommandQueue. Graphics threads must be quiescent.
    #[cfg(windows)]
    pub unsafe fn hook_dx12_objects(
        &self,
        swap_chain: *mut c_void,
        queue: *mut c_void,
    ) -> io::Result<()> {
        unsafe {
            self.install(
                HookKind::Dx12,
                &[
                    (5, vtable(swap_chain, 8), dx12_present as *const () as usize),
                    (6, vtable(swap_chain, 13), dx12_resize as *const () as usize),
                    (7, vtable(queue, 10), execute as *const () as usize),
                ],
            )
        }
    }
    /// Discover DX9 functions using a temporary device.
    /// # Safety
    /// All threads executing the target functions must be quiescent.
    #[cfg(windows)]
    pub unsafe fn hook_dx9(&self) -> io::Result<()> {
        unsafe { discovery::dx9(self) }
    }
    /// Discover DX11 functions using a temporary swap chain.
    /// # Safety
    /// All threads executing the target functions must be quiescent.
    #[cfg(windows)]
    pub unsafe fn hook_dx11(&self) -> io::Result<()> {
        unsafe { discovery::dx11(self) }
    }
    /// Discover DX12 functions using a temporary command queue and swap chain.
    /// # Safety
    /// All threads executing the target functions must be quiescent.
    #[cfg(windows)]
    pub unsafe fn hook_dx12(&self) -> io::Result<()> {
        unsafe { discovery::dx12(self) }
    }
    /// # Safety
    /// OpenGL swap functions must be quiescent. Call only while the application uses OpenGL.
    pub unsafe fn hook_opengl(&self) -> io::Result<()> {
        if self.is_hooked(HookKind::OpenGl) {
            return Ok(());
        }
        #[cfg(windows)]
        {
            let gl =
                unsafe { libloading::Library::new("opengl32.dll") }.map_err(io::Error::other)?;
            let gdi = unsafe { libloading::Library::new("gdi32.dll") }.map_err(io::Error::other)?;
            let wgl = *unsafe { gl.get::<*const c_void>(b"wglSwapBuffers\0") }
                .map_err(io::Error::other)? as usize;
            let swap = *unsafe { gdi.get::<*const c_void>(b"SwapBuffers\0") }
                .map_err(io::Error::other)? as usize;
            unsafe {
                self.install(
                    HookKind::OpenGl,
                    &[
                        (8, wgl, opengl_wgl as *const () as usize),
                        (9, swap, opengl_gdi as *const () as usize),
                    ],
                )
            }?;
            self.state
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .libraries
                .extend([gl, gdi]);
            Ok(())
        }
        #[cfg(target_os = "linux")]
        {
            let gl = unsafe { libloading::Library::new("libGL.so.1") }.map_err(io::Error::other)?;
            let target = *unsafe { gl.get::<*const c_void>(b"glXSwapBuffers\0") }
                .map_err(io::Error::other)? as usize;
            unsafe {
                self.install(
                    HookKind::OpenGl,
                    &[(8, target, opengl_glx as *const () as usize)],
                )
            }?;
            self.state
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .libraries
                .push(gl);
            Ok(())
        }
        #[cfg(not(any(windows, target_os = "linux")))]
        {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "OpenGL hooks require Windows or Linux",
            ))
        }
    }
    /// # Safety
    /// Must run on the window's owner thread. `hwnd` must remain alive until unhooked.
    #[cfg(windows)]
    pub unsafe fn hook_wnd_proc(&self, hwnd: usize) -> io::Result<()> {
        use windows::Win32::{
            Foundation::{GetLastError, HWND, SetLastError, WIN32_ERROR},
            UI::WindowsAndMessaging::{GWLP_WNDPROC, SetWindowLongPtrW},
        };
        let mut window = self.window.lock().unwrap_or_else(|e| e.into_inner());
        if window.is_some() {
            return Err(io::Error::other("a window procedure is already hooked"));
        }
        unsafe {
            SetLastError(WIN32_ERROR(0));
        }
        let original =
            unsafe { SetWindowLongPtrW(HWND(hwnd as _), GWLP_WNDPROC, wnd_proc as *const () as _) };
        if original == 0 && unsafe { GetLastError() }.0 != 0 {
            return Err(io::Error::last_os_error());
        }
        ORIGINALS[10].store(original as usize, Ordering::Release);
        *window = Some((hwnd, original as isize));
        Ok(())
    }
    /// # Safety
    /// Run on the window's owner thread, after all callbacks return and before window destruction.
    #[cfg(windows)]
    pub unsafe fn unhook_wnd_proc(&self) -> io::Result<()> {
        use windows::Win32::{
            Foundation::HWND,
            UI::WindowsAndMessaging::{GWLP_WNDPROC, GetWindowLongPtrW, SetWindowLongPtrW},
        };
        let mut window = self.window.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((hwnd, original)) = *window {
            if unsafe { GetWindowLongPtrW(HWND(hwnd as _), GWLP_WNDPROC) }
                != wnd_proc as *const () as _
            {
                return Err(io::Error::other(
                    "another component replaced the window procedure",
                ));
            }
            unsafe {
                SetWindowLongPtrW(HWND(hwnd as _), GWLP_WNDPROC, original as _);
            }
            *window = None;
            ORIGINALS[10].store(0, Ordering::Release);
        }
        Ok(())
    }
    #[cfg(windows)]
    pub fn set_on_wnd_proc(&self, callback: Option<WndCallback>) {
        *self.wnd_callback.write().unwrap_or_else(|e| e.into_inner()) = callback;
    }
}
#[cfg(windows)]
unsafe fn vtable(object: *mut c_void, index: usize) -> usize {
    unsafe { *(*(object as *const *const usize)).add(index) }
}
fn event(kind: EventKind, object: *mut c_void, args: [usize; 5]) {
    HooksManager::get().dispatch(HookEvent { kind, object, args });
}
#[cfg(windows)]
unsafe extern "system" fn wnd_proc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    w: windows::Win32::Foundation::WPARAM,
    l: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::UI::WindowsAndMessaging::CallWindowProcW;
    let callback = HooksManager::get()
        .wnd_callback
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    if let Some(cb) = callback {
        if let Ok(Some(result)) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            cb(hwnd.0 as usize, msg, w.0, l.0)
        })) {
            return windows::Win32::Foundation::LRESULT(result);
        }
    }
    let original = ORIGINALS[10].load(Ordering::Acquire);
    unsafe {
        CallWindowProcW(
            Some(std::mem::transmute::<
                usize,
                unsafe extern "system" fn(_, _, _, _) -> _,
            >(original)),
            hwnd,
            msg,
            w,
            l,
        )
    }
}
#[cfg(windows)]
thread_local! {static INSIDE_SWAP:std::cell::Cell<bool>=const{std::cell::Cell::new(false)};}
#[cfg(windows)]
unsafe fn swap(slot: usize, hdc: *mut c_void) -> i32 {
    let outer = INSIDE_SWAP.with(|flag| {
        let outer = !flag.get();
        flag.set(true);
        outer
    });
    if outer {
        event(EventKind::OpenGlSwapBuffers, hdc, [0; 5]);
    }
    let original: unsafe extern "system" fn(*mut c_void) -> i32 =
        unsafe { std::mem::transmute(ORIGINALS[slot].load(Ordering::Acquire)) };
    let result = unsafe { original(hdc) };
    if outer {
        INSIDE_SWAP.with(|flag| flag.set(false));
    }
    result
}
#[cfg(windows)]
unsafe extern "system" fn opengl_wgl(hdc: *mut c_void) -> i32 {
    unsafe { swap(8, hdc) }
}
#[cfg(windows)]
unsafe extern "system" fn opengl_gdi(hdc: *mut c_void) -> i32 {
    unsafe { swap(9, hdc) }
}
#[cfg(target_os = "linux")]
unsafe extern "C" fn opengl_glx(display: *mut c_void, drawable: libc::c_ulong) {
    event(
        EventKind::OpenGlSwapBuffers,
        display,
        [drawable as usize, 0, 0, 0, 0],
    );
    let original: unsafe extern "C" fn(*mut c_void, libc::c_ulong) =
        unsafe { std::mem::transmute(ORIGINALS[8].load(Ordering::Acquire)) };
    unsafe {
        original(display, drawable);
    }
}

#[cfg(windows)]
unsafe extern "system" fn dx9_present(
    a: *mut c_void,
    b: *const c_void,
    c: *const c_void,
    d: *mut c_void,
    e: *const c_void,
) -> i32 {
    event(
        EventKind::Dx9Present,
        a,
        [b as usize, c as usize, d as usize, e as usize, 0],
    );
    let original: unsafe extern "system" fn(
        *mut c_void,
        *const c_void,
        *const c_void,
        *mut c_void,
        *const c_void,
    ) -> i32 = unsafe { std::mem::transmute(ORIGINALS[0].load(Ordering::Acquire)) };
    unsafe { original(a, b, c, d, e) }
}

#[cfg(windows)]
unsafe extern "system" fn dx9_reset(a: *mut c_void, b: *mut c_void) -> i32 {
    event(EventKind::Dx9Reset, a, [b as usize, 0, 0, 0, 0]);
    let original: unsafe extern "system" fn(*mut c_void, *mut c_void) -> i32 =
        unsafe { std::mem::transmute(ORIGINALS[1].load(Ordering::Acquire)) };
    unsafe { original(a, b) }
}

#[cfg(windows)]
unsafe extern "system" fn dx9_end_scene(a: *mut c_void) -> i32 {
    event(EventKind::Dx9EndScene, a, [0, 0, 0, 0, 0]);
    let original: unsafe extern "system" fn(*mut c_void) -> i32 =
        unsafe { std::mem::transmute(ORIGINALS[2].load(Ordering::Acquire)) };
    unsafe { original(a) }
}

#[cfg(windows)]
unsafe extern "system" fn dx11_present(a: *mut c_void, b: u32, c: u32) -> i32 {
    event(EventKind::Present, a, [b as usize, c as usize, 0, 0, 0]);
    let original: unsafe extern "system" fn(*mut c_void, u32, u32) -> i32 =
        unsafe { std::mem::transmute(ORIGINALS[3].load(Ordering::Acquire)) };
    unsafe { original(a, b, c) }
}

#[cfg(windows)]
unsafe extern "system" fn dx11_resize(
    a: *mut c_void,
    b: u32,
    c: u32,
    d: u32,
    e: i32,
    f: u32,
) -> i32 {
    event(
        EventKind::ResizeBuffers,
        a,
        [b as usize, c as usize, d as usize, e as usize, f as usize],
    );
    let original: unsafe extern "system" fn(*mut c_void, u32, u32, u32, i32, u32) -> i32 =
        unsafe { std::mem::transmute(ORIGINALS[4].load(Ordering::Acquire)) };
    unsafe { original(a, b, c, d, e, f) }
}

#[cfg(windows)]
unsafe extern "system" fn dx12_present(a: *mut c_void, b: u32, c: u32) -> i32 {
    event(EventKind::Present, a, [b as usize, c as usize, 0, 0, 0]);
    let original: unsafe extern "system" fn(*mut c_void, u32, u32) -> i32 =
        unsafe { std::mem::transmute(ORIGINALS[5].load(Ordering::Acquire)) };
    unsafe { original(a, b, c) }
}

#[cfg(windows)]
unsafe extern "system" fn dx12_resize(
    a: *mut c_void,
    b: u32,
    c: u32,
    d: u32,
    e: i32,
    f: u32,
) -> i32 {
    event(
        EventKind::ResizeBuffers,
        a,
        [b as usize, c as usize, d as usize, e as usize, f as usize],
    );
    let original: unsafe extern "system" fn(*mut c_void, u32, u32, u32, i32, u32) -> i32 =
        unsafe { std::mem::transmute(ORIGINALS[6].load(Ordering::Acquire)) };
    unsafe { original(a, b, c, d, e, f) }
}

#[cfg(windows)]
unsafe extern "system" fn execute(a: *mut c_void, b: u32, c: *const *mut c_void) {
    event(
        EventKind::ExecuteCommandLists,
        a,
        [b as usize, c as usize, 0, 0, 0],
    );
    let original: unsafe extern "system" fn(*mut c_void, u32, *const *mut c_void) -> () =
        unsafe { std::mem::transmute(ORIGINALS[7].load(Ordering::Acquire)) };
    unsafe { original(a, b, c) }
}
