//! `omath.Triangle`.

use super::support::{LuaScalar, expect_args, from_lua_userdata};
use super::vector::Vec3f;
use crate::Triangle;
use mlua::{Lua, MetaMethod, MultiValue, Result, Table, UserData, UserDataFields, UserDataMethods};

pub(super) type Triangle3f = Triangle<Vec3f>;

from_lua_userdata!(Triangle3f);

macro_rules! vertex_fields {
    ($fields:ident, $($vertex:ident),+) => {$(
        $fields.add_field_method_get(stringify!($vertex), |_, this| Ok(this.$vertex));
        $fields.add_field_method_set(stringify!($vertex), |_, this, value: Vec3f| {
            this.$vertex = value;
            Ok(())
        });
    )+};
}

impl UserData for Triangle3f {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        vertex_fields!(fields, vertex1, vertex2, vertex3);
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("new", |lua: &Lua, args: MultiValue| {
            if args.is_empty() {
                return Ok(Triangle3f::default());
            }
            expect_args(&args, 3)?;
            let (vertex1, vertex2, vertex3) = lua.unpack_multi(args)?;
            Ok(Triangle3f::new(vertex1, vertex2, vertex3))
        });

        methods.add_method("calculate_normal", |_, this, ()| Ok(this.calculate_normal()));
        methods.add_method("side_a_length", |_, this, ()| Ok(this.side_a_length()));
        methods.add_method("side_b_length", |_, this, ()| Ok(this.side_b_length()));
        methods.add_method("side_a_vector", |_, this, ()| Ok(this.side_a_vector()));
        methods.add_method("side_b_vector", |_, this, ()| Ok(this.side_b_vector()));
        methods.add_method("hypot", |_, this, ()| Ok(this.hypot()));
        methods.add_method("is_rectangular", |_, this, ()| Ok(this.is_rectangular()));
        methods.add_method("mid_point", |_, this, ()| Ok(this.mid_point()));

        methods.add_meta_method(MetaMethod::ToString, |_, this, ()| {
            let vertex = |vertex: Vec3f| vertex.as_array().map(LuaScalar::cpp_format).join(", ");
            Ok(format!(
                "Triangle(({}), ({}), ({}))",
                vertex(this.vertex1),
                vertex(this.vertex2),
                vertex(this.vertex3)
            ))
        });
    }
}

pub(super) fn register(lua: &Lua, omath: &Table) -> Result<()> {
    omath.set("Triangle", lua.create_proxy::<Triangle3f>()?)
}
