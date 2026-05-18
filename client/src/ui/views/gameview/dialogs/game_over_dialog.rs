use bevy::feathers::controls::FeathersButton;
use bevy::feathers::theme::{InheritableThemeTextColor, ThemedText};
use bevy::feathers::tokens;
use bevy::prelude::*;

use crate::client::game::GameOverEvent;
use crate::ui::theme::{WOOD_BORDER, WOOD_GOLD, WOOD_SURFACE};
use crate::ui::{Dialog, Overlay, Screen};

#[derive(Component, Clone, Copy, Default)]
pub(crate) struct GameOverMessageText;
#[derive(Resource)]
pub(crate) struct GameOverMessage(String);

pub fn on_game_over(
    event: On<GameOverEvent>,
    mut commands: Commands,
    mut next_overlay: ResMut<NextState<Overlay>>,
) {
    next_overlay.set(Overlay::GameOver);
    let reason = event.reason;
    let message = match reason.get_winner() {
        Some(player) => format!("{player} wins by {reason}!"),
        None => format!("Draw by {reason}!"),
    };
    commands.insert_resource(GameOverMessage(message));
}

pub fn game_over_dialog_scene() -> impl Scene {
    bsn! {
        Dialog
        Children[(
            Node { width: px(420), padding: UiRect::all(px(28)), flex_direction: FlexDirection::Column, row_gap: px(16), align_items: AlignItems::Center, border: UiRect::all(px(2)), border_radius: BorderRadius::all(px(10)) }
            BackgroundColor(WOOD_SURFACE)
            BorderColor::all(WOOD_BORDER)
            BoxShadow::new(Color::srgba(0.0, 0.0, 0.0, 0.6), px(0), px(6), px(2), px(10))
            InheritableThemeTextColor(tokens::TEXT_MAIN)
            Children[
                (Text("Game Over") ThemedText TextFont { font_size: FontSize::Px(28.0) } TextLayout::justify(Justify::Center)),
                (Text("") ThemedText GameOverMessageText TextLayout::justify(Justify::Center)),
                (Node { width: percent(100), height: px(1) } BackgroundColor(WOOD_GOLD)),
                (@FeathersButton { @caption: bsn! { Text("OK") ThemedText } } Node { width: px(180), height: px(50) }
                    on(|_: On<bevy::ui_widgets::Activate>, mut screen: ResMut<NextState<Screen>>, mut overlay: ResMut<NextState<Overlay>>| {
                        screen.set(Screen::Menu); overlay.set(Overlay::None);
                    }))
            ]
        )]
    }
}

pub(crate) fn set_game_over_message(
    message: Res<GameOverMessage>,
    mut text: Single<&mut Text, With<GameOverMessageText>>,
) {
    text.0 = message.0.clone();
}

pub fn cleanup_game_over_dialog(mut commands: Commands, query: Query<Entity, With<Dialog>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<GameOverMessage>();
}
