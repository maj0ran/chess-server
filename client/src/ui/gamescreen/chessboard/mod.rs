use bevy::prelude::*;
use chess_core::Promotion;

mod assets;
mod piece;
mod square;
mod board;


pub use board::*;
use piece::*;
use square::*;

pub use assets::ChessAssets;
pub use board::{ChessBoard, ResetSelection, RotateBoardEvent};

#[derive(Resource)]
pub struct PendingMove {
    pub src: String,
    pub dst: String,
}

/// Event to send the selected source and destination squares to the client.
#[derive(Event)]
pub struct RequestMove {
    pub source: String,
    pub destination: String,
    pub promotion: Option<Promotion>,
}

pub struct ChessboardPlugin;

impl Plugin for ChessboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChessAssets>()
            .add_observer(draw_chessboard)
            .add_observer(handle_move)
            .add_observer(draw_pieces)
            .add_observer(on_move_request)
            .add_observer(reset_selections);
    }
}
