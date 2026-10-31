use omath::Vector3;
use omath::collision::{Collider, EpaParams, Simplex, epa_solve};

type Vector3f = Vector3<f32>;

// Minimal collider matching Epa's expectations
struct DegenerateCollider;

impl Collider for DegenerateCollider {
    type Scalar = f32;

    // returns furthest point along dir
    fn find_abs_furthest_vertex_position(&self, dir: Vector3f) -> Vector3f {
        // Always return points on a small circle in XY plane so some faces become degenerate
        if dir.x > 0.5 {
            return Vector3f::new(0.01, 0.0, 0.0);
        }
        if dir.x < -0.5 {
            return Vector3f::new(-0.01, 0.0, 0.0);
        }
        if dir.y > 0.5 {
            return Vector3f::new(0.0, 0.01, 0.0);
        }
        if dir.y < -0.5 {
            return Vector3f::new(0.0, -0.01, 0.0);
        }
        Vector3f::new(0.0, 0.0, 0.01)
    }

    fn get_origin(&self) -> Vector3f {
        Vector3f::default()
    }

    fn set_origin(&mut self, _: Vector3f) {}
}

#[test]
fn degenerate_face_handled() {
    // Prepare a simplex with near-collinear points to force degenerate face handling
    let mut s = Simplex::default();
    s.assign(&[
        Vector3f::new(0.01, 0.0, 0.0),
        Vector3f::new(0.02, 0.0, 0.0),
        Vector3f::new(0.03, 0.0, 0.0),
        Vector3f::new(0.0, 0.0, 0.01),
    ]);

    let params = EpaParams {
        max_iterations: 4,
        tolerance: 1e-6,
    };

    // The algorithm should either return a valid result or gracefully exit (not crash)
    if let Some(result) = epa_solve(&DegenerateCollider, &DegenerateCollider, &s, params) {
        assert!(result.depth.is_finite());
        assert!(result.normal.x.is_finite());
    }
}
