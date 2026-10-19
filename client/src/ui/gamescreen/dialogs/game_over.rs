use bevy::{
    feathers::controls::FeathersButton,
    feathers::display::{caption, label},
    feathers::theme::{InheritableThemeTextColor, ThemedText},
    feathers::tokens,
    ui_widgets::Activate,
};

use crate::{
    backend::GameOverEvent,
    ui::*,
    ui::theme::*,
};

#[derive(Component, Clone, Copy, Default)]
pub(crate) struct GameOverMessageText;
#[derive(Resource)]
pub(crate) struct GameOverMessage(String);

/// Composes the message for the 'GameOverMessageText' Component.
/// The message is composed using the Server Message and is either:
/// - {side} wins by {reason}
/// - Draw by {reason}
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
        Children [
            theme::default_dialog_node()
            theme::default_ui_shadow()
            BackgroundColor(THEME_SURFACE)
            BorderColor::all(THEME_BORDER)
            InheritableThemeTextColor(tokens::TEXT_MAIN)
            Children [
                @label("Game Over")
                TextFont { font_size: FontSize::Rem(1.5) }
                TextLayout::justify(Justify::Center)
                --
                // no Feather component here as this gets changed by `set_game_over_message()`.
                // idk if it is possible to use Feather here.
                Text("")
                ThemedText
                GameOverMessageText
                TextLayout::justify(Justify::Center)
                --
                // line separator
                Node { width: percent(100), height: px(1) }
                BackgroundColor(THEME_GOLD)
                --
                @FeathersButton { @caption: bsn! { @caption("OK") } }
                default_button_size_180()
                    on(|_: On<Activate>,
                        mut screen: ResMut<NextState<Screen>>,
                        mut overlay: ResMut<NextState<Overlay>>| {
                            screen.set(Screen::Menu); overlay.set(Overlay::None);
                    })
            ]
        ]
    }
}

/// Sets the message in the 'GameOverMessageText' Component. (see `on_game_over()`)
pub(crate) fn set_game_over_message(
    message: Res<GameOverMessage>,
    mut text: Single<&mut Text, With<GameOverMessageText>>,
) {
    text.0 = message.0.clone();
}