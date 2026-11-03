use super::support::{
    add_raising_function, add_raising_method, from_lua_userdata, read_only_table, subtable,
};
use crate::{
    Color,
    hud::{widget::*, *},
};
use mlua::{
    AnyUserData, Function, IntoLuaMulti, Lua, Result, Table, UserData, UserDataFields,
    UserDataMethods, Value,
};
use std::{cell::RefCell, rc::Rc};
from_lua_userdata!(CanvasBox);
impl UserData for CanvasBox {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        macro_rules! field {
            ($name:ident) => {
                fields.add_field_method_get(stringify!($name), |_, this| Ok(this.$name));
                fields.add_field_method_set(stringify!($name), |_, this, value: Point| {
                    this.$name = value;
                    Ok(())
                });
            };
        }
        field!(top_left_corner);
        field!(top_right_corner);
        field!(bottom_left_corner);
        field!(bottom_right_corner);
    }
    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_function(
            "new",
            |_, (top, bottom, ratio): (Point, Point, Option<f32>)| {
                Ok(Self::new(top, bottom, ratio.unwrap_or(4.0)))
            },
        );
        m.add_function("from_bounds", |_, (min, max): (Point, Point)| {
            Ok(Self::from_bounds(min, max))
        });
        m.add_method("as_table", |lua, this, ()| {
            lua.create_sequence_from(this.as_array())
        });
    }
}
struct RendererState {
    callbacks: Table,
    error: Option<mlua::Error>,
    textures: Vec<Value>,
}
impl RendererState {
    fn call(&mut self, name: &str, args: impl IntoLuaMulti) {
        if self.error.is_some() {
            return;
        }
        let result = (|| {
            if let Some(f) = self.callbacks.get::<Option<Function>>(name)? {
                f.call::<()>(args)?;
            }
            Ok(())
        })();
        if let Err(e) = result {
            self.error = Some(e);
        }
    }
    fn finish(&mut self) -> Result<()> {
        self.error.take().map_or(Ok(()), Err)
    }
}
impl HudRenderer for RendererState {
    fn add_line(&mut self, a: Point, b: Point, c: Color, t: f32) {
        self.call("add_line", (a, b, c, t));
    }
    fn add_polyline(&mut self, v: &[Point], c: Color, t: f32) {
        self.call("add_polyline", (v.to_vec(), c, t));
    }
    fn add_filled_polyline(&mut self, v: &[Point], c: Color) {
        self.call("add_filled_polyline", (v.to_vec(), c));
    }
    fn add_rectangle(&mut self, a: Point, b: Point, c: Color) {
        self.call("add_rectangle", (a, b, c));
    }
    fn add_filled_rectangle(&mut self, a: Point, b: Point, c: Color) {
        self.call("add_filled_rectangle", (a, b, c));
    }
    fn add_circle(&mut self, a: Point, r: f32, c: Color, t: f32, s: i32) {
        self.call("add_circle", (a, r, c, t, s));
    }
    fn add_filled_circle(&mut self, a: Point, r: f32, c: Color, s: i32) {
        self.call("add_filled_circle", (a, r, c, s));
    }
    fn add_arc(&mut self, a: Point, r: f32, min: f32, max: f32, c: Color, t: f32, s: i32) {
        self.call("add_arc", (a, r, min, max, c, t, s));
    }
    fn add_image(&mut self, id: usize, a: Point, b: Point, c: Color) {
        self.call(
            "add_image",
            (
                self.textures.get(id).cloned().unwrap_or(Value::Nil),
                a,
                b,
                c,
            ),
        );
    }
    fn add_text(&mut self, a: Point, c: Color, text: &str) {
        self.call("add_text", (a, c, text));
    }
    fn calc_text_size(&mut self, text: &str) -> Point {
        if self.error.is_some() {
            return Point::default();
        }
        let result = (|| match self.callbacks.get::<Option<Function>>("calc_text_size")? {
            Some(f) => f.call::<Point>(text),
            None => Ok(Point::default()),
        })();
        match result {
            Ok(p) => p,
            Err(e) => {
                self.error = Some(e);
                Point::default()
            }
        }
    }
}
#[derive(Clone)]
struct LuaRenderer(Rc<RefCell<RendererState>>);
impl UserData for LuaRenderer {
    fn add_fields<F: UserDataFields<Self>>(m: &mut F) {
        add_raising_function(m, "new", |_, callbacks: Table| {
            Ok(Self(Rc::new(RefCell::new(RendererState {
                callbacks,
                error: None,
                textures: Vec::new(),
            }))))
        });
        macro_rules! direct {($name:ident,($($arg:ident:$ty:ty),*))=>{add_raising_method(m,stringify!($name),|_,this,($($arg,)*):($($ty,)*)|{let mut r=this.0.try_borrow_mut().map_err(mlua::Error::external)?;r.$name($($arg),*);r.finish()});};}
        direct!(add_line,(a:Point,b:Point,c:Color,t:f32));
        direct!(add_rectangle,(a:Point,b:Point,c:Color));
        direct!(add_filled_rectangle,(a:Point,b:Point,c:Color));
        add_raising_method(
            m,
            "add_text",
            |_, this, (a, c, text): (Point, Color, String)| {
                let mut r = this.0.try_borrow_mut().map_err(mlua::Error::external)?;
                r.add_text(a, c, &text);
                r.finish()
            },
        );
        add_raising_method(
            m,
            "add_polyline",
            |_, this, (points, color, t): (Table, Color, f32)| {
                let points = points
                    .sequence_values::<Point>()
                    .collect::<Result<Vec<_>>>()?;
                let mut r = this.0.try_borrow_mut().map_err(mlua::Error::external)?;
                r.add_polyline(&points, color, t);
                r.finish()
            },
        );
        add_raising_method(
            m,
            "add_filled_polyline",
            |_, this, (points, color): (Table, Color)| {
                let points = points
                    .sequence_values::<Point>()
                    .collect::<Result<Vec<_>>>()?;
                let mut r = this.0.try_borrow_mut().map_err(mlua::Error::external)?;
                r.add_filled_polyline(&points, color);
                r.finish()
            },
        );
        add_raising_method(
            m,
            "add_circle",
            |_, this, (a, r, c, t, s): (Point, f32, Color, f32, Option<i32>)| {
                let mut renderer = this.0.try_borrow_mut().map_err(mlua::Error::external)?;
                renderer.add_circle(a, r, c, t, s.unwrap_or(0));
                renderer.finish()
            },
        );
        add_raising_method(
            m,
            "add_filled_circle",
            |_, this, (a, r, c, s): (Point, f32, Color, Option<i32>)| {
                let mut renderer = this.0.try_borrow_mut().map_err(mlua::Error::external)?;
                renderer.add_filled_circle(a, r, c, s.unwrap_or(0));
                renderer.finish()
            },
        );
        add_raising_method(m,"add_arc",|_,this,(a,r,min,max,c,t,s):(Point,f32,f32,f32,Color,f32,Option<i32>)|{let mut renderer=this.0.try_borrow_mut().map_err(mlua::Error::external)?;renderer.add_arc(a,r,min,max,c,t,s.unwrap_or(0));renderer.finish()});
        add_raising_method(
            m,
            "add_image",
            |_, this, (texture, a, b, c): (Value, Point, Point, Option<Color>)| {
                let mut renderer = this.0.try_borrow_mut().map_err(mlua::Error::external)?;
                renderer.call(
                    "add_image",
                    (texture, a, b, c.unwrap_or(Color::new(1.0, 1.0, 1.0, 1.0))),
                );
                renderer.finish()
            },
        );
        add_raising_method(m, "calc_text_size", |_, this, text: String| {
            let mut r = this.0.try_borrow_mut().map_err(mlua::Error::external)?;
            let size = r.calc_text_size(&text);
            r.finish()?;
            Ok(size)
        });
    }
}
#[derive(Clone)]
struct LuaOverlay {
    state: Rc<RefCell<EntityOverlay>>,
    renderer: LuaRenderer,
}
impl LuaOverlay {
    fn draw(&self, w: EntityWidget) -> Result<Self> {
        let mut state = self.state.try_borrow_mut().map_err(mlua::Error::external)?;
        let mut r = self
            .renderer
            .0
            .try_borrow_mut()
            .map_err(mlua::Error::external)?;
        state.draw(&mut *r, &w);
        r.finish()?;
        Ok(self.clone())
    }
    fn side(&self, side: Side, w: SideWidget) -> Result<Self> {
        self.draw(EntityWidget::Side(side, vec![w]))
    }
}
impl UserData for LuaOverlay {
    fn add_fields<F: UserDataFields<Self>>(m: &mut F) {
        add_raising_function(
            m,
            "new",
            |_, (top, bottom, renderer): (Point, Point, AnyUserData)| {
                let renderer = renderer.borrow::<LuaRenderer>()?.clone();
                Ok(Self {
                    state: Rc::new(RefCell::new(EntityOverlay::new(top, bottom, 4.0))),
                    renderer,
                })
            },
        );
        add_raising_function(
            m,
            "from_canvas",
            |_, (canvas, renderer): (CanvasBox, AnyUserData)| {
                let renderer = renderer.borrow::<LuaRenderer>()?.clone();
                Ok(Self {
                    state: Rc::new(RefCell::new(EntityOverlay::from_canvas(canvas))),
                    renderer,
                })
            },
        );
        add_raising_method(m,"add_2d_box",|_,this,(color,fill,outline,t):(Color,Option<Color>,Option<Color>,Option<f32>)|this.draw(EntityWidget::Box(Box{color,fill:fill.unwrap_or_default().into(),outline:outline.unwrap_or_default(),thickness:t.unwrap_or(1.0),glow:None})));
        add_raising_method(
            m,
            "add_cornered_2d_box",
            |_,
             this,
             (color, fill, outline, ratio, t): (
                Color,
                Option<Color>,
                Option<Color>,
                Option<f32>,
                Option<f32>,
            )| {
                this.draw(EntityWidget::CorneredBox(CorneredBox {
                    color,
                    fill: fill.unwrap_or_default(),
                    outline: outline.unwrap_or_default(),
                    corner_ratio: ratio.unwrap_or(0.2),
                    thickness: t.unwrap_or(1.0),
                    glow: None,
                }))
            },
        );
        add_raising_method(
            m,
            "add_dashed_box",
            |_, this, (color, dash, gap, t): (Color, Option<f32>, Option<f32>, Option<f32>)| {
                this.draw(EntityWidget::DashedBox(DashedBox {
                    color,
                    dash_len: dash.unwrap_or(8.0),
                    gap_len: gap.unwrap_or(5.0),
                    thickness: t.unwrap_or(1.0),
                }))
            },
        );
        for (prefix, side) in [
            ("right", Side::Right),
            ("left", Side::Left),
            ("top", Side::Top),
            ("bottom", Side::Bottom),
        ] {
            add_raising_method(
                m,
                &format!("add_{prefix}_bar"),
                move |_,
                      this,
                      (color, outline, bg, size, ratio, offset): (
                    Color,
                    Color,
                    Color,
                    f32,
                    f32,
                    Option<f32>,
                )| {
                    this.side(
                        side,
                        SideWidget::Bar(Bar {
                            color: color.into(),
                            outline,
                            bg,
                            size,
                            ratio,
                            offset: offset.unwrap_or(5.0),
                            glow: None,
                        }),
                    )
                },
            );
            add_raising_method(
                m,
                &format!("add_{prefix}_dashed_bar"),
                move |_,
                      this,
                      (color, outline, bg, size, ratio, dash, gap, offset): (
                    Color,
                    Color,
                    Color,
                    f32,
                    f32,
                    f32,
                    f32,
                    Option<f32>,
                )| {
                    this.side(
                        side,
                        SideWidget::DashedBar(DashedBar {
                            bar: Bar {
                                color: color.into(),
                                outline,
                                bg,
                                size,
                                ratio,
                                offset: offset.unwrap_or(5.0),
                                glow: None,
                            },
                            dash_len: dash,
                            gap_len: gap,
                        }),
                    )
                },
            );
            for centered in [false, true] {
                if centered && matches!(side, Side::Left | Side::Right) {
                    continue;
                }
                let name = if centered {
                    &format!("add_centered_{prefix}_label")
                } else {
                    &format!("add_{prefix}_label")
                };
                add_raising_method(
                    m,
                    name,
                    move |_, this, (color, offset, outlined, text): (Color, f32, bool, String)| {
                        let label = Label {
                            color: color.into(),
                            offset,
                            outlined: if outlined {
                                Outlined::On
                            } else {
                                Outlined::Off
                            },
                            text,
                            glow: None,
                        };
                        this.side(
                            side,
                            if centered {
                                SideWidget::Centered(label)
                            } else {
                                SideWidget::Label(label)
                            },
                        )
                    },
                );
            }
            add_raising_method(
                m,
                &format!("add_{prefix}_space_vertical"),
                move |_, this, size: f32| this.side(side, SideWidget::SpaceVertical(size)),
            );
            add_raising_method(
                m,
                &format!("add_{prefix}_space_horizontal"),
                move |_, this, size: f32| this.side(side, SideWidget::SpaceHorizontal(size)),
            );
            add_raising_method(
                m,
                &format!("add_{prefix}_progress_ring"),
                move |_,
                      this,
                      (color, bg, radius, ratio, t, offset, segments): (
                    Color,
                    Color,
                    f32,
                    f32,
                    Option<f32>,
                    Option<f32>,
                    Option<i32>,
                )| {
                    this.side(
                        side,
                        SideWidget::ProgressRing(ProgressRing {
                            color,
                            bg,
                            radius,
                            ratio,
                            thickness: t.unwrap_or(2.0),
                            offset: offset.unwrap_or(5.0),
                            segments: segments.unwrap_or(0),
                        }),
                    )
                },
            );
            add_raising_method(
                m,
                &format!("add_{prefix}_icon"),
                move |_,
                      this,
                      (texture, width, height, tint, offset): (
                    Value,
                    f32,
                    f32,
                    Option<Color>,
                    Option<f32>,
                )| {
                    let id = {
                        let mut r = this
                            .renderer
                            .0
                            .try_borrow_mut()
                            .map_err(mlua::Error::external)?;
                        let id = r.textures.len();
                        r.textures.push(texture);
                        id
                    };
                    let result = this.side(
                        side,
                        SideWidget::Icon(Icon {
                            texture_id: id,
                            width,
                            height,
                            tint: tint.unwrap_or(Color::new(1.0, 1.0, 1.0, 1.0)),
                            offset: offset.unwrap_or(5.0),
                        }),
                    );
                    this.renderer
                        .0
                        .try_borrow_mut()
                        .map_err(mlua::Error::external)?
                        .textures
                        .truncate(id);
                    result
                },
            );
        }
        add_raising_method(
            m,
            "add_snap_line",
            |_, this, (start, color, width): (Point, Color, f32)| {
                this.draw(EntityWidget::SnapLine(SnapLine {
                    start,
                    color,
                    width,
                }))
            },
        );
        add_raising_method(
            m,
            "add_skeleton",
            |_, this, (color, t): (Color, Option<f32>)| {
                this.draw(EntityWidget::Skeleton(Skeleton {
                    color,
                    thickness: t.unwrap_or(1.0),
                    bones: vec![],
                }))
            },
        );
        add_raising_method(
            m,
            "add_scan_marker",
            |_, this, (color, outline, t): (Color, Option<Color>, Option<f32>)| {
                this.draw(EntityWidget::ScanMarker(ScanMarker {
                    color,
                    outline: outline.unwrap_or_default(),
                    outline_thickness: t.unwrap_or(1.0),
                }))
            },
        );
        add_raising_method(
            m,
            "add_aim_dot",
            |_, this, (position, color, radius): (Point, Color, Option<f32>)| {
                this.draw(EntityWidget::AimDot(AimDot {
                    position,
                    color,
                    radius: radius.unwrap_or(3.0),
                }))
            },
        );
        add_raising_method(
            m,
            "add_projectile_aim",
            |_,
             this,
             (position, color, size, line, figure): (
                Point,
                Color,
                Option<f32>,
                Option<f32>,
                Option<i32>,
            )| {
                let figure = match figure.unwrap_or(1) {
                    0 => Figure::Circle,
                    1 => Figure::Square,
                    _ => return Err(mlua::Error::external("invalid ProjectileAimFigure")),
                };
                this.draw(EntityWidget::ProjectileAim(ProjectileAim {
                    position,
                    color,
                    size: size.unwrap_or(3.0),
                    line_size: line.unwrap_or(1.0),
                    figure,
                }))
            },
        );
    }
}
pub(super) fn register(lua: &Lua, omath: &Table) -> Result<()> {
    let hud = subtable(lua, omath, "hud")?;
    hud.set("CanvasBox", lua.create_proxy::<CanvasBox>()?)?;
    hud.set("Renderer", lua.create_proxy::<LuaRenderer>()?)?;
    hud.set("EntityOverlay", lua.create_proxy::<LuaOverlay>()?)?;
    let figures = lua.create_table()?;
    figures.set("CIRCLE", 0)?;
    figures.set("SQUARE", 1)?;
    hud.set("ProjectileAimFigure", read_only_table(lua, figures)?)
}
