//! `omath.Color` and `omath.Hsv`.

use super::support::{
    CppInt, add_eq, expect_args, float_fields, from_lua_userdata, no_matching_overload, number,
    userdata,
};
use crate::{Color, Hsv};
use mlua::{Lua, MetaMethod, MultiValue, Result, Table, UserData, UserDataFields, UserDataMethods};

from_lua_userdata!(Color, Hsv);

impl UserData for Color {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("r", |_, this| Ok(this.value().x));
        fields.add_field_method_get("g", |_, this| Ok(this.value().y));
        fields.add_field_method_get("b", |_, this| Ok(this.value().z));
        fields.add_field_method_get("a", |_, this| Ok(this.value().w));
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("new", |lua: &Lua, args: MultiValue| {
            if args.is_empty() {
                return Ok(Color::default());
            }
            expect_args(&args, 4)?;
            let (r, g, b, a) = lua.unpack_multi(args)?;
            Ok(Color::new(r, g, b, a))
        });
        methods.add_function(
            "from_rgba",
            |_, (r, g, b, a): (CppInt, CppInt, CppInt, CppInt)| {
                Ok(Color::from_rgba(r.0 as u8, g.0 as u8, b.0 as u8, a.0 as u8))
            },
        );
        methods.add_function("from_hsv", |_, args: MultiValue| match args.len() {
            1 => userdata::<Hsv>(&args[0])
                .map(Color::from)
                .ok_or_else(no_matching_overload),
            3 => match (number(&args[0]), number(&args[1]), number(&args[2])) {
                (Some(hue), Some(saturation), Some(value)) => {
                    Ok(Color::from_hsv(hue as f32, saturation as f32, value as f32))
                }
                _ => Err(no_matching_overload()),
            },
            _ => Err(no_matching_overload()),
        });
        methods.add_function("red", |_, ()| Ok(Color::RED));
        methods.add_function("green", |_, ()| Ok(Color::GREEN));
        methods.add_function("blue", |_, ()| Ok(Color::BLUE));

        methods.add_method("to_hsv", |_, this, ()| Ok(this.to_hsv()));
        methods.add_method_mut("set_hue", |_, this, hue: f32| {
            this.set_hue(hue);
            Ok(())
        });
        methods.add_method_mut("set_saturation", |_, this, saturation: f32| {
            this.set_saturation(saturation);
            Ok(())
        });
        methods.add_method_mut("set_value", |_, this, value: f32| {
            this.set_value(value);
            Ok(())
        });
        methods.add_method("blend", |_, this, (other, ratio): (Color, f32)| {
            Ok(this.blend(&other, ratio))
        });

        add_eq(methods);
        methods.add_meta_method(MetaMethod::ToString, |_, this, ()| Ok(this.to_string()));
    }
}

impl UserData for Hsv {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        float_fields!(fields, hue, saturation, value);
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("new", |_, args: MultiValue| {
            expect_args(&args, 0)?;
            Ok(Hsv::default())
        });
    }
}

pub(super) fn register(lua: &Lua, omath: &Table) -> Result<()> {
    omath.set("Color", lua.create_proxy::<Color>()?)?;
    omath.set("Hsv", lua.create_proxy::<Hsv>()?)
}
