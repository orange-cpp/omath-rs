//! Conventions of the supported engines: axes, angle ranges, matrix layout, projection and world units.
//!
//! Every engine module has the same shape, with `formulas` for free functions, `CameraTrait` and `MeshTrait` for the
//! generic [`Camera`](crate::projection::Camera) and [`Mesh`](crate::primitives::Mesh), and type aliases for those. The
//! macros below hold what is identical between engines; whatever an engine's module spells out itself is what differs.

crate::angle_range!(pub Clamped89, -89.0, 89.0, Clamped);
crate::angle_range!(pub Clamped90, -90.0, 90.0, Clamped);
crate::angle_range!(pub Normalized180, -180.0, 180.0, Normalized);

/// The matrix and angle type aliases, for a scalar, a store order and a pitch range.
macro_rules! engine_types {
    ($scalar:ty, $order:ty, $pitch:ty) => {
        pub type Mat4X4 = crate::Mat<4, 4, $scalar, $order>;
        pub type Mat3X3 = crate::Mat<3, 3, $scalar, $order>;
        pub type Mat1X3 = crate::Mat<1, 3, $scalar, $order>;
        pub type PitchAngle = crate::Angle<$scalar, $pitch>;
        pub type YawAngle = crate::Angle<$scalar, crate::engines::Normalized180>;
        pub type RollAngle = crate::Angle<$scalar, crate::engines::Normalized180>;
        pub type ViewAngles = crate::ViewAngles<
            $scalar,
            $pitch,
            crate::engines::Normalized180,
            crate::engines::Normalized180,
        >;
    };
}

/// The formulas that only depend on the engine's `rotation_matrix` and axes.
macro_rules! engine_formulas {
    ($scalar:ty) => {
        pub fn forward_vector(angles: ViewAngles) -> Vector3<$scalar> {
            rotation_matrix(angles).rotate_vector(K_ABS_FORWARD)
        }

        pub fn right_vector(angles: ViewAngles) -> Vector3<$scalar> {
            rotation_matrix(angles).rotate_vector(K_ABS_RIGHT)
        }

        pub fn up_vector(angles: ViewAngles) -> Vector3<$scalar> {
            rotation_matrix(angles).rotate_vector(K_ABS_UP)
        }

        pub fn extract_origin(mat: &Mat4X4) -> Vector3<$scalar> {
            mat.extract_origin()
        }

        pub fn extract_scale(mat: &Mat4X4) -> Vector3<$scalar> {
            mat.extract_scale()
        }
    };
}

/// Conversions between world units and real lengths, one arm per way an engine defines its unit.
macro_rules! engine_units {
    (metric) => {
        // One unit is one meter.
        engine_units!(@emit value,
            value * T::from_f64(100.0),
            value,
            value / T::from_f64(100.0),
            value);
    };
    (inch) => {
        // One unit is one inch.
        engine_units!(@emit value,
            value * T::from_f64(2.54),
            units_to_centimeters(value) / T::from_f64(100.0),
            value / T::from_f64(2.54),
            centimeters_to_units(value * T::from_f64(100.0)));
    };
    (centimeter) => {
        // One unit is one centimeter.
        engine_units!(@emit value,
            value,
            value / T::from_f64(100.0),
            value,
            value * T::from_f64(100.0));
    };
    (rage) => {
        // One unit is one meter, but the C++ has the centimeter factors the wrong way around. Kept as is.
        engine_units!(@emit value,
            value / T::from_f64(100.0),
            value,
            value * T::from_f64(100.0),
            value);
    };
    (@emit $value:ident, $to_centimeters:expr, $to_meters:expr, $from_centimeters:expr, $from_meters:expr) => {
        pub fn units_to_centimeters<T: crate::Real>($value: T) -> T {
            $to_centimeters
        }

        pub fn units_to_meters<T: crate::Real>($value: T) -> T {
            $to_meters
        }

        pub fn units_to_kilometers<T: crate::Real>(units: T) -> T {
            units_to_meters(units) / T::from_f64(1000.0)
        }

        pub fn centimeters_to_units<T: crate::Real>($value: T) -> T {
            $from_centimeters
        }

        pub fn meters_to_units<T: crate::Real>($value: T) -> T {
            $from_meters
        }

        pub fn kilometers_to_units<T: crate::Real>(kilometers: T) -> T {
            meters_to_units(kilometers * T::from_f64(1000.0))
        }
    };
}

/// `CameraTrait`, the unit struct implementing [`CameraEngine`](crate::projection::CameraEngine), and the `Camera`
/// alias. Anything after the look-at expression goes into the trait impl as is.
macro_rules! engine_camera {
    (
        $scalar:ty, $order:ty, $pitch:ty, $depth_range:ident,
        look_at($direction:ident) => $look_at:expr;
        $($overrides:tt)*
    ) => {
        pub struct CameraTrait;

        impl crate::projection::CameraEngine for CameraTrait {
            type Scalar = $scalar;
            type Order = $order;
            type PitchRange = $pitch;
            type YawRange = crate::engines::Normalized180;
            type RollRange = crate::engines::Normalized180;

            const DEPTH_RANGE: crate::NdcDepthRange = crate::NdcDepthRange::$depth_range;

            fn calc_look_at_angle(cam_origin: Vector3<$scalar>, look_at: Vector3<$scalar>) -> ViewAngles {
                let $direction = (look_at - cam_origin).normalized();
                $look_at
            }

            fn calc_view_matrix(angles: ViewAngles, cam_origin: Vector3<$scalar>) -> Mat4X4 {
                formulas::calc_view_matrix(angles, cam_origin)
            }

            /// Inverse of `rotation_matrix`; lets `Camera::calc_view_angles_from_view_matrix` recover all three angles.
            fn extract_rotation_angles(rotation: &Mat4X4) -> Option<ViewAngles> {
                Some(formulas::extract_rotation_angles(rotation))
            }

            fn calc_projection_matrix(
                fov: crate::projection::FieldOfView,
                view_port: crate::projection::ViewPort,
                near_plane: $scalar,
                far_plane: $scalar,
                ndc_depth_range: crate::NdcDepthRange,
            ) -> Mat4X4 {
                formulas::calc_perspective_projection_matrix(
                    <$scalar>::from(fov.as_degrees()),
                    <$scalar>::from(view_port.aspect_ratio()),
                    near_plane,
                    far_plane,
                    ndc_depth_range,
                )
            }

            $($overrides)*
        }

        pub type Camera = crate::projection::Camera<CameraTrait>;
    };
}

