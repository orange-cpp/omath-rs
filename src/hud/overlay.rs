use super::{widget::*, *};
use crate::{
    primitives::{Aabb, UpAxisKind},
    projection::{Camera, CameraEngine},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityOverlayError {
    NoProjectedVertex,
    CenterProjectionFailed,
}
impl std::fmt::Display for EntityOverlayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for EntityOverlayError {}
/// Layout state is separate from the renderer so it can be stored across Lua calls without self-references.
pub struct EntityOverlay {
    pub canvas: CanvasBox,
    cursors: [Point; 4],
}
impl EntityOverlay {
    pub fn new(top: Point, bottom: Point, aspect: f32) -> Self {
        Self::from_canvas(CanvasBox::new(top, bottom, aspect))
    }
    pub fn from_canvas(canvas: CanvasBox) -> Self {
        Self {
            cursors: [
                canvas.top_right_corner,
                canvas.top_left_corner,
                canvas.top_left_corner,
                canvas.bottom_left_corner,
            ],
            canvas,
        }
    }
    pub fn from_aabb<E: CameraEngine, A: UpAxisKind>(
        camera: &Camera<E>,
        aabb: &Aabb<E::Scalar, A>,
    ) -> Result<Self, EntityOverlayError> {
        use crate::Scalar;
        let mut min = Point::new(f32::INFINITY, f32::INFINITY);
        let mut max = Point::new(f32::NEG_INFINITY, f32::NEG_INFINITY);
        for vertex in aabb.vertices() {
            let p = camera
                .world_to_screen_unclipped(vertex)
                .map_err(|_| EntityOverlayError::NoProjectedVertex)?;
            let p = Point::new(p.x.to_f64() as f32, p.y.to_f64() as f32);
            min.x = min.x.min(p.x);
            min.y = min.y.min(p.y);
            max.x = max.x.max(p.x);
            max.y = max.y.max(p.y);
        }
        Ok(Self::from_canvas(CanvasBox::from_bounds(min, max)))
    }
    pub fn contents(
        &mut self,
        renderer: &mut dyn HudRenderer,
        widgets: impl IntoIterator<Item = EntityWidget>,
    ) -> &mut Self {
        for widget in widgets {
            self.draw(renderer, &widget);
        }
        self
    }
    pub fn draw(&mut self, r: &mut dyn HudRenderer, widget: &EntityWidget) -> &mut Self {
        let points = self.canvas.as_array();
        let min = points[0];
        let max = points[2];
        match widget {
            EntityWidget::Box(w) => {
                if let Some(g) = w.glow {
                    draw_glow_polyline(r, &points, g, w.thickness);
                }
                if w.outline.value().w > 0.0 {
                    r.add_polyline(&points, w.outline, w.thickness + 2.0);
                }
                r.add_polyline(&points, w.color, w.thickness);
                match w.fill {
                    Paint::Color(c) if c.value().w > 0.0 => r.add_filled_polyline(&points, c),
                    Paint::Gradient(g) => r.add_gradient_rectangle(min, max, g),
                    _ => {}
                }
            }
            EntityWidget::CanvasGlow(w) => draw_edge_glow(r, min, max, *w),
            EntityWidget::CorneredBox(w) => {
                let length = (max.x - min.x).abs() * w.corner_ratio;
                let lines = [
                    (points[0], points[0] + Point::new(length, 0.0)),
                    (points[0], points[0] + Point::new(0.0, length)),
                    (points[3], points[3] - Point::new(0.0, length)),
                    (points[3], points[3] + Point::new(length, 0.0)),
                    (points[1], points[1] - Point::new(length, 0.0)),
                    (points[1], points[1] + Point::new(0.0, length)),
                    (points[2], points[2] - Point::new(0.0, length)),
                    (points[2], points[2] - Point::new(length, 0.0)),
                ];
                if let Some(g) = w.glow {
                    for (a, b) in lines {
                        glow_line(r, a, b, g, w.thickness);
                    }
                }
                if w.fill.value().w > 0.0 {
                    r.add_polyline(&points, w.fill, 1.0);
                    r.add_filled_polyline(&points, w.fill);
                }
                for (a, b) in lines {
                    if w.outline.value().w > 0.0 {
                        r.add_line(a, b, w.outline, w.thickness + 2.0);
                    }
                    r.add_line(a, b, w.color, w.thickness);
                }
            }
            EntityWidget::DashedBox(w) => {
                if !w.dash_len.is_finite()
                    || w.dash_len <= 0.0
                    || !w.gap_len.is_finite()
                    || w.gap_len < 0.0
                {
                    return self;
                }
                let edge = (points[1] - points[0])
                    .length()
                    .min((points[2] - points[1]).length());
                let corner = w.dash_len.min(edge / 2.0);
                for i in 0..4 {
                    let a = points[i];
                    let b = points[(i + 1) % 4];
                    let direction = (b - a).normalized();
                    r.add_line(a, a + direction * corner, w.color, w.thickness);
                    dashed_line(
                        r,
                        a + direction * corner,
                        b - direction * corner,
                        w.color,
                        w.dash_len,
                        w.gap_len,
                        w.thickness,
                    );
                    r.add_line(b - direction * corner, b, w.color, w.thickness);
                }
            }
            EntityWidget::Side(side, children) => {
                for child in children {
                    self.add_side_widget(r, *side, child);
                }
            }
            EntityWidget::SnapLine(w) => {
                r.add_line(w.start, (points[2] + points[3]) / 2.0, w.color, w.width)
            }
            EntityWidget::Skeleton(w) => {
                if w.bones.is_empty() {
                    let bones = [
                        ((0.5, 0.13), (0.5, 0.22)),
                        ((0.5, 0.22), (0.5, 0.38)),
                        ((0.5, 0.38), (0.5, 0.55)),
                        ((0.5, 0.22), (0.25, 0.25)),
                        ((0.25, 0.25), (0.13, 0.42)),
                        ((0.13, 0.42), (0.08, 0.56)),
                        ((0.5, 0.22), (0.75, 0.25)),
                        ((0.75, 0.25), (0.87, 0.42)),
                        ((0.87, 0.42), (0.92, 0.56)),
                        ((0.5, 0.55), (0.36, 0.58)),
                        ((0.36, 0.58), (0.32, 0.77)),
                        ((0.32, 0.77), (0.27, 0.97)),
                        ((0.5, 0.55), (0.64, 0.58)),
                        ((0.64, 0.58), (0.68, 0.77)),
                        ((0.68, 0.77), (0.73, 0.97)),
                    ];
                    let joint = |(x, y)| {
                        let top = points[0] + (points[1] - points[0]) * x;
                        let bot = points[3] + (points[2] - points[3]) * x;
                        top + (bot - top) * y
                    };
                    for (a, b) in bones {
                        r.add_line(joint(a), joint(b), w.color, w.thickness);
                    }
                } else {
                    for bone in &w.bones {
                        r.add_line(bone.start, bone.end, w.color, w.thickness);
                    }
                }
            }
            EntityWidget::ScanMarker(w) => {
                let size = (max.x - min.x).abs().min((max.y - min.y).abs()) * 0.5;
                let h = size * 3.0_f32.sqrt() / 2.0;
                let center =
                    Point::new((min.x + max.x) / 2.0, min.y + (max.y - min.y).abs() * 0.44);
                let tri = [
                    center + Point::new(0.0, -h * 2.0 / 3.0),
                    center + Point::new(-size / 2.0, h / 3.0),
                    center + Point::new(size / 2.0, h / 3.0),
                ];
                r.add_filled_polyline(&tri, w.color);
                if w.outline.value().w > 0.0 {
                    r.add_polyline(&tri, w.outline, w.outline_thickness);
                }
            }
            EntityWidget::AimDot(w) => r.add_filled_circle(w.position, w.radius, w.color, 0),
            EntityWidget::ProjectileAim(w) => {
                r.add_line((min + max) / 2.0, w.position, w.color, w.line_size);
                match w.figure {
                    Figure::Circle => r.add_filled_circle(w.position, w.size, w.color, 0),
                    Figure::Square => {
                        let half = Point::new(w.size, w.size) / 2.0;
                        r.add_filled_rectangle(w.position - half, w.position + half, w.color);
                    }
                }
            }
        }
        self
    }
    pub fn add_side_widget(
        &mut self,
        r: &mut dyn HudRenderer,
        side: Side,
        widget: &SideWidget,
    ) -> &mut Self {
        let index = match side {
            Side::Right => 0,
            Side::Left => 1,
            Side::Top => 2,
            Side::Bottom => 3,
        };
        let cursor = &mut self.cursors[index];
        let vertical = side == Side::Left || side == Side::Right;
        let negative = side == Side::Left || side == Side::Top;
        match widget {
            SideWidget::None => {}
            SideWidget::SpaceVertical(size) => {
                cursor.y += if side == Side::Top { -size } else { *size }
            }
            SideWidget::SpaceHorizontal(size) => {
                cursor.x += if side == Side::Left { -size } else { *size }
            }
            SideWidget::Bar(w) => draw_bar(r, self.canvas, cursor, side, w, None),
            SideWidget::DashedBar(w) => draw_bar(
                r,
                self.canvas,
                cursor,
                side,
                &w.bar,
                Some((w.dash_len, w.gap_len)),
            ),
            SideWidget::Label(w) | SideWidget::Centered(w) => {
                let size = r.calc_text_size(&w.text);
                if side == Side::Top {
                    cursor.y -= size.y;
                }
                let mut position = *cursor
                    + if vertical {
                        Point::new(
                            if negative {
                                -(w.offset + size.x)
                            } else {
                                w.offset
                            },
                            0.0,
                        )
                    } else {
                        Point::new(0.0, if negative { -w.offset } else { w.offset })
                    };
                if !vertical && matches!(widget, SideWidget::Centered(_)) {
                    position.x = (self.canvas.top_left_corner.x + self.canvas.top_right_corner.x
                        - size.x)
                        / 2.0;
                }
                draw_label(r, position, w.color, w.outlined, &w.text, w.glow);
                if side != Side::Top {
                    cursor.y += size.y;
                }
            }
            SideWidget::Icon(w) => {
                if side == Side::Top {
                    cursor.y -= w.height;
                }
                let position = *cursor
                    + if vertical {
                        Point::new(
                            if negative {
                                -(w.offset + w.width)
                            } else {
                                w.offset
                            },
                            0.0,
                        )
                    } else {
                        Point::new(0.0, if negative { -w.offset } else { w.offset })
                    };
                r.add_image(
                    w.texture_id,
                    position,
                    position + Point::new(w.width, w.height),
                    w.tint,
                );
                if side != Side::Top {
                    cursor.y += w.height;
                }
            }
            SideWidget::ProgressRing(w) => {
                if side == Side::Top {
                    cursor.y -= w.radius * 2.0;
                }
                let center = *cursor
                    + if vertical {
                        Point::new(
                            if negative {
                                -w.offset - w.radius
                            } else {
                                w.offset + w.radius
                            },
                            w.radius,
                        )
                    } else {
                        Point::new(
                            w.radius,
                            if negative {
                                -w.offset + w.radius
                            } else {
                                w.offset + w.radius
                            },
                        )
                    };
                r.add_circle(center, w.radius, w.bg, w.thickness, w.segments);
                let ratio = w.ratio.clamp(0.0, 1.0);
                if ratio > 0.0 {
                    r.add_arc(
                        center,
                        w.radius,
                        -std::f32::consts::FRAC_PI_2,
                        -std::f32::consts::FRAC_PI_2 + ratio * std::f32::consts::TAU,
                        w.color,
                        w.thickness,
                        w.segments,
                    );
                }
                if side != Side::Top {
                    cursor.y += w.radius * 2.0;
                }
            }
        }
        self
    }
}
fn draw_bar(
    r: &mut dyn HudRenderer,
    canvas: CanvasBox,
    cursor: &mut Point,
    side: Side,
    w: &Bar,
    dashes: Option<(f32, f32)>,
) {
    let vertical = side == Side::Left || side == Side::Right;
    let negative = side == Side::Left || side == Side::Top;
    let min = canvas.top_left_corner;
    let max = canvas.bottom_right_corner;
    let ratio = w.ratio.clamp(0.0, 1.0);
    let (start, step, perp, length) = if vertical {
        (
            Point::new(
                cursor.x
                    + if negative {
                        -(w.offset + w.size)
                    } else {
                        w.offset
                    },
                max.y,
            ),
            Point::new(0.0, -1.0),
            Point::new(w.size, 0.0),
            (max.y - min.y).abs(),
        )
    } else {
        (
            Point::new(
                min.x,
                cursor.y + if negative { -w.offset } else { w.offset },
            ),
            Point::new(1.0, 0.0),
            Point::new(0.0, if negative { -w.size } else { w.size }),
            (max.x - min.x).abs(),
        )
    };
    let (a, b) = bounds(start, start + step * length + perp);
    let (fill_a, fill_b) = if vertical {
        bounds(start + step * (length * ratio), start + perp)
    } else {
        bounds(start, start + step * (length * ratio) + perp)
    };
    if dashes.is_some() {
        r.add_filled_rectangle(start, start + step * length + perp, w.bg);
    } else {
        r.add_filled_rectangle(a, b, w.bg);
    }
    if dashes.is_none() {
        draw_glow_rectangle(r, fill_a, fill_b, w.glow);
    }
    draw_filled_rectangle(
        r,
        fill_a,
        fill_b,
        resolve_bar_paint(w.color, ratio, vertical),
    );
    if let Some((dash, gap)) = dashes {
        draw_dash_gaps(r, start, step, perp, length, w.outline, dash, gap);
    }
    if dashes.is_some() {
        r.add_rectangle(
            start
                - if vertical {
                    Point::new(1.0, 0.0)
                } else {
                    Point::default()
                },
            start + step * length + perp,
            w.outline,
        );
    } else {
        r.add_rectangle(a, b, w.outline);
    }
    if vertical {
        cursor.x += if negative {
            -(w.offset + w.size)
        } else {
            w.offset + w.size
        };
    } else {
        cursor.y += if negative {
            -(w.offset + w.size)
        } else {
            w.offset + w.size
        };
    }
}
fn glow_line(r: &mut dyn HudRenderer, a: Point, b: Point, g: Glow, thickness: f32) {
    let radius = g.radius.max(0.0).ceil() as i32;
    for layer in (1..=radius).rev() {
        r.add_line(
            a,
            b,
            super::render::alpha(
                g.color,
                g.intensity.max(0.0) * (1.0 - (layer - 1) as f32 / (radius + 1) as f32)
                    / radius as f32,
            ),
            thickness + layer as f32 * 2.0,
        );
    }
}
fn dashed_line(
    r: &mut dyn HudRenderer,
    a: Point,
    b: Point,
    color: Color,
    dash: f32,
    gap: f32,
    thickness: f32,
) {
    let length = (b - a).length();
    if !length.is_finite()
        || length <= 0.0
        || !dash.is_finite()
        || dash <= 0.0
        || !gap.is_finite()
        || gap < 0.0
    {
        return;
    }
    let count = ((length + gap) / (dash + gap)).floor();
    if count < 1.0 {
        return;
    }
    let offset = (length - (count * dash + (count - 1.0) * gap)) / 2.0;
    let dir = (b - a) / length;
    for i in 0..count as usize {
        let start = offset + i as f32 * (dash + gap);
        r.add_line(a + dir * start, a + dir * (start + dash), color, thickness);
    }
}

pub struct ScreenOverlay {
    pub screen_size: Point,
}
impl ScreenOverlay {
    pub fn new(screen_size: Point) -> Self {
        Self { screen_size }
    }
    pub fn contents(
        &mut self,
        r: &mut dyn HudRenderer,
        widgets: impl IntoIterator<Item = ScreenWidget>,
    ) -> &mut Self {
        for w in widgets {
            self.draw(r, &w);
        }
        self
    }
    pub fn draw(&mut self, r: &mut dyn HudRenderer, w: &ScreenWidget) -> &mut Self {
        let center = self.screen_size / 2.0;
        match w {
            ScreenWidget::Crosshair(w) => {
                let inner = w.gap;
                let outer = w.gap + w.length;
                let half = w.thickness / 2.0;
                for (a, b) in [
                    (Point::new(-half, -outer), Point::new(half, -inner)),
                    (Point::new(-half, inner), Point::new(half, outer)),
                    (Point::new(-outer, -half), Point::new(-inner, half)),
                    (Point::new(inner, -half), Point::new(outer, half)),
                ] {
                    let a = center + a;
                    let b = center + b;
                    if w.outline.value().w > 0.0 {
                        r.add_filled_rectangle(
                            a - Point::new(1.0, 1.0),
                            b + Point::new(1.0, 1.0),
                            w.outline,
                        );
                    }
                    r.add_filled_rectangle(a, b, w.color);
                }
                if w.dot_radius > 0.0 {
                    if w.outline.value().w > 0.0 {
                        r.add_filled_circle(center, w.dot_radius + 1.0, w.outline, 0);
                    }
                    r.add_filled_circle(center, w.dot_radius, w.color, 0);
                }
            }
            ScreenWidget::FovCircle(w) => {
                draw_filled_circle(r, center, w.radius, w.fill, w.segments);
                r.add_circle(center, w.radius, w.color, w.thickness, w.segments);
            }
            ScreenWidget::ThreatArrow(w) => {
                let dir = Point::new(w.angle.cos(), w.angle.sin());
                let perp = Point::new(-dir.y, dir.x);
                let base = center + dir * (w.radius - w.size);
                let wing = perp * (w.size * 0.6);
                let tri = [center + dir * w.radius, base + wing, base - wing];
                if w.outline.value().w > 0.0 {
                    r.add_polyline(&tri, w.outline, 2.0);
                }
                r.add_filled_polyline(&tri, w.color);
            }
            ScreenWidget::HitMarker(w) => {
                let fade = w.alpha.clamp(0.0, 1.0);
                for (x, y) in [(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)] {
                    let dir = Point::new(x, y) * std::f32::consts::FRAC_1_SQRT_2;
                    let a = center + dir * w.gap;
                    let b = center + dir * (w.gap + w.size);
                    if w.outline.value().w > 0.0 {
                        r.add_line(
                            a,
                            b,
                            super::render::alpha(w.outline, fade),
                            w.thickness + 2.0,
                        );
                    }
                    r.add_line(a, b, super::render::alpha(w.color, fade), w.thickness);
                }
            }
            ScreenWidget::CenterBar(w) => draw_center_bar(r, center, w, None),
            ScreenWidget::DashedCenterBar(w) => {
                draw_center_bar(r, center, &w.bar, Some((w.dash_len, w.gap_len)))
            }
            ScreenWidget::Corner(w) => {
                let top = matches!(
                    w.anchor,
                    Anchor::TopLeft | Anchor::TopCenter | Anchor::TopRight
                );
                let left = matches!(w.anchor, Anchor::TopLeft | Anchor::BottomLeft);
                let right = matches!(w.anchor, Anchor::TopRight | Anchor::BottomRight);
                let mut y = if top {
                    w.padding
                } else {
                    self.screen_size.y - w.padding
                };
                for child in &w.children {
                    match child {
                        CornerWidget::None => {}
                        CornerWidget::SpaceVertical(size) => y += if top { *size } else { -size },
                        CornerWidget::Label(label) => {
                            let size = r.calc_text_size(&label.text);
                            let x = if left {
                                w.padding
                            } else if right {
                                self.screen_size.x - w.padding - size.x
                            } else {
                                (self.screen_size.x - size.x) / 2.0
                            };
                            if !top {
                                y -= size.y;
                            }
                            draw_label(
                                r,
                                Point::new(x, y + if top { label.offset } else { -label.offset }),
                                label.color,
                                label.outlined,
                                &label.text,
                                label.glow,
                            );
                            if top {
                                y += size.y;
                            }
                        }
                    }
                }
            }
        }
        self
    }
}
fn draw_center_bar(
    r: &mut dyn HudRenderer,
    center: Point,
    w: &CenterBar,
    dashes: Option<(f32, f32)>,
) {
    let x = center.x
        + if w.side == Side::Right {
            w.offset
        } else {
            -w.offset - w.thickness
        };
    let a = Point::new(x, center.y - w.length / 2.0);
    let b = a + Point::new(w.thickness, w.length);
    let ratio = w.ratio.clamp(0.0, 1.0);
    let fill = Point::new(a.x, b.y - w.length * ratio);
    r.add_filled_rectangle(a, b, w.bg);
    if dashes.is_none() {
        if let Some(glow) = w.glow {
            draw_edge_glow(r, fill, b, glow);
        }
    }
    draw_filled_rectangle(r, fill, b, resolve_bar_paint(w.color, ratio, true));
    if let Some((dash, gap)) = dashes {
        draw_dash_gaps(
            r,
            Point::new(a.x, b.y),
            Point::new(0.0, -1.0),
            Point::new(w.thickness, 0.0),
            w.length,
            w.outline,
            dash,
            gap,
        );
    }
    r.add_rectangle(a, b, w.outline);
}
