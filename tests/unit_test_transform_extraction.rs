#[macro_use]
mod projection_support;

use omath::engines::{cry, frostbite, iw, opengl, source, unity, unreal};
use omath::{Mat, Vector3};

use omath::{AngleRange, Real, StoreOrder, ViewAngles};

fn expect_vector_near<T: Real>(actual: Vector3<T>, expected: Vector3<T>, epsilon: f64) {
    expect_near!(actual.x.to_f64(), expected.x.to_f64(), epsilon);
    expect_near!(actual.y.to_f64(), expected.y.to_f64(), epsilon);
    expect_near!(actual.z.to_f64(), expected.z.to_f64(), epsilon);
}

fn verify_transform_extractors<
    T: Real,
    S: StoreOrder,
    P: AngleRange,
    Y: AngleRange,
    R: AngleRange,
>(
    origin: Vector3<T>,
    scale: Vector3<T>,
    angles: ViewAngles<T, P, Y, R>,
    rotation_matrix: impl Fn(ViewAngles<T, P, Y, R>) -> Mat<4, 4, T, S>,
    extract_origin: impl Fn(&Mat<4, 4, T, S>) -> Vector3<T>,
    extract_scale: impl Fn(&Mat<4, 4, T, S>) -> Vector3<T>,
    extract_rotation_angles: impl Fn(&Mat<4, 4, T, S>) -> ViewAngles<T, P, Y, R>,
) {
    let transform = Mat::translation(origin) * rotation_matrix(angles) * Mat::scale(scale);

    expect_vector_near(extract_origin(&transform), origin, 1e-4);
    expect_vector_near(extract_scale(&transform), scale, 1e-4);
    expect_vector_near(
        extract_rotation_angles(&transform).as_vector3(),
        angles.as_vector3(),
        1e-3,
    );
}

#[test]
fn source_engine() {
    let angles: source::ViewAngles = source::ViewAngles::new(
        source::PitchAngle::from_degrees(20.0),
        source::YawAngle::from_degrees(-35.0),
        source::RollAngle::from_degrees(15.0),
    );

    verify_transform_extractors(
        Vector3::new(12.0, -3.0, 8.0),
        Vector3::new(2.0, 3.0, 4.0),
        angles,
        source::rotation_matrix,
        source::extract_origin,
        source::extract_scale,
        source::extract_rotation_angles,
    );
}

#[test]
fn iw_engine() {
    let angles: iw::ViewAngles = iw::ViewAngles::new(
        iw::PitchAngle::from_degrees(-18.0),
        iw::YawAngle::from_degrees(42.0),
        iw::RollAngle::from_degrees(-11.0),
    );

    verify_transform_extractors(
        Vector3::new(-7.0, 5.0, 13.0),
        Vector3::new(1.5, 2.5, 3.5),
        angles,
        iw::rotation_matrix,
        iw::extract_origin,
        iw::extract_scale,
        iw::extract_rotation_angles,
    );
}

#[test]
fn unreal_engine() {
    let angles: unreal::ViewAngles = unreal::ViewAngles::new(
        unreal::PitchAngle::from_degrees(30.0),
        unreal::YawAngle::from_degrees(45.0),
        unreal::RollAngle::from_degrees(-20.0),
    );

    verify_transform_extractors(
        Vector3::new(100.0, -50.0, 25.0),
        Vector3::new(2.0, 4.0, 6.0),
        angles,
        unreal::rotation_matrix,
        unreal::extract_origin,
        unreal::extract_scale,
        unreal::extract_rotation_angles,
    );
}

#[test]
fn unity_engine() {
    let angles: unity::ViewAngles = unity::ViewAngles::new(
        unity::PitchAngle::from_degrees(15.0),
        unity::YawAngle::from_degrees(-25.0),
        unity::RollAngle::from_degrees(35.0),
    );

    verify_transform_extractors(
        Vector3::new(4.0, 9.0, -2.0),
        Vector3::new(0.5, 3.0, 7.0),
        angles,
        unity::rotation_matrix,
        unity::extract_origin,
        unity::extract_scale,
        unity::extract_rotation_angles,
    );
}

#[test]
fn frostbite_engine() {
    let angles: frostbite::ViewAngles = frostbite::ViewAngles::new(
        frostbite::PitchAngle::from_degrees(-12.0),
        frostbite::YawAngle::from_degrees(33.0),
        frostbite::RollAngle::from_degrees(-27.0),
    );

    verify_transform_extractors(
        Vector3::new(6.0, -8.0, 10.0),
        Vector3::new(1.25, 2.75, 4.25),
        angles,
        frostbite::rotation_matrix,
        frostbite::extract_origin,
        frostbite::extract_scale,
        frostbite::extract_rotation_angles,
    );
}

#[test]
fn cry_engine() {
    let angles: cry::ViewAngles = cry::ViewAngles::new(
        cry::PitchAngle::from_degrees(-18.0),
        cry::YawAngle::from_degrees(40.0),
        cry::RollAngle::from_degrees(22.0),
    );

    verify_transform_extractors(
        Vector3::new(3.0, 14.0, -6.0),
        Vector3::new(2.0, 5.0, 8.0),
        angles,
        cry::rotation_matrix,
        cry::extract_origin,
        cry::extract_scale,
        cry::extract_rotation_angles,
    );
}

#[test]
fn opengl_engine() {
    let angles: opengl::ViewAngles = opengl::ViewAngles::new(
        opengl::PitchAngle::from_degrees(24.0),
        opengl::YawAngle::from_degrees(-31.0),
        opengl::RollAngle::from_degrees(17.0),
    );

    verify_transform_extractors(
        Vector3::new(-9.0, 2.0, 11.0),
        Vector3::new(3.0, 6.0, 9.0),
        angles,
        opengl::rotation_matrix,
        opengl::extract_origin,
        opengl::extract_scale,
        opengl::extract_rotation_angles,
    );
}
