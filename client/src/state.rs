use crate::network::NetworkInterface;
use bevy::prelude::*;
use chess_core::UserRoleSelection;
use chess_core::{ChessColor, GameId, Tile};
use std::collections::HashMap;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum Screen {
    #[default]
    Menu,
    Game,
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum Overlay {
    #[default]
    None,
    CreateDialog,
    JoinDialog,
    QuitGameDialog,
    GameOver,
    Promotion,
}

#[derive(Debug, Clone)]
pub struct GameDetails {
    pub white_player: Option<usize>,
    pub black_player: Option<usize>,
    pub _time: u32,
    pub _time_inc: u32,
}

pub struct MenuState {
    pub games: HashMap<GameId, Option<GameDetails>>,
    pub selected_game: Option<GameId>,
    pub is_loading: bool,
    pub error_msg: Option<String>,
}

pub struct GameState {
    pub board: HashMap<String, char>,
    pub dragging_piece: Option<(char, f32, f32, String)>, // piece, x, y, from_tile
    pub pending_promotion: Option<(Tile, Tile)>,
    pub winner: Option<ChessColor>,
    pub dirty: bool,
}

/// ClientState is the shared resource for all our bevy UI.
#[derive(Resource)]
pub struct ClientState {
    pub network: NetworkInterface,
    pub menu_state: MenuState,
    pub game_state: GameState,
    pub create_settings: CreateGameSettings,
}

pub struct CreateGameSettings {
    pub time: i32,
    pub increment: i32,
    pub selected_color: UserRoleSelection,
}

/// Creating a new client state sets up its own thread for the network messaging with the server.
/// To communicate from the UI with this thread, we use channels.
impl ClientState {
    pub fn new() -> Self {
        Self {
            // interface for the network logic to communicate with the server.
            network: NetworkInterface::new(),
            // state of the main menu, i.e., list of games, selected game.
            menu_state: MenuState {
                games: HashMap::new(),
                selected_game: None,
                is_loading: false,
                error_msg: None,
            },
            // the state of the currently active game
            game_state: GameState {
                board: HashMap::new(),
                dragging_piece: None,
                pending_promotion: None,
                winner: None,
                dirty: true,
            },
            // default settings for creating a new game.
            // TODO: Time is not settable by the user for now.
            create_settings: CreateGameSettings {
                time: 10,
                increment: 5,
                selected_color: UserRoleSelection::White,
            },
        }
    }

    /// Sets up a board from a given FEN. Used for game start.
    pub fn update_board_from_fen(&mut self, fen: &str) {
        self.game_state.board.clear();
        let fen_parts: Vec<&str> = fen.split(' ').collect();
        if fen_parts.is_empty() {
            return;
        }
        let rows: Vec<&str> = fen_parts[0].split('/').collect();
        for (r, row_str) in rows.iter().enumerate() {
            let mut f = 0;
            for c in row_str.chars() {
                if let Some(digit) = c.to_digit(10) {
                    f += digit as usize;
                } else {
                    if f < 8 {
                        let tile =
                            format!("{}{}", (97 + f) as u8 as char, (8 - r + 48) as u8 as char);
                        self.game_state.board.insert(tile, c);
                        f += 1;
                    }
                }
            }
        }
    }
}
