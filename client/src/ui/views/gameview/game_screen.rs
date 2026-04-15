use super::GameScreenComponent;
use super::chessboard::board::draw_chessboard;
use crate::backend::client::{BoardUpdate, ClientBackend, GameJoinedEvent, Overlay, Screen};

use bevy::prelude::*;

pub const SOURCE_COLOR: Color = Color::srgb_u8(250, 113, 113);
pub const DESTINATION_COLOR: Color = Color::srgb_u8(113, 250, 113);

pub struct ChessPlugin;

impl Plugin for ChessPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Ingame), setup_gamescreen)
            .add_systems(OnExit(Screen::Ingame), cleanup_game)
            .add_observer(on_game_joined);
    }
}

pub fn on_game_joined(
    ev: On<GameJoinedEvent>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_overlay: ResMut<NextState<Overlay>>,
    mut backend: ResMut<ClientBackend>,
) {
    backend.update_internal_board_from_fen(&ev.fen);
    next_screen.set(Screen::Ingame);
    next_overlay.set(Overlay::None);
}

/// Set-ups the in-game screen. Draws the chessboard and the two players.
pub fn setup_gamescreen(mut commands: Commands) {
    log::info!("Setting up gamescreen");

    draw_chessboard(&mut commands);

    commands.trigger(BoardUpdate);
}

/// Despawn all entities that are part of the in-game screen.
/// Obviously happens when we leave a game.
pub fn cleanup_game(mut commands: Commands, query: Query<Entity, With<GameScreenComponent>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn listen_keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_overlay: ResMut<NextState<Overlay>>,
    overlay: Res<State<Overlay>>,
) {
    if keys.just_pressed(KeyCode::Escape) && *overlay.get() == Overlay::None {
        next_overlay.set(Overlay::QuitGameDialog);
    }
}
