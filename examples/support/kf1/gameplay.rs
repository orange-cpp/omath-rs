//! Reflected KF1 aim and reversible fire-mode edits. Native bone/trace queries
//! are supplied by the injected host; memory-image tests use the same logic.
use super::{overlay::LiveActor, reader::*};
use omath::{
    Vector3,
    engines::unreal::*,
    rev_eng::{MemoryRead, MemoryWrite, shift_address},
};
#[derive(Clone, Debug)]
pub struct AimSettings {
    pub enabled: bool,
    pub key: usize,
    pub cone: f64,
    pub range: f64,
    pub smoothing: f64,
    pub head_drop: f64,
    pub visible_only: bool,
    pub silent: bool,
    pub no_recoil: bool,
    pub no_spread: bool,
}
impl Default for AimSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            key: 0,
            cone: 25.0,
            range: 4000.0,
            smoothing: 0.35,
            head_drop: 0.1,
            visible_only: true,
            silent: false,
            no_recoil: false,
            no_spread: false,
        }
    }
}
pub fn choose_target<M: MemoryRead>(
    target: &Target<M>,
    player: &LocalPlayer,
    zeds: &[LiveActor],
    settings: &AimSettings,
    mut bone: impl FnMut(usize) -> Option<Vector3<f64>>,
    mut visible: impl FnMut(Vector3<f64>, Vector3<f64>, usize) -> bool,
) -> Option<ViewAngles> {
    let state = target.read_camera(player);
    let forward = forward_vector(state.angles);
    let mut best = settings.cone;
    let mut selected = None;
    for zed in zeds {
        let Some(bounds) = &zed.bounds else { continue };
        let head = bone(zed.actor.address).unwrap_or_else(|| {
            let mut p = (bounds.min + bounds.max) * 0.5;
            p.z = bounds.max.z - (bounds.max.z - bounds.min.z) * settings.head_drop;
            p
        });
        let delta = head - state.position;
        let distance = delta.length();
        if distance <= 0.0 || distance > settings.range {
            continue;
        }
        let angle = forward
            .dot(delta / distance)
            .clamp(-1.0, 1.0)
            .acos()
            .to_degrees();
        if angle >= best
            || settings.visible_only && !visible(state.position, head, zed.actor.address)
        {
            continue;
        }
        best = angle;
        selected = Some(head);
    }
    selected.map(|point| {
        let d = point - state.position;
        ViewAngles::new(
            PitchAngle::from_degrees(d.z.atan2(d.x.hypot(d.y)).to_degrees()),
            YawAngle::from_degrees(d.y.atan2(d.x).to_degrees()),
            RollAngle::from_degrees(0.0),
        )
    })
}
pub fn smooth(current: ViewAngles, wanted: ViewAngles, factor: f64) -> ViewAngles {
    let p = current.pitch.as_degrees();
    let y = current.yaw.as_degrees();
    let mut dy = (wanted.yaw.as_degrees() - y) % 360.0;
    if dy > 180.0 {
        dy -= 360.0;
    }
    if dy < -180.0 {
        dy += 360.0;
    }
    ViewAngles::new(
        PitchAngle::from_degrees(p + (wanted.pitch.as_degrees() - p) * factor),
        YawAngle::from_degrees(y + dy * factor),
        RollAngle::from_degrees(0.0),
    )
}
#[derive(Clone, Debug)]
pub struct SilentAim {
    controller: usize,
    pawn: usize,
    weapon: usize,
    weapon_offset: isize,
    real: [i32; 3],
    written: [i32; 3],
    pawn_real: [i32; 3],
    weapon_real: [i32; 3],
}
impl SilentAim {
    pub fn begin<M: MemoryWrite>(
        target: &Target<M>,
        classes: &ReflectionClasses,
        player: &LocalPlayer,
        wanted: ViewAngles,
    ) -> std::io::Result<Self> {
        let real: [i32; 3] = target.read(shift_address(player.controller, ACTOR_ROTATION_OFFSET));
        let written = [
            degrees_to_rotator_units(wanted.pitch.as_degrees()),
            degrees_to_rotator_units(wanted.yaw.as_degrees()),
            real[2],
        ];
        let weapon_offset = target
            .find_property_offset(
                target.read_ptr(player.pawn, OBJECT_CLASS_OFFSET),
                "Weapon",
                classes.property,
            )
            .unwrap_or(0) as isize;
        let weapon = if weapon_offset != 0 {
            target.read_ptr(player.pawn, weapon_offset)
        } else {
            0
        };
        let value = Self {
            controller: player.controller,
            pawn: player.pawn,
            weapon,
            weapon_offset,
            real,
            written,
            pawn_real: target.read(shift_address(player.pawn, ACTOR_ROTATION_OFFSET)),
            weapon_real: target.read(shift_address(weapon, ACTOR_ROTATION_OFFSET)),
        };
        target.memory.write_memory(
            shift_address(player.controller, ACTOR_ROTATION_OFFSET),
            &written,
        )?;
        Ok(value)
    }
    pub fn restore<M: MemoryWrite>(&self, target: &Target<M>) -> std::io::Result<()> {
        let address = shift_address(self.controller, ACTOR_ROTATION_OFFSET);
        let now = target.read::<[i32; 3]>(address);
        let moved = std::array::from_fn::<_, 3, _>(|i| now[i].wrapping_sub(self.written[i]));
        let restored =
            std::array::from_fn::<_, 3, _>(|i| self.real[i].wrapping_add(moved[i]) & 0xffff);
        target.memory.write_memory(address, &restored)?;
        if self.pawn != 0 {
            let address = shift_address(self.pawn, ACTOR_ROTATION_OFFSET);
            let mut value = target.read::<[i32; 3]>(address);
            value[1] = self.pawn_real[1].wrapping_add(moved[1]) & 0xffff;
            target.memory.write_memory(address, &value)?;
        }
        if self.weapon != 0 && target.read_ptr(self.pawn, self.weapon_offset) == self.weapon {
            let value = std::array::from_fn::<_, 3, _>(|i| {
                self.weapon_real[i].wrapping_add(moved[i]) & 0xffff
            });
            target
                .memory
                .write_memory(shift_address(self.weapon, ACTOR_ROTATION_OFFSET), &value)?;
        }
        Ok(())
    }
}
#[derive(Default)]
pub struct FireEdits {
    values: Vec<(usize, usize, usize, i32)>,
}
impl FireEdits {
    pub fn restore<M: MemoryWrite>(&mut self, target: &Target<M>) {
        for &(object, class, address, value) in self.values.iter().rev() {
            if target.read_ptr(object, OBJECT_CLASS_OFFSET) == class {
                let _ = target.memory.write_memory(address, &value);
            }
        }
        self.values.clear();
    }
    fn zero<M: MemoryWrite>(&mut self, target: &Target<M>, object: usize, offset: i32) {
        let address = shift_address(object, offset as isize);
        if self.values.iter().any(|v| v.2 == address) {
            return;
        }
        let class = target.read_ptr(object, OBJECT_CLASS_OFFSET);
        if let Ok(value) = target.memory.read_memory::<i32>(address) {
            if target.memory.write_memory(address, &0i32).is_ok() {
                self.values.push((object, class, address, value));
            }
        }
    }
    pub fn apply<M: MemoryWrite>(
        &mut self,
        target: &Target<M>,
        classes: &ReflectionClasses,
        player: &LocalPlayer,
        names: &[&str],
        defaults: bool,
    ) {
        let pawn_class = target.read_ptr(player.pawn, OBJECT_CLASS_OFFSET);
        let Some(offset) = target.find_property_offset(pawn_class, "Weapon", classes.property)
        else {
            return;
        };
        let weapon = target.read_ptr(player.pawn, offset as isize);
        if weapon == 0 {
            return;
        }
        let class = target.read_ptr(weapon, OBJECT_CLASS_OFFSET);
        let Some(offset) = target.find_property_offset(class, "FireMode", classes.property) else {
            return;
        };
        for slot in 0..2 {
            let fire = target.read_ptr(weapon, offset as isize + slot * 4);
            if fire == 0 {
                continue;
            }
            let class = target.read_ptr(fire, OBJECT_CLASS_OFFSET);
            let default = if defaults {
                (0x30..0x200)
                    .step_by(4)
                    .find_map(|offset| {
                        let data = target.read_ptr(class, offset);
                        (data != 0
                            && target.read::<i32>(shift_address(class, offset + 4)) > 0x28
                            && target.read_ptr(data, 0x28) == class)
                            .then_some(data)
                    })
                    .unwrap_or(0)
            } else {
                0
            };
            for name in names {
                if let Some(offset) = target.find_property_offset(class, name, classes.property) {
                    self.zero(target, fire, offset);
                    if default != 0 {
                        self.zero(target, default, offset);
                    }
                }
            }
        }
    }
}
#[cfg(feature = "imgui")]
pub fn menu(ui: &imgui::Ui, s: &mut AimSettings) {
    ui.window("Aim and weapon settings").build(|| {
        ui.checkbox("Aim enabled", &mut s.enabled);
        ui.combo_simple_string(
            "Aim key",
            &mut s.key,
            &[
                "Right mouse",
                "Left mouse",
                "Mouse X1",
                "Mouse X2",
                "Shift",
                "Alt",
            ],
        );
        let mut cone = s.cone as f32;
        if ui.slider("Aim cone", 1.0, 180.0, &mut cone) {
            s.cone = cone as f64;
        }
        let mut range = s.range as f32;
        if ui.slider("Aim range", 100.0, 20000.0, &mut range) {
            s.range = range as f64;
        }
        let mut smoothing = s.smoothing as f32;
        if ui.slider("Smoothing", 0.0, 1.0, &mut smoothing) {
            s.smoothing = smoothing as f64;
        }
        let mut drop = s.head_drop as f32;
        if ui.slider("Head fallback drop", 0.0, 1.0, &mut drop) {
            s.head_drop = drop as f64;
        }
        ui.checkbox("Only visible", &mut s.visible_only);
        ui.checkbox("Silent aim", &mut s.silent);
        ui.checkbox("No recoil", &mut s.no_recoil);
        ui.checkbox("No spread", &mut s.no_spread);
    });
}
