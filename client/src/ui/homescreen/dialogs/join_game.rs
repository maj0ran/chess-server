use bevy::feathers::controls::FeathersButton;
use bevy::feathers::display::{caption, label};
use bevy::feathers::theme::InheritableThemeTextColor;
use bevy::feathers::tokens;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use chess_core::protocol::messages::ClientMessage;
use chess_core::protocol::{JoinGameParams, UserRoleSelection};

use crate::backend::LobbyState;
use crate::backend::net::NetworkSend;
use crate::ui::theme;
use crate::ui::{Dialog, Overlay};

/// Visual definition of the Join Game dialog.
pub fn join_dialog_scene() -> impl Scene {
    bsn! {
        Dialog
        Children [
            theme::default_dialog_node()
            theme::default_ui_shadow()
            BackgroundColor(theme::THEME_SURFACE)
            BorderColor::all(theme::THEME_BORDER)
            InheritableThemeTextColor(tokens::TEXT_MAIN)
            Children [
                @label("Select Side")
                TextFont { font_size: FontSize::Rem(1.5) }
                TextLayout::justify(Justify::Center)
                --
                // line separator
                Node { width: percent(100), height: px(1) }
                BackgroundColor(theme::THEME_GOLD)
                --
                @FeathersButton { @caption: bsn! { @caption("White") } }
                theme::default_button_size_180()
                on(|_: On<Activate>,
                    lobby: ResMut<LobbyState>,
                    next: ResMut<NextState<Overlay>>,
                    commands: Commands| {
                        join(UserRoleSelection::White, lobby, next, commands);
                })
                --
                @FeathersButton { @caption: bsn! { @caption("Black") } }
                theme::default_button_size_180()
                on(|_: On<Activate>,
                    lobby: ResMut<LobbyState>,
                    next: ResMut<NextState<Overlay>>,
                    commands: Commands| {
                        join(UserRoleSelection::Black, lobby, next, commands);
                })
                --
                @FeathersButton { @caption: bsn! { @caption("Random") } }
                theme::default_button_size_180()
                on(|_: On<Activate>,
                    lobby: ResMut<LobbyState>,
                    next: ResMut<NextState<Overlay>>,
                    commands: Commands| {
                        join(UserRoleSelection::Random, lobby, next, commands);
                })
                --
                @FeathersButton { @caption: bsn! { @caption("Spectate") } }
                theme::default_button_size_180()
                on(|_: On<Activate>,
                    lobby: ResMut<LobbyState>,
                    next: ResMut<NextState<Overlay>>,
                    commands: Commands| {
                        join(UserRoleSelection::Spectator, lobby, next, commands);
                })
                --
                // line separator
                Node { width: percent(100), height: px(1) }
                BackgroundColor(theme::THEME_BORDER)
                --
                @FeathersButton { @caption: bsn! { @caption("Cancel") } }
                theme::default_button_size_180()
                on(|_: On<Activate>, mut next: ResMut<NextState<Overlay>>| {
                    next.set(Overlay::None);
                })
            ]
        ]
    }
}

/// Callback to join a game.
/// This will send the JoinGame message to the server.
fn join(
    side: UserRoleSelection,
    mut lobby: ResMut<LobbyState>,
    mut next: ResMut<NextState<Overlay>>,
    mut commands: Commands,
) {
    if let Some(game_id) = lobby.pending_join_game.take() {
        commands.trigger(NetworkSend(ClientMessage::JoinGame(JoinGameParams {
            game_id,
            side,
        })));
    }
    next.set(Overlay::None);
}
