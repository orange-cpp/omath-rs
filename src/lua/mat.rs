//! `omath.Mat3`, `omath.Mat4`, `omath.Mat4ColumnMajor` and `omath.Mat4d`.

use super::support::{
    CppInt, add_div, add_eq, add_mul, expect_args, from_lua_userdata, number, raising_function,
    set_raising_function,
};
use super::vector::Vec3f;
use crate::{ColumnMajor, Mat, NdcDepthRange, RowMajor};
use mlua::{
    AnyUserData, Lua, MetaMethod, MultiValue, Result, Table, UserData, UserDataFields,
    UserDataMethods, Value,
};

type Mat3f = Mat<3, 3, f32, RowMajor>;
type Mat4f = Mat<4, 4, f32, RowMajor>;
type Mat4ColumnMajor = Mat<4, 4, f32, ColumnMajor>;
type Mat4d = Mat<4, 4, f64, RowMajor>;

from_lua_userdata!(Mat3f, Mat4f, Mat4ColumnMajor, Mat4d);

const OUT_OF_RANGE: &str = "matrix index is out of range";
const NOT_A_ROW: &str = "matrix rows must be tables";
const MISSING_VALUE: &str = "matrix row has missing value";

/// The C++ index is an `int`, so a wide integer wraps before the range check.
fn checked_index(index: CppInt, limit: usize) -> Option<usize> {
    usize::try_from(index.0 as i32)
        .ok()
        .filter(|&index| index < limit)
}

/// The helpers only `Mat4` and `Mat4ColumnMajor` have.
macro_rules! mat4_helpers {
    ($methods:ident, $mat:ty) => {
        $methods.add_function("to_screen_mat", |_, (width, height): (f32, f32)| {
            Ok(<$mat>::to_screen(width, height))
        });
        $methods.add_function("translation", |_, diff: Vec3f| {
            Ok(<$mat>::translation(diff))
        });
        $methods.add_function("scale", |_, scale: Vec3f| Ok(<$mat>::scale(scale)));
        $methods.add_function(
            "look_at_left_handed",
            |_, (eye, center, up): (Vec3f, Vec3f, Vec3f)| {
                Ok(<$mat>::look_at_left_handed(eye, center, up))
            },
        );
        $methods.add_function(
            "look_at_right_handed",
            |_, (eye, center, up): (Vec3f, Vec3f, Vec3f)| {
                Ok(<$mat>::look_at_right_handed(eye, center, up))
            },
        );
        $methods.add_function(
            "perspective_left_handed_vertical_fov",
            |_, (field_of_view, aspect_ratio, near, far): (f32, f32, f32, f32)| {
                Ok(<$mat>::perspective_left_handed_vertical_fov(
                    field_of_view,
                    aspect_ratio,
                    near,
                    far,
                    NdcDepthRange::NegativeOneToOne,
                ))
            },
        );
        $methods.add_function(
            "perspective_right_handed_vertical_fov",
            |_, (field_of_view, aspect_ratio, near, far): (f32, f32, f32, f32)| {
                Ok(<$mat>::perspective_right_handed_vertical_fov(
                    field_of_view,
                    aspect_ratio,
                    near,
                    far,
                    NdcDepthRange::NegativeOneToOne,
                ))
            },
        );
    };
}

macro_rules! square_mat {
    ($mat:ty, $scalar:ty, $size:literal, $name:literal, $register:ident $(, $helpers:ident)?) => {
        impl UserData for $mat {
            fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
                fields.add_field_function_get("from_rows", |lua, _| {
                    raising_function(lua, concat!("omath.", $name, ".from_rows"))
                });
                fields.add_field_function_get("at", |lua, _| raising_function(lua, concat!("omath.", $name, ".at")));
                fields.add_field_function_get("set_at", |lua, _| {
                    raising_function(lua, concat!("omath.", $name, ".set_at"))
                });
            }

            fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
                methods.add_function("new", |_, args: MultiValue| {
                    expect_args(&args, 0)?;
                    Ok(<$mat>::zero())
                });
                methods.add_function("row_count", |_, ()| Ok($size));
                methods.add_function("columns_count", |_, ()| Ok($size));

                add_mul(methods, Some(|left, right| left * right), |mat, scalar| mat * scalar as $scalar);
                add_div(methods, None, |mat, scalar| mat / scalar as $scalar);
                add_eq(methods);
                methods.add_meta_method(MetaMethod::ToString, |_, this, ()| Ok(this.to_string().replace("NaN", "nan")));

                methods.add_method("transposed", |_, this, ()| Ok(this.transposed()));
                methods.add_method("determinant", |_, this, ()| Ok(this.determinant()));
                methods.add_method("sum", |_, this, ()| Ok(this.sum()));
                methods.add_method_mut("clear", |_, this, ()| {
                    this.clear();
                    Ok(())
                });
                methods.add_method_mut("set", |_, this, value: $scalar| {
                    this.set(value);
                    Ok(())
                });
                methods.add_method("inverted", |_, this, ()| Ok(this.inverted()));
                methods.add_method("as_table", |_, this, ()| {
                    Ok((0..$size)
                        .map(|row| (0..$size).map(|column| this.at(row, column)).collect::<Vec<$scalar>>())
                        .collect::<Vec<_>>())
                });

                $($helpers!(methods, $mat);)?
            }
        }

        fn $register(lua: &Lua, omath: &Table) -> Result<()> {
            set_raising_function(lua, concat!("omath.", $name, ".from_rows"), |_, rows: Table| {
                let mut result = <$mat>::zero();

                for row in 0..$size {
                    let Value::Table(row_table) = rows.get::<Value>(row + 1)? else {
                        return Ok(Err(NOT_A_ROW));
                    };

                    for column in 0..$size {
                        let Some(value) = number(&row_table.get::<Value>(column + 1)?) else {
                            return Ok(Err(MISSING_VALUE));
                        };
                        *result.at_mut(row, column) = value as $scalar;
                    }
                }

                Ok(Ok(result))
            })?;
            set_raising_function(
                lua,
                concat!("omath.", $name, ".at"),
                |_, (mat, row, column): ($mat, CppInt, CppInt)| {
                    Ok(match (checked_index(row, $size), checked_index(column, $size)) {
                        (Some(row), Some(column)) => Ok(mat.at(row, column)),
                        _ => Err(OUT_OF_RANGE),
                    })
                },
            )?;
            set_raising_function(
                lua,
                concat!("omath.", $name, ".set_at"),
                |_, (mat, row, column, value): (AnyUserData, CppInt, CppInt, $scalar)| {
                    let (Some(row), Some(column)) = (checked_index(row, $size), checked_index(column, $size)) else {
                        return Ok(Err(OUT_OF_RANGE));
                    };
                    let mut mat = mat.borrow_mut::<$mat>()?;
                    *mat.at_mut(row, column) = value;

                    Ok(Ok(*mat))
                },
            )?;

            omath.set($name, lua.create_proxy::<$mat>()?)
        }
    };
}

square_mat!(Mat3f, f32, 3, "Mat3", register_mat3);
square_mat!(Mat4f, f32, 4, "Mat4", register_mat4, mat4_helpers);
square_mat!(
    Mat4ColumnMajor,
    f32,
    4,
    "Mat4ColumnMajor",
    register_mat4_column_major,
    mat4_helpers
);
square_mat!(Mat4d, f64, 4, "Mat4d", register_mat4d);

pub(super) fn register(lua: &Lua, omath: &Table) -> Result<()> {
    register_mat3(lua, omath)?;
    register_mat4(lua, omath)?;
    register_mat4_column_major(lua, omath)?;
    register_mat4d(lua, omath)
}
