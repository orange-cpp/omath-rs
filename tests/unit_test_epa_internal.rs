use omath::Vector3;
use omath::collision::{Collider, EpaParams, Simplex, epa_solve};

type Vector3f = Vector3<f32>;

// Dummy collider that returns small offsets
struct DummyCollider;

impl Collider for DummyCollider {
    type Scalar = f32;

    fn find_abs_furthest_vertex_position(&self, dir: Vector3f) -> Vector3f {
        // map direction to a small point so support_point is finite
        Vector3f::new(dir.x * 0.01, dir.y * 0.01, dir.z * 0.01)
    }

    fn get_origin(&self) -> Vector3f {
        Vector3f::default()
    }

    fn set_origin(&mut self, _: Vector3f) {}
}

#[test]
fn solve_handles_small_polytope() {
    // Create a simplex that is nearly degenerate but valid for solve
    let mut s = Simplex::default();
    s.assign(&[
        Vector3f::new(0.01, 0.0, 0.0),
        Vector3f::new(0.0, 0.01, 0.0),
        Vector3f::new(0.0, 0.0, 0.01),
        Vector3f::new(-0.01, -0.01, -0.01),
    ]);

    let params = EpaParams { max_iterations: 16, tolerance: 1e-6 };

    // Should either return a valid result or gracefully return None
    if let Some(result) = epa_solve(&DummyCollider, &DummyCollider, &s, params) {
        assert!(result.depth.is_finite());
        assert!(result.normal.x.is_finite());
        assert!(result.num_faces > 0);
    }
}
