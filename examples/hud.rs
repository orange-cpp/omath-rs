use omath::{
    Color, Vector2,
    hud::{widget::*, *},
};
fn main() {
    let mut commands = CommandBuffer {
        glyph_size: Vector2::new(8.0, 12.0),
        ..Default::default()
    };
    let mut entity =
        EntityOverlay::new(Vector2::new(400.0, 200.0), Vector2::new(400.0, 500.0), 4.0);
    entity.contents(
        &mut commands,
        [
            EntityWidget::Box(Box {
                color: Color::GREEN,
                ..Default::default()
            }),
            EntityWidget::Side(
                Side::Right,
                vec![
                    SideWidget::Bar(Bar {
                        color: BarGradient {
                            empty_color: Color::RED,
                            full_color: Color::GREEN,
                        }
                        .into(),
                        ratio: 0.75,
                        ..Default::default()
                    }),
                    SideWidget::Label(Label {
                        color: Color::GREEN.into(),
                        text: "75 HP".into(),
                        offset: 4.0,
                        ..Default::default()
                    }),
                ],
            ),
        ],
    );
    ScreenOverlay::new(Vector2::new(800.0, 600.0)).contents(
        &mut commands,
        [ScreenWidget::Crosshair(Crosshair {
            color: Color::GREEN,
            ..Default::default()
        })],
    );
    println!("{:#?}", commands.commands);
}
