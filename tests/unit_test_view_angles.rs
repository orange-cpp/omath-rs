//! The C++ file only includes the header, so there is no test to port. This keeps the same compile check.

use omath::{Angle, ViewAngles, angle_range};

angle_range!(PitchRange, -90.0, 90.0, Clamped);
angle_range!(YawRange, -180.0, 180.0, Normalized);
angle_range!(RollRange, -180.0, 180.0, Clamped);

type TestAngles = ViewAngles<f32, PitchRange, YawRange, RollRange>;

#[test]
fn header_instantiates() {
    let angles = TestAngles::new(
        Angle::from_degrees(10.0f32),
        Angle::from_degrees(20.0f32),
        Angle::from_degrees(30.0f32),
    );

    assert_eq!(angles.as_vector3().as_array(), [10.0f32, 20.0f32, 30.0f32]);
    assert_eq!(TestAngles::default(), TestAngles::default());
}
