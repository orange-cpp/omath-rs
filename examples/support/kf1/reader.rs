use omath::{
    Vector3,
    engines::unreal::*,
    rev_eng::{MemoryRead, MemoryValue, MemoryWrite, ProcessMemory, shift_address},
};
use std::{cell::Cell, io};
pub const G_OBJECTS_RVA: usize = 0x16a70c;
pub const G_NAMES_RVA: usize = 0x166674;
pub const AUDIO_SUBSYSTEM_RVA: usize = 0x4c6934;
pub const AUDIO_LISTENER_OFFSET: isize = 0x38;
pub const OBJECT_CLASS_OFFSET: isize = 0x28;
pub const STRUCT_SUPER_OFFSET: isize = 0x2c;
pub const STRUCT_CHILDREN_OFFSET: isize = 0x40;
pub const FIELD_NEXT_OFFSET: isize = 0x30;
pub const PROPERTY_OFFSET_OFFSET: isize = 0x4c;
pub const ACTOR_LOCATION_OFFSET: isize = 0x14c;
pub const ACTOR_ROTATION_OFFSET: isize = 0x158;
pub const ACTOR_CONTROLLER_OFFSET: isize = 0x360;
pub const ACTOR_LEVEL_OFFSET: isize = 0x9c;
pub const CONTROLLER_PLAYER_OFFSET: isize = 0x514;
pub const LEVEL_SINGLE_LINE_CHECK_OFFSET: isize = 0xdc;
pub const TRACE_WORLD_FLAGS: u32 = 0x286;

