//! `omath._types` and the `omath.<engine>` tables. Every engine goes through [`register_engine`].

use super::support::{
    LuaScalar, add_eq, copy_from_userdata, expect_args, float_fields, from_lua_userdata, subtable,
};
use super::vector::Vec3f;
use crate::engines::{
    Clamped89, Clamped90, Normalized180, cry, frostbite, iw, opengl, rage, serious, source, unity,
    unreal,
};
use crate::projection::{
    Camera, CameraEngine, Error, FieldOfView, Mat4X4Of, ViewAnglesOf, ViewPort,
};
use crate::{Angle, AngleRange, Vector3, ViewAngles};
use mlua::{
    AnyUserData, FromLua, Lua, MetaMethod, MultiValue, Result, Table, UserData, UserDataFields,
    UserDataMethods, Value,
};

type PitchAngle90 = Angle<f32, Clamped90>;
type PitchAngle89 = Angle<f32, Clamped89>;
type SharedYawRoll = Angle<f32, Normalized180>;
type ViewAngles90 = ViewAngles<f32, Clamped90, Normalized180, Normalized180>;
type ViewAngles89 = ViewAngles<f32, Clamped89, Normalized180, Normalized180>;

from_lua_userdata!(ViewPort);

/// The texts of the C++ bindings, which are not the [`Display`](std::fmt::Display) of [`Error`].
fn projection_error_to_string(error: Error) -> &'static str {
    match error {
        Error::WorldPositionIsOutOfScreenBounds => "world position is out of screen bounds",
        Error::InvViewProjMatDetEqZero => "inverse view-projection matrix determinant is zero",
        Error::PerspectiveDividerLessEqZero => "perspective divider is less or equal to zero",
    }
}

impl<T: LuaScalar, R: AngleRange> FromLua for Angle<T, R> {
    fn from_lua(value: Value, _: &Lua) -> Result<Self> {
        copy_from_userdata(value)
    }
}

impl<T: LuaScalar, R: AngleRange> UserData for Angle<T, R> {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("from_degrees", |_, degrees: T| {
            Ok(Self::from_degrees(degrees))
        });
        methods.add_function("from_radians", |_, radians: T| {
            Ok(Self::from_radians(radians))
        });
        methods.add_method("as_degrees", |_, this, ()| Ok(this.as_degrees()));
        methods.add_method("as_radians", |_, this, ()| Ok(this.as_radians()));
        methods.add_method("sin", |_, this, ()| Ok(Self::sin(*this)));
        methods.add_method("cos", |_, this, ()| Ok(Self::cos(*this)));
        methods.add_method("tan", |_, this, ()| Ok(Self::tan(*this)));
        methods.add_method("cot", |_, this, ()| Ok(Self::cot(*this)));

        methods.add_meta_function(MetaMethod::Add, |_, (left, right): (Self, Self)| {
            Ok(left + right)
        });
        methods.add_meta_function(MetaMethod::Sub, |_, (left, right): (Self, Self)| {
            Ok(left - right)
        });
        methods.add_meta_method(MetaMethod::Unm, |_, this, ()| Ok(-*this));
        add_eq(methods);
        // sol2 derives these two from the C++ `operator<=>`
        methods.add_meta_function(MetaMethod::Lt, |_, (left, right): (Self, Self)| {
            Ok(left < right)
        });
        methods.add_meta_function(MetaMethod::Le, |_, (left, right): (Self, Self)| {
            Ok(left <= right)
        });
        methods.add_meta_method(MetaMethod::ToString, |_, this, ()| {
            Ok(format!("{}deg", this.as_degrees().cpp_format()))
        });
    }
}

impl UserData for ViewPort {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        float_fields!(fields, width, height);
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("new", |lua: &Lua, args: MultiValue| {
            expect_args(&args, 2)?;
            let (width, height) = lua.unpack_multi(args)?;
            Ok(ViewPort::new(width, height))
        });
        methods.add_method("aspect_ratio", |_, this, ()| Ok(this.aspect_ratio()));
    }
}

impl<T, P, Y, R> FromLua for ViewAngles<T, P, Y, R>
where
    T: LuaScalar,
    P: AngleRange + Default,
    Y: AngleRange + Default,
    R: AngleRange + Default,
{
    fn from_lua(value: Value, _: &Lua) -> Result<Self> {
        copy_from_userdata(value)
    }
}

