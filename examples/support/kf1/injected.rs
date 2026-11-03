use crate::kf1::{dx9_imgui::Renderer, gameplay::*, glow::ActorGlow, overlay::*, reader::*};
use omath::{Vector3, hooks::*, rev_eng::ProcessMemory};
use retour::RawDetour;
use std::{
    cell::{Cell, RefCell},
    ffi::c_void,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Instant,
};
use windows::{
    Win32::{
        Foundation::*,
        Graphics::Direct3D9::*,
        System::LibraryLoader::*,
        UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
    },
    core::{Interface, PCSTR, w},
};
static HOOKS: Mutex<Vec<RawDetour>> = Mutex::new(Vec::new());
static ORIGINAL: [AtomicUsize; 6] = [const { AtomicUsize::new(0) }; 6];
static STOP: AtomicBool = AtomicBool::new(false);
static MENU: AtomicBool = AtomicBool::new(false);
static INPUT: Mutex<Vec<(u32, usize)>> = Mutex::new(Vec::new());
thread_local! {static RUNTIME:RefCell<Option<Runtime>>=const{RefCell::new(None)};static GROUP:Cell<usize>=const{Cell::new(0)};static LEVEL:Cell<u32>=const{Cell::new(0)};static CUTOUT:Cell<bool>=const{Cell::new(false)};static WATCH:Cell<bool>=const{Cell::new(false)};}
struct Runtime {
    target: Target,
    classes: ReflectionClasses,
    player: LocalPlayer,
    tracker: ActorTracker,
    zeds: Vec<LiveActor>,
    allies: Vec<LiveActor>,
    weapons: Vec<usize>,
    settings: EspSettings,
    aim: AimSettings,
    silent: Option<SilentAim>,
    recoil: FireEdits,
    spread: FireEdits,
    glow: [ActorGlow; 3],
    glow_enabled: bool,
    glow_allies: bool,
    glow_weapons: bool,
    glow_color: [f32; 3],
    weapon_color: [f32; 3],
    thickness: f32,
    // WATCH: the level is done and the HUD overlay is not drawn yet. hud_overlay: the last frame had one, so glows wait
    // for it. glows_waiting: blurred, not laid yet. silhouette: a silhouette was drawn during this level.
    hud_overlay: bool,
    glows_waiting: bool,
    silhouette: bool,
    context: imgui::Context,
    renderer: Renderer,
    device: IDirect3DDevice9,
    hwnd: HWND,
    menu: bool,
    insert: bool,
    end: bool,
    last: Instant,
    search: Instant,
}
impl Runtime {
    fn new(device: &IDirect3DDevice9) -> Result<Self, std::boxed::Box<dyn std::error::Error>> {
        let pid = std::process::id();
        let core = platform::find_module_base(pid, "Core.dll")?;
        let target = Target::new(
            ProcessMemory::open(pid, true)?,
            pid,
            platform::find_module_base(pid, "Engine.dll")?,
            core + G_OBJECTS_RVA,
            core + G_NAMES_RVA,
        )?;
        let classes = target.find_reflection_classes();
        if !classes.complete() {
            return Err("reflection classes are not ready".into());
        }
        let player = target.find_local_player(&classes);
        let mut parameters = D3DDEVICE_CREATION_PARAMETERS::default();
        unsafe {
            device.GetCreationParameters(&mut parameters)?;
        }
        let mut context = imgui::Context::create();
        context.set_ini_filename(None);
        Ok(Self {
            target,
            classes,
            player,
            tracker: ActorTracker::default(),
            zeds: Vec::new(),
            allies: Vec::new(),
            weapons: Vec::new(),
            settings: EspSettings::default(),
            aim: AimSettings::default(),
            silent: None,
            recoil: FireEdits::default(),
            spread: FireEdits::default(),
            glow: Default::default(),
            glow_enabled: true,
            glow_allies: true,
            glow_weapons: true,
            glow_color: [0.31, 0.86, 0.47],
            weapon_color: [1.0; 3],
            thickness: 5.0,
            hud_overlay: false,
            glows_waiting: false,
            silhouette: false,
            context,
            renderer: Renderer::default(),
            device: device.clone(),
            hwnd: parameters.hFocusWindow,
            menu: false,
            insert: false,
            end: false,
            last: Instant::now(),
            search: Instant::now() - PLAYER_SEARCH_INTERVAL,
        })
    }
    fn restore(&mut self) {
        if let Some(silent) = self.silent.take() {
            let _ = silent.restore(&self.target);
        }
        self.recoil.restore(&self.target);
        self.spread.restore(&self.target);
    }
    fn reset(&mut self) {
        self.renderer.release();
        for glow in &mut self.glow {
            glow.release();
        }
    }
    fn present(&mut self) {
        if STOP.load(Ordering::Relaxed) {
            return;
        }
        let foreground = unsafe { GetForegroundWindow() } == self.hwnd;
        if platform::key_pressed(VK_END.0 as i32, &mut self.end) && foreground {
            STOP.store(true, Ordering::Relaxed);
            self.restore();
            self.reset();
            return;
        }
        if platform::key_pressed(VK_INSERT.0 as i32, &mut self.insert) && foreground {
            self.menu = !self.menu;
            MENU.store(self.menu, Ordering::Release);
        }
        let mut desc = D3DSURFACE_DESC::default();
        if unsafe {
            self.device
                .GetRenderTarget(0)
                .and_then(|s| s.GetDesc(&mut desc))
        }
        .is_err()
        {
            return;
        }
        refresh_local_player(
            &self.target,
            &self.classes,
            &mut self.player,
            &mut self.search,
        );
        self.tracker.advance(&self.target, &self.classes);
        self.zeds = read_living_zeds(&self.target, &self.tracker, &self.player);
        self.allies = read_living_allies(&self.target, &self.tracker, &self.player);
        self.weapons = read_ally_weapons(&self.target, &self.tracker, &self.allies);
        if self
            .renderer
            .prepare(&mut self.context, &self.device)
            .is_err()
        {
            return;
        }
        for glow in &mut self.glow {
            glow.prepare(&self.device, desc.Width, desc.Height);
        }
        let io = self.context.io_mut();
        io.display_size = [desc.Width as f32, desc.Height as f32];
        io.update_delta_time(self.last.elapsed());
        self.last = Instant::now();
        io.mouse_draw_cursor = self.menu;
        io.font_global_scale = self.settings.font_size / 13.0;
        for (message, param) in INPUT.lock().unwrap_or_else(|e| e.into_inner()).drain(..) {
            if message == WM_MOUSEWHEEL {
                io.add_mouse_wheel_event([0.0, ((param >> 16) as u16 as i16) as f32 / 120.0]);
            }
            if message == WM_MOUSEHWHEEL {
                io.add_mouse_wheel_event([((param >> 16) as u16 as i16) as f32 / 120.0, 0.0]);
            }
            if message == WM_CHAR {
                if let Some(c) = char::from_u32(param as u32) {
                    io.add_input_character(c);
                }
            }
        }
        let mut mouse = POINT::default();
        if self.menu {
            unsafe {
                let _ = GetCursorPos(&mut mouse);
                let _ = windows::Win32::Graphics::Gdi::ScreenToClient(self.hwnd, &mut mouse);
            }
            io.add_mouse_pos_event([mouse.x as f32, mouse.y as f32]);
        } else {
            io.add_mouse_pos_event([-f32::MAX, -f32::MAX]);
        }
        for (button, key) in [
            (imgui::MouseButton::Left, VK_LBUTTON),
            (imgui::MouseButton::Right, VK_RBUTTON),
            (imgui::MouseButton::Middle, VK_MBUTTON),
        ] {
            io.add_mouse_button_event(
                button,
                self.menu && unsafe { GetAsyncKeyState(key.0 as i32) } < 0,
            );
        }
        let ui = self.context.frame();
        if self.menu {
            crate::kf1::overlay::menu(ui, &mut self.settings, &self.tracker);
            crate::kf1::gameplay::menu(ui, &mut self.aim);
            ui.window("Glow and renderer").build(|| {
                ui.checkbox("Glow", &mut self.glow_enabled);
                ui.checkbox("Ally glow", &mut self.glow_allies);
                ui.checkbox("Ally weapon glow", &mut self.glow_weapons);
                ui.color_edit3("Zed glow", &mut self.glow_color);
                ui.color_edit3("Weapon glow", &mut self.weapon_color);
                ui.slider("Thickness", 1.0, 20.0, &mut self.thickness);
                ui.text("INSERT menu / END disable");
                for glow in &self.glow {
                    if !glow.error.is_empty() {
                        ui.text_wrapped(&glow.error);
                    }
                }
            });
        }
        let mut renderer = omath::hud::ImguiHudRenderer::new(ui);
        draw_overlay(
            &mut renderer,
            &self.target,
            WindowRect {
                x: 0,
                y: 0,
                width: desc.Width as i32,
                height: desc.Height as i32,
            },
            &self.player,
            &self.tracker,
            &self.zeds,
            &self.allies,
            &self.settings,
        );
        drop(renderer);
        if unsafe { self.device.BeginScene() }.is_ok() {
            let _ = self.renderer.render(self.context.render(), &self.device);
            let _ = unsafe { self.device.EndScene() };
        }
    }
    fn aim(&mut self) {
        if STOP.load(Ordering::Relaxed) {
            return;
        }
        if self.aim.no_recoil {
            self.recoil.apply(
                &self.target,
                &self.classes,
                &self.player,
                &["maxVerticalRecoilAngle", "maxHorizontalRecoilAngle"],
                false,
            );
        } else {
            self.recoil.restore(&self.target);
        }
        if self.aim.no_spread {
            self.spread.apply(
                &self.target,
                &self.classes,
                &self.player,
                &["Spread", "MaxSpread"],
                true,
            );
        } else {
            self.spread.restore(&self.target);
        }
        let keys = [
            VK_RBUTTON,
            VK_LBUTTON,
            VK_XBUTTON1,
            VK_XBUTTON2,
            VK_LSHIFT,
            VK_LMENU,
        ];
        if !self.aim.enabled
            || !self.player.valid()
            || unsafe { GetForegroundWindow() } != self.hwnd
            || unsafe { GetAsyncKeyState(keys[self.aim.key.min(5)].0 as i32) } >= 0
        {
            return;
        }
        let wanted = choose_target(
            &self.target,
            &self.player,
            &self.zeds,
            &self.aim,
            |actor| unsafe { head(&self.target, &self.classes, actor) },
            |from, to, actor| unsafe { visible(&self.target, &self.player, from, to, actor) },
        );
        if let Some(wanted) = wanted {
            if self.aim.silent {
                self.silent =
                    SilentAim::begin(&self.target, &self.classes, &self.player, wanted).ok();
            } else {
                let current = self.target.read_camera(&self.player).angles;
                let _ = self.target.write_view_angles(
                    self.player.controller,
                    smooth(current, wanted, self.aim.smoothing),
                    ACTOR_ROTATION_OFFSET,
                );
            }
        }
    }
    fn group(&self, actor: usize) -> usize {
        if !self.glow_enabled {
            return 0;
        }
        if self.zeds.iter().any(|z| z.actor.address == actor) {
            1
        } else if self.glow_allies && self.allies.iter().any(|a| a.actor.address == actor) {
            2
        } else if self.glow_weapons && self.weapons.contains(&actor) {
            3
        } else {
            0
        }
    }
    fn lay(&mut self, device: &IDirect3DDevice9) {
        if STOP.load(Ordering::Relaxed) {
            return;
        }
        let ally = self.settings.ally_color.value();
        for (glow, color) in
            self.glow
                .iter_mut()
                .zip([self.glow_color, [ally.x, ally.y, ally.z], self.weapon_color])
        {
            let _ = glow.lay(device, color);
        }
    }
    // An overlay that did not turn up last frame is one this level or the player's state does without
    fn begin_level(&mut self) {
        if WATCH.with(|v| v.replace(false)) {
            self.hud_overlay = false;
        }
        if std::mem::take(&mut self.glows_waiting) {
            for glow in &mut self.glow {
                glow.discard();
            }
        }
    }
    // Without the HUD overlay nothing multiplies the frame, so the glows go on right here
    fn end_level(&mut self) {
        WATCH.with(|v| v.set(true));
        if !std::mem::take(&mut self.silhouette) || STOP.load(Ordering::Relaxed) {
            return;
        }
        for glow in &mut self.glow {
            let _ = glow.blur(&self.device, self.thickness);
        }
        if self.hud_overlay {
            self.glows_waiting = true;
        } else {
            self.lay(&self.device.clone());
        }
    }
    // HUDKillingFloor::DrawModOverlay's SepiaShader tile: OB_Modulate is the only DESTCOLOR/SRCCOLOR blend
    fn watch_overlay(&mut self, device: &IDirect3DDevice9) {
        let (mut blending, mut source, mut destination) = (0, 0, 0);
        unsafe {
            let _ = device.GetRenderState(D3DRS_ALPHABLENDENABLE, &mut blending);
            let _ = device.GetRenderState(D3DRS_SRCBLEND, &mut source);
            let _ = device.GetRenderState(D3DRS_DESTBLEND, &mut destination);
        }
        if blending == 0
            || source != D3DBLEND_DESTCOLOR.0 as u32
            || destination != D3DBLEND_SRCCOLOR.0 as u32
        {
            return;
        }
        WATCH.with(|v| v.set(false));
        self.hud_overlay = true;
        if std::mem::take(&mut self.glows_waiting) {
            self.lay(device);
        }
    }
}
unsafe fn module_virtual(target: &Target, object: usize, slot: isize) -> usize {
    unsafe fn loaded(address: usize) -> bool {
        let mut module = HMODULE::default();
        address != 0
            && unsafe {
                GetModuleHandleExA(
                    GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS
                        | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
                    PCSTR(address as *const u8),
                    &mut module,
                )
            }
            .is_ok()
    }
    let table = target.read_ptr(object, 0);
    if !unsafe { loaded(table) } {
        return 0;
    }
    let function = target.read_ptr(table, slot);
    if unsafe { loaded(function) } {
        function
    } else {
        0
    }
}
unsafe fn head(target: &Target, classes: &ReflectionClasses, actor: usize) -> Option<Vector3<f64>> {
    unsafe {
        let class = target.read_ptr(actor, OBJECT_CLASS_OFFSET);
        let bone = target.find_property_offset(class, "HeadBone", classes.property)?;
        let height = target.find_property_offset(class, "HeadHeight", classes.property)?;
        let scale = target.find_property_offset(class, "HeadScale", classes.property)?;
        let engine = GetModuleHandleW(w!("Engine.dll")).ok()?;
        let coords = GetProcAddress(
            engine,
            PCSTR(
                c"?GetBoneCoords@USkeletalMeshInstance@@QAE?AVFCoords@@K@Z"
                    .as_ptr()
                    .cast(),
            ),
        )?;
        let pending = module_virtual(target, actor, 0x30);
        if pending == 0
            || std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize) -> i32>(pending)(
                actor,
            ) != 0
        {
            return None;
        }
        let mesh = target.read_ptr(actor, 0xb8);
        if target.object_name(target.read_ptr(mesh, OBJECT_CLASS_OFFSET)) != "SkeletalMesh" {
            return None;
        }
        let get_instance = module_virtual(target, mesh, 0x98);
        if get_instance == 0 {
            return None;
        }
        let instance = std::mem::transmute::<
            usize,
            unsafe extern "thiscall" fn(usize, usize) -> usize,
        >(get_instance)(mesh, actor);
        if instance == 0 {
            return None;
        }
        let match_bone = module_virtual(target, instance, 0x130);
        if match_bone == 0 {
            return None;
        }
        let index = std::mem::transmute::<usize, unsafe extern "thiscall" fn(usize, i32) -> i32>(
            match_bone,
        )(instance, target.read(actor + bone as usize));
        if index < 0 {
            return None;
        }
        let mut output = [0.0f32; 12];
        std::mem::transmute::<
            unsafe extern "system" fn() -> isize,
            unsafe extern "thiscall" fn(usize, *mut f32, u32) -> usize,
        >(coords)(instance, output.as_mut_ptr(), index as u32);
        let along = target.read::<f32>(actor + height as usize)
            * target.read::<f32>(actor + scale as usize);
        Some(Vector3::new(
            (output[0] + output[3] * along) as f64,
            (output[1] + output[4] * along) as f64,
            (output[2] + output[5] * along) as f64,
        ))
    }
}
unsafe fn visible(
    target: &Target,
    player: &LocalPlayer,
    from: Vector3<f64>,
    to: Vector3<f64>,
    actor: usize,
) -> bool {
    let level = target.read_ptr(player.controller, ACTOR_LEVEL_OFFSET);
    let function = unsafe { module_virtual(target, level, LEVEL_SINGLE_LINE_CHECK_OFFSET) };
    if function == 0 {
        return false;
    }
    let mut hit = [0u32; 32];
    hit[9] = 1f32.to_bits();
    hit[10] = u32::MAX;
    let start = [from.x as f32, from.y as f32, from.z as f32];
    let end = [to.x as f32, to.y as f32, to.z as f32];
    unsafe {
        std::mem::transmute::<
            usize,
            unsafe extern "thiscall" fn(
                usize,
                *mut u32,
                usize,
                *const f32,
                *const f32,
                u32,
                f32,
                f32,
                f32,
            ) -> i32,
        >(function)(
            level,
            hit.as_mut_ptr(),
            player.controller,
            end.as_ptr(),
            start.as_ptr(),
            TRACE_WORLD_FLAGS,
            0.0,
            0.0,
            0.0,
        );
    }
    hit[1] == 0 || hit[1] as usize == actor
}
fn with_runtime(f: impl FnOnce(&mut Runtime)) {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        RUNTIME.with(|runtime| {
            if let Ok(mut value) = runtime.try_borrow_mut() {
                if let Some(runtime) = value.as_mut() {
                    f(runtime);
                }
            }
        })
    }));
    if result.is_err() {
        STOP.store(true, Ordering::Release);
    }
}
type Draw = unsafe extern "thiscall" fn(usize, usize, i32, *mut u8, *mut i32);
unsafe extern "thiscall" fn draw(
    this: usize,
    viewport: usize,
    flag: i32,
    data: *mut u8,
    size: *mut i32,
) {
    with_runtime(|r| {
        if let Some(s) = r.silent.take() {
            let _ = s.restore(&r.target);
        }
    });
    unsafe {
        std::mem::transmute::<usize, Draw>(ORIGINAL[0].load(Ordering::Acquire))(
            this, viewport, flag, data, size,
        );
    }
    with_runtime(Runtime::aim);
}
type Level = unsafe extern "thiscall" fn(usize, usize);
unsafe extern "thiscall" fn level(this: usize, interface: usize) {
    if LEVEL.with(Cell::get) == 0 {
        with_runtime(Runtime::begin_level);
    }
    LEVEL.with(|v| v.set(v.get() + 1));
    unsafe {
        std::mem::transmute::<usize, Level>(ORIGINAL[1].load(Ordering::Acquire))(this, interface);
    }
    LEVEL.with(|v| {
        v.set(v.get() - 1);
        if v.get() == 0 {
            with_runtime(Runtime::end_level);
        }
    });
}
type Mesh = unsafe extern "thiscall" fn(usize, usize, usize, usize, usize, usize);
unsafe extern "thiscall" fn mesh(
    this: usize,
    actor: usize,
    node: usize,
    lights: usize,
    projectors: usize,
    interface: usize,
) {
    let mut group = 0;
    let mut cutout = false;
    // Outside the level with glows waiting this is the first person weapon, cut out of the glow
    if LEVEL.with(Cell::get) > 0 {
        with_runtime(|r| group = r.group(r.target.read_ptr(actor, 0)));
    } else {
        with_runtime(|r| cutout = r.glows_waiting);
    }
    let previous = GROUP.with(|v| v.replace(group));
    let previous_cutout = CUTOUT.with(|v| v.replace(cutout));
    unsafe {
        std::mem::transmute::<usize, Mesh>(ORIGINAL[2].load(Ordering::Acquire))(
            this, actor, node, lights, projectors, interface,
        );
    }
    GROUP.with(|v| v.set(previous));
    CUTOUT.with(|v| v.set(previous_cutout));
}
fn watch_overlay(raw: *mut c_void) {
    if WATCH.with(Cell::get) {
        with_runtime(|r| {
            if let Some(device) = unsafe { IDirect3DDevice9::from_raw_borrowed(&raw) } {
                r.watch_overlay(device);
            }
        });
    }
}
type Indexed = unsafe extern "system" fn(
    *mut c_void,
    D3DPRIMITIVETYPE,
    i32,
    u32,
    u32,
    u32,
    u32,
) -> windows::core::HRESULT;
unsafe extern "system" fn indexed(
    raw: *mut c_void,
    kind: D3DPRIMITIVETYPE,
    base: i32,
    min: u32,
    count: u32,
    start: u32,
    primitives: u32,
) -> windows::core::HRESULT {
    let original =
        unsafe { std::mem::transmute::<usize, Indexed>(ORIGINAL[3].load(Ordering::Acquire)) };
    let result = unsafe { original(raw, kind, base, min, count, start, primitives) };
    let group = GROUP.with(Cell::get);
    if result.is_ok() && (group > 0 || CUTOUT.with(Cell::get)) {
        with_runtime(|r| {
            if let Some(device) = unsafe { IDirect3DDevice9::from_raw_borrowed(&raw) } {
                let again = move || unsafe {
                    original(raw, kind, base, min, count, start, primitives).ok()
                };
                if group > 0 {
                    r.silhouette = true;
                    let _ = r.glow[group - 1].draw_silhouette(device, again);
                } else {
                    for glow in &mut r.glow {
                        let _ = glow.draw_cutout(device, again);
                    }
                }
            }
        });
    }
    watch_overlay(raw);
    result
}
type Primitive =
    unsafe extern "system" fn(*mut c_void, D3DPRIMITIVETYPE, u32, u32) -> windows::core::HRESULT;
