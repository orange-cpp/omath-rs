//! `omath.Vec2`, `omath.Vec3` and `omath.Vec4`.

use super::support::{
    LuaScalar, add_div, add_eq, add_mul, expect_args, float_fields, from_lua_userdata,
};
use crate::{Vector2, Vector3, Vector4};
use mlua::{Lua, MetaMethod, MultiValue, Result, Table, UserData, UserDataFields, UserDataMethods};

pub(super) type Vec2f = Vector2<f32>;
pub(super) type Vec3f = Vector3<f32>;
pub(super) type Vec4f = Vector4<f32>;

from_lua_userdata!(Vec2f, Vec3f, Vec4f);

/// What all three vectors bind the same way. Ordering is by squared length, like the C++ operators.
macro_rules! vector_methods {
    ($methods:ident, $vector:ty, $name:literal, $($axis:ident),+) => {
        $methods.add_function("new", |lua: &Lua, args: MultiValue| {
            if args.is_empty() {
                return Ok(<$vector>::default());
            }
            expect_args(&args, [$(stringify!($axis)),+].len())?;
            let ($($axis,)+) = lua.unpack_multi(args)?;
            Ok(<$vector>::new($($axis),+))
        });

        $methods.add_meta_function(MetaMethod::Add, |_, (left, right): ($vector, $vector)| Ok(left + right));
        $methods.add_meta_function(MetaMethod::Sub, |_, (left, right): ($vector, $vector)| Ok(left - right));
        $methods.add_meta_method(MetaMethod::Unm, |_, this, ()| Ok(-*this));
        add_eq($methods);
        $methods.add_meta_function(MetaMethod::Lt, |_, (left, right): ($vector, $vector)| {
            Ok(left.length_sqr() < right.length_sqr())
        });
        $methods.add_meta_function(MetaMethod::Le, |_, (left, right): ($vector, $vector)| {
            Ok(left.length_sqr() <= right.length_sqr())
        });
        $methods.add_meta_method(MetaMethod::ToString, |_, this, ()| {
            let components = this.as_array().map(LuaScalar::cpp_format).join(", ");
            Ok(format!(concat!($name, "({})"), components))
        });

        $methods.add_method("length", |_, this, ()| Ok(this.length()));
        $methods.add_method("length_sqr", |_, this, ()| Ok(this.length_sqr()));
        $methods.add_method("dot", |_, this, other: $vector| Ok(this.dot(other)));
        $methods.add_method("sum", |_, this, ()| Ok(this.sum()));
        $methods.add_method("abs", |_, this, ()| Ok(this.abs()));
    };
}

impl UserData for Vec2f {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        float_fields!(fields, x, y);
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        vector_methods!(methods, Vec2f, "Vec2", x, y);

        add_mul(methods, None, |vector, scalar| vector * scalar as f32);
        add_div(methods, None, |vector, scalar| vector / scalar as f32);

        methods.add_method("normalized", |_, this, ()| Ok(this.normalized()));
        methods.add_method("distance_to", |_, this, other: Vec2f| {
            Ok(this.distance_to(other))
        });
        methods.add_method("distance_to_sqr", |_, this, other: Vec2f| {
            Ok(this.distance_to_sqr(other))
        });
    }
}

impl UserData for Vec3f {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        float_fields!(fields, x, y, z);
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        vector_methods!(methods, Vec3f, "Vec3", x, y, z);

        add_mul(
            methods,
            Some(|left, right| left * right),
            |vector, scalar| vector * scalar as f32,
        );
        add_div(
            methods,
            Some(|left, right| left / right),
            |vector, scalar| vector / scalar as f32,
        );

        methods.add_method("length_2d", |_, this, ()| Ok(this.length_2d()));
        methods.add_method("normalized", |_, this, ()| Ok(this.normalized()));
        methods.add_method("cross", |_, this, other: Vec3f| Ok(this.cross(other)));
        methods.add_method("distance_to", |_, this, other: Vec3f| {
            Ok(this.distance_to(other))
        });
        methods.add_method("distance_to_sqr", |_, this, other: Vec3f| {
            Ok(this.distance_to_sqr(other))
        });
        methods.add_method("sum_2d", |_, this, ()| Ok(this.sum_2d()));
        methods.add_method("point_to_same_direction", |_, this, other: Vec3f| {
            Ok(this.point_to_same_direction(other))
        });
        methods.add_method("as_array", |_, this, ()| Ok(this.as_array()));
        methods.add_method("angle_between", |_, this, other: Vec3f| {
            Ok(match this.angle_between(other) {
                Ok(angle) => (Some(angle.as_degrees()), None),
                Err(_) => (None, Some("impossible angle (zero-length vector)")),
            })
        });
        methods.add_method(
            "is_perpendicular",
            |_, this, (other, epsilon): (Vec3f, Option<f32>)| {
                Ok(this.is_perpendicular(other, epsilon.unwrap_or(0.0001)))
            },
        );
        methods.add_method("as_table", |lua, this, ()| {
            let table = lua.create_table()?;
            table.set("x", this.x)?;
            table.set("y", this.y)?;
            table.set("z", this.z)?;
            Ok(table)
        });
    }
}

impl UserData for Vec4f {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        float_fields!(fields, x, y, z, w);
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        vector_methods!(methods, Vec4f, "Vec4", x, y, z, w);

        add_mul(
            methods,
            Some(|left, right| left * right),
            |vector, scalar| vector * scalar as f32,
        );
        add_div(
            methods,
            Some(|left, right| left / right),
            |vector, scalar| vector / scalar as f32,
        );

        methods.add_method_mut("clamp", |_, this, (min, max): (f32, f32)| {
            *this = this.clamped(min, max);
            Ok(*this)
        });
    }
}

pub(super) fn register(lua: &Lua, omath: &Table) -> Result<()> {
    omath.set("Vec2", lua.create_proxy::<Vec2f>()?)?;
    omath.set("Vec3", lua.create_proxy::<Vec3f>()?)?;
    omath.set("Vec4", lua.create_proxy::<Vec4f>()?)
}