impl<T, P, Y, R> UserData for ViewAngles<T, P, Y, R>
where
    T: LuaScalar,
    P: AngleRange + Default,
    Y: AngleRange + Default,
    R: AngleRange + Default,
{
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("pitch", |_, this| Ok(this.pitch));
        fields.add_field_method_set("pitch", |_, this, value: Angle<T, P>| {
            this.pitch = value;
            Ok(())
        });
        fields.add_field_method_get("yaw", |_, this| Ok(this.yaw));
        fields.add_field_method_set("yaw", |_, this, value: Angle<T, Y>| {
            this.yaw = value;
            Ok(())
        });
        fields.add_field_method_get("roll", |_, this| Ok(this.roll));
        fields.add_field_method_set("roll", |_, this, value: Angle<T, R>| {
            this.roll = value;
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("new", |lua: &Lua, args: MultiValue| {
            expect_args(&args, 3)?;
            let (pitch, yaw, roll) = lua.unpack_multi(args)?;
            Ok(Self::new(pitch, yaw, roll))
        });
    }
}

/// Vectors cross the Lua boundary as `omath.Vec3`, also for the `f64` Unreal camera, which a `Vec3` rounds to `f32`.
fn engine_vector<T: LuaScalar>(vector: Vec3f) -> Vector3<T> {
    vector.cast()
}

fn lua_result<T: LuaScalar>(
    result: std::result::Result<Vector3<T>, Error>,
) -> (Option<Vec3f>, Option<&'static str>) {
    match result {
        Ok(position) => (Some(position.cast()), None),
        Err(error) => (None, Some(projection_error_to_string(error))),
    }
}

impl<E> UserData for Camera<E>
where
    E: CameraEngine,
    E::Scalar: LuaScalar,
    Mat4X4Of<E>: UserData + FromLua + 'static,
    ViewAnglesOf<E>: UserData + FromLua + 'static,
{
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("new", |lua: &Lua, args: MultiValue| {
            expect_args(&args, 6)?;
            let (position, view_angles, view_port, field_of_view, near, far) =
                lua.unpack_multi(args)?;
            Ok(Self::new(
                engine_vector(position),
                view_angles,
                view_port,
                field_of_view,
                near,
                far,
            ))
        });

        methods.add_method_mut("look_at", |_, this, target: Vec3f| {
            this.look_at(engine_vector(target));
            Ok(())
        });
        methods.add_method("get_forward", |_, this, ()| {
            Ok(this.get_forward().cast::<f32>())
        });
        methods.add_method("get_right", |_, this, ()| {
            Ok(this.get_right().cast::<f32>())
        });
        methods.add_method("get_up", |_, this, ()| Ok(this.get_up().cast::<f32>()));
        methods.add_method("get_origin", |_, this, ()| {
            Ok(this.get_origin().cast::<f32>())
        });
        methods.add_method("get_view_angles", |_, this, ()| Ok(this.get_view_angles()));
        methods.add_method("get_near_plane", |_, this, ()| Ok(this.get_near_plane()));
        methods.add_method("get_far_plane", |_, this, ()| Ok(this.get_far_plane()));
        methods.add_method("get_field_of_view", |_, this, ()| {
            Ok(this.get_field_of_view())
        });
        methods.add_method_mut("set_origin", |_, this, origin: Vec3f| {
            this.set_origin(engine_vector(origin));
            Ok(())
        });
        methods.add_method_mut(
            "set_view_angles",
            |_, this, view_angles: ViewAnglesOf<E>| {
                this.set_view_angles(view_angles);
                Ok(())
            },
        );
        methods.add_method_mut("set_view_port", |_, this, view_port: ViewPort| {
            this.set_view_port(view_port);
            Ok(())
        });
        methods.add_method_mut(
            "set_field_of_view",
            |_, this, field_of_view: FieldOfView| {
                this.set_field_of_view(field_of_view);
                Ok(())
            },
        );
        methods.add_method_mut("set_near_plane", |_, this, near_plane: E::Scalar| {
            this.set_near_plane(near_plane);
            Ok(())
        });
        methods.add_method_mut("set_far_plane", |_, this, far_plane: E::Scalar| {
            this.set_far_plane(far_plane);
            Ok(())
        });
        methods.add_method("get_view_matrix", |_, this, ()| Ok(this.get_view_matrix()));
        methods.add_method("get_projection_matrix", |_, this, ()| {
            Ok(this.get_projection_matrix())
        });
        methods.add_method("get_view_projection_matrix", |_, this, ()| {
            Ok(this.get_view_projection_matrix())
        });
        methods.add_function(
            "extract_projection_params",
            |_, projection_matrix: Mat4X4Of<E>| {
                let params = Self::extract_projection_params(&projection_matrix);
                Ok((params.fov, params.aspect_ratio))
            },
        );
        methods.add_function(
            "calc_view_angles_from_view_matrix",
            |_, view_matrix: Mat4X4Of<E>| Ok(Self::calc_view_angles_from_view_matrix(&view_matrix)),
        );
        methods.add_function(
            "calc_origin_from_view_matrix",
            |_, view_matrix: Mat4X4Of<E>| {
                Ok(Self::calc_origin_from_view_matrix(&view_matrix).cast::<f32>())
            },
        );
        methods.add_method("world_to_screen", |_, this, position: Vec3f| {
            Ok(lua_result(this.world_to_screen(engine_vector(position))))
        });
        methods.add_method("screen_to_world", |_, this, position: Vec3f| {
            Ok(lua_result(this.screen_to_world(engine_vector(position))))
        });
    }
}