unsafe extern "system" fn primitive(
    raw: *mut c_void,
    kind: D3DPRIMITIVETYPE,
    start: u32,
    primitives: u32,
) -> windows::core::HRESULT {
    let result = unsafe {
        std::mem::transmute::<usize, Primitive>(ORIGINAL[4].load(Ordering::Acquire))(
            raw, kind, start, primitives,
        )
    };
    watch_overlay(raw);
    result
}
// D3D9Drv's TripPostFX. A display change makes UD3D9RenderDevice::SetRes fail its Reset, drop the device and create
// a new one, writing new post effects over the old ones without releasing them: ~140 MB of address space per change.
// The missing CleanUp runs where SetRes is about to create the device, the render device being in esi.
const POST_EFFECTS_CLEAN_UP_RVA: usize = 0x2120;
const POST_EFFECTS_CLEAN_UP_CODE: [u8; 10] =
    [0x51, 0x53, 0x55, 0x56, 0x57, 0x83, 0xec, 0x1c, 0x8b, 0xf1];
const SET_RES_CREATE_DEVICE_RVA: usize = 0x15eee;
const SET_RES_CREATE_DEVICE_CODE: [u8; 2] = [0x8b, 0x0d];
const RENDER_DEVICE_POST_EFFECTS_OFFSET: usize = 0xc0;
static POST_EFFECTS_CLEAN_UP: AtomicUsize = AtomicUsize::new(0);
#[unsafe(naked)]
unsafe extern "C" fn create_device() {
    std::arch::naked_asm!(
        "pushad",
        "pushfd",
        "lea ecx, [esi + {offset}]",
        "call dword ptr [{clean_up}]",
        "popfd",
        "popad",
        "jmp dword ptr [{original} + {slot}]",
        offset = const RENDER_DEVICE_POST_EFFECTS_OFFSET,
        clean_up = sym POST_EFFECTS_CLEAN_UP,
        original = sym ORIGINAL,
        slot = const 5 * std::mem::size_of::<AtomicUsize>(),
    );
}
unsafe fn hook_create_device() -> Result<(), std::boxed::Box<dyn std::error::Error>> {
    let driver = unsafe { GetModuleHandleW(w!("D3D9Drv.dll"))? }.0 as usize;
    let code = |rva: usize, len: usize| unsafe {
        std::slice::from_raw_parts((driver + rva) as *const u8, len)
    };
    if code(POST_EFFECTS_CLEAN_UP_RVA, POST_EFFECTS_CLEAN_UP_CODE.len())
        != POST_EFFECTS_CLEAN_UP_CODE
        || code(SET_RES_CREATE_DEVICE_RVA, SET_RES_CREATE_DEVICE_CODE.len())
            != SET_RES_CREATE_DEVICE_CODE
    {
        return Err("D3D9Drv.dll is not the known build, the post effects keep leaking".into());
    }
    POST_EFFECTS_CLEAN_UP.store(driver + POST_EFFECTS_CLEAN_UP_RVA, Ordering::Release);
    unsafe {
        install(
            5,
            driver + SET_RES_CREATE_DEVICE_RVA,
            create_device as *const () as usize,
        )
    }
}
unsafe fn install(
    index: usize,
    address: usize,
    detour: usize,
) -> Result<(), std::boxed::Box<dyn std::error::Error>> {
    let hook = unsafe { RawDetour::new(address as _, detour as _)? };
    ORIGINAL[index].store(hook.trampoline() as *const _ as usize, Ordering::Release);
    unsafe {
        hook.enable()?;
    }
    HOOKS.lock().unwrap().push(hook);
    Ok(())
}
/// # Safety
/// Call outside loader lock with render/engine threads quiescent. KF1 build 1065 only.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn omath_kf1_start() -> i32 {
    if !HOOKS.lock().unwrap_or_else(|e| e.into_inner()).is_empty() {
        return 1;
    }
    let result = std::panic::catch_unwind(
        || -> Result<(), std::boxed::Box<dyn std::error::Error>> {
            if !HOOKS.lock().unwrap().is_empty() {
                return Err("already started".into());
            }
            STOP.store(false, Ordering::Release);
            let hwnd = platform::find_main_window(std::process::id()).ok_or("no game window")?;
            if unsafe { GetWindowThreadProcessId(hwnd, None) }
                != unsafe { windows::Win32::System::Threading::GetCurrentThreadId() }
            {
                return Err("start on the game window owner thread".into());
            }
            HooksManager::get().set_on_wnd_proc(Some(Arc::new(
                |hwnd, message, param, argument| {
                    if !MENU.load(Ordering::Acquire) {
                        return None;
                    }
                    if message == WM_INPUT {
                        return Some(
                            unsafe {
                                DefWindowProcW(
                                    HWND(hwnd as _),
                                    message,
                                    WPARAM(param),
                                    LPARAM(argument),
                                )
                            }
                            .0,
                        );
                    }
                    if matches!(message, WM_MOUSEWHEEL | WM_MOUSEHWHEEL | WM_CHAR) {
                        INPUT
                            .lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .push((message, param));
                    }
                    if (WM_MOUSEFIRST..=WM_MOUSELAST).contains(&message)
                        || matches!(
                            message,
                            WM_INPUT
                                | WM_KEYDOWN
                                | WM_KEYUP
                                | WM_SYSKEYDOWN
                                | WM_SYSKEYUP
                                | WM_CHAR
                        )
                    {
                        Some(0)
                    } else {
                        None
                    }
                },
            )));
            unsafe {
                HooksManager::get().hook_wnd_proc(hwnd.0 as usize)?;
            }
            let engine = unsafe { GetModuleHandleW(w!("Engine.dll"))? };
            let exports=[(c"?Draw@UGameEngine@@UAEXPAVUViewport@@HPAEPAH@Z",draw as *const ()as usize),(c"?Render@FLevelSceneNode@@UAEXPAVFRenderInterface@@@Z",level as *const ()as usize),(c"?Render@USkeletalMeshInstance@@UAEXPAVFDynamicActor@@PAVFLevelSceneNode@@PAV?$TList@PAVFDynamicLight@@@@PAV?$TList@PAUFProjectorRenderInfo@@@@PAVFRenderInterface@@@Z",mesh as *const ()as usize)];
            for (index, (name, detour)) in exports.into_iter().enumerate() {
                let address = unsafe { GetProcAddress(engine, PCSTR(name.as_ptr().cast())) }
                    .ok_or("missing Engine export")?;
                unsafe {
                    install(index, address as *const () as usize, detour)?;
                }
            }
            let _ = unsafe { hook_create_device() };
            HooksManager::get().set_on_dx9_present(Some(Arc::new(|raw, _, _, _, _| {
                RUNTIME.with(|cell| {
                    let Ok(mut runtime) = cell.try_borrow_mut() else {
                        return;
                    };
                    if runtime.is_none() && !STOP.load(Ordering::Relaxed) {
                        if let Some(device) = unsafe { IDirect3DDevice9::from_raw_borrowed(&raw) } {
                            if let Ok(value) = Runtime::new(device) {
                                let table = unsafe { *(raw as *const *const usize) };
                                let address = unsafe { *table.add(82) };
                                if unsafe { install(3, address, indexed as *const () as usize) }
                                    .is_ok()
                                {
                                    let _ = unsafe {
                                        install(4, *table.add(81), primitive as *const () as usize)
                                    };
                                    *runtime = Some(value);
                                }
                            }
                        }
                    }
                    if let Some(runtime) = runtime.as_mut() {
                        runtime.present();
                    }
                });
            })));
            HooksManager::get()
                .set_on_dx9_reset(Some(Arc::new(|_, _| with_runtime(Runtime::reset))));
            unsafe {
                HooksManager::get().hook_dx9()?;
            }
            Ok(())
        },
    );
    if matches!(result, Ok(Ok(()))) {
        0
    } else {
        let _ = unsafe { omath_kf1_stop() };
        -1
    }
}
/// # Safety
/// Run on the render thread outside callbacks, with all hooked functions quiescent.
/// Stop successfully before unloading. END only disables; it does not unload.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn omath_kf1_stop() -> i32 {
    STOP.store(true, Ordering::Release);
    MENU.store(false, Ordering::Release);
    let result =
        std::panic::catch_unwind(|| -> Result<(), std::boxed::Box<dyn std::error::Error>> {
            unsafe {
                HooksManager::get().unhook_wnd_proc()?;
                HooksManager::get().unhook_dx9()?;
            }
            HooksManager::get().set_on_wnd_proc(None);
            HooksManager::get().set_on_dx9_present(None);
            HooksManager::get().set_on_dx9_reset(None);
            let mut hooks = HOOKS.lock().unwrap();
            for hook in hooks.iter().rev() {
                unsafe {
                    hook.disable()?;
                }
            }
            hooks.clear();
            RUNTIME.with(|cell| {
                if let Some(mut runtime) = cell.borrow_mut().take() {
                    runtime.restore();
                    runtime.reset();
                }
            });
            Ok(())
        });
    if matches!(result, Ok(Ok(()))) { 0 } else { -1 }
}
