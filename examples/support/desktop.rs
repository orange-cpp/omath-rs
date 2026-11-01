use imgui::{Condition, DrawCmd, MouseButton, TextureId};
use omath::{
    Color, Vector2, Vector3,
    engines::source,
    hud::{widget::*, *},
    projection::{FieldOfView, ViewPort},
};
use std::{error::Error, mem::size_of, time::Instant};
use windows::{
    Win32::{
        Foundation::*,
        Graphics::{Gdi::*, OpenGL::*},
        UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
    },
    core::w,
};

pub struct Window {
    pub hwnd: HWND,
    pub dc: HDC,
    context: HGLRC,
}
impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            let _ = wglMakeCurrent(self.dc, HGLRC::default());
            let _ = wglDeleteContext(self.context);
            ReleaseDC(Some(self.hwnd), self.dc);
            let _ = DestroyWindow(self.hwnd);
        }
    }
}
unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        WM_CLOSE => {
            unsafe {
                PostQuitMessage(0);
            }
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, w, l) },
    }
}
impl Window {
    pub fn new(visible: bool) -> Result<Self, std::boxed::Box<dyn Error>> {
        unsafe {
            let class = WNDCLASSW {
                lpfnWndProc: Some(window_proc),
                lpszClassName: w!("OmathRustShowcase"),
                style: CS_OWNDC,
                ..Default::default()
            };
            if RegisterClassW(&class) == 0 {
                return Err(windows::core::Error::from_win32().into());
            }
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                class.lpszClassName,
                w!("omath Rust showcase"),
                WS_OVERLAPPEDWINDOW,
                80,
                80,
                1100,
                800,
                None,
                None,
                None,
                None,
            )?;
            let dc = GetDC(Some(hwnd));
            let pixel = PIXELFORMATDESCRIPTOR {
                nSize: size_of::<PIXELFORMATDESCRIPTOR>() as u16,
                nVersion: 1,
                dwFlags: PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
                iPixelType: PFD_TYPE_RGBA,
                cColorBits: 32,
                cAlphaBits: 8,
                cDepthBits: 24,
                iLayerType: PFD_MAIN_PLANE.0 as u8,
                ..Default::default()
            };
            let format = ChoosePixelFormat(dc, &pixel);
            if format == 0 {
                ReleaseDC(Some(hwnd), dc);
                let _ = DestroyWindow(hwnd);
                return Err(windows::core::Error::from_win32().into());
            }
            if let Err(e) = SetPixelFormat(dc, format, &pixel) {
                ReleaseDC(Some(hwnd), dc);
                let _ = DestroyWindow(hwnd);
                return Err(e.into());
            }
            let context = match wglCreateContext(dc) {
                Ok(c) => c,
                Err(e) => {
                    ReleaseDC(Some(hwnd), dc);
                    let _ = DestroyWindow(hwnd);
                    return Err(e.into());
                }
            };
            let result = Self { hwnd, dc, context };
            wglMakeCurrent(dc, context)?;
            if visible {
                let _ = ShowWindow(hwnd, SW_SHOW);
            }
            Ok(result)
        }
    }
}
fn point(x: f32, y: f32) -> Point {
    Vector2::new(x, y)
}
struct State {
    mode: i32,
    health: f32,
    fov: f32,
    yaw: f32,
    pitch: f32,
    u: f32,
    v: f32,
    glow: bool,
    animate: bool,
}
impl Default for State {
    fn default() -> Self {
        Self {
            mode: 0,
            health: 0.75,
            fov: 90.0,
            yaw: 0.0,
            pitch: 0.0,
            u: 0.25,
            v: 0.25,
            glow: true,
            animate: true,
        }
    }
}
fn scene(ui: &imgui::Ui, state: &mut State, size: Point) {
    ui.window("omath controls")
        .position([16.0, 16.0], Condition::FirstUseEver)
        .size([270.0, 310.0], Condition::FirstUseEver)
        .build(|| {
            ui.radio_button("HUD", &mut state.mode, 0);
            ui.same_line();
            ui.radio_button("Barycentric", &mut state.mode, 1);
            ui.radio_button("Projection", &mut state.mode, 2);
            ui.separator();
            match state.mode {
                0 => {
                    ui.slider("Health", 0.0, 1.0, &mut state.health);
                    ui.checkbox("Glow", &mut state.glow);
                    ui.checkbox("Animate gradient", &mut state.animate);
                }
                1 => {
                    ui.slider("U", 0.0, 1.0, &mut state.u);
                    ui.slider("V", 0.0, 1.0 - state.u, &mut state.v);
                    state.v = state.v.min(1.0 - state.u);
                    ui.text(format!("W = {:.3}", 1.0 - state.u - state.v));
                }
                _ => {
                    ui.slider("FOV", 30.0, 130.0, &mut state.fov);
                    ui.slider("Yaw", -50.0, 50.0, &mut state.yaw);
                    ui.slider("Pitch", -35.0, 35.0, &mut state.pitch);
                }
            }
            ui.separator();
            ui.text("All scene geometry uses the Rust port.");
            ui.text("Close the window to exit.");
        });
    let mut r = ImguiHudRenderer::new(ui);
    match state.mode {
        0 => {
            let x = size.x * 0.62;
            let top = point(x, size.y * 0.2);
            let bottom = point(x, size.y * 0.82);
            let mut entity = EntityOverlay::new(top, bottom, 4.0);
            let mut gradient = Gradient::new(
                Color::new(0.3, 0.2, 0.8, 0.18),
                Color::new(0.1, 0.7, 0.9, 0.25),
                Color::new(0.1, 0.2, 0.4, 0.08),
                Color::new(0.6, 0.2, 0.8, 0.12),
            );
            gradient.animated = state.animate;
            let glow = state.glow.then_some(Glow {
                color: Color::new(0.1, 0.7, 1.0, 0.5),
                radius: 8.0,
                intensity: 0.7,
            });
            entity.contents(
                &mut r,
                [
                    EntityWidget::Box(Box {
                        color: Color::new(0.1, 0.7, 1.0, 1.0),
                        fill: gradient.into(),
                        outline: Color::new(0.0, 0.0, 0.0, 1.0),
                        glow,
                        ..Default::default()
                    }),
                    EntityWidget::Skeleton(Skeleton {
                        color: Color::new(0.9, 0.9, 1.0, 0.9),
                        ..Default::default()
                    }),
                    EntityWidget::Side(
                        Side::Left,
                        vec![SideWidget::Bar(Bar {
                            color: BarGradient {
                                empty_color: Color::RED,
                                full_color: Color::GREEN,
                            }
                            .into(),
                            outline: Color::new(0.0, 0.0, 0.0, 1.0),
                            bg: Color::new(0.05, 0.05, 0.05, 1.0),
                            size: 6.0,
                            ratio: state.health,
                            offset: 8.0,
                            glow,
                        })],
                    ),
                    EntityWidget::Side(
                        Side::Top,
                        vec![SideWidget::Centered(Label {
                            color: Color::new(0.8, 0.9, 1.0, 1.0).into(),
                            text: "Rust entity overlay".into(),
                            offset: 10.0,
                            outlined: Outlined::On,
                            ..Default::default()
                        })],
                    ),
                    EntityWidget::Side(
                        Side::Right,
                        vec![
                            SideWidget::Label(Label {
                                color: Color::GREEN.into(),
                                text: format!("{:.0} HP", state.health * 100.0),
                                offset: 10.0,
                                ..Default::default()
                            }),
                            SideWidget::ProgressRing(ProgressRing {
                                color: Color::GREEN,
                                bg: Color::new(0.2, 0.2, 0.2, 0.7),
                                radius: 18.0,
                                ratio: state.health,
                                thickness: 3.0,
                                offset: 10.0,
                                segments: 32,
                            }),
                        ],
                    ),
                ],
            );
            ScreenOverlay::new(size).contents(
                &mut r,
                [
                    ScreenWidget::Crosshair(Crosshair {
                        color: Color::new(1.0, 1.0, 1.0, 0.8),
                        dot_radius: 1.5,
                        ..Default::default()
                    }),
                    ScreenWidget::FovCircle(FovCircle {
                        color: Color::new(0.7, 0.8, 1.0, 0.35),
                        radius: 90.0,
                        fill: Color::default().into(),
                        thickness: 1.0,
                        segments: 64,
                    }),
                ],
            );
        }
        1 => {
            let a = point(size.x * 0.65, size.y * 0.17);
            let b = point(size.x * 0.35, size.y * 0.82);
            let c = point(size.x * 0.92, size.y * 0.82);
            let u = state.u;
            let v = state.v;
            let w = 1.0 - u - v;
            let p = a * u + b * v + c * w;
            r.add_polyline(&[a, b, c], Color::new(0.5, 0.6, 0.8, 1.0), 2.0);
            for (vertex, color, label) in [
                (a, Color::RED, "A"),
                (b, Color::GREEN, "B"),
                (c, Color::BLUE, "C"),
            ] {
                r.add_line(p, vertex, color, 1.0);
                r.add_filled_circle(vertex, 6.0, color, 16);
                r.add_text(vertex + point(10.0, 0.0), color, label);
            }
            r.add_filled_circle(p, 8.0, Color::new(u, v, w, 1.0), 24);
            r.add_text(
                p + point(12.0, -12.0),
                Color::new(1.0, 1.0, 1.0, 1.0),
                &format!("({u:.2}, {v:.2}, {w:.2})"),
            );
        }
        _ => {
            let angles = source::ViewAngles::new(
                source::PitchAngle::from_degrees(state.pitch),
                source::YawAngle::from_degrees(state.yaw),
                source::RollAngle::from_degrees(0.0),
            );
            let camera = source::Camera::new(
                Vector3::new(0.0, 0.0, 30.0),
                angles,
                ViewPort::new(size.x, size.y),
                FieldOfView::from_degrees(state.fov),
                0.1,
                10000.0,
            );
            for i in -10..=10 {
                let y = i as f32 * 20.0;
                for (a, b) in [
                    (Vector3::new(40.0, y, 0.0), Vector3::new(400.0, y, 0.0)),
                    (
                        Vector3::new(40.0 + (i + 10) as f32 * 18.0, -200.0, 0.0),
                        Vector3::new(40.0 + (i + 10) as f32 * 18.0, 200.0, 0.0),
                    ),
                ] {
                    if let (Ok(a), Ok(b)) = (
                        camera.world_to_screen_unclipped(a),
                        camera.world_to_screen_unclipped(b),
                    ) {
                        r.add_line(
                            point(a.x, a.y),
                            point(b.x, b.y),
                            Color::new(0.2, 0.4, 0.6, 1.0),
                            1.0,
                        );
                    }
                }
            }
            let aabb = omath::primitives::Aabb::<f32>::new(
                Vector3::new(160.0, -25.0, 0.0),
                Vector3::new(210.0, 25.0, 70.0),
            );
            if let Ok(mut overlay) = EntityOverlay::from_aabb(&camera, &aabb) {
                overlay.draw(
                    &mut r,
                    &EntityWidget::Box(Box {
                        color: Color::GREEN,
                        ..Default::default()
                    }),
                );
            }
        }
    }
}
pub unsafe fn render(data: &imgui::DrawData, width: i32, height: i32) {
    unsafe {
        render_with_background(data, width, height, [0.035, 0.045, 0.065, 1.0]);
    }
}
pub unsafe fn render_with_background(
    data: &imgui::DrawData,
    width: i32,
    height: i32,
    background: [f32; 4],
) {
    unsafe {
        glViewport(0, 0, width, height);
        glClearColor(background[0], background[1], background[2], background[3]);
        glClear(GL_COLOR_BUFFER_BIT);
        glEnable(GL_BLEND);
        glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
        glDisable(GL_CULL_FACE);
        glDisable(GL_DEPTH_TEST);
        glEnable(GL_SCISSOR_TEST);
        glEnable(GL_TEXTURE_2D);
        glMatrixMode(GL_PROJECTION);
        glLoadIdentity();
        glOrtho(0.0, width as f64, height as f64, 0.0, -1.0, 1.0);
        glMatrixMode(GL_MODELVIEW);
        glLoadIdentity();
        for list in data.draw_lists() {
            for command in list.commands() {
                if let DrawCmd::Elements { count, cmd_params } = command {
                    let [x1, y1, x2, y2] = cmd_params.clip_rect;
                    let x = x1.max(0.0) as i32;
                    let y = y1.max(0.0) as i32;
                    let right = (x2 as i32).min(width);
                    let bottom = (y2 as i32).min(height);
                    if right <= x || bottom <= y {
                        continue;
                    }
                    glScissor(x, height - bottom, right - x, bottom - y);
                    glBindTexture(GL_TEXTURE_2D, cmd_params.texture_id.id() as u32);
                    glBegin(GL_TRIANGLES);
                    for index in
                        &list.idx_buffer()[cmd_params.idx_offset..cmd_params.idx_offset + count]
                    {
                        let vertex = list.vtx_buffer()[cmd_params.vtx_offset + *index as usize];
                        let [red, green, blue, alpha] = vertex.col;
                        glColor4ub(red, green, blue, alpha);
                        glTexCoord2f(vertex.uv[0], vertex.uv[1]);
                        glVertex2f(vertex.pos[0], vertex.pos[1]);
                    }
                    glEnd();
                }
            }
        }
        glDisable(GL_SCISSOR_TEST);
    }
}
pub fn run() -> Result<(), std::boxed::Box<dyn Error>> {
    let frames = std::env::args().skip(1).find_map(|arg| {
        arg.strip_prefix("--frames=")
            .and_then(|s| s.parse::<u32>().ok())
    });
    let window = Window::new(frames.is_none())?;
    let mut context = imgui::Context::create();
    context.set_ini_filename(None);
    let texture = create_font_texture(&mut context);
    context.fonts().tex_id = TextureId::new(texture as usize);
    let mut state = State {
        mode: std::env::args()
            .skip(1)
            .find_map(|arg| {
                arg.strip_prefix("--mode=")
                    .and_then(|s| s.parse::<i32>().ok())
            })
            .unwrap_or(0),
        ..Default::default()
    };
    let hook_test = std::env::args().any(|arg| arg == "--hook-test");
    let swaps = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    if hook_test {
        let swaps = swaps.clone();
        omath::hooks::HooksManager::get().set_on_opengl_swap_buffers(Some(std::sync::Arc::new(
            move |_| {
                swaps.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            },
        )));
        unsafe {
            omath::hooks::HooksManager::get().hook_opengl()?;
        }
    }
    let mut last = Instant::now();
    let mut frame = 0;
    let result = (|| -> Result<(), std::boxed::Box<dyn Error>> {
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
            let mut rect = RECT::default();
            unsafe {
                GetClientRect(window.hwnd, &mut rect)?;
            }
            let width = rect.right.max(1);
            let height = rect.bottom.max(1);
            let io = context.io_mut();
            io.display_size = [width as f32, height as f32];
            io.update_delta_time(last.elapsed());
            last = Instant::now();
            let mut mouse = POINT::default();
            unsafe {
                GetCursorPos(&mut mouse)?;
                let _ = ScreenToClient(window.hwnd, &mut mouse);
            }
            io.add_mouse_pos_event([mouse.x as f32, mouse.y as f32]);
            for (button, key) in [
                (MouseButton::Left, VK_LBUTTON),
                (MouseButton::Right, VK_RBUTTON),
                (MouseButton::Middle, VK_MBUTTON),
            ] {
                io.add_mouse_button_event(button, unsafe { GetAsyncKeyState(key.0 as i32) } < 0);
            }
            scene(
                context.frame(),
                &mut state,
                point(width as f32, height as f32),
            );
            unsafe {
                render(context.render(), width, height);
                SwapBuffers(window.dc)?;
                let error = glGetError();
                if error != GL_NO_ERROR {
                    return Err(format!("OpenGL error {error}").into());
                }
            }
            frame += 1;
            if frames.is_some_and(|limit| frame >= limit) {
                return Ok(());
            }
            std::thread::sleep(std::time::Duration::from_millis(8));
        }
    })();
    unsafe {
        glDeleteTextures(1, &texture);
    }
    if hook_test {
        unsafe {
            omath::hooks::HooksManager::get().unhook_opengl()?;
        }
        omath::hooks::HooksManager::get().set_on_opengl_swap_buffers(None);
        let observed = swaps.load(std::sync::atomic::Ordering::Relaxed);
        if observed != frame as usize {
            return Err(format!("expected {frame} swap callbacks, observed {observed}").into());
        }
    }
    result
}

pub fn create_font_texture(context: &mut imgui::Context) -> u32 {
    unsafe {
        let atlas = context.fonts().build_rgba32_texture();
        let mut texture = 0;
        glGenTextures(1, &mut texture);
        glBindTexture(GL_TEXTURE_2D, texture);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR as i32);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as i32);
        glTexImage2D(
            GL_TEXTURE_2D,
            0,
            GL_RGBA as i32,
            atlas.width as i32,
            atlas.height as i32,
            0,
            GL_RGBA,
            GL_UNSIGNED_BYTE,
            atlas.data.as_ptr().cast(),
        );
        texture
    }
}