pub struct Target<M: MemoryRead = ProcessMemory> {
    pub memory: M,
    pub process_id: u32,
    pub engine_base: usize,
    pub g_objects: usize,
    pub g_names: usize,
    pub object_array: ObjectArrayLayout,
    pub names: NameLayout,
    pub object_count: u32,
    failed: Cell<bool>,
}
impl<M: MemoryRead> Target<M> {
    pub fn new(
        memory: M,
        process_id: u32,
        engine_base: usize,
        g_objects: usize,
        g_names: usize,
    ) -> io::Result<Self> {
        let object_array = ObjectArrayLayout::ue2_5();
        let object_count =
            get_object_count(&memory, g_objects, &object_array).map_err(io::Error::other)?;
        Ok(Self {
            memory,
            process_id,
            engine_base,
            g_objects,
            g_names,
            object_array,
            names: NameLayout::ue2_5(),
            object_count,
            failed: Cell::new(false),
        })
    }
    pub fn clear_read_error(&self) {
        self.failed.set(false);
    }
    pub fn read_failed(&self) -> bool {
        self.failed.get()
    }
    pub fn read<T: MemoryValue + Default>(&self, address: usize) -> T {
        match self.memory.read_memory(address) {
            Ok(v) => v,
            Err(_) => {
                self.failed.set(true);
                T::default()
            }
        }
    }
    pub fn read_ptr(&self, base: usize, offset: isize) -> usize {
        self.read::<u32>(shift_address(base, offset)) as usize
    }
    pub fn read_string(&self, address: usize) -> String {
        let data = self.read_ptr(address, 0);
        let length = self.read::<i32>(address + 4).saturating_sub(1);
        if data == 0 || !(1..=256).contains(&length) {
            return String::new();
        }
        let mut bytes = vec![0; length as usize * 2];
        if self.memory.read_bytes(data, &mut bytes).is_err() {
            self.failed.set(true);
            return String::new();
        }
        String::from_utf16_lossy(
            &bytes
                .chunks_exact(2)
                .map(|b| u16::from_le_bytes([b[0], b[1]]))
                .collect::<Vec<_>>(),
        )
    }
    pub fn live_object_count(&self) -> u32 {
        get_object_count(&self.memory, self.g_objects, &self.object_array)
            .unwrap_or(self.object_count)
    }
    pub fn object_at(&self, index: u32) -> Option<usize> {
        get_object_by_index(&self.memory, self.g_objects, index, &self.object_array).ok()
    }
    pub fn read_object_table(&self) -> Vec<u32> {
        let data = self.read_ptr(self.g_objects, self.object_array.objects_offset);
        let count = self.live_object_count() as usize;
        if data == 0 || count == 0 || count > 4_000_000 {
            return Vec::new();
        }
        let mut bytes = vec![0; count * 4];
        if self.memory.read_bytes(data, &mut bytes).is_err() {
            self.failed.set(true);
            return Vec::new();
        }
        bytes
            .chunks_exact(4)
            .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
            .collect()
    }
    pub fn object_name(&self, object: usize) -> String {
        get_actor_name(&self.memory, object, self.g_names, &self.names)
            .unwrap_or_else(|_| "?".into())
    }
    pub fn is_kind_of(&self, mut class: usize, wanted: usize) -> bool {
        for _ in 0..64 {
            if class == 0 {
                break;
            }
            if class == wanted {
                return true;
            }
            class = self.read_ptr(class, STRUCT_SUPER_OFFSET);
        }
        false
    }
    pub fn find_base_class(&self, mut class: usize, name: &str) -> usize {
        for _ in 0..64 {
            if class == 0 {
                break;
            }
            if self.object_name(class) == name {
                return class;
            }
            class = self.read_ptr(class, STRUCT_SUPER_OFFSET);
        }
        0
    }
    pub fn is_local_controller(&self, controller: usize) -> bool {
        self.find_base_class(
            self.read_ptr(
                self.read_ptr(controller, CONTROLLER_PLAYER_OFFSET),
                OBJECT_CLASS_OFFSET,
            ),
            "Viewport",
        ) != 0
    }
    pub fn find_reflection_classes(&self) -> ReflectionClasses {
        let mut result = ReflectionClasses::default();
        for index in 0..self.live_object_count() {
            let Some(object) = self.object_at(index) else {
                continue;
            };
            let name = self.object_name(object);
            if !matches!(
                name.as_str(),
                "Actor" | "PlayerController" | "Pawn" | "Property"
            ) || self.object_name(self.read_ptr(object, OBJECT_CLASS_OFFSET)) != "Class"
            {
                continue;
            }
            match name.as_str() {
                "Actor" => result.actor = object,
                "PlayerController" => result.player_controller = object,
                "Pawn" => result.pawn = object,
                _ => result.property = object,
            }
            if result.complete() {
                break;
            }
        }
        result
    }
    pub fn find_property_offset(
        &self,
        mut owner: usize,
        name: &str,
        property_class: usize,
    ) -> Option<i32> {
        for _ in 0..32 {
            if owner == 0 {
                break;
            }
            let mut field = self.read_ptr(owner, STRUCT_CHILDREN_OFFSET);
            for _ in 0..4096 {
                if field == 0 {
                    break;
                }
                if self.object_name(field) == name
                    && self.is_kind_of(self.read_ptr(field, OBJECT_CLASS_OFFSET), property_class)
                {
                    return Some(self.read(shift_address(field, PROPERTY_OFFSET_OFFSET)));
                }
                field = self.read_ptr(field, FIELD_NEXT_OFFSET);
            }
            owner = self.read_ptr(owner, STRUCT_SUPER_OFFSET);
        }
        None
    }
    pub fn list_properties(
        &self,
        mut owner: usize,
        property_class: usize,
        limit: usize,
    ) -> Vec<String> {
        let mut result = Vec::new();
        for _ in 0..32 {
            if owner == 0 || result.len() >= limit {
                break;
            }
            let mut field = self.read_ptr(owner, STRUCT_CHILDREN_OFFSET);
            for _ in 0..4096 {
                if field == 0 || result.len() >= limit {
                    break;
                }
                if self.is_kind_of(self.read_ptr(field, OBJECT_CLASS_OFFSET), property_class) {
                    result.push(format!(
                        "{}::{}",
                        self.object_name(owner),
                        self.object_name(field)
                    ));
                }
                field = self.read_ptr(field, FIELD_NEXT_OFFSET);
            }
            owner = self.read_ptr(owner, STRUCT_SUPER_OFFSET);
        }
        result
    }
    pub fn read_location(&self, actor: usize, offset: isize) -> Vector3<f64> {
        let [x, y, z] = self.read::<[f32; 3]>(shift_address(actor, offset));
        Vector3::new(x as f64, y as f64, z as f64)
    }
    pub fn read_view_angles(&self, actor: usize, offset: isize) -> ViewAngles {
        let [p, y, r] = self.read::<[i32; 3]>(shift_address(actor, offset));
        ViewAngles::new(
            PitchAngle::from_degrees(rotator_units_to_degrees(p)),
            YawAngle::from_degrees(rotator_units_to_degrees(y)),
            RollAngle::from_degrees(rotator_units_to_degrees(r)),
        )
    }
    pub fn find_possessed_pawn(&self, pawn_class: usize, controller: usize) -> usize {
        for i in 0..self.live_object_count() {
            if let Some(object) = self.object_at(i) {
                if self.is_kind_of(self.read_ptr(object, OBJECT_CLASS_OFFSET), pawn_class)
                    && self.read_ptr(object, ACTOR_CONTROLLER_OFFSET) == controller
                {
                    return object;
                }
            }
        }
        0
    }
    pub fn resolve_player_properties(&self, classes: &ReflectionClasses, player: &mut LocalPlayer) {
        let class = self.read_ptr(player.controller, OBJECT_CLASS_OFFSET);
        player.fov_offset = self.find_property_offset(class, "FovAngle", classes.property);
        player.view_rotation_offset =
            self.find_property_offset(class, "CalcViewRotation", classes.property);
        player.view_location_offset =
            self.find_property_offset(class, "CalcViewLocation", classes.property);
    }
    pub fn find_local_player_by_scan(&self, classes: &ReflectionClasses) -> LocalPlayer {
        let mut local = 0;
        let mut controller_count = 0;
        for i in 0..self.live_object_count() {
            if let Some(object) = self.object_at(i) {
                if self.is_kind_of(
                    self.read_ptr(object, OBJECT_CLASS_OFFSET),
                    classes.player_controller,
                ) {
                    controller_count += 1;
                    if self.is_local_controller(object) {
                        local = object;
                    }
                }
            }
        }
        if local == 0 {
            return LocalPlayer::default();
        }
        let mut player = LocalPlayer {
            controller: local,
            pawn: self.find_possessed_pawn(classes.pawn, local),
            controller_count,
            ..Default::default()
        };
        self.resolve_player_properties(classes, &mut player);
        player
    }
    pub fn audio_listener(&self) -> usize {
        let audio = self.read_ptr(self.engine_base + AUDIO_SUBSYSTEM_RVA, 0);
        if audio == 0 {
            0
        } else {
            self.read_ptr(audio, AUDIO_LISTENER_OFFSET)
        }
    }
    pub fn find_local_player(&self, classes: &ReflectionClasses) -> LocalPlayer {
        let pawn = self.audio_listener();
        let controller = if pawn != 0 {
            self.read_ptr(pawn, ACTOR_CONTROLLER_OFFSET)
        } else {
            0
        };
        if controller == 0
            || !self.is_kind_of(self.read_ptr(pawn, OBJECT_CLASS_OFFSET), classes.pawn)
            || !self.is_kind_of(
                self.read_ptr(controller, OBJECT_CLASS_OFFSET),
                classes.player_controller,
            )
            || !self.is_local_controller(controller)
        {
            return self.find_local_player_by_scan(classes);
        }
        let mut player = LocalPlayer {
            controller,
            pawn,
            ..Default::default()
        };
        self.resolve_player_properties(classes, &mut player);
        player
    }
    pub fn still_valid(&self, classes: &ReflectionClasses, player: &LocalPlayer) -> bool {
        player.valid()
            && self.is_kind_of(
                self.read_ptr(player.controller, OBJECT_CLASS_OFFSET),
                classes.player_controller,
            )
            && self.read_ptr(player.controller, CONTROLLER_PLAYER_OFFSET) != 0
            && (player.pawn == 0
                || self.read_ptr(player.pawn, ACTOR_CONTROLLER_OFFSET) == player.controller)
    }
    pub fn read_camera(&self, player: &LocalPlayer) -> CameraState {
        CameraState {
            position: self.read_location(
                player.controller,
                player
                    .view_location_offset
                    .map_or(ACTOR_LOCATION_OFFSET, |v| v as isize),
            ),
            angles: self.read_view_angles(
                player.controller,
                player
                    .view_rotation_offset
                    .map_or(ACTOR_ROTATION_OFFSET, |v| v as isize),
            ),
            fov: player.fov_offset.map_or(90.0, |offset| {
                self.read::<f32>(shift_address(player.controller, offset as isize)) as f64
            }),
        }
    }
}
impl<M: MemoryWrite> Target<M> {
    pub fn write_view_angles(
        &self,
        actor: usize,
        angles: ViewAngles,
        offset: isize,
    ) -> io::Result<()> {
        self.memory.write_memory(
            shift_address(actor, offset),
            &[
                degrees_to_rotator_units(angles.pitch.as_degrees()),
                degrees_to_rotator_units(angles.yaw.as_degrees()),
                degrees_to_rotator_units(angles.roll.as_degrees()),
            ],
        )
    }
}
pub fn rotator_units_to_degrees(units: i32) -> f64 {
    let wrapped = units as u32 & 0xffff;
    let signed = if wrapped >= 32768 {
        wrapped as i32 - 65536
    } else {
        wrapped as i32
    };
    signed as f64 * 360.0 / 65536.0
}
pub fn degrees_to_rotator_units(degrees: f64) -> i32 {
    (degrees * 65536.0 / 360.0).round() as i32 & 0xffff
}
#[derive(Default, Clone, Copy, Debug)]
pub struct ReflectionClasses {
    pub actor: usize,
    pub player_controller: usize,
    pub pawn: usize,
    pub property: usize,
}
impl ReflectionClasses {
    pub fn complete(&self) -> bool {
        self.actor != 0 && self.player_controller != 0 && self.pawn != 0 && self.property != 0
    }
}
#[derive(Default, Clone, Copy, Debug)]
pub struct LocalPlayer {
    pub controller: usize,
    pub pawn: usize,
    pub controller_count: usize,
    pub fov_offset: Option<i32>,
    pub view_rotation_offset: Option<i32>,
    pub view_location_offset: Option<i32>,
}
impl LocalPlayer {
    pub fn valid(&self) -> bool {
        self.controller != 0
    }
}
#[derive(Clone, Copy, Debug)]
pub struct CameraState {
    pub position: Vector3<f64>,
    pub angles: ViewAngles,
    pub fov: f64,
}

