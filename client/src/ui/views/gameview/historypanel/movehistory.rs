use crate::client::game::ActiveGame;
use crate::ui::views::gameview::GameScreenComponent;
use bevy::color::Color;
use bevy::prelude::*;

#[derive(Event)]
pub struct MoveHistoryUpdated;

#[derive(Component)]
pub struct MoveHistory;

impl MoveHistory {
    pub fn new() -> (
        GameScreenComponent,
        MoveHistory,
        Node,
        BackgroundColor,
        Text,
        TextFont,
        TextLayout,
    ) {
        (
            GameScreenComponent,
            MoveHistory,
            Node {
                position_type: PositionType::Absolute,
                height: Val::Percent(100.0),
                width: Val::Px(250.0),
                top: Val::Px(100.0),
                justify_items: JustifyItems::Start,
                ..default()
            },
            BackgroundColor(Color::srgb(1.0, 0.0, 0.0)),
            Text::new(""),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextLayout {
                justify: Justify::Justified,
                ..default()
            },
        )
    }
}

pub fn update_move_history(
    _ev: On<MoveHistoryUpdated>,
    mut query_display: Query<&mut Text, With<MoveHistory>>,
    history: ResMut<ActiveGame>,
) {
    if let Ok(mut text) = query_display.single_mut() {
        log::debug!("Updating move history");
        let last_move = &history.move_history.last().unwrap();
        text.0.push_str(&format!("{} ", last_move));
        if &history.move_history.len() % 2 == 0 {
            text.0.push_str("\n");
        } else {
            let spacing = " ".repeat(10 - last_move.len());
            text.0.push_str(&spacing);
        }
    }
}
