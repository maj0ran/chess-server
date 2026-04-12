use bevy::prelude::Component;

pub mod chessboard;
pub mod dialogs;
pub mod game_screen;

/// Marker component for everything that is on the in-game screen.
/// (The board, player names, move list, etc. ...)
#[derive(Component)]
pub struct GameScreenComponent;
