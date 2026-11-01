//! Deterministic counterpart to tests/cpp/extended_reference.cpp.
use omath::{collision::*, engines::source, primitives::*, projectile_prediction::*, *};
struct Rng(u64);
impl Rng {
    fn next(&mut self, min: f32, max: f32) -> f32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        min + (max - min) * ((self.0 >> 40) as f32 / (1u32 << 24) as f32)
    }
}
fn vector(kind: &str, index: i32, v: Vector3<f32>) {
    println!("{kind} {index} {} {} {}", v.x, v.y, v.z);
}
fn main() {
    let mut rng = Rng(0x853c49e6748fea9b);
    let triangle = Triangle::new(
        Vector3::new(-1.0, 0.0, -1.0),
        Vector3::new(1.0, 0.0, -1.0),
        Vector3::new(0.0, 0.0, 1.0),
    );
    let aabb = Aabb::<f32>::new(Vector3::splat(-1.0), Vector3::splat(1.0));
    let obb = Obb {
        center: Vector3::default(),
        axis_x: Vector3::new(0.8, 0.6, 0.0),
        axis_y: Vector3::new(-0.6, 0.8, 0.0),
        axis_z: Vector3::new(0.0, 0.0, 1.0),
        half_extents: Vector3::new(1.0, 2.0, 0.5),
    };
    for i in 0..2048 {
        let ray = Ray {
            start: Vector3::new(
                rng.next(-8.0, 8.0),
                rng.next(-8.0, 8.0),
                rng.next(-8.0, 8.0),
            ),
            end: Vector3::new(
                rng.next(-8.0, 8.0),
                rng.next(-8.0, 8.0),
                rng.next(-8.0, 8.0),
            ),
            infinite_length: i % 2 == 0,
        };
        vector("triangle", i, get_ray_hit_point(&ray, &triangle));
        vector("aabb", i, get_ray_hit_point(&ray, &aabb));
        vector("obb", i, get_ray_hit_point(&ray, &obb));
    }
    let mut vertices = Vec::new();
    for x in [-1.0, 1.0] {
        for y in [-1.0, 1.0] {
            for z in [-1.0, 1.0] {
                vertices.push(Vertex {
                    position: Vector3::new(x, y, z),
                    ..Default::default()
                });
            }
        }
    }
    let a = source::MeshCollider::new(source::Mesh::new(vertices.clone(), Vec::new()));
    let mut workspace = EpaWorkspace::default();
    for i in 0..128 {
        let mut mesh = source::Mesh::new(vertices.clone(), Vec::new());
        mesh.set_origin(Vector3::new(
            rng.next(-3.0, 3.0),
            rng.next(-3.0, 3.0),
            rng.next(-3.0, 3.0),
        ));
        let b = source::MeshCollider::new(mesh);
        let hit = is_collide_with_simplex_info(&a, &b, &GjkSettings::default());
        println!("gjk {i} {}", u8::from(hit.hit));
        if hit.hit {
            let epa = epa_solve(&a, &b, &hit.simplex, EpaParams::default());
            let reused = epa_solve_with_workspace(
                &a,
                &b,
                &hit.simplex,
                EpaParams::default(),
                &mut workspace,
            );
            assert_eq!(epa, reused);
            print!("epa {i} {}", u8::from(epa.is_some()));
            if let Some(epa) = epa {
                print!(
                    " {} {} {} {}",
                    epa.depth, epa.normal.x, epa.normal.y, epa.normal.z
                );
            }
            println!();
        }
    }
    let engine = ProjPredEngineLegacy::<source::PredEngineTrait>::new(400.0, 0.005, 5.0, 5.0);
    for i in 0..64 {
        let target = Target {
            origin: Vector3::new(
                rng.next(50.0, 1000.0),
                rng.next(-100.0, 100.0),
                rng.next(0.0, 100.0),
            ),
            velocity: Vector3::new(rng.next(-20.0, 20.0), rng.next(-20.0, 20.0), 0.0),
            is_airborne: false,
        };
        let p = Projectile {
            origin: Vector3::new(3.0, 2.0, 1.0),
            launch_speed: 1000.0,
            gravity_scale: 0.4,
            ..Default::default()
        };
        let point = engine.maybe_calculate_aim_point(&p, &target);
        print!("projectile {i} {}", u8::from(point.is_some()));
        if let Some(p) = point {
            print!(" {} {} {}", p.x, p.y, p.z);
        }
        println!();
    }
}
