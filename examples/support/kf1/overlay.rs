use super::reader::*;
use omath::{
    Color, Vector3,
    engines::unreal::Camera,
    hud::{widget::*, *},
    primitives::{Aabb, UpZ},
    projection::{FieldOfView, ViewPort},
    rev_eng::{MemoryRead, shift_address},
};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
pub const SCAN_BUDGET: usize = 8192;
pub const MAX_TRACKED: usize = 512;
pub const RADAR_RADIUS: f32 = 110.0;
pub const RADAR_MARGIN: f32 = 24.0;
pub const PLAYER_SEARCH_INTERVAL: Duration = Duration::from_secs(1);
pub const MAX_PLAUSIBLE_HEALTH: i32 = 100000;
pub type ActorBounds = Aabb<f64, UpZ>;
#[derive(Clone, Debug)]
pub struct EspSettings {
    pub zeds: bool,
    pub allies: bool,
    pub ally_color: Color,
    pub max_distance: f64,
    pub boxes: bool,
    pub color: Color,
    pub fill_alpha: f32,
    pub health_bar: bool,
    pub name: bool,
    pub health_text: bool,
    pub distance: bool,
    pub radar: bool,
    pub radar_range: f32,
    pub origin_marker: bool,
    pub status_panel: bool,
    pub font_size: f32,
    #[cfg(feature = "imgui")]
    pub font: Option<imgui::FontId>,
}
impl Default for EspSettings {
    fn default() -> Self {
        Self {
            zeds: true,
            allies: true,
            ally_color: Color::new(0.6, 0.8, 1.0, 1.0),
            max_distance: 8000.0,
            boxes: true,
            color: Color::new(0.31, 0.86, 0.47, 1.0),
            fill_alpha: 0.12,
            health_bar: true,
            name: true,
            health_text: true,
            distance: true,
            radar: true,
            radar_range: 3000.0,
            origin_marker: true,
            status_panel: true,
            font_size: 16.0,
            #[cfg(feature = "imgui")]
            font: None,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WindowRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct ActorLayout {
    pub health: Option<i32>,
    pub collision_radius: Option<i32>,
    pub collision_height: Option<i32>,
    pub health_max: Option<i32>,
    pub base: Option<i32>,
}
pub fn zed_display_name(class_name: &str) -> String {
    let name = class_name
        .strip_prefix("Zombie")
        .unwrap_or(class_name)
        .split('_')
        .next()
        .unwrap_or("");
    if name.eq_ignore_ascii_case("Boss") {
        return "Patriarch".into();
    }
    [
        "Bloat",
        "Clot",
        "Crawler",
        "Fleshpound",
        "Gorefast",
        "Husk",
        "Scrake",
        "Siren",
        "Stalker",
    ]
    .into_iter()
    .find(|known| name.eq_ignore_ascii_case(known))
    .unwrap_or(name)
    .into()
}
#[derive(Clone, Debug)]
pub struct ClassInfo {
    pub name: String,
    pub is_zed: bool,
    pub is_ally: bool,
    pub is_weapon: bool,
    pub name_index: i32,
    pub checked_sweep: usize,
}
#[derive(Clone, Debug)]
pub struct TrackedActor {
    pub address: usize,
    pub name: String,
    pub display_name: String,
    pub object_class: usize,
}
#[derive(Default)]
pub struct ActorTracker {
    table: Vec<u32>,
    cursor: usize,
    sweep: usize,
    pub reused_classes: usize,
    pub last_reused_class: String,
    classes: HashMap<usize, ClassInfo>,
    pub layout: ActorLayout,
    progress: Vec<TrackedActor>,
    pub tracked: Vec<TrackedActor>,
    allies_progress: Vec<TrackedActor>,
    pub allies: Vec<TrackedActor>,
    weapons_progress: Vec<TrackedActor>,
    pub weapons: Vec<TrackedActor>,
    replication_info_offset: Option<i32>,
    player_name_offset: Option<i32>,
    capped_progress: bool,
    pub capped: bool,
}
impl ActorTracker {
    fn start_sweep<M: MemoryRead>(&mut self, target: &Target<M>) {
        self.tracked = std::mem::take(&mut self.progress);
        self.allies = std::mem::take(&mut self.allies_progress);
        self.weapons = std::mem::take(&mut self.weapons_progress);
        self.capped = self.capped_progress;
        self.capped_progress = false;
        self.table = target.read_object_table();
        self.cursor = 0;
        self.sweep += 1;
    }
    fn class_info<M: MemoryRead>(
        &mut self,
        target: &Target<M>,
        reflection: &ReflectionClasses,
        class: usize,
    ) -> ClassInfo {
        let name_address = shift_address(class, target.names.object_name_offset);
        let index = target.read::<i32>(name_address);
        let mut replaced = String::new();
        if let Some(info) = self.classes.get_mut(&class) {
            if info.checked_sweep == self.sweep || index == info.name_index {
                info.checked_sweep = self.sweep;
                return info.clone();
            }
            replaced = info.name.clone();
            self.classes.remove(&class);
        }
        let zed = target.find_base_class(class, "KFMonster");
        let ally = if zed != 0 || target.find_base_class(class, "KFHumanPawnEnemy") != 0 {
            0
        } else {
            target.find_base_class(class, "KFHumanPawn")
        };
        let layout_class = if zed != 0 { zed } else { ally };
        if layout_class != 0 && self.layout.health.is_none() {
            let property =
                |name| target.find_property_offset(layout_class, name, reflection.property);
            self.layout = ActorLayout {
                health: property("Health"),
                collision_radius: property("CollisionRadius"),
                collision_height: property("CollisionHeight"),
                health_max: property("HealthMax"),
                base: property("Base"),
            };
        }
        let info = ClassInfo {
            name: target.object_name(class),
            is_zed: zed != 0,
            is_ally: ally != 0,
            is_weapon: target.find_base_class(class, "WeaponAttachment") != 0,
            name_index: index,
            checked_sweep: self.sweep,
        };
        if !replaced.is_empty() && (info.is_zed || info.is_ally) {
            self.last_reused_class = format!("{} took over the address of {replaced}", info.name);
            self.reused_classes += 1;
        }
        self.classes.insert(class, info.clone());
        info
    }
    fn ally_name<M: MemoryRead>(
        &mut self,
        target: &Target<M>,
        reflection: &ReflectionClasses,
        pawn: usize,
    ) -> String {
        if self.replication_info_offset.is_none() {
            self.replication_info_offset = target.find_property_offset(
                target.read_ptr(pawn, OBJECT_CLASS_OFFSET),
                "PlayerReplicationInfo",
                reflection.property,
            );
        }
        let info = self
            .replication_info_offset
            .map_or(0, |offset| target.read_ptr(pawn, offset as isize));
        if info == 0 {
            return "Ally".into();
        }
        if self.player_name_offset.is_none() {
            self.player_name_offset = target.find_property_offset(
                target.read_ptr(info, OBJECT_CLASS_OFFSET),
                "PlayerName",
                reflection.property,
            );
        }
        let name = self.player_name_offset.map_or_else(String::new, |offset| {
            target.read_string(shift_address(info, offset as isize))
        });
        if name.is_empty() { "Ally".into() } else { name }
    }
    pub fn advance<M: MemoryRead>(&mut self, target: &Target<M>, reflection: &ReflectionClasses) {
        self.advance_budget(target, reflection, SCAN_BUDGET);
    }
    pub fn advance_budget<M: MemoryRead>(
        &mut self,
        target: &Target<M>,
        reflection: &ReflectionClasses,
        budget: usize,
    ) {
        if self.cursor >= self.table.len() {
            self.start_sweep(target);
        }
        let end = (self.cursor + budget).min(self.table.len());
        while self.cursor < end {
            let object = self.table[self.cursor] as usize;
            self.cursor += 1;
            if object == 0 {
                continue;
            }
            let class = target.read_ptr(object, OBJECT_CLASS_OFFSET);
            if class == 0 {
                continue;
            }
            let info = self.class_info(target, reflection, class);
            if info.is_ally {
                let display_name = self.ally_name(target, reflection, object);
                self.allies_progress.push(TrackedActor {
                    address: object,
                    name: target.object_name(object),
                    display_name,
                    object_class: class,
                });
            } else if info.is_weapon {
                self.weapons_progress.push(TrackedActor {
                    address: object,
                    name: String::new(),
                    display_name: String::new(),
                    object_class: class,
                });
            } else if info.is_zed {
                if self.progress.len() >= MAX_TRACKED {
                    self.capped_progress = true;
                    continue;
                }
                self.progress.push(TrackedActor {
                    address: object,
                    name: target.object_name(object),
                    display_name: zed_display_name(&info.name),
                    object_class: class,
                });
            }
        }
    }
}
pub fn refresh_local_player<M: MemoryRead>(
    target: &Target<M>,
    reflection: &ReflectionClasses,
    player: &mut LocalPlayer,
    last_search: &mut Instant,
) {
    if player.pawn != 0
        && target.read_ptr(player.pawn, ACTOR_CONTROLLER_OFFSET) != player.controller
    {
        player.pawn = 0;
    }
    let listener = target.audio_listener();
    let controller = if listener != 0 {
        target.read_ptr(listener, ACTOR_CONTROLLER_OFFSET)
    } else {
        0
    };
    let new_pawn = listener != player.pawn
        && controller != 0
        && target.is_kind_of(
            target.read_ptr(controller, OBJECT_CLASS_OFFSET),
            reflection.player_controller,
        )
        && target.is_local_controller(controller);
    if (target.still_valid(reflection, player) && !new_pawn)
        || last_search.elapsed() < PLAYER_SEARCH_INTERVAL
    {
        return;
    }
    *last_search = Instant::now();
    *player = target.find_local_player(reflection);
}
pub fn read_bounds<M: MemoryRead>(
    target: &Target<M>,
    actor: usize,
    layout: &ActorLayout,
) -> Option<ActorBounds> {
    let radius = target.read::<f32>(shift_address(actor, layout.collision_radius? as isize)) as f64;
    let half = target.read::<f32>(shift_address(actor, layout.collision_height? as isize)) as f64;
    if !(radius > 0.0 && radius < 1000.0 && half > 0.0 && half < 1000.0) {
        return None;
    }
    let center = target.read_location(actor, ACTOR_LOCATION_OFFSET);
    Some(ActorBounds::new(
        center - Vector3::new(radius, radius, half),
        center + Vector3::new(radius, radius, half),
    ))
}
pub fn read_health_max<M: MemoryRead>(
    target: &Target<M>,
    actor: usize,
    layout: &ActorLayout,
) -> Option<i32> {
    let address = shift_address(actor, layout.health_max? as isize);
    let value = target.read::<i32>(address);
    if (1..=MAX_PLAUSIBLE_HEALTH).contains(&value) {
        return Some(value);
    }
    let value = target.read::<f32>(address);
    if value >= 1.0 && value <= MAX_PLAUSIBLE_HEALTH as f32 {
        Some(value as i32)
    } else {
        None
    }
}
#[derive(Clone, Debug)]
pub struct LiveActor {
    pub actor: TrackedActor,
    pub position: Vector3<f64>,
    pub bounds: Option<ActorBounds>,
    pub health: Option<i32>,
    pub health_max: Option<i32>,
}
pub fn read_living<M: MemoryRead>(
    target: &Target<M>,
    tracked: &[TrackedActor],
    layout: &ActorLayout,
) -> Vec<LiveActor> {
    let mut result = Vec::new();
    for actor in tracked {
        target.clear_read_error();
        if target.read_ptr(actor.address, OBJECT_CLASS_OFFSET) != actor.object_class {
            continue;
        }
        let health = layout
            .health
            .map(|offset| target.read::<i32>(shift_address(actor.address, offset as isize)));
        if health.is_some_and(|h| h <= 0) {
            continue;
        }
        let live = LiveActor {
            actor: actor.clone(),
            position: target.read_location(actor.address, ACTOR_LOCATION_OFFSET),
            bounds: read_bounds(target, actor.address, layout),
            health,
            health_max: read_health_max(target, actor.address, layout),
        };
        if !target.read_failed() {
            result.push(live);
        }
    }
    result
}
pub fn read_living_allies<M: MemoryRead>(
    target: &Target<M>,
    tracker: &ActorTracker,
    local_pawn: usize,
) -> Vec<LiveActor> {
    read_living(target, &tracker.allies, &tracker.layout)
        .into_iter()
        .filter(|ally| ally.actor.address != local_pawn)
        .collect()
}
pub fn read_ally_weapons<M: MemoryRead>(
    target: &Target<M>,
    tracker: &ActorTracker,
    allies: &[LiveActor],
) -> Vec<usize> {
    let Some(base) = tracker.layout.base else {
        return Vec::new();
    };
    tracker
        .weapons
        .iter()
        .filter(|weapon| {
            target.read_ptr(weapon.address, OBJECT_CLASS_OFFSET) == weapon.object_class
                && allies.iter().any(|ally| {
                    target.read_ptr(weapon.address, base as isize) == ally.actor.address
                })
        })
        .map(|weapon| weapon.address)
        .collect()
}
pub fn draw_actor_box(
    r: &mut dyn HudRenderer,
    camera: &Camera,
    live: &LiveActor,
    bounds: &ActorBounds,
    distance: f64,
    color: Color,
    settings: &EspSettings,
) -> bool {
    let Ok(mut overlay) = EntityOverlay::from_aabb(camera, bounds) else {
        return false;
    };
    let v = color.value();
    let black = Color::new(0.0, 0.0, 0.0, 1.0);
    let white = Color::new(1.0, 1.0, 1.0, 1.0);
    let outlined = {
        #[cfg(feature = "imgui")]
        {
            if settings.font.is_some() {
                Outlined::Off
            } else {
                Outlined::On
            }
        }
        #[cfg(not(feature = "imgui"))]
        {
            Outlined::On
        }
    };
    if settings.boxes {
        overlay.draw(
            r,
            &EntityWidget::CorneredBox(CorneredBox {
                color,
                fill: Color::new(v.x, v.y, v.z, settings.fill_alpha),
                outline: black,
                ..Default::default()
            }),
        );
    }
    if settings.health_bar {
        if let Some(max) = live.health_max {
            overlay.add_side_widget(
                r,
                Side::Left,
                &SideWidget::Bar(Bar {
                    color: BarGradient {
                        empty_color: Color::new(0.9, 0.2, 0.2, 1.0),
                        full_color: color,
                    }
                    .into(),
                    outline: black,
                    bg: Color::new(0.1, 0.1, 0.1, 0.8),
                    size: 4.0,
                    ratio: live
                        .health
                        .map_or(1.0, |h| (h as f32 / max as f32).clamp(0.0, 1.0)),
                    ..Default::default()
                }),
            );
        }
    }
    if settings.name {
        overlay.add_side_widget(
            r,
            Side::Top,
            &SideWidget::Centered(Label {
                color: white.into(),
                offset: 2.0,
                outlined,
                text: live.actor.display_name.clone(),
                glow: None,
            }),
        );
    }
    if settings.health_text {
        if let Some(health) = live.health {
            overlay.add_side_widget(
                r,
                Side::Right,
                &SideWidget::Label(Label {
                    color: color.into(),
                    offset: 2.0,
                    outlined,
                    text: format!("{health} HP"),
                    glow: None,
                }),
            );
        }
    }
    if settings.distance {
        overlay.add_side_widget(
            r,
            Side::Bottom,
            &SideWidget::Centered(Label {
                color: white.into(),
                offset: 2.0,
                outlined,
                text: format!("{distance:.0}u"),
                glow: None,
            }),
        );
    }
    true
}
pub fn radar_center(rect: WindowRect) -> Point {
    Point::new(
        rect.width as f32 - RADAR_MARGIN - RADAR_RADIUS,
        RADAR_MARGIN + RADAR_RADIUS,
    )
}
pub fn draw_radar_background(r: &mut dyn HudRenderer, rect: WindowRect, range: f32) {
    let c = radar_center(rect);
    let line = Color::from_rgba(210, 210, 210, 40);
    r.add_filled_circle(c, RADAR_RADIUS, Color::from_rgba(0, 0, 0, 120), 0);
    r.add_circle(c, RADAR_RADIUS, Color::from_rgba(210, 210, 210, 90), 1.5, 0);
    r.add_circle(c, RADAR_RADIUS * 0.5, line, 1.0, 0);
    r.add_line(
        c - Point::new(RADAR_RADIUS, 0.0),
        c + Point::new(RADAR_RADIUS, 0.0),
        line,
        1.0,
    );
    r.add_line(
        c - Point::new(0.0, RADAR_RADIUS),
        c + Point::new(0.0, RADAR_RADIUS),
        line,
        1.0,
    );
    r.add_filled_polyline(
        &[
            c + Point::new(0.0, -7.0),
            c + Point::new(-5.0, 5.0),
            c + Point::new(5.0, 5.0),
        ],
        Color::from_rgba(255, 255, 255, 220),
    );
    r.add_text(
        c + Point::new(-RADAR_RADIUS, RADAR_RADIUS + 4.0),
        Color::from_rgba(210, 210, 210, 140),
        &format!("radar {range:.0}u"),
    );
}
pub fn draw_radar_blip(
    r: &mut dyn HudRenderer,
    rect: WindowRect,
    camera: &Camera,
    position: Vector3<f64>,
    range: f32,
    color: Color,
) {
    if range <= 0.0 || !range.is_finite() {
        return;
    }
    let mut offset =
        omath::algorithm::world_to_radar(camera, position, (RADAR_RADIUS / range) as f64, None);
    let length = offset.length();
    let pinned = length > RADAR_RADIUS;
    if pinned {
        offset *= RADAR_RADIUS / length;
    }
    r.add_filled_circle(
        radar_center(rect) + Point::new(offset.x as f32, offset.y as f32),
        if pinned { 2.0 } else { 3.0 },
        color,
        0,
    );
}
pub fn draw_status(r: &mut dyn HudRenderer, text: &str) {
    r.add_filled_rectangle(
        Point::new(12.0, 12.0),
        Point::new(392.0, 152.0),
        Color::from_rgba(0, 0, 0, 140),
    );
    r.add_text(
        Point::new(24.0, 24.0),
        Color::from_rgba(220, 220, 220, 255),
        text,
    );
}
#[allow(clippy::too_many_arguments)] // Mirrors the shared observer/injected frame inputs.
pub fn draw_overlay<M: MemoryRead>(
    r: &mut dyn HudRenderer,
    target: &Target<M>,
    rect: WindowRect,
    player: &LocalPlayer,
    tracker: &ActorTracker,
    zeds: &[LiveActor],
    allies: &[LiveActor],
    settings: &EspSettings,
) {
    if !player.valid() {
        if settings.status_panel {
            draw_status(r, "kf1 overlay\nno local player, waiting for a level");
        }
        return;
    }
    let state = target.read_camera(player);
    let camera = Camera::new(
        state.position,
        state.angles,
        ViewPort::new(rect.width as f32, rect.height as f32),
        FieldOfView::from_degrees(state.fov as f32),
        1.0,
        32768.0,
    );
    let origin = Vector3::default();
    let projected = camera.world_to_screen(origin);
    if settings.radar {
        draw_radar_background(r, rect, settings.radar_range);
        draw_radar_blip(
            r,
            rect,
            &camera,
            origin,
            settings.radar_range,
            Color::from_rgba(255, 170, 0, 255),
        );
    }
    if settings.origin_marker {
        if let Ok(p) = projected {
            let p = Point::new(p.x as f32, p.y as f32);
            let c = Color::from_rgba(255, 170, 0, 200);
            r.add_filled_circle(p, 4.0, Color::from_rgba(255, 170, 0, 255), 0);
            r.add_circle(p, 14.0, c, 2.0, 0);
            for direction in [-1.0, 1.0] {
                r.add_line(
                    p + Point::new(direction * 24.0, 0.0),
                    p + Point::new(direction * 16.0, 0.0),
                    c,
                    2.0,
                );
            }
            r.add_text(
                p + Point::new(20.0, -8.0),
                Color::new(1.0, 1.0, 1.0, 1.0),
                &format!(
                    "world origin {{0, 0, 0}}\n{:.0} units away",
                    state.position.distance_to(origin)
                ),
            );
        }
    }
    let mut drawn = 0;
    for zed in zeds {
        let distance = state.position.distance_to(zed.position);
        if settings.radar {
            draw_radar_blip(
                r,
                rect,
                &camera,
                zed.position,
                settings.radar_range,
                Color::from_rgba(80, 220, 120, 255),
            );
        }
        if settings.zeds && distance <= settings.max_distance {
            if let Some(bounds) = &zed.bounds {
                if draw_actor_box(r, &camera, zed, bounds, distance, settings.color, settings) {
                    drawn += 1;
                }
            }
        }
    }
    if settings.allies {
        for ally in allies {
            let distance = state.position.distance_to(ally.position);
            if settings.radar {
                draw_radar_blip(
                    r,
                    rect,
                    &camera,
                    ally.position,
                    settings.radar_range,
                    settings.ally_color,
                );
            }
            if distance <= settings.max_distance {
                if let Some(bounds) = &ally.bounds {
                    draw_actor_box(
                        r,
                        &camera,
                        ally,
                        bounds,
                        distance,
                        settings.ally_color,
                        settings,
                    );
                }
            }
        }
    }
    if settings.status_panel {
        draw_status(
            r,
            &format!(
                "kf1 overlay {}x{}   END quit\ncamera  {:.0} {:.0} {:.0}\nangles  pitch {:.1} yaw {:.1} roll {:.1}\nfov     {:.1}\norigin  {}\nzeds    {}{} tracked, {} on screen",
                rect.width,
                rect.height,
                state.position.x,
                state.position.y,
                state.position.z,
                state.angles.pitch.as_degrees(),
                state.angles.yaw.as_degrees(),
                state.angles.roll.as_degrees(),
                state.fov,
                projected.map_or_else(
                    |_| "off screen".into(),
                    |p| format!("{:.0}, {:.0} on screen", p.x, p.y)
                ),
                tracker.tracked.len(),
                if tracker.capped { " (capped)" } else { "" },
                drawn
            ),
        );
    }
}
#[cfg(feature = "imgui")]
pub fn menu(ui: &imgui::Ui, settings: &mut EspSettings, tracker: &ActorTracker) {
    ui.window("Killing Floor overlay")
        .size([350.0, 460.0], imgui::Condition::FirstUseEver)
        .build(|| {
            ui.checkbox("Zeds", &mut settings.zeds);
            ui.checkbox("Allies", &mut settings.allies);
            ui.checkbox("Boxes", &mut settings.boxes);
            ui.checkbox("Health bar", &mut settings.health_bar);
            ui.checkbox("Name", &mut settings.name);
            ui.checkbox("Health text", &mut settings.health_text);
            ui.checkbox("Distance", &mut settings.distance);
            ui.checkbox("Radar", &mut settings.radar);
            ui.checkbox("World origin", &mut settings.origin_marker);
            ui.checkbox("Status panel", &mut settings.status_panel);
            let mut color = settings.color.value().as_array();
            if ui.color_edit4("Zed color", &mut color) {
                settings.color = Color::from_vector(color.into());
            }
            let mut color = settings.ally_color.value().as_array();
            if ui.color_edit4("Ally color", &mut color) {
                settings.ally_color = Color::from_vector(color.into());
            }
            let mut distance = settings.max_distance as f32;
            if ui.slider("Range", 100.0, 20000.0, &mut distance) {
                settings.max_distance = distance as f64;
            }
            ui.slider("Radar range", 100.0, 10000.0, &mut settings.radar_range);
            ui.slider("Fill alpha", 0.0, 1.0, &mut settings.fill_alpha);
            ui.slider("Font size", 10.0, 30.0, &mut settings.font_size);
            ui.text(format!(
                "{} zeds / {} allies / {} weapons",
                tracker.tracked.len(),
                tracker.allies.len(),
                tracker.weapons.len()
            ));
            ui.text(format!("Reused classes: {}", tracker.reused_classes));
            ui.text_wrapped(&tracker.last_reused_class);
        });
}
