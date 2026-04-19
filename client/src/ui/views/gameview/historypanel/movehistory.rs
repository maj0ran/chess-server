use crate::client::game::ActiveGame;
use crate::ui::views::gameview::GameScreenComponent;
use bevy::color::Color;
use bevy::prelude::*;

#[derive(Event)]
pub struct MoveHistoryUpdated;
#[derive(Event)]
pub struct MoveHistoryFullRefresh;

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

pub fn refresh_move_history(
    _ev: On<MoveHistoryFullRefresh>,
    query_display: Single<&mut Text, With<MoveHistory>>,
    history: ResMut<ActiveGame>,
) {
    let mut text = query_display.into_inner();
    text.0.clear();

    let history = &history.move_history;

    for (i, move_str) in history.iter().enumerate() {
        let half_move_num = i + 1;
        if half_move_num % 2 == 1 {
            let move_num = half_move_num / 2 + 1;
            let move_num_spacing = " ".repeat(4 - move_num.to_string().len());
            let half_move_spacing = " ".repeat(10 - move_str.len());
            text.0.push_str(&format!(
                "{}.{}{}{}",
                move_num, move_num_spacing, move_str, &half_move_spacing
            ));
        } else {
            text.0.push_str(&format!("{}\n", move_str));
        }
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

    let last_move = &history.last().unwrap();
    if &history.len() % 2 == 1 {
        let move_num = half_move_num / 2 + 1;
        let move_num_spacing = " ".repeat(4 - move_num.to_string().len());
        let half_move_spacing = " ".repeat(10 - last_move.len());
        text.0.push_str(&format!(
            "{}.{}{}{}",
            move_num, move_num_spacing, last_move, &half_move_spacing
        ));
    } else {
        text.0.push_str(&format!("{}\n", last_move));
    }
}