#[cfg(all(windows, feature = "hooks"))]
pub mod platform {
    use super::*;
    use windows::{
        Win32::{Foundation::*, System::Diagnostics::ToolHelp::*, UI::WindowsAndMessaging::*},
        core::PCWSTR,
    };
    struct Snapshot(HANDLE);
    impl Drop for Snapshot {
        fn drop(&mut self) {
            let _ = unsafe { CloseHandle(self.0) };
        }
    }
    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain([0]).collect()
    }
    fn name(bytes: &[u16]) -> String {
        String::from_utf16_lossy(
            &bytes[..bytes.iter().position(|&v| v == 0).unwrap_or(bytes.len())],
        )
    }
    pub fn find_process_id(process: &str) -> io::Result<u32> {
        unsafe {
            let snapshot = Snapshot(
                CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).map_err(io::Error::other)?,
            );
            let mut entry = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };
            let mut found = Process32FirstW(snapshot.0, &mut entry).is_ok();
            while found {
                if name(&entry.szExeFile).eq_ignore_ascii_case(process) {
                    return Ok(entry.th32ProcessID);
                }
                found = Process32NextW(snapshot.0, &mut entry).is_ok();
            }
        }
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{process} is not running"),
        ))
    }
    pub fn find_module_base(pid: u32, module: &str) -> io::Result<usize> {
        unsafe {
            let snapshot = Snapshot(
                CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid)
                    .map_err(io::Error::other)?,
            );
            let mut entry = MODULEENTRY32W {
                dwSize: std::mem::size_of::<MODULEENTRY32W>() as u32,
                ..Default::default()
            };
            let mut found = Module32FirstW(snapshot.0, &mut entry).is_ok();
            while found {
                if name(&entry.szModule).eq_ignore_ascii_case(module) {
                    return Ok(entry.modBaseAddr as usize);
                }
                found = Module32NextW(snapshot.0, &mut entry).is_ok();
            }
        }
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{module} is not loaded"),
        ))
    }
    pub fn find_main_window(pid: u32) -> Option<HWND> {
        struct Search {
            pid: u32,
            window: Option<HWND>,
        }
        unsafe extern "system" fn find(hwnd: HWND, param: LPARAM) -> windows::core::BOOL {
            let search = unsafe { &mut *(param.0 as *mut Search) };
            let mut pid = 0;
            unsafe {
                GetWindowThreadProcessId(hwnd, Some(&mut pid));
            }
            if pid == search.pid
                && unsafe { IsWindowVisible(hwnd) }.as_bool()
                && unsafe { GetWindow(hwnd, GW_OWNER) }.is_err()
            {
                search.window = Some(hwnd);
                return false.into();
            }
            true.into()
        }
        let mut search = Search { pid, window: None };
        let _ = unsafe { EnumWindows(Some(find), LPARAM(&mut search as *mut _ as isize)) };
        search.window
    }
    pub fn attach() -> io::Result<Target<ProcessMemory>> {
        let pid = find_process_id("KillingFloor.exe")?;
        let core = find_module_base(pid, "Core.dll")?;
        Target::new(
            ProcessMemory::open(pid, false)?,
            pid,
            find_module_base(pid, "Engine.dll")?,
            core + G_OBJECTS_RVA,
            core + G_NAMES_RVA,
        )
    }
    pub fn window_rect(hwnd: HWND) -> io::Result<(i32, i32, i32, i32)> {
        let mut rect = RECT::default();
        unsafe {
            GetClientRect(hwnd, &mut rect).map_err(io::Error::other)?;
        }
        let mut point = POINT {
            x: rect.left,
            y: rect.top,
        };
        unsafe {
            windows::Win32::Graphics::Gdi::ClientToScreen(hwnd, &mut point)
                .ok()
                .map_err(io::Error::other)?;
        }
        Ok((
            point.x,
            point.y,
            rect.right - rect.left,
            rect.bottom - rect.top,
        ))
    }
    pub fn key_pressed(key: i32, was_down: &mut bool) -> bool {
        let down =
            unsafe { windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(key) } < 0;
        let pressed = down && !*was_down;
        *was_down = down;
        pressed
    }
    pub fn module_name_wide(value: &str) -> Vec<u16> {
        wide(value)
    }
    pub fn wide_pointer(value: &[u16]) -> PCWSTR {
        PCWSTR(value.as_ptr())
    }
}
