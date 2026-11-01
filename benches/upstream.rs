//! Workloads from the seven upstream benchmark suites. `--smoke` runs one
//! iteration; `--iterations=N` overrides upstream counts; `--list` lists names.
#![allow(clippy::approx_constant)] // Preserve the exact upstream OBB workload constants.
use omath::{
    collision::*,
    engines::*,
    linear_algebra::{ColumnMajor, RowMajor},
    primitives::*,
    projectile_prediction::*,
    projection::*,
    *,
};
use std::{hint::black_box, time::Instant};
struct Runner {
    filter: String,
    list: bool,
    iterations: Option<usize>,
}
impl Runner {
    fn counter(&self, name: &str, key: &str, value: f64) {
        if !self.list && name.contains(&self.filter) {
            println!("counter {name} {key} {value}");
        }
    }
    fn run<T>(&self, name: &str, count: usize, mut f: impl FnMut(usize) -> T) {
        if !name.contains(&self.filter) {
            return;
        }
        if self.list {
            println!("{name}");
            return;
        }
        let n = self.iterations.unwrap_or(count);
        let start = Instant::now();
        for i in 0..n {
            black_box(f(i & 1023));
        }
        println!("{name} {n} {}", start.elapsed().as_nanos());
    }
}
struct Rng(u64);
impl Rng {
    fn new() -> Self {
        Self(0x853c49e6748fea9b)
    }
    fn next(&mut self, min: f32, max: f32) -> f32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        min + (max - min) * ((self.0 >> 40) as f32 / (1u32 << 24) as f32)
    }
}
fn camera<E: CameraEngine>(r: &Runner, suffix: &str) {
    use omath::scalar::Scalar;
    let v = |x: f64, y: f64, z: f64| {
        Vector3::new(
            E::Scalar::from_f64(x),
            E::Scalar::from_f64(y),
            E::Scalar::from_f64(z),
        )
    };
    let mut c = projection::Camera::<E>::new(
        v(0.0, 0.0, 0.0),
        Default::default(),
        ViewPort::new(1920.0, 1080.0),
        FieldOfView::from_degrees(90.0),
        E::Scalar::from_f64(0.1),
        E::Scalar::from_f64(10000.0),
    );
    c.look_at(v(512.0, 96.0, 48.0));
    let mut mixed = Vec::new();
    let mut front = Vec::new();
    for kind in ["visible", "behind", "mixed"] {
        let mut rng = Rng::new();
        let points = (0..1024)
            .map(|i| {
                let d = E::Scalar::from_f64(rng.next(128.0, 2048.0) as f64);
                let spread = d * E::Scalar::from_f64(0.4);
                let x = E::Scalar::from_f64(rng.next(-1.0, 1.0) as f64) * spread;
                let y = E::Scalar::from_f64(rng.next(-1.0, 1.0) as f64) * spread;
                let depth = if kind == "visible" || kind == "mixed" && i % 2 == 0 {
                    d
                } else {
                    -d
                };
                c.get_origin()
                    + c.get_abs_forward() * depth
                    + c.get_abs_right() * x
                    + c.get_abs_up() * y
            })
            .collect::<Vec<_>>();
        if kind != "mixed" || suffix == "source_engine" {
            r.counter(
                &format!("camera_world_to_screen_{kind}_{suffix}"),
                "on_screen_pct",
                points
                    .iter()
                    .filter(|&&p| c.world_to_screen(p).is_ok())
                    .count() as f64
                    * 100.0
                    / 1024.0,
            );
            r.run(
                &format!("camera_world_to_screen_{kind}_{suffix}"),
                2_000_000,
                |i| black_box(&c).world_to_screen(points[i]),
            );
        }
        if kind == "mixed" {
            mixed = points;
        } else if kind == "visible" {
            front = points;
        }
    }
    if suffix != "source_engine" {
        return;
    }
    r.run("camera_world_to_view_coordinates", 2_000_000, |i| {
        black_box(&c).world_to_view_coordinates(front[i])
    });
    r.run("camera_project_entity_box", 500_000, |i| {
        let b = Aabb::<E::Scalar>::new(
            mixed[i] - v(16.0, 16.0, 36.0),
            mixed[i] + v(16.0, 16.0, 36.0),
        );
        for p in b.vertices() {
            let _ = black_box(c.world_to_screen_unclipped(p));
        }
        black_box(c.world_to_screen_unclipped(b.center()))
    });
    let mut rng = Rng::new();
    let screen = (0..1024)
        .map(|_| {
            v(
                rng.next(0.0, 1920.0) as f64,
                rng.next(0.0, 1080.0) as f64,
                rng.next(0.0, 1.0) as f64,
            )
        })
        .collect::<Vec<_>>();
    r.run("camera_screen_to_world", 500_000, |i| {
        black_box(&c).screen_to_world(screen[i])
    });
    r.run("camera_screen_to_world_moving", 500_000, |i| {
        c.set_origin(v(i as f64, 0.0, 64.0));
        black_box(&c).screen_to_world(screen[i])
    });
    c.set_origin(v(0.0, 0.0, 0.0));
    let boxes = mixed
        .iter()
        .map(|p| Aabb::<E::Scalar>::new(*p - v(16.0, 16.0, 36.0), *p + v(16.0, 16.0, 36.0)))
        .collect::<Vec<_>>();
    r.run("camera_is_aabb_culled_by_frustum", 2_000_000, |i| {
        black_box(&c).is_aabb_culled_by_frustum(&boxes[i])
    });
    let boxes = mixed
        .iter()
        .map(|p| Obb {
            center: *p,
            axis_x: v(1.0, 0.0, 0.0),
            axis_y: v(0.0, 1.0, 0.0),
            axis_z: v(0.0, 0.0, 1.0),
            half_extents: v(16.0, 16.0, 36.0),
        })
        .collect::<Vec<_>>();
    r.run("camera_is_obb_culled_by_frustum", 2_000_000, |i| {
        black_box(&c).is_obb_culled_by_frustum(&boxes[i])
    });
    let triangles = mixed
        .iter()
        .map(|p| Triangle::new(*p, *p + v(32.0, 0.0, 0.0), *p + v(0.0, 32.0, 24.0)))
        .collect::<Vec<_>>();
    r.run("camera_is_triangle_culled_by_frustum", 2_000_000, |i| {
        black_box(&c).is_culled_by_frustum(&triangles[i])
    });
    r.run("camera_set_origin_then_project", 1_000_000, |i| {
        c.set_origin(v(i as f64, 0.0, 64.0));
        black_box(&c).world_to_screen(front[i])
    });
}
fn matrices(r: &Runner) {
    macro_rules! mat {
        ($name:literal,$n:literal,$t:ty,$order:ty,$count:expr) => {{
            let a = Mat::<$n, $n, $t, $order>::splat(3.0);
            let b = Mat::<$n, $n, $t, $order>::splat(7.0);
            r.run($name, $count, |_| black_box(a) * black_box(b));
        }};
    }
    mat!(
        "mat_float_multiplication_col_major",
        128,
        f32,
        ColumnMajor,
        5000
    );
    mat!(
        "mat_float_multiplication_row_major",
        128,
        f32,
        RowMajor,
        5000
    );
    mat!(
        "mat_double_multiplication_col_major",
        128,
        f64,
        ColumnMajor,
        5000
    );
    mat!(
        "mat_double_multiplication_row_major",
        128,
        f64,
        RowMajor,
        5000
    );
    mat!(
        "mat_float_multiplication_4x4_row_major",
        4,
        f32,
        RowMajor,
        5_000_000
    );
    mat!(
        "mat_float_multiplication_4x4_col_major",
        4,
        f32,
        ColumnMajor,
        5_000_000
    );
    mat!(
        "mat_double_multiplication_4x4_row_major",
        4,
        f64,
        RowMajor,
        5_000_000
    );
    mat!(
        "mat_float_multiplication_8x8_row_major",
        8,
        f32,
        RowMajor,
        2_000_000
    );
    mat!(
        "mat_float_multiplication_16x16_row_major",
        16,
        f32,
        RowMajor,
        500_000
    );
    let a = Mat::<4, 4>::splat(3.0);
    let p = Vector3::new(128.0, -512.0, 64.0);
    r.run(
        "mat_float_multiplication_4x4_by_4x1_row_major",
        5_000_000,
        |_| black_box(a) * Mat::<4, 1>::column_from_vector(black_box(p)),
    );
    let a = Mat::<4, 4>::from_rows([
        [0.9, 0.1, 0.05, 12.0],
        [0.1, 1.1, 0.2, -4.0],
        [0.0, 0.15, 0.95, 3.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);
    r.run("mat_float_4x4_inverted", 2_000_000, |_| {
        black_box(a).inverted()
    });
}
fn views(r: &Runner) {
    macro_rules! view {
        ($engine:ident,$name:literal) => {{
            use $engine as e;
            let angles = e::ViewAngles::new(
                e::PitchAngle::from_degrees(23.0),
                e::YawAngle::from_degrees(114.0),
                e::RollAngle::from_degrees(7.0),
            );
            let origin = Vector3::new(128.0, -512.0, 64.0);
            r.run($name, 2_000_000, |_| {
                e::calc_view_matrix(black_box(angles), black_box(origin))
            });
        }};
    }
    view!(source, "view_matrix_source_engine");
    view!(iw, "view_matrix_iw_engine");
    view!(cry, "view_matrix_cry_engine");
    view!(frostbite, "view_matrix_frostbite_engine");
    view!(rage, "view_matrix_rage_engine");
    view!(unity, "view_matrix_unity_engine");
    view!(opengl, "view_matrix_opengl_engine");
    view!(unreal, "view_matrix_unreal_engine");
    let angles = source::ViewAngles::new(
        source::PitchAngle::from_degrees(23.0),
        source::YawAngle::from_degrees(114.0),
        source::RollAngle::from_degrees(7.0),
    );
    r.run("view_matrix_rotation_only", 2_000_000, |_| {
        source::rotation_matrix(black_box(angles))
    });
    r.run("view_matrix_camera_view_only", 2_000_000, |_| {
        Mat::<4, 4>::camera_view(
            black_box(Vector3::new(0.91, 0.39, -0.12)),
            black_box(Vector3::new(-0.39, 0.92, 0.0)),
            black_box(Vector3::new(0.11, 0.05, 0.99)),
            black_box(Vector3::new(128.0, -512.0, 64.0)),
        )
    });
}
fn sphere(n: u32) -> source::Mesh {
    let mut vertices = Vec::new();
    let mut elements = Vec::new();
    for ring in 0..=n {
        let phi = std::f32::consts::PI * ring as f32 / n as f32;
        for sector in 0..=n {
            let theta = 2.0 * std::f32::consts::PI * sector as f32 / n as f32;
            let position =
                Vector3::new(phi.sin() * theta.cos(), phi.cos(), phi.sin() * theta.sin());
            vertices.push(Vertex {
                position,
                normal: position,
                uv: Default::default(),
            });
        }
    }
    for ring in 0..n {
        for sector in 0..n {
            let tl = ring * (n + 1) + sector;
            let bl = tl + n + 1;
            elements.extend([
                Vector3::new(tl, bl, tl + 1),
                Vector3::new(tl + 1, bl, bl + 1),
            ]);
        }
    }
    source::Mesh::new(vertices, elements)
}
fn rays(kind: &str) -> Vec<Ray> {
    let mut rng = Rng::new();
    (0..1024)
        .map(|_| {
            if kind == "parallel" {
                return Ray {
                    start: Vector3::new(-8.0, rng.next(1.0, 4.0), 0.0),
                    end: Vector3::new(16.0, 0.0, 0.0),
                    infinite_length: false,
                };
            }
            let p = rng.next(0.0, std::f32::consts::PI);
            let t = rng.next(0.0, 2.0 * std::f32::consts::PI);
            let d = Vector3::new(p.sin() * t.cos(), p.cos(), p.sin() * t.sin());
            let perpendicular = d.cross(Vector3::new(0.0, 1.0, 0.0)).normalized();
            let offset = if kind == "hit" {
                0.0
            } else if kind == "graze" {
                1.9
            } else {
                6.0
            };
            Ray {
                start: d * 8.0,
                end: perpendicular * offset - d * 8.0,
                infinite_length: false,
            }
        })
        .collect()
}
fn trace<G: RayTraceable<Scalar = f32>>(
    r: &Runner,
    name: &str,
    shape: &G,
    kind: &str,
    count: usize,
) {
    let rays = rays(kind);
    r.counter(
        name,
        "hit_pct",
        rays.iter()
            .filter(|ray| get_ray_hit_point(ray, shape) != ray.end)
            .count() as f64
            * 100.0
            / 1024.0,
    );
    r.run(name, count, |i| {
        get_ray_hit_point(&rays[i], black_box(shape))
    });
}
fn lines(r: &Runner) {
    for (kind, size) in [("hit", 4.0), ("miss", 0.5), ("parallel", 1.0)] {
        let t = Triangle::new(
            Vector3::new(-size, 0.0, -size),
            Vector3::new(size, 0.0, -size),
            Vector3::new(0.0, 0.0, size),
        );
        trace(
            r,
            &format!("line_tracer_triangle_{kind}"),
            &t,
            kind,
            2_000_000,
        );
    }
    for kind in ["hit", "miss"] {
        let b = Aabb::<f32>::new(Vector3::splat(-1.0), Vector3::splat(1.0));
        trace(r, &format!("line_tracer_aabb_{kind}"), &b, kind, 2_000_000);
        let o = Obb {
            center: Vector3::default(),
            axis_x: Vector3::new(0.7071068, 0.7071068, 0.0),
            axis_y: Vector3::new(-0.7071068, 0.7071068, 0.0),
            axis_z: Vector3::new(0.0, 0.0, 1.0),
            half_extents: Vector3::splat(1.0),
        };
        trace(r, &format!("line_tracer_obb_{kind}"), &o, kind, 2_000_000);
    }
    for kind in ["hit", "miss", "graze"] {
        for n in [8, 16, 32] {
            let mesh = sphere(n);
            r.counter(
                &format!("line_tracer_mesh_{kind}/{n}"),
                "faces",
                (2 * n * n) as f64,
            );
            trace(
                r,
                &format!("line_tracer_mesh_{kind}/{n}"),
                &mesh,
                kind,
                20_000,
            );
        }
    }
    let mut mesh = sphere(16);
    r.counter("line_tracer_mesh_moving", "faces", 512.0);
    let rays = rays("hit");
    r.run("line_tracer_mesh_moving", 20_000, |i| {
        mesh.set_origin(Vector3::new(i as f32 * 0.001, 0.0, 0.0));
        get_ray_hit_point(&rays[i], &mesh)
    });
}
fn collision(r: &Runner) {
    let cube = || {
        let positions = [
            [-1.0, -1.0, -1.0],
            [-1.0, -1.0, 1.0],
            [-1.0, 1.0, -1.0],
            [-1.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, -1.0],
            [1.0, -1.0, 1.0],
            [1.0, -1.0, -1.0],
        ];
        source::Mesh::new(
            positions
                .map(|p| Vertex {
                    position: Vector3::new(p[0], p[1], p[2]),
                    ..Default::default()
                })
                .to_vec(),
            Vec::new(),
        )
    };
    let a = source::MeshCollider::new(cube());
    for (name, offset) in [
        ("BM_Gjk_Separated", Vector3::new(0.0, 2.1, 0.0)),
        ("BM_Gjk_Overlapping", Vector3::new(0.5, 0.0, 0.0)),
        ("BM_Gjk_SameOrigin", Vector3::default()),
    ] {
        let mut mesh = cube();
        mesh.set_origin(offset);
        let b = source::MeshCollider::new(mesh);
        r.run(name, 100_000, |_| is_collide(black_box(&a), black_box(&b)));
    }
    let mut mesh = cube();
    mesh.set_origin(Vector3::new(0.5, 0.0, 0.0));
    let b = source::MeshCollider::new(mesh);
    let hit = is_collide_with_simplex_info(&a, &b, &GjkSettings::default());
    assert!(hit.hit);
    let mut workspace = EpaWorkspace::default();
    black_box(epa_solve_with_workspace(
        &a,
        &b,
        &hit.simplex,
        EpaParams::default(),
        &mut workspace,
    ));
    r.run("BM_Epa_MonotonicBuffer", 100_000, |_| {
        epa_solve_with_workspace(
            black_box(&a),
            black_box(&b),
            &hit.simplex,
            EpaParams::default(),
            &mut workspace,
        )
    });
    r.run("BM_Epa_DefaultResource", 100_000, |_| {
        epa_solve(
            black_box(&a),
            black_box(&b),
            &hit.simplex,
            EpaParams::default(),
        )
    });
    r.run("BM_GjkEpa_Pipeline", 100_000, |_| {
        let hit =
            is_collide_with_simplex_info(black_box(&a), black_box(&b), &GjkSettings::default());
        if hit.hit {
            epa_solve_with_workspace(&a, &b, &hit.simplex, EpaParams::default(), &mut workspace)
        } else {
            None
        }
    });
}
fn projectile(r: &Runner) {
    for (
        name,
        count,
        gravity,
        limit,
        tolerance,
        origin,
        velocity,
        projectile_origin,
        speed,
        scale,
    ) in [
        (
            "source_engine_projectile_prediction",
            10_000,
            400.0,
            50.0,
            5.0,
            [100.0, 0.0, 90.0],
            [0.0; 3],
            [3.0, 2.0, 1.0],
            5000.0,
            0.4,
        ),
        (
            "projectile_prediction_short_range",
            200_000,
            400.0,
            50.0,
            5.0,
            [100.0, 0.0, 90.0],
            [0.0; 3],
            [3.0, 2.0, 1.0],
            5000.0,
            0.4,
        ),
        (
            "projectile_prediction_long_range_moving",
            20_000,
            800.0,
            50.0,
            10.0,
            [4000.0, 1500.0, 0.0],
            [-50.0, 20.0, 0.0],
            [0.0; 3],
            3000.0,
            1.0,
        ),
        (
            "projectile_prediction_no_solution",
            20_000,
            400.0,
            5.0,
            5.0,
            [100000.0, 0.0, 0.0],
            [0.0; 3],
            [0.0; 3],
            1.0,
            1.0,
        ),
        (
            "projectile_prediction_aim_angles",
            200_000,
            400.0,
            50.0,
            5.0,
            [100.0, 0.0, 90.0],
            [0.0; 3],
            [3.0, 2.0, 1.0],
            5000.0,
            0.4,
        ),
        (
            "projectile_prediction_muzzle_offset",
            200_000,
            400.0,
            50.0,
            5.0,
            [100.0, 0.0, 90.0],
            [0.0; 3],
            [0.0; 3],
            5000.0,
            0.4,
        ),
    ] {
        let target = Target {
            origin: origin.into(),
            velocity: velocity.into(),
            is_airborne: false,
        };
        let p = Projectile {
            origin: projectile_origin.into(),
            launch_speed: speed,
            gravity_scale: scale,
            ..Default::default()
        };
        let engine =
            ProjPredEngineLegacy::<source::PredEngineTrait>::new(gravity, 0.001, limit, tolerance);
        if name.ends_with("muzzle_offset") {
            let launcher = Launcher {
                eye_origin: Vector3::new(3.0, 2.0, 1.0),
                muzzle_offset: MuzzleOffset {
                    forward: 16.0,
                    right: 8.0,
                    up: -6.0,
                },
                ..Default::default()
            };
            r.run(name, count, |_| {
                engine.maybe_calculate_aim(&p, &launcher, &target)
            });
        } else if name.ends_with("aim_angles") {
            r.run(name, count, |_| {
                engine.maybe_calculate_aim_angles(&p, &target)
            });
        } else {
            r.run(name, count, |_| {
                if name.starts_with("source_engine") {
                    ProjPredEngineLegacy::<source::PredEngineTrait>::new(
                        gravity, 0.001, limit, tolerance,
                    )
                    .maybe_calculate_aim_point(&p, &target)
                } else {
                    engine.maybe_calculate_aim_point(&p, &target)
                }
            });
        }
    }
}
fn patterns(r: &Runner) {
    let mut mt = [0u32; 624];
    mt[0] = 1337;
    for i in 1..624 {
        mt[i] = 1812433253u32
            .wrapping_mul(mt[i - 1] ^ (mt[i - 1] >> 30))
            .wrapping_add(i as u32);
    }
    let mut index = 624;
    let mut bytes = vec![0; 16 * 1024 * 1024];
    for byte in &mut bytes {
        if index == 624 {
            for i in 0..624 {
                let y = (mt[i] & 0x80000000) | (mt[(i + 1) % 624] & 0x7fffffff);
                mt[i] = mt[(i + 397) % 624] ^ (y >> 1) ^ if y & 1 != 0 { 0x9908b0df } else { 0 };
            }
            index = 0;
        }
        let mut y = mt[index];
        index += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c5680;
        y ^= (y << 15) & 0xefc60000;
        y ^= y >> 18;
        *byte = y as u8;
    }
    r.run("pattern_scan_rare_candidates", 100, |_| {
        utility::PatternScanner::scan_for_pattern(
            black_box(&bytes),
            "48 8B 05 ? ? ? ? 48 85 C0 74 ? DE AD BE EF",
        )
    });
    bytes.fill(0xcc);
    for i in (7..bytes.len()).step_by(8) {
        bytes[i] = 0;
    }
    r.run("pattern_scan_dense_candidates", 100, |_| {
        utility::PatternScanner::scan_for_pattern(black_box(&bytes), "CC CC CC CC CC CC CC CC CC")
    });
}
fn main() {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let args = std::env::args().collect::<Vec<_>>();
            let r = Runner {
                filter: args
                    .iter()
                    .find_map(|s| s.strip_prefix("--filter="))
                    .unwrap_or("")
                    .into(),
                list: args.iter().any(|s| s == "--list"),
                iterations: if args.iter().any(|s| s == "--smoke") {
                    Some(1)
                } else {
                    args.iter().find_map(|s| {
                        s.strip_prefix("--iterations=")
                            .map(|n| n.parse().expect("positive iteration count"))
                    })
                },
            };
            camera::<source::CameraTrait>(&r, "source_engine");
            camera::<opengl::CameraTrait>(&r, "opengl_engine");
            camera::<unreal::CameraTrait>(&r, "unreal_engine");
            matrices(&r);
            views(&r);
            lines(&r);
            collision(&r);
            projectile(&r);
            patterns(&r);
        })
        .unwrap()
        .join()
        .unwrap();
}
