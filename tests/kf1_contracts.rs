//! Deterministic UE2.5 snapshots exercise the actual example implementation.
#[path = "../examples/support/kf1/mod.rs"]
mod kf1;
use kf1::{gameplay::*, overlay::*, reader::*};
use omath::{
    Vector3,
    engines::unreal::*,
    rev_eng::{MemoryRead, MemoryValue, MemoryWrite},
};
use std::{cell::RefCell, collections::HashMap, io};
struct Image(RefCell<Vec<u8>>);
impl MemoryRead for Image {
    fn read_bytes(&self, address: usize, out: &mut [u8]) -> io::Result<()> {
        let data = self.0.borrow();
        out.copy_from_slice(
            data.get(address..address + out.len())
                .ok_or_else(|| io::Error::other("outside image"))?,
        );
        Ok(())
    }
}
impl MemoryWrite for Image {
    fn write_bytes(&self, address: usize, value: &[u8]) -> io::Result<()> {
        let mut data = self.0.borrow_mut();
        data.get_mut(address..address + value.len())
            .ok_or_else(|| io::Error::other("outside image"))?
            .copy_from_slice(value);
        Ok(())
    }
}
struct Fixture {
    target: Target<Image>,
    names: HashMap<String, u32>,
    objects: Vec<u32>,
    next: usize,
}
impl Fixture {
    fn new() -> Self {
        let memory = Image(RefCell::new(vec![0; 0x100000]));
        memory.write_memory(0x200, &0x800u32).unwrap();
        memory.write_memory(0x204, &0u32).unwrap();
        memory.write_memory(0x100, &0x400u32).unwrap();
        let target = Target::new(memory, 0, 0, 0x200, 0x100).unwrap();
        Self {
            target,
            names: HashMap::new(),
            objects: Vec::new(),
            next: 0x1000,
        }
    }
    fn set<T: MemoryValue>(&self, address: usize, value: T) {
        self.target.memory.write_memory(address, &value).unwrap();
    }
    fn name(&mut self, object: usize, name: &str) {
        let index = if let Some(index) = self.names.get(name) {
            *index
        } else {
            let index = self.names.len() as u32;
            self.names.insert(name.into(), index);
            let address = 0x80000 + index as usize * 0x800;
            self.set(0x400 + index as usize * 4, address as u32);
            for (i, c) in name.encode_utf16().chain([0]).enumerate() {
                self.set(address + 0xc + i * 2, c);
            }
            index
        };
        self.set(object + 0x24, index);
    }
    fn object(&mut self, name: &str, class: usize, parent: usize) -> usize {
        let address = self.next;
        self.next += 0x1000;
        self.name(address, name);
        self.set(address + 0x28, class as u32);
        self.set(address + 0x2c, parent as u32);
        self.objects.push(address as u32);
        for (i, &object) in self.objects.iter().enumerate() {
            self.set(0x800 + i * 4, object);
        }
        self.set(0x204, self.objects.len() as u32);
        address
    }
    fn property(&mut self, owner: usize, name: &str, property_class: usize, offset: i32) {
        let next = self.target.read_ptr(owner, STRUCT_CHILDREN_OFFSET);
        let field = self.object(name, property_class, 0);
        self.set(field + 0x4c, offset);
        self.set(field + 0x30, next as u32);
        self.set(owner + 0x40, field as u32);
    }
    fn classes(&mut self) -> ReflectionClasses {
        let class = self.object("Class", 0, 0);
        self.set(class + 0x28, class as u32);
        ReflectionClasses {
            actor: self.object("Actor", class, 0),
            player_controller: self.object("PlayerController", class, 0),
            pawn: self.object("Pawn", class, 0),
            property: self.object("Property", class, 0),
        }
    }
}
#[test]
fn reflection_tracker_health_and_reused_class_snapshot() {
    let mut f = Fixture::new();
    let classes = f.classes();
    let class_class = f.target.read_ptr(classes.actor, OBJECT_CLASS_OFFSET);
    let monster = f.object("KFMonster", class_class, classes.pawn);
    for (name, offset) in [
        ("Health", 0x500),
        ("HealthMax", 0x504),
        ("CollisionRadius", 0x508),
        ("CollisionHeight", 0x50c),
    ] {
        f.property(monster, name, classes.property, offset);
    }
    let child = f.object("ZombieClot_STANDARD", class_class, monster);
    let actor = f.object("Clot0", child, 0);
    f.set(actor + 0x500, 90i32);
    f.set(actor + 0x504, 150.0f32);
    f.set(actor + 0x508, 16.0f32);
    f.set(actor + 0x50c, 36.0f32);
    f.set(actor + 0x14c, [100.0f32, 20.0, 36.0]);
    assert!(f.target.find_reflection_classes().complete());
    assert_eq!(
        f.target
            .find_property_offset(child, "Health", classes.property),
        Some(0x500)
    );
    let mut tracker = ActorTracker::default();
    for _ in 0..3 {
        tracker.advance_budget(&f.target, &classes, 8192);
    }
    assert_eq!(tracker.tracked.len(), 1);
    let live = read_living(&f.target, &tracker.tracked, &tracker.layout);
    assert_eq!(live.len(), 1);
    assert_eq!(live[0].health_max, Some(150));
    assert_eq!(live[0].actor.display_name, "Clot");
    assert_eq!(live[0].bounds.unwrap().max.z, 72.0);
    f.name(child, "ZombieScrake_STANDARD");
    for _ in 0..3 {
        tracker.advance_budget(&f.target, &classes, 8192);
    }
    assert_eq!(tracker.reused_classes, 1);
    assert_eq!(tracker.tracked[0].display_name, "Scrake");
    f.set(actor + 0x500, 0i32);
    assert!(read_living(&f.target, &tracker.tracked, &tracker.layout).is_empty());
    f.set(actor + 0x500, 90i32);
    f.set(actor + 0x28, classes.actor as u32);
    assert!(read_living(&f.target, &tracker.tracked, &tracker.layout).is_empty());
}
#[test]
fn viewport_local_player_scan_and_camera_properties() {
    let mut f = Fixture::new();
    let classes = f.classes();
    let cc = f.target.read_ptr(classes.actor, OBJECT_CLASS_OFFSET);
    for (name, offset) in [
        ("FovAngle", 0x600),
        ("CalcViewLocation", 0x604),
        ("CalcViewRotation", 0x610),
    ] {
        f.property(classes.player_controller, name, classes.property, offset);
    }
    let viewport_class = f.object("Viewport", cc, 0);
    let viewport = f.object("Viewport0", viewport_class, 0);
    let controller = f.object("Controller0", classes.player_controller, 0);
    let pawn = f.object("Pawn0", classes.pawn, 0);
    f.set(controller + 0x514, viewport as u32);
    f.set(pawn + 0x360, controller as u32);
    f.set(controller + 0x600, 100.0f32);
    f.set(controller + 0x604, [1.0f32, 2.0, 3.0]);
    f.set(controller + 0x610, [0i32, 16384, 0]);
    let player = f.target.find_local_player(&classes);
    assert_eq!(player.controller, controller);
    assert_eq!(player.pawn, pawn);
    assert!(f.target.still_valid(&classes, &player));
    let camera = f.target.read_camera(&player);
    assert_eq!(camera.position, Vector3::new(1.0, 2.0, 3.0));
    assert_eq!(camera.angles.yaw.as_degrees(), 90.0);
    assert_eq!(camera.fov, 100.0);
}
#[test]
fn silent_rotation_restores_tick_input_and_pawn_yaw() {
    let mut f = Fixture::new();
    let classes = f.classes();
    let controller = f.object("controller", classes.player_controller, 0);
    let pawn = f.object("pawn", classes.pawn, 0);
    f.set(controller + 0x158, [100i32, 200, 300]);
    f.set(pawn + 0x158, [400i32, 500, 600]);
    let player = LocalPlayer {
        controller,
        pawn,
        ..Default::default()
    };
    let wanted = ViewAngles::new(
        PitchAngle::from_degrees(45.0),
        YawAngle::from_degrees(90.0),
        RollAngle::from_degrees(0.0),
    );
    let silent = SilentAim::begin(&f.target, &classes, &player, wanted).unwrap();
    let mut written = f.target.read::<[i32; 3]>(controller + 0x158);
    written[0] += 3;
    written[1] -= 7;
    written[2] += 11;
    f.set(controller + 0x158, written);
    silent.restore(&f.target).unwrap();
    assert_eq!(
        f.target.read::<[i32; 3]>(controller + 0x158),
        [103, 193, 311]
    );
    assert_eq!(f.target.read::<[i32; 3]>(pawn + 0x158), [400, 493, 600]);
}
#[test]
fn weapon_edits_restore_values_and_skip_reused_objects() {
    let mut f = Fixture::new();
    let classes = f.classes();
    let cc = f.target.read_ptr(classes.actor, OBJECT_CLASS_OFFSET);
    let weapon_class = f.object("Weapon", cc, 0);
    let fire_class = f.object("Fire", cc, 0);
    f.property(classes.pawn, "Weapon", classes.property, 0x500);
    f.property(weapon_class, "FireMode", classes.property, 0x500);
    f.property(fire_class, "Spread", classes.property, 0x500);
    let pawn = f.object("pawn", classes.pawn, 0);
    let weapon = f.object("weapon", weapon_class, 0);
    let fire = f.object("fire", fire_class, 0);
    f.set(pawn + 0x500, weapon as u32);
    f.set(weapon + 0x500, fire as u32);
    f.set(fire + 0x500, 0.5f32);
    let player = LocalPlayer {
        pawn,
        ..Default::default()
    };
    let mut edits = FireEdits::default();
    edits.apply(&f.target, &classes, &player, &["Spread"], false);
    assert_eq!(f.target.read::<f32>(fire + 0x500), 0.0);
    edits.apply(&f.target, &classes, &player, &["Spread"], false);
    edits.restore(&f.target);
    assert_eq!(f.target.read::<f32>(fire + 0x500), 0.5);
    edits.apply(&f.target, &classes, &player, &["Spread"], false);
    f.set(fire + 0x28, weapon_class as u32);
    f.set(fire + 0x500, 123i32);
    edits.restore(&f.target);
    assert_eq!(f.target.read::<i32>(fire + 0x500), 123);
}
#[test]
fn rotations_and_display_names_follow_kf1_contract() {
    assert_eq!(rotator_units_to_degrees(65535), -360.0 / 65536.0);
    assert_eq!(degrees_to_rotator_units(450.0), 16384);
    assert_eq!(zed_display_name("ZombieBoss_STANDARD"), "Patriarch");
    let a = ViewAngles::new(
        PitchAngle::from_degrees(0.0),
        YawAngle::from_degrees(179.0),
        RollAngle::from_degrees(0.0),
    );
    let b = ViewAngles::new(
        PitchAngle::from_degrees(20.0),
        YawAngle::from_degrees(-179.0),
        RollAngle::from_degrees(0.0),
    );
    let c = smooth(a, b, 0.5);
    assert_eq!(c.pitch.as_degrees(), 10.0);
    assert_eq!(c.yaw.as_degrees().abs(), 180.0);
}

