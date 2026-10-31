mod common;

use common::expect_near;
use omath::Vector3;
use omath::collision::Simplex;

type Vec3 = Vector3<f32>;

#[test]
fn region_ac_selects_ac() {
    // Construct points that force the Region AC branch where ac points toward the origin
    let a = Vec3::new(1.0, 0.0, 0.0);
    let b = Vec3::new(2.0, 0.0, 0.0);
    let c = Vec3::new(0.0, 1.0, 0.0);

    let mut s = Simplex::default();
    s.assign(&[a, b, c]);

    let mut dir = Vec3::default();
    let hit = s.handle(&mut dir);

    // Should not report a collision; simplex should reduce to {a, c}
    assert!(!hit);
    assert_eq!(s.size(), 2);
    assert!(s[0] == a);
    assert!(s[1] == c);
    // direction should be finite and non-zero
    assert!(dir.x.is_finite());
    assert!(dir.y.is_finite());
    assert!(dir.z.is_finite());
}

#[test]
fn abc_above_sets_direction() {
    // Choose triangle so abc points roughly toward the origin (abc · ao > 0)
    let a = Vec3::new(-1.0, 0.0, 0.0);
    let b = Vec3::new(0.0, 1.0, 0.0);
    let c = Vec3::new(0.0, 0.0, 1.0);

    let mut s = Simplex::default();
    s.assign(&[a, b, c]);

    let mut dir = Vec3::default();
    let hit = s.handle(&mut dir);

    assert!(!hit);

    let ab = b - a;
    let ac = c - a;
    let abc = ab.cross(ac);

    // direction should equal abc (above triangle case)
    expect_near(dir.x, abc.x, 1e-6f32);
    expect_near(dir.y, abc.y, 1e-6f32);
    expect_near(dir.z, abc.z, 1e-6f32);
}
