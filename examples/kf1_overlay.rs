//! Read-only KF1 1065 observer: reflected actor tracking, health, radar, names and camera projection.
#[cfg(all(windows, feature = "hooks", feature = "imgui"))]
#[allow(dead_code)]
#[path = "support/desktop.rs"]
mod desktop;
#[path = "support/kf1/mod.rs"]
mod kf1;
#[cfg(all(windows, feature = "hooks", feature = "imgui"))]
fn main() -> Result<(), std::boxed::Box<dyn std::error::Error>> {
    use kf1::{overlay::*, reader::*};
    use std::time::{Duration, Instant};
    use windows::Win32::{
        Foundation::*,
        Graphics::OpenGL::*,
        UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
    };
    let target = platform::attach()?;
    let game = platform::find_main_window(target.process_id).ok_or("game has no visible window")?;
    let classes = target.find_reflection_classes();
    if !classes.complete() {
        return Err(
            "Actor/PlayerController/Pawn/Property classes are not loaded; enter a level first"
                .into(),
        );
    }
    let mut player = target.find_local_player(&classes);
    let mut tracker = ActorTracker::default();
    let mut settings = EspSettings::default();
    let mut menu = false;
    let mut end = false;
    let mut insert = false;
    let window = desktop::Window::new(false)?;
    unsafe {
        SetWindowLongPtrW(window.hwnd, GWL_STYLE, WS_POPUP.0 as _);
        SetWindowLongPtrW(
            window.hwnd,
            GWL_EXSTYLE,
            (WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT).0 as _,
        );
        SetLayeredWindowAttributes(window.hwnd, COLORREF(0), 255, LWA_COLORKEY)?;
    }
    let mut context = imgui::Context::create();
    context.set_ini_filename(None);
    let texture = desktop::create_font_texture(&mut context);
    context.fonts().tex_id = imgui::TextureId::new(texture as usize);
    let mut last = Instant::now();
    let mut search = Instant::now() - PLAYER_SEARCH_INTERVAL;
    let result = (|| -> Result<(), std::boxed::Box<dyn std::error::Error>> {
        loop {
            let mut message = MSG::default();
            while unsafe { PeekMessageW(&mut message, None, 0, 0, PM_REMOVE) }.as_bool() {
                if message.message == WM_QUIT {
                    return Ok(());
                }
                unsafe {
                    let _ = TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
            let foreground = unsafe { GetForegroundWindow() } == game;
            let quit = platform::key_pressed(VK_END.0 as i32, &mut end);
            let toggle = platform::key_pressed(VK_INSERT.0 as i32, &mut insert);
            if foreground && quit {
                return Ok(());
            }
            if foreground && toggle {
                menu = !menu;
                unsafe {
                    SetWindowLongPtrW(
                        window.hwnd,
                        GWL_EXSTYLE,
                        (WS_EX_LAYERED
                            | WS_EX_TOOLWINDOW
                            | WS_EX_NOACTIVATE
                            | if menu {
                                WINDOW_EX_STYLE(0)
                            } else {
                                WS_EX_TRANSPARENT
                            })
                        .0 as _,
                    );
                }
            }
            let (x, y, width, height) = platform::window_rect(game)?;
            if width <= 0 || height <= 0 || !foreground {
                unsafe {
                    let _ = ShowWindow(window.hwnd, SW_HIDE);
                }
                std::thread::sleep(Duration::from_millis(40));
                continue;
            }
            unsafe {
                SetWindowPos(
                    window.hwnd,
                    Some(HWND_TOPMOST),
                    x,
                    y,
                    width,
                    height,
                    SWP_NOACTIVATE | SWP_SHOWWINDOW,
                )?;
            }
            refresh_local_player(&target, &classes, &mut player, &mut search);
            tracker.advance(&target, &classes);
            let zeds = read_living_zeds(&target, &tracker, &player);
            let allies = read_living_allies(&target, &tracker, &player);
            let io = context.io_mut();
            io.display_size = [width as f32, height as f32];
            io.font_global_scale = settings.font_size / 13.0;
            io.update_delta_time(last.elapsed());
            last = Instant::now();
            if menu {
                let mut mouse = POINT::default();
                unsafe {
                    GetCursorPos(&mut mouse)?;
                    let _ = windows::Win32::Graphics::Gdi::ScreenToClient(window.hwnd, &mut mouse);
                }
                io.add_mouse_pos_event([mouse.x as f32, mouse.y as f32]);
                for (button, key) in [
                    (imgui::MouseButton::Left, VK_LBUTTON),
                    (imgui::MouseButton::Right, VK_RBUTTON),
                ] {
                    io.add_mouse_button_event(
                        button,
                        unsafe { GetAsyncKeyState(key.0 as i32) } < 0,
                    );
                }
            }
            let ui = context.frame();
            if menu {
                kf1::overlay::menu(ui, &mut settings, &tracker);
            }
            let _font = settings.font.map(|font| ui.push_font(font));
            {
                let mut r = omath::hud::ImguiHudRenderer::new(ui);
                draw_overlay(
                    &mut r,
                    &target,
                    WindowRect {
                        x,
                        y,
                        width,
                        height,
                    },
                    &player,
                    &tracker,
                    &zeds,
                    &allies,
                    &settings,
                );
            }
            drop(_font);
            unsafe {
                desktop::render_with_background(context.render(), width, height, [0.0; 4]);
                SwapBuffers(window.dc)?;
            }
            std::thread::sleep(Duration::from_millis(8));
        }
    })();
    unsafe {
        glDeleteTextures(1, &texture);
    }
    result
}
#[cfg(not(all(windows, feature = "hooks", feature = "imgui")))]
fn main() {
    println!("Run on Windows with --features hooks,imgui; KF1 build 1065 must be running.");
}
