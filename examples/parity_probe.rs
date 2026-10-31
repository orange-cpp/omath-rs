//! Machine-readable counterpart to tests/cpp/reference.cpp.
use omath::{
    Mat, Vector3,
    hud::{widget::*, *},
    pathfinding::NavigationMesh,
};
use std::{
    io::{self, Write},
    time::Instant,
};
fn p(x: f32, y: f32) -> Point {
    Point::new(x, y)
}
fn color(c: omath::Color) -> String {
    let v = c.value();
    format!(" {} {} {} {}", v.x, v.y, v.z, v.w)
}
fn point(p: Point) -> String {
    format!(" {} {}", p.x, p.y)
}
fn hud() {
    let (red, green, blue, clear) = (
        omath::Color::RED,
        omath::Color::GREEN,
        omath::Color::BLUE,
        omath::Color::default(),
    );
    let mut r = CommandBuffer {
        glyph_size: p(8.0, 12.0),
        ..Default::default()
    };
    let mut e = EntityOverlay::new(p(100.0, 10.0), p(100.0, 110.0), 4.0);
    e.draw(
        &mut r,
        &EntityWidget::Box(Box {
            color: red,
            ..Default::default()
        }),
    );
    e.add_side_widget(
        &mut r,
        Side::Right,
        &SideWidget::Bar(Bar {
            color: green.into(),
            outline: blue,
            bg: clear,
            size: 4.0,
            ratio: 0.25,
            ..Default::default()
        }),
    );
    e.add_side_widget(
        &mut r,
        Side::Right,
        &SideWidget::Label(Label {
            color: red.into(),
            offset: 2.0,
            text: "HP".into(),
            ..Default::default()
        }),
    );
    e.add_side_widget(
        &mut r,
        Side::Top,
        &SideWidget::Bar(Bar {
            color: red.into(),
            outline: blue,
            bg: clear,
            size: 3.0,
            ratio: 0.75,
            ..Default::default()
        }),
    );
    e.add_side_widget(
        &mut r,
        Side::Top,
        &SideWidget::Centered(Label {
            color: blue.into(),
            offset: 2.0,
            text: "name".into(),
            outlined: Outlined::On,
            ..Default::default()
        }),
    );
    e.add_side_widget(
        &mut r,
        Side::Left,
        &SideWidget::ProgressRing(ProgressRing {
            color: red,
            bg: blue,
            radius: 6.0,
            ratio: 0.5,
            thickness: 2.0,
            offset: 5.0,
            segments: 16,
        }),
    );
    e.add_side_widget(
        &mut r,
        Side::Bottom,
        &SideWidget::Icon(Icon {
            texture_id: 7,
            width: 16.0,
            height: 16.0,
            tint: omath::Color::new(1.0, 1.0, 1.0, 1.0),
            offset: 5.0,
        }),
    );
    e.draw(
        &mut r,
        &EntityWidget::SnapLine(SnapLine {
            start: p(0.0, 0.0),
            color: red,
            width: 1.0,
        }),
    );
    e.draw(
        &mut r,
        &EntityWidget::DashedBox(DashedBox {
            color: blue,
            ..Default::default()
        }),
    );
    e.draw(
        &mut r,
        &EntityWidget::Skeleton(Skeleton {
            color: green,
            ..Default::default()
        }),
    );
    ScreenOverlay::new(p(800.0, 600.0)).contents(
        &mut r,
        [
            ScreenWidget::Crosshair(Crosshair {
                color: red,
                ..Default::default()
            }),
            ScreenWidget::FovCircle(FovCircle {
                color: green,
                radius: 40.0,
                fill: clear.into(),
                thickness: 1.0,
                segments: 0,
            }),
            ScreenWidget::ThreatArrow(ThreatArrow {
                angle: 0.5,
                color: blue,
                radius: 100.0,
                size: 12.0,
                outline: clear,
            }),
            ScreenWidget::HitMarker(HitMarker {
                color: red,
                size: 8.0,
                gap: 3.0,
                thickness: 2.0,
                outline: clear,
                alpha: 1.0,
            }),
        ],
    );
    for c in r.commands {
        match c {
            DrawCommand::Line(a, b, c, t) => {
                println!("line{}{}{} {t}", point(a), point(b), color(c))
            }
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
            DrawCommand::Circle(a, r, c, t, s) => {
                println!("circle{} {r}{} {t} {s}", point(a), color(c))
            }
            DrawCommand::FilledCircle(a, r, c, s) => {
                println!("fillcircle{} {r}{} {s}", point(a), color(c))
            }
            DrawCommand::Arc(a, r, lo, hi, c, t, s) => {
                println!("arc{} {r} {lo} {hi}{} {t} {s}", point(a), color(c))
            }
            DrawCommand::Image(id, a, b, c) => {
                println!("image {id}{}{}{}", point(a), point(b), color(c))
            }
            DrawCommand::Text(a, c, text) => println!("text {text}{}{}", point(a), color(c)),
            other => panic!("unexpected command: {other:?}"),
        }
    }
}
pub fn benchmarks() {
    for kernel in ["vector", "matrix", "scan"] {
        let iterations = match kernel {
            "vector" => 500000,
            "matrix" => 100000,
            _ => 256,
        };
        let mut checksum = 0.0_f64;
        let mut a = Mat::<4, 4>::from_rows([
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 10.0, 11.0, 12.0],
            [13.0, 14.0, 15.0, 16.0],
        ]);
        let mut bytes = vec![0; 65536];
        *bytes.last_mut().unwrap() = 0xab;
        let begin = Instant::now();
        for i in 0..iterations {
            let value = unsafe { std::ptr::read_volatile(&i) };
            match kernel {
                "vector" => {
                    let v = Vector3::new(
                        (value & 1023) as f32 / 7.0,
                        (value % 31) as f32,
                        (value % 17) as f32,
                    );
                    checksum += v.dot(Vector3::new(1.25, -2.5, 0.75)) as f64;
                }
                "matrix" => {
                    *a.at_mut(0, 0) = (value % 31) as f32;
                    let result = a * a;
                    checksum += (result.at(0, 0) + result.at(3, 3)) as f64;
                }
                _ => {
                    checksum += omath::utility::PatternScanner::scan_for_pattern(&bytes, "00 00 AB")
                        .unwrap() as f64;
                }
            }
        }
        println!(
            "{kernel} {iterations} {} {checksum}",
            begin.elapsed().as_nanos()
        );
    }
}
fn main() -> Result<(), std::boxed::Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    match args.get(1).map(String::as_str).unwrap_or("cases") {
        "nav" => {
            let mut nav = NavigationMesh::new();
            nav.deserialize(&std::fs::read_to_string(&args[2])?)?;
            print!("{}", nav.serialize());
        }
        "hud" => hud(),
        "bench" => benchmarks(),
        _ => {
            {
                use omath::engines::source::*;
                for i in 0..12 {
                    let camera = Camera::new(
                        Vector3::new(3.0, 4.0, 5.0),
                        ViewAngles::new(
                            PitchAngle::from_degrees((i - 6) as f32),
                            YawAngle::from_degrees((i * 5) as f32),
                            RollAngle::from_degrees((i % 3) as f32),
                        ),
                        omath::projection::ViewPort::new(800.0, 600.0),
                        omath::projection::FieldOfView::from_degrees(90.0),
                        0.1,
                        1000.0,
                    );
                    let target = K_ABS_FORWARD * 100.0
                        + K_ABS_RIGHT * ((i * 2) as f32)
                        + K_ABS_UP * (i as f32);
                    print!("camera source {i}");
                    match camera.world_to_screen_unclipped(target) {
                        Ok(p) => print!(" {} {}", p.x, p.y),
                        Err(_) => print!(" none"),
                    }
                    let matrix = camera.get_view_matrix();
                    for r in 0..4 {
                        for c in 0..4 {
                            print!(" {}", matrix.at(r, c));
                        }
                    }
                    println!();
                }
            }
            {
                use omath::engines::iw::*;
                for i in 0..12 {
                    let camera = Camera::new(
                        Vector3::new(3.0, 4.0, 5.0),
                        ViewAngles::new(
                            PitchAngle::from_degrees((i - 6) as f32),
                            YawAngle::from_degrees((i * 5) as f32),
                            RollAngle::from_degrees((i % 3) as f32),
                        ),
                        omath::projection::ViewPort::new(800.0, 600.0),
                        omath::projection::FieldOfView::from_degrees(90.0),
                        0.1,
                        1000.0,
                    );
                    let target = K_ABS_FORWARD * 100.0
                        + K_ABS_RIGHT * ((i * 2) as f32)
                        + K_ABS_UP * (i as f32);
                    print!("camera iw {i}");
                    match camera.world_to_screen_unclipped(target) {
                        Ok(p) => print!(" {} {}", p.x, p.y),
                        Err(_) => print!(" none"),
                    }
                    let matrix = camera.get_view_matrix();
                    for r in 0..4 {
                        for c in 0..4 {
                            print!(" {}", matrix.at(r, c));
                        }
                    }
                    println!();
                }
            }
            {
                use omath::engines::unity::*;
                for i in 0..12 {
                    let camera = Camera::new(
                        Vector3::new(3.0, 4.0, 5.0),
                        ViewAngles::new(
                            PitchAngle::from_degrees((i - 6) as f32),
                            YawAngle::from_degrees((i * 5) as f32),
                            RollAngle::from_degrees((i % 3) as f32),
                        ),
                        omath::projection::ViewPort::new(800.0, 600.0),
                        omath::projection::FieldOfView::from_degrees(90.0),
                        0.1,
                        1000.0,
                    );
                    let target = K_ABS_FORWARD * 100.0
                        + K_ABS_RIGHT * ((i * 2) as f32)
                        + K_ABS_UP * (i as f32);
                    print!("camera unity {i}");
                    match camera.world_to_screen_unclipped(target) {
                        Ok(p) => print!(" {} {}", p.x, p.y),
                        Err(_) => print!(" none"),
                    }
                    let matrix = camera.get_view_matrix();
                    for r in 0..4 {
                        for c in 0..4 {
                            print!(" {}", matrix.at(r, c));
                        }
                    }
                    println!();
                }
            }
            {
                use omath::engines::unreal::*;
                for i in 0..12 {
                    let camera = Camera::new(
                        Vector3::new(3.0, 4.0, 5.0),
                        ViewAngles::new(
                            PitchAngle::from_degrees((i - 6) as f64),
                            YawAngle::from_degrees((i * 5) as f64),
                            RollAngle::from_degrees((i % 3) as f64),
                        ),
                        omath::projection::ViewPort::new(800.0, 600.0),
                        omath::projection::FieldOfView::from_degrees(90.0),
                        0.1,
                        1000.0,
                    );
                    let target = K_ABS_FORWARD * 100.0
                        + K_ABS_RIGHT * ((i * 2) as f64)
                        + K_ABS_UP * (i as f64);
                    print!("camera unreal {i}");
                    match camera.world_to_screen_unclipped(target) {
                        Ok(p) => print!(" {} {}", p.x, p.y),
                        Err(_) => print!(" none"),
                    }
                    let matrix = camera.get_view_matrix();
                    for r in 0..4 {
                        for c in 0..4 {
                            print!(" {}", matrix.at(r, c));
                        }
                    }
                    println!();
                }
            }
            {
                use omath::engines::frostbite::*;
                for i in 0..12 {
                    let camera = Camera::new(
                        Vector3::new(3.0, 4.0, 5.0),
                        ViewAngles::new(
                            PitchAngle::from_degrees((i - 6) as f32),
                            YawAngle::from_degrees((i * 5) as f32),
                            RollAngle::from_degrees((i % 3) as f32),
                        ),
                        omath::projection::ViewPort::new(800.0, 600.0),
                        omath::projection::FieldOfView::from_degrees(90.0),
                        0.1,
                        1000.0,
                    );
                    let target = K_ABS_FORWARD * 100.0
                        + K_ABS_RIGHT * ((i * 2) as f32)
                        + K_ABS_UP * (i as f32);
                    print!("camera frostbite {i}");
                    match camera.world_to_screen_unclipped(target) {
                        Ok(p) => print!(" {} {}", p.x, p.y),
                        Err(_) => print!(" none"),
                    }
                    let matrix = camera.get_view_matrix();
                    for r in 0..4 {
                        for c in 0..4 {
                            print!(" {}", matrix.at(r, c));
                        }
                    }
                    println!();
                }
            }
            {
                use omath::engines::rage::*;
                for i in 0..12 {
                    let camera = Camera::new(
                        Vector3::new(3.0, 4.0, 5.0),
                        ViewAngles::new(
                            PitchAngle::from_degrees((i - 6) as f32),
                            YawAngle::from_degrees((i * 5) as f32),
                            RollAngle::from_degrees((i % 3) as f32),
                        ),
                        omath::projection::ViewPort::new(800.0, 600.0),
                        omath::projection::FieldOfView::from_degrees(90.0),
                        0.1,
                        1000.0,
                    );
                    let target = K_ABS_FORWARD * 100.0
                        + K_ABS_RIGHT * ((i * 2) as f32)
                        + K_ABS_UP * (i as f32);
                    print!("camera rage {i}");
                    match camera.world_to_screen_unclipped(target) {
                        Ok(p) => print!(" {} {}", p.x, p.y),
                        Err(_) => print!(" none"),
                    }
                    let matrix = camera.get_view_matrix();
                    for r in 0..4 {
                        for c in 0..4 {
                            print!(" {}", matrix.at(r, c));
                        }
                    }
                    println!();
                }
            }
            {
                use omath::engines::serious::*;
                for i in 0..12 {
                    let camera = Camera::new(
                        Vector3::new(3.0, 4.0, 5.0),
                        ViewAngles::new(
                            PitchAngle::from_degrees((i - 6) as f32),
                            YawAngle::from_degrees((i * 5) as f32),
                            RollAngle::from_degrees((i % 3) as f32),
                        ),
                        omath::projection::ViewPort::new(800.0, 600.0),
                        omath::projection::FieldOfView::from_degrees(90.0),
                        0.1,
                        1000.0,
                    );
                    let target = K_ABS_FORWARD * 100.0
                        + K_ABS_RIGHT * ((i * 2) as f32)
                        + K_ABS_UP * (i as f32);
                    print!("camera serious {i}");
                    match camera.world_to_screen_unclipped(target) {
                        Ok(p) => print!(" {} {}", p.x, p.y),
                        Err(_) => print!(" none"),
                    }
                    let matrix = camera.get_view_matrix();
                    for r in 0..4 {
                        for c in 0..4 {
                            print!(" {}", matrix.at(r, c));
                        }
                    }
                    println!();
                }
            }
            {
                use omath::engines::cry::*;
                for i in 0..12 {
                    let camera = Camera::new(
                        Vector3::new(3.0, 4.0, 5.0),
                        ViewAngles::new(
                            PitchAngle::from_degrees((i - 6) as f32),
                            YawAngle::from_degrees((i * 5) as f32),
                            RollAngle::from_degrees((i % 3) as f32),
                        ),
                        omath::projection::ViewPort::new(800.0, 600.0),
                        omath::projection::FieldOfView::from_degrees(90.0),
                        0.1,
                        1000.0,
                    );
                    let target = K_ABS_FORWARD * 100.0
                        + K_ABS_RIGHT * ((i * 2) as f32)
                        + K_ABS_UP * (i as f32);
                    print!("camera cry {i}");
                    match camera.world_to_screen_unclipped(target) {
                        Ok(p) => print!(" {} {}", p.x, p.y),
                        Err(_) => print!(" none"),
                    }
                    let matrix = camera.get_view_matrix();
                    for r in 0..4 {
                        for c in 0..4 {
                            print!(" {}", matrix.at(r, c));
                        }
                    }
                    println!();
                }
            }
            {
                use omath::engines::opengl::*;
                for i in 0..12 {
                    let camera = Camera::new(
                        Vector3::new(3.0, 4.0, 5.0),
                        ViewAngles::new(
                            PitchAngle::from_degrees((i - 6) as f32),
                            YawAngle::from_degrees((i * 5) as f32),
                            RollAngle::from_degrees((i % 3) as f32),
                        ),
                        omath::projection::ViewPort::new(800.0, 600.0),
                        omath::projection::FieldOfView::from_degrees(90.0),
                        0.1,
                        1000.0,
                    );
                    let target = K_ABS_FORWARD * 100.0
                        + K_ABS_RIGHT * ((i * 2) as f32)
                        + K_ABS_UP * (i as f32);
                    print!("camera opengl {i}");
                    match camera.world_to_screen_unclipped(target) {
                        Ok(p) => print!(" {} {}", p.x, p.y),
                        Err(_) => print!(" none"),
                    }
                    let matrix = camera.get_view_matrix();
                    for r in 0..4 {
                        for c in 0..4 {
                            print!(" {}", matrix.at(r, c));
                        }
                    }
                    println!();
                }
            }
            let mut output = io::BufWriter::new(io::stdout().lock());
            for i in 0..256 {
                let a = Vector3::new(
                    (i - 128) as f32 / 7.0,
                    (i % 31) as f32 - 15.0,
                    (i % 17) as f32 - 8.0,
                );
                let b = Vector3::new((i % 23) as f32, -3.5, 2.25);
                let c = a.cross(b);
                writeln!(
                    output,
                    "vec {i} {} {} {} {} {}",
                    a.dot(b),
                    c.x,
                    c.y,
                    c.z,
                    a.length()
                )?;
            }
            for i in 0..64 {
                let mut a = Mat::<4, 4>::default();
                for row in 0..4 {
                    for col in 0..4 {
                        *a.at_mut(row, col) = ((i + row * 4 + col) % 17) as f32 - 8.0;
                    }
                }
                let result = a * a;
                write!(output, "mat {i}")?;
                for row in 0..4 {
                    for col in 0..4 {
                        write!(output, " {}", result.at(row, col))?;
                    }
                }
                writeln!(output, " {}", a.determinant())?;
            }
        }
    }
    Ok(())
}
