//! Rust DLL/shared-library counterpart of the graphics hook examples.
//! Initialize through the exports after loading, outside the Windows loader lock.
//! The host owns thread quiescence during installation/removal and the renderer callback.
#[cfg(all(
    any(windows, target_os = "linux"),
    any(target_arch = "x86", target_arch = "x86_64")
))]
mod implementation {
    use omath::hooks::*;
    use std::{ffi::c_void, sync::Arc};
    type NativeCallback = unsafe extern "system" fn(u32, *mut c_void, *const usize, usize);
    /// # Safety
    /// The callback must remain valid until replaced/cleared and must not unwind. Pointers
    /// passed to it are borrowed for that call only, with validity defined by the graphics API.
    #[unsafe(no_mangle)]
    pub unsafe extern "system" fn omath_set_graphics_callback(callback: Option<NativeCallback>) {
        for kind in [
            EventKind::Dx9Present,
            EventKind::Dx9Reset,
            EventKind::Dx9EndScene,
            EventKind::Present,
            EventKind::ResizeBuffers,
            EventKind::ExecuteCommandLists,
            EventKind::OpenGlSwapBuffers,
        ] {
            HooksManager::get().set_callback(
                kind,
                callback.map(|callback| {
                    Arc::new(move |event: HookEvent| unsafe {
                        callback(
                            kind as u32,
                            event.object,
                            event.args.as_ptr(),
                            event.args.len(),
                        )
                    }) as Callback
                }),
            );
        }
    }
    /// Modes: 0 OpenGL, 9 DX9, 11 DX11, 12 DX12. Returns 0 on success, -1 on error.
    /// # Safety
    /// Threads must be outside affected graphics functions throughout installation.
    #[unsafe(no_mangle)]
    pub unsafe extern "system" fn omath_start_graphics_hooks(mode: u32) -> i32 {
        let result = std::panic::catch_unwind(|| unsafe {
            match mode {
                0 => HooksManager::get().hook_opengl(),
                #[cfg(windows)]
                9 => HooksManager::get().hook_dx9(),
                #[cfg(windows)]
                11 => HooksManager::get().hook_dx11(),
                #[cfg(windows)]
                12 => HooksManager::get().hook_dx12(),
                _ => Err(std::io::Error::other("unsupported graphics API")),
            }
        });
        if matches!(result, Ok(Ok(()))) { 0 } else { -1 }
    }
    /// # Safety
    /// Graphics originals/detours must be quiescent until this returns. Call before unloading
    /// this library; unloading without successful removal leaves native branches into unmapped code.
    #[unsafe(no_mangle)]
    pub unsafe extern "system" fn omath_stop_graphics_hooks() -> i32 {
        let result = std::panic::catch_unwind(|| unsafe {
            for kind in [
                HookKind::Dx9,
                HookKind::Dx11,
                HookKind::Dx12,
                HookKind::OpenGl,
            ] {
                HooksManager::get().unhook(kind)?;
            }
            Ok::<_, std::io::Error>(())
        });
        if matches!(result, Ok(Ok(()))) { 0 } else { -1 }
    }
}
