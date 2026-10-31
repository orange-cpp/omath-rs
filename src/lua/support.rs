//! What the bindings share: argument checks, `std::format` compatible floats, and raising plain string errors.

use crate::Real;
use mlua::{
    Error, FromLua, FromLuaMulti, Function, IntoLua, IntoLuaMulti, Lua, MetaMethod, MultiValue,
    Result, Table, UserData, UserDataMethods, Value,
};

const FAILURE_KEY: &str = "omath.failure";
const RAISING_KEY: &str = "omath.raising";
const READ_ONLY_KEY: &str = "omath.read_only";

// A Rust callback's `Err` reaches Lua as an opaque userdata, while sol2 raises the `what()` of a C++ exception as a
// plain string. Scripts can match on that string, so the functions that throw on purpose go through `raising`.
const HELPERS: &str = r#"
local failure = ...
local error, next, setmetatable = error, next, setmetatable

local function finish(first, ...)
    if first == failure then
        error((...), 0)
    end
    return first, ...
end

local function raising(raw)
    return function(...)
        return finish(raw(...))
    end
end

local function fail_on_newindex()
    error("sol: cannot modify the elements of an enumeration table", 0)
end

local function read_only(values)
    return setmetatable({}, {
        __index = values,
        __newindex = fail_on_newindex,
        __pairs = function()
            return next, values, nil
        end,
    })
end

return raising, read_only
"#;

pub(super) fn install_helpers(lua: &Lua) -> Result<()> {
    let failure = lua.create_table()?;
    let (raising, read_only): (Function, Function) =
        lua.load(HELPERS).set_name("=omath").call(failure.clone())?;

    lua.set_named_registry_value(FAILURE_KEY, failure)?;
    lua.set_named_registry_value(RAISING_KEY, raising)?;
    lua.set_named_registry_value(READ_ONLY_KEY, read_only)
}

/// Stores under `key` a function that raises the `&str` returned by `function` as a plain string error. A userdata
/// exposes it with [`raising_function`] in an `add_field_function_get`.
pub(super) fn set_raising_function<A, R, F>(lua: &Lua, key: &str, function: F) -> Result<()>
where
    A: FromLuaMulti,
    R: IntoLuaMulti,
    F: Fn(&Lua, A) -> Result<std::result::Result<R, &'static str>> + 'static,
{
    let raw = lua.create_function(move |lua, args: A| match function(lua, args)? {
        Ok(value) => value.into_lua_multi(lua),
        Err(message) => {
            (lua.named_registry_value::<Table>(FAILURE_KEY)?, message).into_lua_multi(lua)
        }
    })?;
    let wrapped: Function = lua
        .named_registry_value::<Function>(RAISING_KEY)?
        .call(raw)?;

    lua.set_named_registry_value(key, wrapped)
}

pub(super) fn raising_function(lua: &Lua, key: &str) -> Result<Function> {
    lua.named_registry_value(key)
}

/// A read-only view of `values`, as sol2's `new_enum` makes.
pub(super) fn read_only_table(lua: &Lua, values: Table) -> Result<Table> {
    lua.named_registry_value::<Function>(READ_ONLY_KEY)?
        .call(values)
}

/// `parent[name]`, created if missing.
pub(super) fn subtable(lua: &Lua, parent: &Table, name: &str) -> Result<Table> {
    match parent.get::<Option<Table>>(name)? {
        Some(table) => Ok(table),
        None => {
            let table = lua.create_table()?;
            parent.set(name, &table)?;
            Ok(table)
        }
    }
}

pub(super) fn no_matching_overload() -> Error {
    Error::runtime(
        "no matching function call takes this number of arguments and the specified types",
    )
}

/// Constructors dispatch on the exact argument count, where a tuple of arguments would ignore extra ones.
pub(super) fn expect_args(args: &MultiValue, count: usize) -> Result<()> {
    if args.len() == count {
        Ok(())
    } else {
        Err(no_matching_overload())
    }
}

/// The number a value holds, without the string coercion of [`FromLua`]: sol2 overloads do not take numeric strings.
pub(super) fn number(value: &Value) -> Option<f64> {
    match value {
        Value::Integer(integer) => Some(*integer as f64),
        Value::Number(number) => Some(*number),
        _ => None,
    }
}

pub(super) fn userdata<T: Copy + 'static>(value: &Value) -> Option<T> {
    value
        .as_userdata()?
        .borrow::<T>()
        .ok()
        .map(|borrowed| *borrowed)
}

pub(super) fn copy_from_userdata<T: Copy + 'static>(value: Value) -> Result<T> {
    match &value {
        Value::UserData(data) => Ok(*data.borrow::<T>()?),
        other => Err(Error::FromLuaConversionError {
            from: other.type_name(),
            to: std::any::type_name::<T>().to_owned(),
            message: None,
        }),
    }
}

/// `FromLua` for `Copy` userdata, so that bindings can take them by value.
macro_rules! from_lua_userdata {
    ($($ty:ty),+ $(,)?) => {$(
        impl mlua::FromLua for $ty {
            fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
                $crate::lua::support::copy_from_userdata(value)
            }
        }
    )+};
}
pub(super) use from_lua_userdata;

