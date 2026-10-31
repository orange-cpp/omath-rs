//! Renderer-independent entity and screen overlays.
mod overlay;
mod render;
pub mod widget;
pub use overlay::*;
pub use render::*;
#[cfg(feature = "imgui")]
mod imgui_renderer;
use crate::{Color, Vector2};
#[cfg(feature = "imgui")]
pub use imgui_renderer::ImguiHudRenderer;
pub type Point = Vector2<f32>;
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CanvasBox {
    pub top_left_corner: Point,
    pub top_right_corner: Point,
    pub bottom_left_corner: Point,
    pub bottom_right_corner: Point,
}
impl CanvasBox {
    pub fn new(top: Point, bottom: Point, ratio: f32) -> Self {
        let half = (top.y - bottom.y).abs() / ratio;
        Self::from_bounds(
            Point::new(top.x - half, top.y),
            Point::new(top.x + half, bottom.y),
        )
    }
    pub fn from_bounds(min: Point, max: Point) -> Self {
        Self {
            top_left_corner: min,
            top_right_corner: Point::new(max.x, min.y),
            bottom_left_corner: Point::new(min.x, max.y),
            bottom_right_corner: max,
        }
    }
    pub fn as_array(&self) -> [Point; 4] {
        [
            self.top_left_corner,
            self.top_right_corner,
            self.bottom_right_corner,
            self.bottom_left_corner,
        ]
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GradientDirection {
    #[default]
    RightToLeft,
    LeftToRight,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gradient {
    pub top_left: Color,
    pub top_right: Color,
    pub bottom_right: Color,
    pub bottom_left: Color,
    pub animated: bool,
    pub animation_speed: f32,
    pub animation_spread: f32,
    pub direction: GradientDirection,
}
impl Gradient {
    pub fn new(top_left: Color, top_right: Color, bottom_right: Color, bottom_left: Color) -> Self {
        Self {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
            animated: false,
            animation_speed: 4.0,
            animation_spread: 0.7,
            direction: GradientDirection::RightToLeft,
        }
    }
}
impl Default for Gradient {
    fn default() -> Self {
        Self::new(
            Color::default(),
            Color::default(),
            Color::default(),
            Color::default(),
        )
    }
}

/// Texture handles belong to the renderer. The ImGui implementation uses `imgui::TextureId::id()`.
pub trait HudRendererInterface {
    fn add_line(&mut self, start: Point, end: Point, color: Color, thickness: f32);
    fn add_polyline(&mut self, vertices: &[Point], color: Color, thickness: f32);
    fn add_polyline_clipped(
        &mut self,
        vertices: &[Point],
        _min: Point,
        _max: Point,
        color: Color,
        thickness: f32,
    ) {
        self.add_polyline(vertices, color, thickness);
    }
    fn add_filled_polyline(&mut self, vertices: &[Point], color: Color);
    fn add_rectangle(&mut self, min: Point, max: Point, color: Color);
    fn add_filled_rectangle(&mut self, min: Point, max: Point, color: Color);
    fn add_gradient_rectangle(&mut self, min: Point, max: Point, gradient: Gradient) {
        self.add_filled_rectangle(min, max, gradient.top_left);
    }
    fn add_circle(
        &mut self,
        center: Point,
        radius: f32,
        color: Color,
        thickness: f32,
        segments: i32,
    );
    fn add_filled_circle(&mut self, center: Point, radius: f32, color: Color, segments: i32);
    fn add_gradient_filled_circle(
        &mut self,
        center: Point,
        radius: f32,
        gradient: Gradient,
        segments: i32,
    ) {
        self.add_filled_circle(center, radius, gradient.top_left, segments);
    }
    #[allow(clippy::too_many_arguments)]
    fn add_arc(
        &mut self,
        center: Point,
        radius: f32,
        min: f32,
        max: f32,
        color: Color,
        thickness: f32,
        segments: i32,
    );
    fn add_image(&mut self, texture: usize, min: Point, max: Point, tint: Color);
    fn add_text(&mut self, position: Point, color: Color, text: &str);
    fn add_gradient_text(&mut self, position: Point, gradient: Gradient, text: &str) {
        self.add_text(position, gradient.top_left, text);
    }
    fn calc_text_size(&mut self, text: &str) -> Point;
}
pub use HudRendererInterface as HudRenderer;

/// Inspectable drawing commands, also useful for headless overlays and tests.
#[derive(Clone, Debug, PartialEq)]
pub enum DrawCommand {
    Line(Point, Point, Color, f32),
    Polyline(Vec<Point>, Color, f32),
    ClippedPolyline(Vec<Point>, Point, Point, Color, f32),
    FilledPolyline(Vec<Point>, Color),
    Rectangle(Point, Point, Color),
    FilledRectangle(Point, Point, Color),
    GradientRectangle(Point, Point, Gradient),
    Circle(Point, f32, Color, f32, i32),
    FilledCircle(Point, f32, Color, i32),
    GradientCircle(Point, f32, Gradient, i32),
    Arc(Point, f32, f32, f32, Color, f32, i32),
    Image(usize, Point, Point, Color),
    Text(Point, Color, String),
    GradientText(Point, Gradient, String),
}
#[derive(Default, Debug)]
pub struct CommandBuffer {
    pub commands: Vec<DrawCommand>,
    pub glyph_size: Point,
}
impl HudRenderer for CommandBuffer {
    fn add_line(&mut self, a: Point, b: Point, c: Color, t: f32) {
        self.commands.push(DrawCommand::Line(a, b, c, t));
    }
    fn add_polyline(&mut self, v: &[Point], c: Color, t: f32) {
        self.commands.push(DrawCommand::Polyline(v.to_vec(), c, t));
    }
    fn add_polyline_clipped(&mut self, v: &[Point], a: Point, b: Point, c: Color, t: f32) {
        self.commands
            .push(DrawCommand::ClippedPolyline(v.to_vec(), a, b, c, t));
    }
    fn add_filled_polyline(&mut self, v: &[Point], c: Color) {
        self.commands
            .push(DrawCommand::FilledPolyline(v.to_vec(), c));
    }
    fn add_rectangle(&mut self, a: Point, b: Point, c: Color) {
        self.commands.push(DrawCommand::Rectangle(a, b, c));
    }
    fn add_filled_rectangle(&mut self, a: Point, b: Point, c: Color) {
        self.commands.push(DrawCommand::FilledRectangle(a, b, c));
    }
    fn add_gradient_rectangle(&mut self, a: Point, b: Point, g: Gradient) {
        self.commands.push(DrawCommand::GradientRectangle(a, b, g));
    }
    fn add_circle(&mut self, a: Point, r: f32, c: Color, t: f32, s: i32) {
        self.commands.push(DrawCommand::Circle(a, r, c, t, s));
    }
    fn add_filled_circle(&mut self, a: Point, r: f32, c: Color, s: i32) {
        self.commands.push(DrawCommand::FilledCircle(a, r, c, s));
    }
    fn add_gradient_filled_circle(&mut self, a: Point, r: f32, g: Gradient, s: i32) {
        self.commands.push(DrawCommand::GradientCircle(a, r, g, s));
    }
    fn add_arc(&mut self, a: Point, r: f32, min: f32, max: f32, c: Color, t: f32, s: i32) {
        self.commands
            .push(DrawCommand::Arc(a, r, min, max, c, t, s));
    }
    fn add_image(&mut self, id: usize, a: Point, b: Point, c: Color) {
        self.commands.push(DrawCommand::Image(id, a, b, c));
    }
    fn add_text(&mut self, a: Point, c: Color, text: &str) {
        self.commands.push(DrawCommand::Text(a, c, text.to_owned()));
    }
    fn add_gradient_text(&mut self, a: Point, g: Gradient, text: &str) {
        self.commands
            .push(DrawCommand::GradientText(a, g, text.to_owned()));
    }
    fn calc_text_size(&mut self, text: &str) -> Point {
        Point::new(
            text.lines()
                .map(|line| line.chars().count())
                .max()
                .unwrap_or(0) as f32
                * self.glyph_size.x,
            text.lines().count() as f32 * self.glyph_size.y,
        )
    }
}
