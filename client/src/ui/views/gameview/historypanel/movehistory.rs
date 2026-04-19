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
                width: Val::Px(400.0),
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
    query_display: Single<&mut Text, With<MoveHistory>>,
    history: ResMut<ActiveGame>,
) {
    let history = &history.move_history;
    let half_move_num = history.len();
    let mut text = query_display.into_inner();

    log::debug!("Updating move history");
    let last_move = &history.last().unwrap();
    if &history.len() % 2 == 1 {
        let move_num = half_move_num / 2 + 1;
        let move_num_spacing = " ".repeat(4 - move_num.to_string().len());
        let white_black_spacing = " ".repeat(10 - last_move.len());
        text.0.push_str(&format!(
            "{}.{} {}{}",
            move_num, move_num_spacing, last_move, &white_black_spacing
        ));
    } else {
        text.0.push_str(&format!("{}\n", last_move));
    }
}
