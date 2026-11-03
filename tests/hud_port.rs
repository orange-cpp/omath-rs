use omath::{
    Color, Vector2,
    hud::{widget::*, *},
};
fn p(x: f32, y: f32) -> Point {
    Vector2::new(x, y)
}
#[test]
fn canvas_and_stacked_bars_and_labels_match_cpp_layout() {
    let canvas = CanvasBox::new(p(100.0, 10.0), p(500.0, 110.0), 4.0);
    assert_eq!(canvas.top_left_corner, p(75.0, 10.0));
    assert_eq!(canvas.bottom_right_corner, p(125.0, 110.0));
    let mut overlay = EntityOverlay::from_canvas(canvas);
    let mut r = CommandBuffer {
        glyph_size: p(8.0, 12.0),
        ..Default::default()
    };
    overlay.add_side_widget(
        &mut r,
        Side::Right,
        &SideWidget::Bar(Bar {
            color: Color::RED.into(),
            size: 4.0,
            ratio: 0.25,
            ..Default::default()
        }),
    );
    overlay.add_side_widget(
        &mut r,
        Side::Right,
        &SideWidget::Label(Label {
            color: Color::GREEN.into(),
            text: "HP".into(),
            offset: 2.0,
            ..Default::default()
        }),
    );
    assert!(
        matches!(r.commands[1],DrawCommand::FilledRectangle(a,b,c) if a==p(130.0,85.0)&&b==p(134.0,110.0)&&c==Color::RED)
    );
    assert!(matches!(&r.commands[3],DrawCommand::Text(a,_,text) if *a==p(136.0,10.0)&&text=="HP"));
}
#[test]
fn gradients_clamp_bar_ratio_and_dash_validation_terminates() {
    let paint = resolve_bar_paint(
        BarGradient {
            empty_color: Color::RED,
            full_color: Color::GREEN,
        }
        .into(),
        2.0,
        true,
    );
    assert!(
        matches!(paint,Paint::Gradient(g) if g.top_left==Color::GREEN&&g.bottom_right==Color::RED)
    );
    let mut r = CommandBuffer::default();
    draw_dash_gaps(
        &mut r,
        p(0.0, 0.0),
        p(1.0, 0.0),
        p(0.0, 4.0),
        100.0,
        Color::BLUE,
        0.0,
        0.0,
    );
    assert!(r.commands.is_empty());
}
#[test]
fn crosshair_and_corner_anchor() {
    let mut screen = ScreenOverlay::new(p(800.0, 600.0));
    let mut r = CommandBuffer {
        glyph_size: p(8.0, 12.0),
        ..Default::default()
    };
    screen.contents(
        &mut r,
        [
            ScreenWidget::Crosshair(Crosshair {
                color: Color::RED,
                ..Default::default()
            }),
            ScreenWidget::Corner(Corner {
                anchor: Anchor::BottomRight,
                padding: 10.0,
                children: vec![CornerWidget::Label(Label {
                    color: Color::BLUE.into(),
                    text: "FPS".into(),
                    ..Default::default()
                })],
            }),
        ],
    );
    assert_eq!(r.commands.len(), 5);
    assert!(matches!(&r.commands[4],DrawCommand::Text(a,_,_) if *a==p(766.0,578.0)));
}
#[cfg(feature = "lua")]
#[test]
fn lua_renderer_chaining_textures_and_errors() {
    let lua = mlua::Lua::new();
    omath::lua::register_lib(&lua).unwrap();
    lua.load(
        r#"
      local calls = 0
      local r = omath.hud.Renderer.new({
        add_line=function(a,b,c,t) calls=calls+1 end,
        add_image=function(id,a,b,c) assert(id.tag=='texture'); calls=calls+1 end,
        calc_text_size=function(s) return omath.Vec2.new(#s*8,12) end
      })
      local box = omath.hud.CanvasBox.new(omath.Vec2.new(100,10),omath.Vec2.new(100,110))
      assert(box.top_left_corner.x==75 and #box:as_table()==4)
      local e = omath.hud.EntityOverlay.from_canvas(box,r)
      e:add_snap_line(omath.Vec2.new(0,0),omath.Color.red(),1):add_right_icon({tag='texture'},16,16)
      assert(calls==2)
      local weak=setmetatable({}, {__mode='v'})
      do local texture={tag='texture'};weak[1]=texture;e:add_right_icon(texture,16,16) end
      collectgarbage('collect');assert(weak[1]==nil, 'renderer retained a transient texture')
      local broken = omath.hud.Renderer.new({add_line=function() error('renderer failed') end})
      local e2 = omath.hud.EntityOverlay.from_canvas(box,broken)
      local ok,err=pcall(function() e2:add_snap_line(omath.Vec2.new(0,0),omath.Color.red(),1) end)
      assert(not ok and type(err)=='string' and err:find('renderer failed'), tostring(err))
      ok,err=pcall(function() e:add_projectile_aim(omath.Vec2.new(0,0),omath.Color.red(),3,1,7) end)
      assert(not ok and err=='invalid ProjectileAimFigure', tostring(err))
      ok,err=pcall(omath.hud.EntityOverlay.from_canvas,box,nil)
      assert(not ok and type(err)=='string', tostring(err))
      ok,err=pcall(function() omath.hud.ProjectileAimFigure.CIRCLE=5 end)
      assert(not ok and omath.hud.ProjectileAimFigure.CIRCLE==0)
    "#,
    )
    .exec()
    .unwrap();
}
#[cfg(feature = "imgui")]
#[test]
fn imgui_emits_gradient_and_closed_outline_geometry() {
    let mut context = imgui::Context::create();
    context.set_ini_filename(None);
    context.io_mut().display_size = [800.0, 600.0];
    context.io_mut().delta_time = 1.0 / 60.0;
    context.fonts().build_rgba32_texture();
    let ui = context.frame();
    {
        let mut r = ImguiHudRenderer::new(ui);
        let g = Gradient::new(Color::RED, Color::GREEN, Color::BLUE, Color::RED);
        r.add_gradient_rectangle(p(10.0, 10.0), p(100.0, 100.0), g);
        r.add_gradient_filled_circle(p(200.0, 200.0), 10.0, g, 16);
        r.add_gradient_text(p(10.0, 200.0), g, "test");
    }
    assert!(context.render().total_vtx_count > 20);
}
