use crate::ui::Overlay;
use crate::ui::views::gameview::chessboard::assets::ChessAssets;
use crate::ui::views::gameview::chessboard::board::{
    draw_chessboard, draw_pieces, handle_move, on_move_request, reset_selections,
};
use crate::ui::views::gameview::dialogs::game_over_dialog::{
    cleanup_game_over_dialog, game_over_dialog_scene, on_game_over, set_game_over_message,
};
use crate::ui::views::gameview::dialogs::promotion_dialog::{
    despawn_promotion_dialog, initialize_promotion_images, promotion_dialog_scene,
};
use crate::ui::views::gameview::dialogs::resign_dialog::{
    cleanup_resign_dialog, resign_dialog_scene,
};
use bevy::prelude::*;
use chess_core::Promotion;

pub mod assets;
pub mod board;
pub mod piece;
pub mod square;

pub const BOARD_SIZE: f32 = 800.0;
pub const SQUARE_SIZE: f32 = BOARD_SIZE / 8.0;

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
            .add_observer(reset_selections)
            .add_observer(on_game_over)
            .add_systems(
                OnEnter(Overlay::GameOver),
                (game_over_dialog_scene.spawn(), set_game_over_message).chain(),
            )
            .add_systems(
                OnEnter(Overlay::Promotion),
                (promotion_dialog_scene.spawn(), initialize_promotion_images).chain(),
            )
            .add_systems(OnExit(Overlay::Promotion), despawn_promotion_dialog)
            .add_systems(OnExit(Overlay::GameOver), cleanup_game_over_dialog)
            .add_systems(OnEnter(Overlay::ResignDialog), resign_dialog_scene.spawn())
            .add_systems(OnExit(Overlay::ResignDialog), cleanup_resign_dialog);
    }
}
