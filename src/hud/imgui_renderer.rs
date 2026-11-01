use super::*;
use imgui::{DrawListMut, ImColor32, TextureId, Ui};
pub struct ImguiHudRenderer<'a> {
    ui: &'a Ui,
    draw: DrawListMut<'a>,
}
fn p(v: Point) -> [f32; 2] {
    [v.x, v.y]
}
impl<'a> ImguiHudRenderer<'a> {
    pub fn new(ui: &'a Ui) -> Self {
        Self {
            ui,
            draw: ui.get_background_draw_list(),
        }
    }
    // Apply corner colors to just the vertices emitted by an operation, preserving AA coverage.
    fn shade(
        &mut self,
        start: i32,
        min: Point,
        max: Point,
        gradient: Gradient,
        radial: Option<Point>,
    ) {
        let raw = unsafe { &mut *imgui::sys::igGetBackgroundDrawList() };
        let size = Point::new((max.x - min.x).max(1.0), (max.y - min.y).max(1.0));
        for i in start..raw.VtxBuffer.Size {
            let vertex = unsafe { &mut *raw.VtxBuffer.Data.add(i as usize) };
            let value = if gradient.animated {
                let phase = radial.map_or((vertex.pos.x - min.x) / size.x, |c| {
                    (vertex.pos.y - c.y).atan2(vertex.pos.x - c.x)
                });
                let direction = if gradient.direction == GradientDirection::RightToLeft {
                    1.0
                } else {
                    -1.0
                };
                let t = ((self.ui.time() as f32 * gradient.animation_speed
                    + direction * phase * gradient.animation_spread)
                    .sin()
                    + 1.0)
                    * 0.5;
                gradient.top_left.value() * (1.0 - t) + gradient.top_right.value() * t
            } else {
                let x = ((vertex.pos.x - min.x) / size.x).clamp(0.0, 1.0);
                let y = ((vertex.pos.y - min.y) / size.y).clamp(0.0, 1.0);
                let top = gradient.top_left.value() * (1.0 - x) + gradient.top_right.value() * x;
                let bottom =
                    gradient.bottom_left.value() * (1.0 - x) + gradient.bottom_right.value() * x;
                top * (1.0 - y) + bottom * y
            };
            let alpha = (((vertex.col >> 24) & 255) as f32 * value.w) as u8;
            vertex.col = u32::from(ImColor32::from_rgba(
                (value.x * 255.0) as u8,
                (value.y * 255.0) as u8,
                (value.z * 255.0) as u8,
                alpha,
            ));
        }
    }
    fn vertex_count(&self) -> i32 {
        unsafe { (*imgui::sys::igGetBackgroundDrawList()).VtxBuffer.Size }
    }
}
impl HudRenderer for ImguiHudRenderer<'_> {
    fn add_line(&mut self, a: Point, b: Point, c: Color, t: f32) {
        self.draw
            .add_line(p(a), p(b), c.to_im_color())
            .thickness(t)
            .build();
    }
    fn add_polyline(&mut self, v: &[Point], c: Color, t: f32) {
        let points: Vec<imgui::sys::ImVec2> = v
            .iter()
            .map(|v| imgui::sys::ImVec2 { x: v.x, y: v.y })
            .collect();
        unsafe {
            imgui::sys::ImDrawList_AddPolyline(
                imgui::sys::igGetBackgroundDrawList(),
                points.as_ptr(),
                points.len() as i32,
                u32::from(c.to_im_color()),
                imgui::sys::ImDrawFlags_Closed as i32,
                t,
            );
        }
    }
    fn add_polyline_clipped(&mut self, v: &[Point], a: Point, b: Point, c: Color, t: f32) {
        let points: Vec<imgui::sys::ImVec2> = v
            .iter()
            .map(|v| imgui::sys::ImVec2 { x: v.x, y: v.y })
            .collect();
        self.draw.with_clip_rect_intersect(p(a), p(b), || unsafe {
            imgui::sys::ImDrawList_AddPolyline(
                imgui::sys::igGetBackgroundDrawList(),
                points.as_ptr(),
                points.len() as i32,
                u32::from(c.to_im_color()),
                imgui::sys::ImDrawFlags_Closed as i32,
                t,
            );
        });
    }
    fn add_filled_polyline(&mut self, v: &[Point], c: Color) {
        self.draw
            .add_polyline(
                v.iter().copied().map(p).collect::<Vec<_>>(),
                c.to_im_color(),
            )
            .filled(true)
            .build();
    }
    fn add_rectangle(&mut self, a: Point, b: Point, c: Color) {
        self.draw.add_rect(p(a), p(b), c.to_im_color()).build();
    }
    fn add_filled_rectangle(&mut self, a: Point, b: Point, c: Color) {
        self.draw
            .add_rect(p(a), p(b), c.to_im_color())
            .filled(true)
            .build();
    }
    fn add_gradient_rectangle(&mut self, a: Point, b: Point, g: Gradient) {
        self.draw.add_rect_filled_multicolor(
            p(a),
            p(b),
            g.top_left.to_im_color(),
            g.top_right.to_im_color(),
            g.bottom_right.to_im_color(),
            g.bottom_left.to_im_color(),
        );
    }
    fn add_circle(&mut self, a: Point, r: f32, c: Color, t: f32, s: i32) {
        self.draw
            .add_circle(p(a), r, c.to_im_color())
            .thickness(t)
            .num_segments(s.max(0) as u32)
            .build();
    }
    fn add_filled_circle(&mut self, a: Point, r: f32, c: Color, s: i32) {
        self.draw
            .add_circle(p(a), r, c.to_im_color())
            .num_segments(s.max(0) as u32)
            .filled(true)
            .build();
    }
    fn add_gradient_filled_circle(&mut self, a: Point, r: f32, g: Gradient, s: i32) {
        let start = self.vertex_count();
        self.add_filled_circle(a, r, Color::new(1.0, 1.0, 1.0, 1.0), s);
        self.shade(
            start,
            a - Point::new(r, r),
            a + Point::new(r, r),
            g,
            Some(a),
        );
    }
    fn add_arc(&mut self, a: Point, r: f32, min: f32, max: f32, c: Color, t: f32, s: i32) {
        unsafe {
            let raw = imgui::sys::igGetBackgroundDrawList();
            imgui::sys::ImDrawList_PathArcTo(raw, p(a).into(), r, min, max, s);
            imgui::sys::ImDrawList_PathStroke(raw, u32::from(c.to_im_color()), 0, t);
        }
    }
    fn add_image(&mut self, id: usize, a: Point, b: Point, c: Color) {
        self.draw
            .add_image(TextureId::new(id), p(a), p(b))
            .col(c.to_im_color())
            .build();
    }
    fn add_text(&mut self, a: Point, c: Color, text: &str) {
        self.draw.add_text(p(a), c.to_im_color(), text);
    }
    fn add_gradient_text(&mut self, a: Point, g: Gradient, text: &str) {
        if g.animated {
            let mut x = a.x;
            let direction = if g.direction == GradientDirection::RightToLeft {
                1.0
            } else {
                -1.0
            };
            for (i, ch) in text.chars().enumerate() {
                let t = ((self.ui.time() as f32 * g.animation_speed
                    + direction * i as f32 * g.animation_spread)
                    .sin()
                    + 1.0)
                    * 0.5;
                let c =
                    Color::from_vector(g.top_left.value() * (1.0 - t) + g.top_right.value() * t);
                let glyph = ch.to_string();
                self.add_text(Point::new(x, a.y), c, &glyph);
                x += self.calc_text_size(&glyph).x;
            }
        } else {
            let start = self.vertex_count();
            self.add_text(a, Color::new(1.0, 1.0, 1.0, 1.0), text);
            let size = self.calc_text_size(text);
            self.shade(start, a, a + size, g, None);
        }
    }
    fn calc_text_size(&mut self, text: &str) -> Point {
        let [x, y] = self.ui.calc_text_size(text);
        Point::new(x, y)
    }
}
