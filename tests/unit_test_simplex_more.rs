mod common;

use omath::Vector3;
use omath::collision::Simplex;

type Vec3 = Vector3<f32>;

mod simplex_extra {
    use super::common::expect_near;
    use super::*;

    #[test]
    fn handle_line_collinear_produces_perp() {
        // a and b placed so ab points roughly same dir as ao and are collinear
        let a = Vec3::new(2.0, 0.0, 0.0);
        let b = Vec3::new(1.0, 0.0, 0.0);

        let mut s = Simplex::default();
        s.assign(&[a, b]);

        let mut dir = Vec3::default();
        let hit = s.handle(&mut dir);

        // Should not report collision for a line simplex
        assert!(!hit);
        // Direction must be finite and not zero
        assert!(dir.x.is_finite());
        assert!(dir.y.is_finite());
        assert!(dir.z.is_finite());
        assert!(dir != Vec3::default());

        // Ensure direction is (approximately) perpendicular to ab
        let ab = b - a;
        expect_near(dir.dot(ab), 0.0f32, 1e-4f32);
    }

    #[test]
    fn handle_line_non_collinear_produces_valid_direction() {
        let a = Vec3::new(2.0, 0.0, 0.0);
        let b = Vec3::new(1.0, 1.0, 0.0);

        let mut s = Simplex::default();
        s.assign(&[a, b]);

        let mut dir = Vec3::default();
        let hit = s.handle(&mut dir);

        assert!(!hit);
        assert!(dir.x.is_finite());
        assert!(dir.y.is_finite());
        assert!(dir.z.is_finite());
    }

    #[test]
    fn handle_triangle_flip_winding() {
        // Construct points where triangle winding will be flipped
        let a = Vec3::new(1.0, 0.0, 0.0);
        let b = Vec3::new(0.0, 1.0, 0.0);
        let c = Vec3::new(0.0, -1.0, 0.0);

        let mut s = Simplex::default();
        s.assign(&[a, b, c]);

        let mut dir = Vec3::default();
        let hit = s.handle(&mut dir);

        assert!(!hit);
        assert!(dir.x.is_finite());
        assert!(dir.y.is_finite());
        assert!(dir.z.is_finite());
    }

    #[test]
    fn handle_tetrahedron_inside_returns_true() {
        // Simple tetra that should contain the origin
        let a = Vec3::new(1.0, 0.0, 0.0);
        let b = Vec3::new(0.0, 1.0, 0.0);
        let c = Vec3::new(0.0, 0.0, 1.0);
        let d = Vec3::new(-0.2, -0.2, -0.2);

        let mut s = Simplex::default();
        s.assign(&[a, b, c, d]);

        let mut dir = Vec3::default();

        // If origin is inside, handle_tetrahedron should return true
        assert!(s.handle(&mut dir));
    }
}

// Additional sanity tests
mod simplex_more {
    use super::common::expect_near;
    use super::*;

    #[test]
    fn push_front_and_access() {
        let mut s = Simplex::default();
        s.push_front(Vec3::new(1.0, 0.0, 0.0));
        s.push_front(Vec3::new(2.0, 0.0, 0.0));
        s.push_front(Vec3::new(3.0, 0.0, 0.0));

        assert_eq!(s.size(), 3);
        let exp_front = Vec3::new(3.0, 0.0, 0.0);
        let exp_back = Vec3::new(1.0, 0.0, 0.0);
        assert!(s.front() == exp_front);
        assert!(s.back() == exp_back);
        let d = s.data();
        assert!(d[0] == exp_front);
    }

    #[test]
    fn clear_and_empty() {
        let mut s = Simplex::default();
        s.push_front(Vec3::new(1.0, 1.0, 1.0));
        assert!(!s.empty());
        s.clear();
        assert!(s.empty());
    }

    #[test]
    fn handle_line_collinear_produces_perp() {
        let mut s = Simplex::default();
        s.assign(&[Vec3::new(2.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0)]);
        let mut dir = Vec3::default();
        assert!(!s.handle(&mut dir));
        assert!(dir.length_sqr() > 0.0);
    }

    #[test]
    fn handle_triangle_flip_winding() {
        let a = Vec3::new(1.0, 0.0, 0.0);
        let b = Vec3::new(0.0, 1.0, 0.0);
        let c = Vec3::new(0.0, 0.0, 1.0);
        let mut s = Simplex::default();
        s.assign(&[a, b, c]);
        let mut dir = Vec3::default();

        let ab = b - a;
        let ac = c - a;
        let abc = ab.cross(ac);

        assert!(!s.handle(&mut dir));
        let expected = -abc;
        expect_near(dir.x, expected.x, 1e-6f32);
        expect_near(dir.y, expected.y, 1e-6f32);
        expect_near(dir.z, expected.z, 1e-6f32);
    }

    #[test]
    fn handle_tetrahedron_inside_true() {
        let mut s = Simplex::default();
        s.assign(&[
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(-1.0, -1.0, -1.0),
        ]);
        let mut dir = Vec3::default();
        assert!(s.handle(&mut dir));
    }

    #[test]
    fn handle_point_sets_direction() {
        let mut s = Simplex::default();
        s.assign(&[Vec3::new(1.0, 2.0, 3.0)]);
        let mut dir = Vec3::default();
        assert!(!s.handle(&mut dir));
        expect_near(dir.x, -1.0f32, 1e-6f32);
        expect_near(dir.y, -2.0f32, 1e-6f32);
        expect_near(dir.z, -3.0f32, 1e-6f32);
    }

    #[test]
    fn handle_line_reduces_to_point_when_ao_opposite() {
        let mut s = Simplex::default();
        s.assign(&[Vec3::new(1.0, 0.0, 0.0), Vec3::new(2.0, 0.0, 0.0)]);
        let mut dir = Vec3::default();
        assert!(!s.handle(&mut dir));
        assert_eq!(s.size(), 1);
        expect_near(dir.x, -1.0f32, 1e-6f32);
    }
}
