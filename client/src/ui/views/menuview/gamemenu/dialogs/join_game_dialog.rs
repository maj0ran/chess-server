use bevy::feathers::controls::FeathersButton;
use bevy::feathers::theme::{InheritableThemeTextColor, ThemedText};
use bevy::feathers::tokens;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use chess_core::protocol::messages::ClientMessage;
use chess_core::protocol::{JoinGameParams, UserRoleSelection};

use crate::client::lobby::LobbyState;
use crate::net::network::NetworkSend;
use crate::ui::theme::{WOOD_BORDER, WOOD_GOLD, WOOD_SURFACE};
use crate::ui::{Dialog, Overlay};

pub fn join_dialog_scene() -> impl Scene {
    bsn! {
        Dialog
        Children[(
            Node {
                width: px(420), padding: UiRect::all(px(28)),
                flex_direction: FlexDirection::Column, row_gap: px(14),
                align_items: AlignItems::Center,
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(10)),
            }
            BackgroundColor(WOOD_SURFACE)
            BorderColor::all(WOOD_BORDER)
            BoxShadow::new(Color::srgba(0.0, 0.0, 0.0, 0.6), px(0), px(6), px(2), px(10))
            InheritableThemeTextColor(tokens::TEXT_MAIN)
            Children[
                (Text("Select Side") ThemedText TextFont { font_size: FontSize::Px(26.0) } TextLayout::justify(Justify::Center)),
                (Node { width: percent(100), height: px(1) } BackgroundColor(WOOD_GOLD)),
                (@FeathersButton { @caption: bsn! { Text("White") ThemedText } } Node { width: px(240), height: px(48) } on(|_: On<Activate>, lobby: ResMut<LobbyState>, next: ResMut<NextState<Overlay>>, commands: Commands| {
                    join(UserRoleSelection::White, lobby, next, commands);
                })),
                (@FeathersButton { @caption: bsn! { Text("Black") ThemedText } } Node { width: px(240), height: px(48) } on(|_: On<Activate>, lobby: ResMut<LobbyState>, next: ResMut<NextState<Overlay>>, commands: Commands| {
                    join(UserRoleSelection::Black, lobby, next, commands);
                })),
                (@FeathersButton { @caption: bsn! { Text("Random") ThemedText } } Node { width: px(240), height: px(48) } on(|_: On<Activate>, lobby: ResMut<LobbyState>, next: ResMut<NextState<Overlay>>, commands: Commands| {
                    join(UserRoleSelection::Random, lobby, next, commands);
                })),
                (@FeathersButton { @caption: bsn! { Text("Spectate") ThemedText } } Node { width: px(240), height: px(48) } on(|_: On<Activate>, lobby: ResMut<LobbyState>, next: ResMut<NextState<Overlay>>, commands: Commands| {
                    join(UserRoleSelection::Spectator, lobby, next, commands);
                })),
                (Node { width: percent(100), height: px(1) } BackgroundColor(WOOD_BORDER)),
                (@FeathersButton { @caption: bsn! { Text("Cancel") ThemedText } } Node { width: px(180), height: px(48) } on(|_: On<Activate>, mut next: ResMut<NextState<Overlay>>| next.set(Overlay::None)))
            ]
        )]
    }
}

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

pub fn cleanup_join_dialog(mut commands: Commands, query: Query<Entity, With<Dialog>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
