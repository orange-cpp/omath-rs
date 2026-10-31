use super::{Color, Gradient, Point};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Paint {
    Color(Color),
    Gradient(Gradient),
}
impl Default for Paint {
    fn default() -> Self {
        Self::Color(Color::default())
    }
}
impl From<Color> for Paint {
    fn from(c: Color) -> Self {
        Self::Color(c)
    }
}
impl From<Gradient> for Paint {
    fn from(g: Gradient) -> Self {
        Self::Gradient(g)
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct BarGradient {
    pub empty_color: Color,
    pub full_color: Color,
}
#[derive(Clone, Copy, Debug)]
pub enum BarPaint {
    Color(Color),
    Gradient(Gradient),
    BarGradient(BarGradient),
}
impl Default for BarPaint {
    fn default() -> Self {
        Self::Color(Color::default())
    }
}
impl From<Color> for BarPaint {
    fn from(c: Color) -> Self {
        Self::Color(c)
    }
}
impl From<Gradient> for BarPaint {
    fn from(g: Gradient) -> Self {
        Self::Gradient(g)
    }
}
impl From<BarGradient> for BarPaint {
    fn from(g: BarGradient) -> Self {
        Self::BarGradient(g)
    }
}
impl From<Paint> for BarPaint {
    fn from(p: Paint) -> Self {
        match p {
            Paint::Color(c) => Self::Color(c),
            Paint::Gradient(g) => Self::Gradient(g),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Glow {
    pub color: Color,
    pub radius: f32,
    pub intensity: f32,
}
impl Default for Glow {
    fn default() -> Self {
        Self {
            color: Color::default(),
            radius: 4.0,
            intensity: 1.0,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct CanvasGlow {
    pub glow: Glow,
    pub layers: i32,
    pub rounding: f32,
}
impl Default for CanvasGlow {
    fn default() -> Self {
        Self {
            glow: Glow::default(),
            layers: 64,
            rounding: 20.0,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Outlined {
    #[default]
    Off,
    On,
}
#[derive(Clone, Debug)]
pub struct Box {
    pub color: Color,
    pub fill: Paint,
    pub outline: Color,
    pub thickness: f32,
    pub glow: Option<Glow>,
}
impl Default for Box {
    fn default() -> Self {
        Self {
            color: Color::default(),
            fill: Paint::default(),
            outline: Color::default(),
            thickness: 1.0,
            glow: None,
        }
    }
}
#[derive(Clone, Debug)]
pub struct CorneredBox {
    pub color: Color,
    pub fill: Color,
    pub outline: Color,
    pub corner_ratio: f32,
    pub thickness: f32,
    pub glow: Option<Glow>,
}
impl Default for CorneredBox {
    fn default() -> Self {
        Self {
            color: Color::default(),
            fill: Color::default(),
            outline: Color::default(),
            corner_ratio: 0.2,
            thickness: 1.0,
            glow: None,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct DashedBox {
    pub color: Color,
    pub dash_len: f32,
    pub gap_len: f32,
    pub thickness: f32,
}
impl Default for DashedBox {
    fn default() -> Self {
        Self {
            color: Color::default(),
            dash_len: 8.0,
            gap_len: 5.0,
            thickness: 1.0,
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Bone {
    pub start: Point,
    pub end: Point,
}
#[derive(Clone, Debug)]
pub struct Skeleton {
    pub bones: Vec<Bone>,
    pub color: Color,
    pub thickness: f32,
}
impl Default for Skeleton {
    fn default() -> Self {
        Self {
            bones: Vec::new(),
            color: Color::default(),
            thickness: 1.0,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct SnapLine {
    pub start: Point,
    pub color: Color,
    pub width: f32,
}
#[derive(Clone, Copy, Debug)]
pub struct ScanMarker {
    pub color: Color,
    pub outline: Color,
    pub outline_thickness: f32,
}
#[derive(Clone, Copy, Debug)]
pub struct AimDot {
    pub position: Point,
    pub color: Color,
    pub radius: f32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Figure {
    Circle,
    #[default]
    Square,
}
#[derive(Clone, Copy, Debug)]
pub struct ProjectileAim {
    pub position: Point,
    pub color: Color,
    pub size: f32,
    pub line_size: f32,
    pub figure: Figure,
}
#[derive(Clone, Debug)]
pub struct Bar {
    pub color: BarPaint,
    pub outline: Color,
    pub bg: Color,
    pub size: f32,
    pub ratio: f32,
    pub offset: f32,
    pub glow: Option<Glow>,
}
impl Default for Bar {
    fn default() -> Self {
        Self {
            color: BarPaint::default(),
            outline: Color::default(),
            bg: Color::default(),
            size: 4.0,
            ratio: 1.0,
            offset: 5.0,
            glow: None,
        }
    }
}
#[derive(Clone, Debug)]
pub struct DashedBar {
    pub bar: Bar,
    pub dash_len: f32,
    pub gap_len: f32,
}
#[derive(Clone, Debug, Default)]
pub struct Label {
    pub color: Paint,
    pub offset: f32,
    pub outlined: Outlined,
    pub text: String,
    pub glow: Option<Glow>,
}
#[derive(Clone, Copy, Debug)]
pub struct ProgressRing {
    pub color: Color,
    pub bg: Color,
    pub radius: f32,
    pub ratio: f32,
    pub thickness: f32,
    pub offset: f32,
    pub segments: i32,
}
#[derive(Clone, Copy, Debug)]
pub struct Icon {
    pub texture_id: usize,
    pub width: f32,
    pub height: f32,
    pub tint: Color,
    pub offset: f32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Right,
    Left,
    Top,
    Bottom,
}
#[derive(Clone, Debug)]
pub enum SideWidget {
    None,
    Bar(Bar),
    DashedBar(DashedBar),
    Label(Label),
    Centered(Label),
    SpaceVertical(f32),
    SpaceHorizontal(f32),
    ProgressRing(ProgressRing),
    Icon(Icon),
}
#[derive(Clone, Debug)]
pub enum EntityWidget {
    Box(Box),
    CanvasGlow(CanvasGlow),
    CorneredBox(CorneredBox),
    DashedBox(DashedBox),
    Side(Side, Vec<SideWidget>),
    Skeleton(Skeleton),
    SnapLine(SnapLine),
    ScanMarker(ScanMarker),
    AimDot(AimDot),
    ProjectileAim(ProjectileAim),
}
#[derive(Clone, Copy, Debug)]
pub struct Crosshair {
    pub color: Color,
    pub gap: f32,
    pub length: f32,
    pub thickness: f32,
    pub outline: Color,
    pub dot_radius: f32,
}
impl Default for Crosshair {
    fn default() -> Self {
        Self {
            color: Color::default(),
            gap: 4.0,
            length: 8.0,
            thickness: 1.0,
            outline: Color::default(),
            dot_radius: 0.0,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct FovCircle {
    pub color: Color,
    pub radius: f32,
    pub fill: Paint,
    pub thickness: f32,
    pub segments: i32,
}
#[derive(Clone, Copy, Debug)]
pub struct ThreatArrow {
    pub angle: f32,
    pub color: Color,
    pub radius: f32,
    pub size: f32,
    pub outline: Color,
}
#[derive(Clone, Copy, Debug)]
pub struct HitMarker {
    pub color: Color,
    pub size: f32,
    pub gap: f32,
    pub thickness: f32,
    pub outline: Color,
    pub alpha: f32,
}
#[derive(Clone, Copy, Debug)]
pub struct CenterBar {
    pub side: Side,
    pub color: BarPaint,
    pub outline: Color,
    pub bg: Color,
    pub length: f32,
    pub thickness: f32,
    pub ratio: f32,
    pub offset: f32,
    pub glow: Option<CanvasGlow>,
}
#[derive(Clone, Copy, Debug)]
pub struct DashedCenterBar {
    pub bar: CenterBar,
    pub dash_len: f32,
    pub gap_len: f32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}
#[derive(Clone, Debug)]
pub enum CornerWidget {
    None,
    Label(Label),
    SpaceVertical(f32),
}
#[derive(Clone, Debug)]
pub struct Corner {
    pub anchor: Anchor,
    pub children: Vec<CornerWidget>,
    pub padding: f32,
}
#[derive(Clone, Debug)]
pub enum ScreenWidget {
    Crosshair(Crosshair),
    FovCircle(FovCircle),
    ThreatArrow(ThreatArrow),
    HitMarker(HitMarker),
    CenterBar(CenterBar),
    DashedCenterBar(DashedCenterBar),
    Corner(Corner),
}
pub fn when<T>(condition: bool, widget: T) -> Option<T> {
    condition.then_some(widget)
}