pub(super) fn register_shared_types(lua: &Lua, omath: &Table) -> Result<()> {
    let types = subtable(lua, omath, "_types")?;

    types.set("PitchAngle90", lua.create_proxy::<PitchAngle90>()?)?;
    types.set("PitchAngle89", lua.create_proxy::<PitchAngle89>()?)?;
    types.set("YawRoll", lua.create_proxy::<SharedYawRoll>()?)?;
    types.set("FieldOfView", lua.create_proxy::<FieldOfView>()?)?;
    types.set("ViewPort", lua.create_proxy::<ViewPort>()?)?;
    types.set("ViewAngles90", lua.create_proxy::<ViewAngles90>()?)?;
    types.set("ViewAngles89", lua.create_proxy::<ViewAngles89>()?)
}

/// The angle types of an engine's table.
struct EngineAngles {
    pitch: AnyUserData,
    yaw_roll: AnyUserData,
    view_angles: AnyUserData,
}

impl EngineAngles {
    /// The `f32` types in `omath._types`, picked by the engine's pitch range.
    fn shared<E: CameraEngine>(types: &Table) -> Result<Self> {
        let (pitch, view_angles) = if E::PitchRange::MAX == Clamped90::MAX {
            ("PitchAngle90", "ViewAngles90")
        } else {
            ("PitchAngle89", "ViewAngles89")
        };

        Ok(Self {
            pitch: types.get(pitch)?,
            yaw_roll: types.get("YawRoll")?,
            view_angles: types.get(view_angles)?,
        })
    }

    /// Unreal's angles are `f64`, unlike every other engine's, so none of the types in `omath._types` fit its camera.
    fn unreal(lua: &Lua) -> Result<Self> {
        Ok(Self {
            pitch: lua.create_proxy::<unreal::PitchAngle>()?,
            yaw_roll: lua.create_proxy::<unreal::YawAngle>()?,
            view_angles: lua.create_proxy::<unreal::ViewAngles>()?,
        })
    }
}

fn register_engine<E>(lua: &Lua, omath: &Table, name: &str, angles: EngineAngles) -> Result<()>
where
    E: CameraEngine,
    Camera<E>: UserData,
{
    let types: Table = omath.get("_types")?;
    let engine = subtable(lua, omath, name)?;

    engine.set("PitchAngle", angles.pitch)?;
    engine.set("YawAngle", &angles.yaw_roll)?;
    engine.set("RollAngle", angles.yaw_roll)?;
    engine.set("FieldOfView", types.get::<AnyUserData>("FieldOfView")?)?;
    engine.set("ViewPort", types.get::<AnyUserData>("ViewPort")?)?;
    engine.set("ViewAngles", angles.view_angles)?;
    engine.set("Camera", lua.create_proxy::<Camera<E>>()?)
}

pub(super) fn register_engines(lua: &Lua, omath: &Table) -> Result<()> {
    let types: Table = omath.get("_types")?;

    macro_rules! register_shared {
        ($($engine:ident),+) => {$(
            register_engine::<$engine::CameraTrait>(
                lua,
                omath,
                stringify!($engine),
                EngineAngles::shared::<$engine::CameraTrait>(&types)?,
            )?;
        )+};
    }

    register_shared!(opengl, frostbite, iw, source, rage, serious, unity, cry);
    register_engine::<unreal::CameraTrait>(lua, omath, "unreal", EngineAngles::unreal(lua)?)
}
