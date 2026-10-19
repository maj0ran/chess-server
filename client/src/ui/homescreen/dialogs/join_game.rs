use bevy::{
    prelude::*,
    feathers::controls::FeathersButton,
    feathers::display::{caption, label},
    feathers::theme::InheritableThemeTextColor,
    feathers::tokens,
    ui_widgets::Activate,
};

use crate::{
    backend::LobbyState,
    backend::net::NetworkSend,
    ui::theme::*,
    ui::{Dialog, Overlay},
};

use chess_core::protocol::messages::ClientMessage;
use chess_core::protocol::{JoinGameParams, UserRoleSelection};


/// Scene definition of the Join Game dialog.
pub fn join_dialog_scene() -> impl Scene {
    bsn! {
        Dialog
        Children [
            default_dialog_node()
            default_ui_shadow()
            BackgroundColor(THEME_SURFACE)
            BorderColor::all(THEME_BORDER)
            InheritableThemeTextColor(tokens::TEXT_MAIN)
            Children [
                @label("Select Side")
                TextFont { font_size: FontSize::Rem(1.5) }
                TextLayout::justify(Justify::Center)
                --
                // line separator
                Node { width: percent(100), height: px(1) }
                BackgroundColor(THEME_GOLD)
                --
                @FeathersButton { @caption: bsn! { @caption("White") } }
                default_button_size_180()
                on(|_: On<Activate>,
                    lobby: ResMut<LobbyState>,
                    next: ResMut<NextState<Overlay>>,
                    commands: Commands| {
                        join(UserRoleSelection::White, lobby, next, commands);
                })
                --
                @FeathersButton { @caption: bsn! { @caption("Black") } }
                default_button_size_180()
                on(|_: On<Activate>,
                    lobby: ResMut<LobbyState>,
                    next: ResMut<NextState<Overlay>>,
                    commands: Commands| {
                        join(UserRoleSelection::Black, lobby, next, commands);
                })
                --
                @FeathersButton { @caption: bsn! { @caption("Random") } }
                default_button_size_180()
                on(|_: On<Activate>,
                    lobby: ResMut<LobbyState>,
                    next: ResMut<NextState<Overlay>>,
                    commands: Commands| {
                        join(UserRoleSelection::Random, lobby, next, commands);
                })
                --
                @FeathersButton { @caption: bsn! { @caption("Spectate") } }
                default_button_size_180()
                on(|_: On<Activate>,
                    lobby: ResMut<LobbyState>,
                    next: ResMut<NextState<Overlay>>,
                    commands: Commands| {
                        join(UserRoleSelection::Spectator, lobby, next, commands);
                })
                --
                // line separator
                Node { width: percent(100), height: px(1) }
                BackgroundColor(THEME_BORDER)
                --
                @FeathersButton { @caption: bsn! { @caption("Cancel") } }
                default_button_size_180()
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
