//! Run the hook setup inside the process that owns the graphics context.
#[cfg(all(
    feature = "hooks",
    any(windows, target_os = "linux"),
    any(target_arch = "x86", target_arch = "x86_64")
))]
fn main() -> Result<(), std::boxed::Box<dyn std::error::Error>> {
    use omath::hooks::*;
    use std::sync::Arc;
    let api = std::env::args().nth(1).unwrap_or_default();
    if api.is_empty() {
        println!("usage: graphics_hook <opengl|dx9|dx11|dx12>");
        return Ok(());
    }
    let manager = HooksManager::get();
    for kind in [
        EventKind::Present,
        EventKind::Dx9Present,
        EventKind::OpenGlSwapBuffers,
    ] {
        manager.set_callback(kind, Some(Arc::new(|event| eprintln!("{:?}", event.kind))));
    }
    // This standalone example owns its only thread, satisfying the installation/removal contract.
    let kind = unsafe {
        match api.as_str() {
            "opengl" => {
                manager.hook_opengl()?;
                HookKind::OpenGl
            }
            #[cfg(windows)]
            "dx9" => {
                manager.hook_dx9()?;
                HookKind::Dx9
            }
            #[cfg(windows)]
            "dx11" => {
                manager.hook_dx11()?;
                HookKind::Dx11
            }
            #[cfg(windows)]
            "dx12" => {
                manager.hook_dx12()?;
                HookKind::Dx12
            }
            _ => return Err("unsupported graphics API on this platform".into()),
        }
    };
    println!("Installed {kind:?}; press Enter to remove.");
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    unsafe {
        manager.unhook(kind)?;
    }
    Ok(())
}
#[cfg(not(all(
    feature = "hooks",
    any(windows, target_os = "linux"),
    any(target_arch = "x86", target_arch = "x86_64")
)))]
fn main() {
    println!("Enable --features hooks on x86/x64 Windows or Linux.");
}
