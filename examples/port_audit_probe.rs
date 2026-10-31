//! Probes for the completeness audit; intentionally exercise paths outside the baseline parity fixture.
use omath::{
    Color,
    hud::{widget::*, *},
};

fn point(p: Point) -> String {
    format!(" {} {}", p.x, p.y)
}
fn color(c: Color) -> String {
    let v = c.value();
    format!(" {} {} {} {}", v.x, v.y, v.z, v.w)
}
fn dump(command: DrawCommand) {
    match command {
        DrawCommand::Line(a, b, c, t) => println!("line{}{}{} {t}", point(a), point(b), color(c)),
        DrawCommand::Polyline(v, c, t) => println!(
            "poly {}{}{} {t}",
            v.len(),
            v.into_iter().map(point).collect::<String>(),
            color(c)
        ),
        DrawCommand::FilledPolyline(v, c) => println!(
            "fillpoly {}{}{}",
            v.len(),
            v.into_iter().map(point).collect::<String>(),
            color(c)
        ),
        DrawCommand::Rectangle(a, b, c) => println!("rect{}{}{}", point(a), point(b), color(c)),
        DrawCommand::FilledRectangle(a, b, c) => {
            println!("fillrect{}{}{}", point(a), point(b), color(c))
        }
        other => panic!("unexpected audit command {other:?}"),
    }
}
#[cfg(feature = "imgui")]
fn gradient_channel() {
    let mut context = imgui::Context::create();
    context.set_ini_filename(None);
    context.io_mut().display_size = [800.0, 600.0];
    context.io_mut().delta_time = 1.0 / 60.0;
    context.fonts().build_rgba32_texture();
    let ui = context.frame();
    let half = Color::new(0.5, 0.5, 0.5, 1.0);
    {
        let mut renderer = ImguiHudRenderer::new(ui);
        renderer.add_gradient_filled_circle(
            Point::new(100.0, 100.0),
            10.0,
            Gradient::new(half, half, half, half),
            16,
        );
    }
    let data = context.render();
    let vertex = data
        .draw_lists()
        .flat_map(|list| list.vtx_buffer())
        .find(|v| v.col[3] == 255)
        .expect("opaque gradient vertex");
    println!("shade {}", vertex.col[0]);
}
fn main() {
    let mode = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "corner_glow".into());
    if mode == "fov_wrap" || mode == "fov_roll" {
        use omath::{
            Vector3,
            engines::source::*,
            projection::{FieldOfView, ViewPort},
        };
        let camera = Camera::new(
            Vector3::default(),
            ViewAngles::new(
                PitchAngle::from_degrees(0.0),
                YawAngle::from_degrees(if mode == "fov_wrap" { 179.0 } else { 0.0 }),
                RollAngle::from_degrees(if mode == "fov_roll" { 45.0 } else { 0.0 }),
            ),
            ViewPort::new(800.0, 600.0),
            FieldOfView::from_degrees(90.0),
            0.1,
            1000.0,
        );
        let targets = if mode == "fov_wrap" {
            [
                Vector3::new(-100.0, -1.7455, 0.0),
                Vector3::new(-100.0, 17.6327, 0.0),
            ]
        } else {
            [
                Vector3::new(100.0, 0.0, 0.0),
                Vector3::new(100.0, 10.0, 0.0),
            ]
        };
        let selected = omath::algorithm::get_closest_target_by_fov(
            targets.iter().enumerate(),
            &camera,
            |target| *target.1,
            |_| true,
        )
        .unwrap();
        println!("target {}", selected.0);
        return;
    }
    if mode == "gradient_channel" {
        #[cfg(feature = "imgui")]
        gradient_channel();
        #[cfg(not(feature = "imgui"))]
        panic!("gradient_channel requires the imgui feature");
        return;
    }
    let mut renderer = CommandBuffer::default();
    let mut entity = EntityOverlay::from_canvas(CanvasBox::from_bounds(
        Point::new(75.0, 10.0),
        Point::new(125.0, 110.0),
    ));
    if mode == "corner_glow" {
        entity.draw(
            &mut renderer,
            &EntityWidget::CorneredBox(CorneredBox {
                color: Color::RED,
                fill: Color::new(0.0, 1.0, 0.0, 0.5),
                outline: Color::default(),
                corner_ratio: 0.2,
                thickness: 1.0,
                glow: Some(Glow {
                    color: Color::BLUE,
                    radius: 2.0,
                    intensity: 1.0,
                }),
            }),
        );
    } else {
        let side = match mode.as_str() {
            "dashed_right" => Side::Right,
            "dashed_left" => Side::Left,
            "dashed_top" => Side::Top,
            "dashed_bottom" => Side::Bottom,
            _ => panic!("unknown audit fixture"),
        };
        entity.add_side_widget(
            &mut renderer,
            side,
            &SideWidget::DashedBar(DashedBar {
                bar: Bar {
                    color: Color::RED.into(),
                    outline: Color::BLUE,
                    bg: Color::default(),
                    size: 4.0,
                    ratio: 0.25,
                    offset: 5.0,
                    glow: None,
                },
                dash_len: 8.0,
                gap_len: 5.0,
            }),
        );
    }
    for command in renderer.commands {
        dump(command);
    }
}