#[test]
fn aim_selection_uses_cone_range_visibility_and_bone_position() {
    let mut f = Fixture::new();
    let classes = f.classes();
    let controller = f.object("controller", classes.player_controller, 0);
    let player = LocalPlayer {
        controller,
        ..Default::default()
    };
    let make = |address, x, y| LiveActor {
        actor: TrackedActor {
            address,
            name: String::new(),
            display_name: String::new(),
            object_class: 0,
        },
        position: Vector3::new(x, y, 0.0),
        bounds: Some(ActorBounds::new(
            Vector3::new(x - 1.0, y - 1.0, -5.0),
            Vector3::new(x + 1.0, y + 1.0, 5.0),
        )),
        health: Some(100),
        health_max: Some(100),
    };
    let zeds = [
        make(1, 100.0, 0.0),
        make(2, 100.0, 20.0),
        make(3, -100.0, 0.0),
        make(4, 9000.0, 0.0),
    ];
    let settings = AimSettings::default();
    let mut checks = Vec::new();
    let wanted = choose_target(
        &f.target,
        &player,
        &zeds,
        &settings,
        |address| {
            if address == 2 {
                Some(Vector3::new(100.0, 10.0, 10.0))
            } else {
                None
            }
        },
        |_, _, actor| {
            checks.push(actor);
            actor != 1
        },
    )
    .unwrap();
    assert!((wanted.yaw.as_degrees() - 10f64.atan2(100.0).to_degrees()).abs() < 1e-9);
    assert!(wanted.pitch.as_degrees() > 0.0);
    assert_eq!(checks, vec![1, 2]);
    assert!(
        choose_target(
            &f.target,
            &player,
            &zeds,
            &settings,
            |_| None,
            |_, _, _| false
        )
        .is_none()
    );
}
