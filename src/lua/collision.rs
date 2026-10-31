//! `omath.primitives` and `omath.collision`.

use super::support::{
    expect_args, from_lua_userdata, no_matching_overload, number, raising_function,
    read_only_table, set_raising_function, subtable, userdata,
};
use super::triangle::Triangle3f;
use super::vector::Vec3f;
use crate::collision::{
    Collider, EpaParams, GjkSettings, Ray, can_trace_line, epa_solve, find_support_vertex,
    get_ray_hit_point, is_collide, is_collide_with_simplex_info,
};
use crate::primitives::{Aabb, Obb, UpAxis, UpX, UpZ};
use mlua::{
    Lua, MultiValue, Result, Table, UserData, UserDataFields, UserDataMethods, UserDataRef, Value,
};

type Aabbf = Aabb<f32>;
type Obbf = Obb<f32>;
type Ray3f = Ray<f32>;
type ColliderRef = UserDataRef<ConvexCollider>;

from_lua_userdata!(Aabbf, Obbf, Ray3f);

const CONVEX_COLLIDER_NEW: &str = "omath.ConvexCollider.new";
const NO_VERTICES: &str = "convex collider must contain at least one vertex";

macro_rules! vec3_fields {
    ($fields:ident, $($field:ident),+) => {$(
        $fields.add_field_method_get(stringify!($field), |_, this| Ok(this.$field));
        $fields.add_field_method_set(stringify!($field), |_, this, value: Vec3f| {
            this.$field = value;
            Ok(())
        });
    )+};
}

/// An enum value from Lua. Like `sol::optional`, whatever is not a number is the default; a number outside the enum is
/// undefined behaviour in C++ and an error here.
fn up_axis(axis: &Value) -> Result<UpAxis> {
    match number(axis).map(|axis| axis.round() as i64) {
        None | Some(1) => Ok(UpAxis::Y),
        Some(0) => Ok(UpAxis::X),
        Some(2) => Ok(UpAxis::Z),
        Some(_) => Err(no_matching_overload()),
    }
}

impl UserData for Aabbf {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        vec3_fields!(fields, min, max);
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("new", |lua: &Lua, args: MultiValue| {
            if args.is_empty() {
                return Ok(Aabbf::default());
            }
            expect_args(&args, 2)?;
            let (min, max) = lua.unpack_multi(args)?;
            Ok(Aabbf::new(min, max))
        });

        methods.add_method("center", |_, this, ()| Ok(this.center()));
        methods.add_method("extents", |_, this, ()| Ok(this.extents()));
        methods.add_method("top", |_, this, axis: Value| {
            Ok(match up_axis(&axis)? {
                UpAxis::X => Aabb::<f32, UpX>::new(this.min, this.max).top(),
                UpAxis::Y => this.top(),
                UpAxis::Z => Aabb::<f32, UpZ>::new(this.min, this.max).top(),
            })
        });
        methods.add_method("bottom", |_, this, axis: Value| {
            Ok(match up_axis(&axis)? {
                UpAxis::X => Aabb::<f32, UpX>::new(this.min, this.max).bottom(),
                UpAxis::Y => this.bottom(),
                UpAxis::Z => Aabb::<f32, UpZ>::new(this.min, this.max).bottom(),
            })
        });
        methods.add_method("is_collide", |_, this, other: Aabbf| {
            Ok(this.is_collide(&other))
        });
        methods.add_method("as_table", |lua, this, ()| {
            let table = lua.create_table()?;
            table.set("min", this.min)?;
            table.set("max", this.max)?;
            Ok(table)
        });
    }
}

impl UserData for Obbf {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        vec3_fields!(fields, center, axis_x, axis_y, axis_z, half_extents);
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("new", |lua: &Lua, args: MultiValue| {
            if args.is_empty() {
                return Ok(Obbf::default());
            }
            expect_args(&args, 5)?;
            let (center, axis_x, axis_y, axis_z, half_extents) = lua.unpack_multi(args)?;
            Ok(Obbf {
                center,
                axis_x,
                axis_y,
                axis_z,
                half_extents,
            })
        });

        methods.add_method("vertices", |_, this, ()| Ok(this.vertices()));
    }
}

impl UserData for Ray3f {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        vec3_fields!(fields, start, end);
        fields.add_field_method_get("infinite_length", |_, this| Ok(this.infinite_length));
        fields.add_field_method_set("infinite_length", |_, this, value: bool| {
            this.infinite_length = value;
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("new", |lua: &Lua, args: MultiValue| {
            let infinite_length = match args.len() {
                0 => return Ok(Ray3f::default()),
                2 => false,
                3 => match args[2] {
                    Value::Boolean(infinite_length) => infinite_length,
                    _ => return Err(no_matching_overload()),
                },
                _ => return Err(no_matching_overload()),
            };
            let (start, end): (Vec3f, Vec3f) = lua.unpack_multi(args)?;
            Ok(Ray3f {
                start,
                end,
                infinite_length,
            })
        });

        methods.add_method(
            "direction_vector",
            |_, this, ()| Ok(this.direction_vector()),
        );
        methods.add_method("direction_vector_normalized", |_, this, ()| {
            Ok(this.direction_vector_normalized())
        });
    }
}

/// A convex hull built from a Lua table of points. Replaces the C++ `LuaConvexCollider`.
struct ConvexCollider {
    vertices: Vec<Vec3f>,
    origin: Vec3f,
}

/// The points up to the first entry that is not a `Vec3`.
fn vertices_from_table(points: &Table) -> Result<Vec<Vec3f>> {
    let mut vertices = Vec::new();
    while let Some(point) = userdata::<Vec3f>(&points.get::<Value>(vertices.len() + 1)?) {
        vertices.push(point);
    }
    Ok(vertices)
}

impl Collider for ConvexCollider {
    type Scalar = f32;