/// `MeshTrait`, the unit struct implementing [`MeshEngine`](crate::primitives::MeshEngine), and the `Mesh` alias.
macro_rules! engine_mesh {
    ($scalar:ty, $order:ty) => {
        pub struct MeshTrait;

        impl crate::primitives::MeshEngine for MeshTrait {
            type Scalar = $scalar;
            type Order = $order;
            type RotationAngles = ViewAngles;

            fn rotation_matrix(rotation: &ViewAngles) -> Mat4X4 {
                formulas::rotation_matrix(*rotation)
            }
        }

        pub type Mesh = crate::primitives::Mesh<MeshTrait>;
    };
}

/// The aliases for the fixed-size meshes.
macro_rules! engine_primitives {
    () => {
        pub type BoxMesh = crate::primitives::BoxMesh<MeshTrait>;
        pub type PlaneMesh = crate::primitives::PlaneMesh<MeshTrait>;
    };
}

/// `PredEngineTrait`, the unit struct implementing [`PredEngine`](crate::projectile_prediction::PredEngine).
///
/// Pitch comes in positive upwards whatever the engine's own convention is; `view_pitch` turns it into the engine's.
/// `height` is the up axis, `ground` the two others. The direct angles spell out what differs per engine.
macro_rules! engine_pred_trait {
    (
        $scalar:ty, height: $height:ident, ground: ($first:ident, $second:ident),
        view_pitch($pitch:ident) => $view_pitch:expr;
        direct_pitch($pitch_origin:ident, $pitch_view_to:ident) => $direct_pitch:expr;
        direct_yaw($yaw_origin:ident, $yaw_view_to:ident) => $direct_yaw:expr;
    ) => {
        #[derive(Clone, Copy, Debug, Default)]
        pub struct PredEngineTrait;

        fn view_angles($pitch: $scalar, yaw: $scalar) -> ViewAngles {
            ViewAngles::new(
                PitchAngle::from_degrees($view_pitch),
                YawAngle::from_degrees(yaw),
                RollAngle::from_degrees(0.0),
            )
        }

        impl crate::projectile_prediction::PredEngine for PredEngineTrait {
            type Scalar = $scalar;

            fn predict_projectile_position(
                launch_origin: Vector3<$scalar>,
                projectile: &crate::projectile_prediction::Projectile<$scalar>,
                pitch: $scalar,
                yaw: $scalar,
                time: $scalar,
                gravity: $scalar,
            ) -> Vector3<$scalar> {
                let mut current_pos = launch_origin
                    + forward_vector(view_angles(pitch, yaw)) * projectile.launch_speed * time;
                current_pos.$height -= (gravity * projectile.gravity_scale) * (time * time) * 0.5;

                current_pos
            }

            fn predict_target_position(
                target: &crate::projectile_prediction::Target<$scalar>,
                time: $scalar,
                gravity: $scalar,
            ) -> Vector3<$scalar> {
                let mut predicted = target.origin + target.velocity * time;

                if target.is_airborne {
                    predicted.$height -= gravity * (time * time) * 0.5;
                }

                predicted
            }

            fn calc_vector_2d_distance(delta: Vector3<$scalar>) -> $scalar {
                (delta.$first * delta.$first + delta.$second * delta.$second).sqrt()
            }

            fn get_vector_height_coordinate(vec: Vector3<$scalar>) -> $scalar {
                vec.$height
            }

            fn calc_view_basis(
                pitch: $scalar,
                yaw: $scalar,
            ) -> crate::projectile_prediction::ViewBasis<$scalar> {
                let rotation = rotation_matrix(view_angles(pitch, yaw));

                crate::projectile_prediction::ViewBasis {
                    forward: rotation.rotate_vector(K_ABS_FORWARD),
                    right: rotation.rotate_vector(K_ABS_RIGHT),
                    up: rotation.rotate_vector(K_ABS_UP),
                }
            }

            fn calc_direct_pitch_angle(
                $pitch_origin: Vector3<$scalar>,
                $pitch_view_to: Vector3<$scalar>,
            ) -> $scalar {
                $direct_pitch
            }

            fn calc_direct_yaw_angle(
                $yaw_origin: Vector3<$scalar>,
                $yaw_view_to: Vector3<$scalar>,
            ) -> $scalar {
                $direct_yaw
            }
        }
    };
}

pub mod cry;
pub mod frostbite;
pub mod iw;
pub mod opengl;
pub mod rage;
pub mod serious;
pub mod source;
pub mod unity;
pub mod unreal;
