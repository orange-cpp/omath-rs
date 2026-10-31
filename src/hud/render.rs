use super::{widget::*, *};
pub(super) fn alpha(c: Color, scale: f32) -> Color {
    let v = c.value();
    Color::new(v.x, v.y, v.z, v.w * scale)
}
pub(super) fn bounds(a: Point, b: Point) -> (Point, Point) {
    (
        Point::new(a.x.min(b.x), a.y.min(b.y)),
        Point::new(a.x.max(b.x), a.y.max(b.y)),
    )
}
pub fn draw_label(
    r: &mut dyn HudRenderer,
    position: Point,
    paint: Paint,
    outlined: Outlined,
    text: &str,
    glow: Option<Glow>,
) {
    if let Some(g) = glow {
        let radius = g.radius.max(0.0).ceil() as i32;
        for layer in (1..=radius).rev() {
            let color = alpha(
                g.color,
                g.intensity.max(0.0) * (1.0 - (layer - 1) as f32 / (radius + 1) as f32)
                    / radius as f32,
            );
            for x in -layer..=layer {
                for y in [-layer, layer] {
                    r.add_text(position + Point::new(x as f32, y as f32), color, text);
                }
            }
            for y in (-layer + 1)..layer {
                for x in [-layer, layer] {
                    r.add_text(position + Point::new(x as f32, y as f32), color, text);
                }
            }
        }
    }
    if outlined == Outlined::On {
        for x in -1..=1 {
            for y in -1..=1 {
                if x != 0 || y != 0 {
                    r.add_text(
                        position + Point::new(x as f32, y as f32),
                        Color::new(0.0, 0.0, 0.0, 1.0),
                        text,
                    );
                }
            }
        }
    }
    match paint {
        Paint::Color(c) => r.add_text(position, c, text),
        Paint::Gradient(g) => r.add_gradient_text(position, g, text),
    }
}
pub fn draw_glow_polyline(r: &mut dyn HudRenderer, points: &[Point], glow: Glow, thickness: f32) {
    let radius = glow.radius.max(0.0).ceil() as i32;
    for layer in (1..=radius).rev() {
        r.add_polyline(
            points,
            alpha(
                glow.color,
                glow.intensity.max(0.0) * (1.0 - (layer - 1) as f32 / (radius + 1) as f32)
                    / radius as f32,
            ),
            thickness + layer as f32 * 2.0,
        );
    }
}
pub fn draw_glow_rectangle(r: &mut dyn HudRenderer, min: Point, max: Point, glow: Option<Glow>) {
    if let Some(g) = glow.filter(|g| g.radius > 0.0) {
        let (min, max) = bounds(min, max);
        draw_glow_polyline(r, &CanvasBox::from_bounds(min, max).as_array(), g, 1.0);
    }
}
pub fn draw_filled_rectangle(r: &mut dyn HudRenderer, min: Point, max: Point, paint: Paint) {
    let (min, max) = bounds(min, max);
    match paint {
        Paint::Color(c) => r.add_filled_rectangle(min, max, c),
        Paint::Gradient(g) => r.add_gradient_rectangle(min, max, g),
    }
}
pub fn draw_filled_circle(
    r: &mut dyn HudRenderer,
    center: Point,
    radius: f32,
    paint: Paint,
    segments: i32,
) {
    match paint {
        Paint::Color(c) if c.value().w > 0.0 => r.add_filled_circle(center, radius, c, segments),
        Paint::Gradient(g) => r.add_gradient_filled_circle(center, radius, g, segments),
        _ => {}
    }
}
pub fn resolve_bar_paint(paint: BarPaint, ratio: f32, vertical: bool) -> Paint {
    match paint {
        BarPaint::Color(c) => c.into(),
        BarPaint::Gradient(g) => g.into(),
        BarPaint::BarGradient(g) => {
            let ratio = ratio.clamp(0.0, 1.0);
            let c = Color::from_vector(
                g.empty_color.value() * (1.0 - ratio) + g.full_color.value() * ratio,
            );
            if vertical {
                Gradient::new(c, c, g.empty_color, g.empty_color).into()
            } else {
                Gradient::new(g.empty_color, c, c, g.empty_color).into()
            }
        }
    }
}
pub fn draw_edge_glow(r: &mut dyn HudRenderer, min: Point, max: Point, g: CanvasGlow) {
    if g.glow.radius <= 0.0 {
        return;
    }
    let layers = g.layers.max(2);
    let rounding = g
        .rounding
        .clamp(0.0, (max.x - min.x).abs().min((max.y - min.y).abs()) * 0.5);
    let stroke = (g.glow.radius / (layers - 1) as f32 * 2.0).max(1.0);
    for layer in 0..layers {
        let progress = layer as f32 / (layers - 1) as f32;
        let expansion = g.glow.radius * (1.0 - progress);
        let color = alpha(
            g.glow.color,
            g.glow.intensity.max(0.0) * progress * progress * (3.0 - 2.0 * progress) * 0.35,
        );
        let a = min - Point::new(expansion, expansion);
        let b = max + Point::new(expansion, expansion);
        let radius = rounding + expansion;
        let centers = [
            a + Point::new(radius, radius),
            Point::new(b.x - radius, a.y + radius),
            b - Point::new(radius, radius),
            Point::new(a.x + radius, b.y - radius),
        ];
        let mut points = Vec::with_capacity(48);
        for (corner, center) in centers.into_iter().enumerate() {
            for segment in 0..12 {
                let angle = std::f32::consts::PI
                    + (corner as f32 + segment as f32 / 11.0) * std::f32::consts::FRAC_PI_2;
                points.push(center + Point::new(angle.cos() * radius, angle.sin() * radius));
            }
        }
        let extent = f32::MAX * 0.25;
        for (a, b) in [
            (Point::new(-extent, -extent), Point::new(extent, min.y)),
            (Point::new(-extent, max.y), Point::new(extent, extent)),
            (Point::new(-extent, min.y), Point::new(min.x, max.y)),
            (Point::new(max.x, min.y), Point::new(extent, max.y)),
        ] {
            r.add_polyline_clipped(&points, a, b, color, stroke);
        }
    }
}
#[allow(clippy::too_many_arguments)]
pub fn draw_dash_gaps(
    r: &mut dyn HudRenderer,
    origin: Point,
    step_dir: Point,
    perp: Point,
    length: f32,
    color: Color,
    dash: f32,
    gap: f32,
) {
    if !length.is_finite()
        || length <= 0.0
        || !dash.is_finite()
        || dash <= 0.0
        || !gap.is_finite()
        || gap < 0.0
    {
        return;
    }
    let step = dash + gap;
    let n = ((length + gap) / step).floor();
    if n < 1.0 {
        return;
    }
    let offset = (length - (n * dash + (n - 1.0) * gap)) / 2.0;
    let mut fill = |start: f32, end: f32| {
        let (a, b) = bounds(origin + step_dir * start, origin + step_dir * end + perp);
        r.add_filled_rectangle(a, b, color);
    };
    if offset > 0.0 {
        fill(0.0, offset);
    }
    for i in 0..n as usize {
        let start = offset + i as f32 * step + dash;
        if i + 1 < n as usize && start < length {
            fill(start, (start + gap).min(length));
        }
    }
    let trail = offset + n * dash + (n - 1.0) * gap;
    if trail < length {
        fill(trail, length);
    }
}
