use crate::backend::config::Config;
use crate::backend::network::NetworkInterface;
use bevy::prelude::*;
use chess_core::GameId;
use chess_core::states::GameOverReason;
use std::collections::HashMap;

#[derive(Event)]
pub struct BoardUpdate;

#[derive(Event)]
pub struct GameJoinedEvent {
    pub fen: String,
}

#[derive(Event)]
pub struct GameOverEvent {
    pub reason: GameOverReason,
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum Screen {
    #[default]
    Menu,
    Game,
    Ingame,
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum MenuTab {
    #[default]
    None,
    Games,
    Analysis,
    Puzzle,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameDetails {
    pub white_player: Option<usize>,
    pub black_player: Option<usize>,
    pub _time: u32,
    pub _time_inc: u32,
}

pub struct MenuState {
    pub games: HashMap<GameId, Option<GameDetails>>,
    // local store for ClientDetails. Note: we only have names currently, but this should be a whole struct later.
    pub client_names: HashMap<usize, String>,
}

pub struct GameState {
    pub internal_board: HashMap<String, char>, // the internal board representation. Can be rendered on a GUI board.
}

/// ClientBackend is the shared resource for all our bevy UI.
#[derive(Resource)]
pub struct ClientBackend {
    pub network: NetworkInterface,
    pub in_game_id: Option<GameId>,
    pub name: String,
    pub menu_state: MenuState,
    pub game_state: GameState,
}

/// Creating a new client state sets up its own thread for the network messaging with the server.
/// To communicate from the UI with this thread, we use channels.
impl ClientBackend {
    pub fn new() -> Self {
        let config = Config::read("settings.cfg");
        Self::with_config(config)
    }

    pub fn with_config(config: Config) -> Self {
        let name = config.name.clone();
        Self {
            // interface for the network logic to communicate with the server.
            network: NetworkInterface::with_config(config),
            in_game_id: None,
            // state of the main menu, i.e., list of games, selected game.
            menu_state: MenuState {
                games: HashMap::new(),
                client_names: HashMap::new(),
            },
            // the state of the currently active game
            game_state: GameState {
                internal_board: HashMap::new(),
            },
            name,
        }
    }

    /// Sets up a board from a given FEN. Used for game start.
    /// TODO: A strange place for this method...
    pub fn update_internal_board_from_fen(&mut self, fen: &str) {
        self.game_state.internal_board.clear();
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
                        self.game_state.internal_board.insert(tile, c);
                        f += 1;
                    }
                }
            }
        }
    }
}
