#![cfg(all(feature = "hooks", windows))]
use omath::hooks::*;
use std::{
    ffi::c_void,
    hint::black_box,
    sync::{Arc, Mutex},
};
#[inline(never)]
unsafe extern "system" fn end_scene(device: *mut c_void) -> i32 {
    black_box(device as usize) as i32 + 42
}
#[inline(never)]
unsafe extern "system" fn reset(device: *mut c_void, params: *mut c_void) -> i32 {
    black_box(device as usize) as i32 + black_box(params as usize) as i32 + 16
}
#[inline(never)]
unsafe extern "system" fn present(
    device: *mut c_void,
    a: *const c_void,
    b: *const c_void,
    window: *mut c_void,
    region: *const c_void,
) -> i32 {
    black_box(device as usize) as i32
        + black_box(a as usize) as i32
        + black_box(b as usize) as i32
        + black_box(window as usize) as i32
        + black_box(region as usize) as i32
        + 17
}
#[test]
fn dx9_detours_deliver_parameters_forward_results_and_restore_code() {
    let mut table = [0_usize; 43];
    table[16] = reset as *const () as usize;
    table[17] = present as *const () as usize;
    table[42] = end_scene as *const () as usize;
    let object = table.as_ptr();
    let device = (&object as *const *const usize) as *mut c_void;
    let events = Arc::new(Mutex::new(Vec::new()));
    let manager = HooksManager::get();
    for kind in [
        EventKind::Dx9Present,
        EventKind::Dx9Reset,
        EventKind::Dx9EndScene,
    ] {
        let events = events.clone();
        manager.set_callback(
            kind,
            Some(Arc::new(move |e| {
                events.lock().unwrap().push((e.kind, e.args)); // replacing a callback inside a callback must not deadlock
                HooksManager::get().set_callback(e.kind, None);
            })),
        );
    }
    let before = unsafe { black_box(end_scene)(device) };
    unsafe {
        manager.hook_dx9_device(device).unwrap();
    }
    assert!(manager.is_hooked(HookKind::Dx9));
    assert_eq!(unsafe { black_box(end_scene)(device) }, before);
    assert_eq!(
        unsafe { black_box(reset)(device, 3 as _) },
        (device as usize) as i32 + 19
    );
    assert_eq!(
        unsafe { black_box(present)(device, 1 as _, 2 as _, 3 as _, 4 as _) },
        (device as usize) as i32 + 27
    );
    unsafe {
        manager.unhook(HookKind::Dx9).unwrap();
    }
    assert!(!manager.is_hooked(HookKind::Dx9));
    assert_eq!(unsafe { black_box(end_scene)(device) }, before);
    let events = events.lock().unwrap();
    assert_eq!(events.len(), 3);
    assert_eq!(events[2], (EventKind::Dx9Present, [1, 2, 3, 4, 0]));
}

// The manager subclasses one window at a time
static WINDOW: Mutex<()> = Mutex::new(());
#[test]
fn window_proc_intercepts_and_restores_on_owner_thread() {
    use windows::{
        Win32::{Foundation::*, UI::WindowsAndMessaging::*},
        core::w,
    };
    let _window = WINDOW.lock().unwrap_or_else(|e| e.into_inner());
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("omath test"),
            WS_OVERLAPPEDWINDOW,
            0,
            0,
            100,
            100,
            None,
            None,
            None,
            None,
        )
    }
    .unwrap();
    let message = WM_APP + 17;
    let manager = HooksManager::get();
    manager.set_on_wnd_proc(Some(Arc::new(move |_, msg, _, _| {
        (msg == message).then_some(123)
    })));
    unsafe {
        manager.hook_wnd_proc(hwnd.0 as usize).unwrap();
    }
    assert_eq!(
        unsafe { SendMessageW(hwnd, message, Some(WPARAM(0)), Some(LPARAM(0))) }.0,
        123
    );
    unsafe {
        manager.unhook_wnd_proc().unwrap();
    }
    manager.set_on_wnd_proc(None);
    assert_ne!(
        unsafe { SendMessageW(hwnd, message, Some(WPARAM(0)), Some(LPARAM(0))) }.0,
        123
    );
    unsafe {
        DestroyWindow(hwnd).unwrap();
    }
}
#[test]
fn window_proc_keeps_an_ansi_window_ansi() {
    use windows::{
        Win32::{Foundation::*, UI::WindowsAndMessaging::*},
        core::s,
    };
    let _window = WINDOW.lock().unwrap_or_else(|e| e.into_inner());
    let hwnd = unsafe {
        CreateWindowExA(
            WINDOW_EX_STYLE::default(),
            s!("STATIC"),
            s!("omath ansi test"),
            WS_OVERLAPPEDWINDOW,
            0,
            0,
            100,
            100,
            None,
            None,
            None,
            None,
        )
    }
    .unwrap();
    assert!(!unsafe { IsWindowUnicode(hwnd) }.as_bool());
    let message = WM_APP + 18;
    let manager = HooksManager::get();
    manager.set_on_wnd_proc(Some(Arc::new(move |_, msg, _, _| {
        (msg == message).then_some(321)
    })));
    unsafe {
        manager.hook_wnd_proc(hwnd.0 as usize).unwrap();
    }
    // Subclassing through the wide API would have turned the window into a Unicode one
    assert!(!unsafe { IsWindowUnicode(hwnd) }.as_bool());
    assert_eq!(
        unsafe { SendMessageA(hwnd, message, WPARAM(0), LPARAM(0)) }.0,
        321
    );
    unsafe {
        manager.unhook_wnd_proc().unwrap();
    }
    manager.set_on_wnd_proc(None);
    assert!(!unsafe { IsWindowUnicode(hwnd) }.as_bool());
    unsafe {
        DestroyWindow(hwnd).unwrap();
    }
}
#[test]
fn graphics_api_the_process_has_not_loaded_is_not_hooked() {
    use windows::{Win32::System::LibraryLoader::GetModuleHandleW, core::w};
    if unsafe { GetModuleHandleW(w!("d3d12.dll")) }.is_ok() {
        return;
    }
    let error = unsafe { HooksManager::get().hook_dx12() }.unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
    assert!(!HooksManager::get().is_hooked(HookKind::Dx12));
    assert!(unsafe { GetModuleHandleW(w!("d3d12.dll")) }.is_err());
}
