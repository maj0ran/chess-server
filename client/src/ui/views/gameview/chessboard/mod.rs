use crate::backend::client::{Overlay, Screen};
use crate::ui::views::gameview::chessboard::assets::ChessAssets;
use crate::ui::views::gameview::chessboard::board::{
    draw_chessboard, draw_pieces, handle_move, on_resize_board, reset_selections, rotate_board,
};
use crate::ui::views::gameview::dialogs::game_over_dialog::{
    cleanup_game_over_dialog, game_over_dialog_action_system, on_game_over,
};
use crate::ui::views::gameview::dialogs::promotion_dialog::{
    despawn_promotion_dialog, promotion_action_system, spawn_promotion_dialog,
};
use crate::ui::views::gameview::dialogs::quit_game_dialog::{
    cleanup_quit_game_dialog, quit_game_dialog_action_system, setup_quit_game_dialog,
};
use bevy::prelude::*;
use chess_core::Promotion;

pub mod assets;
pub mod board;
pub mod piece;
pub mod square;

pub const BOARD_SIZE: f32 = 800.0;
pub const SQUARE_SIZE: f32 = BOARD_SIZE / 8.0;

/// Resource to hold the selected source square
#[derive(Resource)]
pub struct SourceSelect {
    pub(crate) entity: Option<Entity>,
}

/// Resource to hold the selected destination square
#[derive(Resource)]
pub struct DestinationSelect {
    pub(crate) entity: Option<Entity>,
}

#[derive(Resource)]
pub struct PendingMove {
    pub src: String,
    pub dst: String,
}

/// Event to send the selected source and destination squares to the backend.
#[derive(Event)]
pub struct RequestMove {
    pub source: String,
    pub destination: String,
    pub promotion: Option<Promotion>,
}

pub struct ChessboardPlugin;

impl Plugin for ChessboardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SourceSelect { entity: None })
            .insert_resource(DestinationSelect { entity: None })
            .init_resource::<ChessAssets>()
            .add_observer(draw_chessboard)
            .add_observer(draw_pieces)
            .add_observer(reset_selections)
            .add_observer(on_game_over)
            .add_observer(rotate_board)
            .add_systems(
                Update,
                handle_move.run_if(
                    resource_changed::<DestinationSelect>
                        .and(not(resource_added::<DestinationSelect>)),
                ),
            )
            .add_systems(OnEnter(Overlay::Promotion), spawn_promotion_dialog)
            .add_systems(OnExit(Overlay::Promotion), despawn_promotion_dialog)
            .add_systems(
                Update,
                promotion_action_system.run_if(in_state(Overlay::Promotion)),
            )
            .add_systems(OnExit(Overlay::GameOver), cleanup_game_over_dialog)
            .add_systems(
                Update,
                game_over_dialog_action_system.run_if(in_state(Overlay::GameOver)),
            )
            // Quit game dialog when pressing Escape
            .add_systems(OnEnter(Overlay::QuitGameDialog), setup_quit_game_dialog)
            .add_systems(OnExit(Overlay::QuitGameDialog), cleanup_quit_game_dialog)
            .add_systems(
                Update,
                quit_game_dialog_action_system.run_if(in_state(Overlay::QuitGameDialog)),
            )
            .add_systems(Update, on_resize_board.run_if(in_state(Screen::Game)));
    }
}
