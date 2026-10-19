use std::collections::HashMap;

use bevy::prelude::*;

use chess_core::protocol::UserRoleSelection;
use chess_core::states::GameOverReason;
use chess_core::{ClientId, GameId};

#[derive(Event)]
pub struct BoardUpdate;

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameJoinedEvent {
    pub gid: GameId,
    pub side: UserRoleSelection,
}

pub type GameJoined = GameJoinedEvent;

#[derive(Event, Debug, Clone, PartialEq, Eq)]
pub struct ClientDetailsEvent {
    pub cid: ClientId,
    pub name: String,
}

pub type ClientDetails = ClientDetailsEvent;

#[derive(Event)]
pub struct GameOverEvent {
    pub reason: GameOverReason,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct GameDetails {
    pub white_player: Option<ClientId>,
    pub black_player: Option<ClientId>,
    pub _time: u32,
    pub _time_inc: u32,
}

impl GameDetails {
    pub fn set_player(&mut self, side: UserRoleSelection, cid: ClientId) {
        match side {
            UserRoleSelection::White => self.white_player = Some(cid),
            UserRoleSelection::Black => self.black_player = Some(cid),
            _ => {}
        }
    }
}

#[derive(Resource, Debug)]
pub struct ActiveGame {
    pub gid: GameId,
    pub side: UserRoleSelection,
    pub internal_board: HashMap<String, char>,
    pub game_info: GameDetails,

    pub move_history: Vec<String>,
}

impl ActiveGame {
    pub fn update_internal_board_from_fen(&mut self, fen: &str) {
        self.internal_board.clear();
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
                        self.internal_board.insert(tile, c);
                        f += 1;
                    }
                }
            }
        }
    }
}