    /// On a tie the first vertex wins, like `std::ranges::max_element`.
    fn find_abs_furthest_vertex_position(&self, direction: Vec3f) -> Vec3f {
        let mut furthest = self.vertices[0];
        for &vertex in &self.vertices[1..] {
            if furthest.dot(direction) < vertex.dot(direction) {
                furthest = vertex;
            }
        }
        self.origin + furthest
    }

    fn get_origin(&self) -> Vec3f {
        self.origin
    }

    fn set_origin(&mut self, new_origin: Vec3f) {
        self.origin = new_origin;
    }
}

impl UserData for ConvexCollider {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_function_get("new", |lua, _| raising_function(lua, CONVEX_COLLIDER_NEW));
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method(
            "find_abs_furthest_vertex_position",
            |_, this, direction: Vec3f| Ok(this.find_abs_furthest_vertex_position(direction)),
        );
        methods.add_method("get_origin", |_, this, ()| Ok(this.get_origin()));
        methods.add_method_mut("set_origin", |_, this, origin: Vec3f| {
            this.set_origin(origin);
            Ok(())
        });
        methods.add_method("vertex_count", |_, this, ()| Ok(this.vertices.len()));
        methods.add_method("vertices", |_, this, ()| Ok(this.vertices.clone()));
    }
}

/// Static functions only: the C++ `LineTracer` has no constructor either.
struct LineTracer;

impl UserData for LineTracer {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function(
            "can_trace_line",
            |_, (ray, triangle): (Ray3f, Triangle3f)| Ok(can_trace_line(&ray, &triangle)),
        );
        methods.add_function("get_ray_hit_point", |_, (ray, target): (Ray3f, Value)| {
            if let Some(triangle) = userdata::<Triangle3f>(&target) {
                Ok(get_ray_hit_point(&ray, &triangle))
            } else if let Some(aabb) = userdata::<Aabbf>(&target) {
                Ok(get_ray_hit_point(&ray, &aabb))
            } else if let Some(obb) = userdata::<Obbf>(&target) {
                Ok(get_ray_hit_point(&ray, &obb))
            } else {
                Err(no_matching_overload())
            }
        });
        methods.add_function(
            "ray_hits_triangle",
            |_, (ray, triangle): (Ray3f, Triangle3f)| {
                Ok(get_ray_hit_point(&ray, &triangle) != ray.end)
            },
        );
        methods.add_function("ray_hits_aabb", |_, (ray, aabb): (Ray3f, Aabbf)| {
            Ok(get_ray_hit_point(&ray, &aabb) != ray.end)
        });
        methods.add_function("ray_hits_obb", |_, (ray, obb): (Ray3f, Obbf)| {
            Ok(get_ray_hit_point(&ray, &obb) != ray.end)
        });
    }
}

pub(super) fn register_primitives(lua: &Lua, omath: &Table) -> Result<()> {
    let primitives = subtable(lua, omath, "primitives")?;

    let up_axis = lua.create_table()?;
    up_axis.set("X", 0)?;
    up_axis.set("Y", 1)?;
    up_axis.set("Z", 2)?;
    primitives.set("UpAxis", read_only_table(lua, up_axis)?)?;

    primitives.set("Aabb", lua.create_proxy::<Aabbf>()?)?;
    primitives.set("Obb", lua.create_proxy::<Obbf>()?)
}

pub(super) fn register_collision(lua: &Lua, omath: &Table) -> Result<()> {
    let collision = subtable(lua, omath, "collision")?;

    set_raising_function(lua, CONVEX_COLLIDER_NEW, |lua, args: MultiValue| {
        let (points, origin): (Table, Vec3f) = match args.len() {
            1 => (lua.unpack_multi(args)?, Vec3f::default()),
            2 => lua.unpack_multi(args)?,
            _ => return Err(no_matching_overload()),
        };
        let vertices = vertices_from_table(&points)?;

        Ok(if vertices.is_empty() {
            Err(NO_VERTICES)
        } else {
            Ok(ConvexCollider { vertices, origin })
        })
    })?;

    collision.set("Ray", lua.create_proxy::<Ray3f>()?)?;
    collision.set("ConvexCollider", lua.create_proxy::<ConvexCollider>()?)?;
    collision.set("LineTracer", lua.create_proxy::<LineTracer>()?)?;

    collision.set(
        "gjk_support_vertex",
        lua.create_function(|_, (a, b, direction): (ColliderRef, ColliderRef, Vec3f)| {
            Ok(find_support_vertex(&*a, &*b, direction))
        })?,
    )?;
    collision.set(
        "gjk_collide",
        lua.create_function(|_, (a, b): (ColliderRef, ColliderRef)| Ok(is_collide(&*a, &*b)))?,
    )?;
    collision.set(
        "epa_solve",
        lua.create_function(|lua, (a, b): (ColliderRef, ColliderRef)| {
            let gjk = is_collide_with_simplex_info(&*a, &*b, &GjkSettings::default());
            if !gjk.hit {
                return Ok(None);
            }
            let Some(epa) = epa_solve(&*a, &*b, &gjk.simplex, EpaParams::default()) else {
                return Ok(None);
            };

            let result = lua.create_table()?;
            result.set("normal", epa.normal)?;
            result.set("penetration_vector", epa.penetration_vector)?;
            result.set("depth", epa.depth)?;
            result.set("iterations", epa.iterations)?;
            result.set("num_vertices", epa.num_vertices)?;
            result.set("num_faces", epa.num_faces)?;
            Ok(Some(result))
        })?,
    )
}