/// Readable and writable `f32` fields.
macro_rules! float_fields {
    ($fields:ident, $($field:ident),+) => {$(
        $fields.add_field_method_get(stringify!($field), |_, this| Ok(this.$field));
        $fields.add_field_method_set(stringify!($field), |_, this, value: f32| {
            this.$field = value;
            Ok(())
        });
    )+};
}
pub(super) use float_fields;

/// A C++ integer argument. sol2 casts an integer and rounds a float, where [`FromLua`] rejects either when it does not
/// fit, so `300` is a valid (wrapping) `uint8_t` and `1.5` a valid index.
#[derive(Clone, Copy)]
pub(super) struct CppInt(pub(super) i64);

impl FromLua for CppInt {
    fn from_lua(value: Value, lua: &Lua) -> Result<Self> {
        Ok(Self(match value {
            Value::Integer(integer) => integer,
            other => f64::from_lua(other, lua)?.round() as i64,
        }))
    }
}

/// `__eq`. Lua also calls it for two userdata of different types, which are never equal.
pub(super) fn add_eq<T, M>(methods: &mut M)
where
    T: Copy + PartialEq + 'static,
    M: UserDataMethods<T>,
{
    methods.add_meta_function(MetaMethod::Eq, |_, (left, right): (Value, Value)| {
        Ok(matches!((userdata::<T>(&left), userdata::<T>(&right)), (Some(left), Some(right)) if left == right))
    });
}

/// `__mul`, with the scalar on either side. Without `by_self`, `T * T` has no overload.
pub(super) fn add_mul<T, M>(
    methods: &mut M,
    by_self: Option<fn(T, T) -> T>,
    by_scalar: fn(T, f64) -> T,
) where
    T: UserData + Copy + 'static,
    M: UserDataMethods<T>,
{
    methods.add_meta_function(
        MetaMethod::Mul,
        move |_, (left, right): (Value, Value)| match (
            userdata::<T>(&left),
            userdata::<T>(&right),
            number(&left),
            number(&right),
            by_self,
        ) {
            (Some(left), Some(right), _, _, Some(by_self)) => Ok(by_self(left, right)),
            (Some(value), _, _, Some(scalar), _) | (_, Some(value), Some(scalar), _, _) => {
                Ok(by_scalar(value, scalar))
            }
            _ => Err(no_matching_overload()),
        },
    );
}

/// `__div`, with the scalar on the right only. Without `by_self`, `T / T` has no overload.
pub(super) fn add_div<T, M>(
    methods: &mut M,
    by_self: Option<fn(T, T) -> T>,
    by_scalar: fn(T, f64) -> T,
) where
    T: UserData + Copy + 'static,
    M: UserDataMethods<T>,
{
    methods.add_meta_function(
        MetaMethod::Div,
        move |_, (left, right): (Value, Value)| match (
            userdata::<T>(&left),
            userdata::<T>(&right),
            number(&right),
            by_self,
        ) {
            (Some(left), Some(right), _, Some(by_self)) => Ok(by_self(left, right)),
            (Some(value), _, Some(scalar), _) => Ok(by_scalar(value, scalar)),
            _ => Err(no_matching_overload()),
        },
    );
}

/// The scalars the bindings use, with `std::format("{}", value)` as their string form.
pub(super) trait LuaScalar: Real + FromLua + IntoLua + 'static {
    fn cpp_format(self) -> String;
}

macro_rules! lua_scalar {
    ($($float:ty),+) => {$(
        impl LuaScalar for $float {
            fn cpp_format(self) -> String {
                let magnitude = self.abs();
                let forms = CppFloatForms {
                    scientific: format!("{magnitude:e}"),
                    integer: format!("{magnitude:.0}"),
                    fixed: magnitude.to_string(),
                };
                forms.format(self.is_sign_negative(), self.is_nan())
            }
        }
    )+};
}
lua_scalar!(f32, f64);

/// A float's magnitude in the shortest scientific, exact integer and shortest fixed forms.
struct CppFloatForms {
    scientific: String,
    integer: String,
    fixed: String,
}

impl CppFloatForms {
    /// `std::to_chars` shortest form: whichever of fixed and scientific is shorter, fixed on a tie, with a two digit
    /// exponent. Rust's `{}` differs on `1e20`, `0.0001`, large integers (C++ prints their exact value) and NaN.
    fn format(self, negative: bool, nan: bool) -> String {
        let sign = if negative { "-" } else { "" };

        if nan {
            return format!("{sign}nan");
        }
        let Some((mantissa, exponent)) = self.scientific.split_once('e') else {
            return format!("{sign}inf");
        };
        let exponent: i32 = exponent.parse().expect("`{:e}` writes an integer exponent");
        let digits = mantissa.len() as i32 - i32::from(mantissa.contains('.'));
        let is_integer = exponent >= digits - 1;

        let fixed_length = if exponent < 0 {
            digits + 1 - exponent
        } else if is_integer {
            exponent + 1
        } else {
            digits + 1
        };
        let exponent_text = format!(
            "{}{:02}",
            if exponent < 0 { '-' } else { '+' },
            exponent.abs()
        );
        let scientific_length = mantissa.len() + 1 + exponent_text.len();

        if fixed_length as usize > scientific_length {
            format!("{sign}{mantissa}e{exponent_text}")
        } else if is_integer {
            format!("{sign}{}", self.integer)
        } else {
            format!("{sign}{}", self.fixed)
        }
    }
}
