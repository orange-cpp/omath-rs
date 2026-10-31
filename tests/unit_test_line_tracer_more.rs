mod common;

use common::expect_near;
use omath::collision::{Ray, get_ray_hit_point};
use omath::{Triangle, Vector3};

type Vec3 = Vector3<f32>;
type Triangle3 = Triangle<Vec3>;

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

fn tri() -> Triangle3 {
    Triangle::new(v(0.0, 0.0, 0.0), v(1.0, 0.0, 0.0), v(0.0, 1.0, 0.0))
}

fn segment(start: Vec3, end: Vec3) -> Ray {
    Ray {
        start,
        end,
        ..Default::default()
    }
}

#[test]
fn parallel_ray_returns_end() {
    // Ray parallel to triangle plane: triangle in XY plane and ray along X axis
    let ray = segment(v(0.0, 0.0, 1.0), v(1.0, 0.0, 1.0));

    let hit = get_ray_hit_point(&ray, &tri());
    assert_eq!(hit, ray.end);
}

#[test]
fn u_out_of_range_returns_end() {
    // Construct a ray that misses due to u < 0
    let ray = segment(v(-1.0, -1.0, -1.0), v(-0.5, -1.0, 1.0));

    let hit = get_ray_hit_point(&ray, &tri());
    assert_eq!(hit, ray.end);
}

#[test]
fn v_out_of_range_returns_end() {
    // Construct ray that has v < 0
    let ray = segment(v(2.0, 2.0, -1.0), v(2.0, 2.0, 1.0));

    let hit = get_ray_hit_point(&ray, &tri());
    assert_eq!(hit, ray.end);
}

#[test]
fn t_hit_too_small_returns_end() {
    let ray = segment(v(0.0, 0.0, 0.0000000001), v(0.0, 0.0, 1.0));

    let hit = get_ray_hit_point(&ray, &tri());
    assert_eq!(hit, ray.end);
}

#[test]
fn t_hit_greater_than_one_returns_end() {
    let tri = tri();
    // Choose a ray and compute t_hit locally to assert consistency
    let ray = segment(v(0.0, 0.0, -1.0), v(0.0, 0.0, -0.5));

    let hit = get_ray_hit_point(&ray, &tri);

    let k_epsilon = f32::EPSILON;
    let side_a = tri.side_a_vector();
    let side_b = tri.side_b_vector();
    let ray_dir = ray.direction_vector();
    let p = ray_dir.cross(side_b);
    let det = side_a.dot(p);

    if det.abs() < k_epsilon {
        assert_eq!(hit, ray.end);
        return;
    }

    let inv_det = 1.0 / det;
    let tvec = ray.start - tri.vertex2;
    let q = tvec.cross(side_a);
    let t_hit = side_b.dot(q) * inv_det;

    if t_hit <= k_epsilon || t_hit > 1.0 {
        assert_eq!(hit, ray.end, "t_hit={t_hit}");
    } else {
        assert_ne!(hit, ray.end, "t_hit={t_hit}");
    }
}

#[test]
fn infinite_length_with_small_t_hit_returns_end() {
    // Create triangle slightly behind so t_hit <= eps
    let tri = Triangle::new(v(0.0, 0.0, -1e-8), v(1.0, 0.0, -1e-8), v(0.0, 1.0, -1e-8));
    let ray = Ray {
        start: v(0.0, 0.0, 0.0),
        end: v(0.0, 0.0, 1.0),
        infinite_length: true,
    };

    let hit = get_ray_hit_point(&ray, &tri);
    assert_eq!(hit, ray.end);
}

#[test]
fn successful_hit_returns_point() {
    let ray = segment(v(0.1, 0.1, -1.0), v(0.1, 0.1, 1.0));

    let hit = get_ray_hit_point(&ray, &tri());
    assert_ne!(hit, ray.end);
    // Hit should be on plane z=0 and near x=0.1,y=0.1
    expect_near(hit.z, 0.0f32, 1e-6f32);
    expect_near(hit.x, 0.1f32, 1e-3f32);
    expect_near(hit.y, 0.1f32, 1e-3f32);
}
