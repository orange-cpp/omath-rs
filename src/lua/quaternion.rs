//! `omath.Quaternion`.

use super::support::{LuaScalar, add_eq, add_mul, expect_args, float_fields, from_lua_userdata};
use super::vector::Vec3f;
use crate::Quaternion;
use mlua::{Lua, MetaMethod, MultiValue, Result, Table, UserData, UserDataFields, UserDataMethods};

type Quatf = Quaternion<f32>;

from_lua_userdata!(Quatf);

impl UserData for Quatf {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        float_fields!(fields, x, y, z, w);
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("new", |lua: &Lua, args: MultiValue| {
            if args.is_empty() {
                return Ok(Quatf::default());
            }
            expect_args(&args, 4)?;
            let (x, y, z, w) = lua.unpack_multi(args)?;
            Ok(Quatf::new(x, y, z, w))
        });
        methods.add_function("from_axis_angle", |_, (axis, angle_rad): (Vec3f, f32)| {
            Ok(Quatf::from_axis_angle(axis, angle_rad))
        });

        methods.add_meta_function(MetaMethod::Add, |_, (left, right): (Quatf, Quatf)| {
            Ok(left + right)
        });
        add_mul(
            methods,
            Some(|left, right| left * right),
            |quaternion, scalar| quaternion * scalar as f32,
        );
        methods.add_meta_method(MetaMethod::Unm, |_, this, ()| Ok(-*this));
        add_eq(methods);
        methods.add_meta_method(MetaMethod::ToString, |_, this, ()| {
            let components = this.as_array().map(LuaScalar::cpp_format).join(", ");
            Ok(format!("Quaternion({components})"))
        });

        methods.add_method("conjugate", |_, this, ()| Ok(this.conjugate()));
        methods.add_method("dot", |_, this, other: Quatf| Ok(this.dot(other)));
        methods.add_method("length", |_, this, ()| Ok(this.length()));
        methods.add_method("length_sqr", |_, this, ()| Ok(this.length_sqr()));
        methods.add_method("normalized", |_, this, ()| Ok(this.normalized()));
        methods.add_method("inverse", |_, this, ()| Ok(this.inverse()));
        methods.add_method("rotate", |_, this, vector: Vec3f| Ok(this.rotate(vector)));
        methods.add_method("to_rotation_matrix3", |_, this, ()| {
            Ok(this.to_rotation_matrix3())
        });
        methods.add_method("to_rotation_matrix4", |_, this, ()| {
            Ok(this.to_rotation_matrix4())
        });
        methods.add_method("as_table", |lua, this, ()| {
            let table = lua.create_table()?;
            table.set("x", this.x)?;
            table.set("y", this.y)?;
            table.set("z", this.z)?;
            table.set("w", this.w)?;
            Ok(table)
        });
    }
}

pub(super) fn register(lua: &Lua, omath: &Table) -> Result<()> {
    omath.set("Quaternion", lua.create_proxy::<Quatf>()?)
}
