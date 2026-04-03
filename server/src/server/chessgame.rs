use crate::chess::chess::Chess;
use crate::chess::pieces::Piece;
use chess_core::*;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct GameDetails {
    pub _started: bool,
    pub white_player: Option<ClientId>,
    pub black_player: Option<ClientId>,
    pub spectators: Vec<ClientId>,

    pub _time: u32,
    pub _time_inc: u32,
}

/// A `ChessGame` represents a real chess game between two players.
/// It wraps the 'raw' `Chess` struct, which is basically only the board and the rules,
/// and adds all the stuff around a chess game: The players, the clock, the move history, etc.
pub struct ChessGame {
    pub id: GameId,
    pub chess: Chess,

    pub details: GameDetails,
}

impl ChessGame {
    /// Starts a chess game.
    /// Chess game can only start when two players are joined.
    pub fn start_game(&mut self) -> GameManagerResult<()> {
        let details = &mut self.details;
        if details.white_player.is_none() {
            return Err(GameManagerError::InvalidGameStatus(
                "White player missing".to_string(),
            ));
        }
        if details.black_player.is_none() {
            return Err(GameManagerError::InvalidGameStatus(
                "Black player missing".to_string(),
            ));
        }
        if details._started {
            return Err(GameManagerError::InvalidGameStatus(
                "Game already started".to_string(),
            ));
        }
        details._started = true;
        Ok(())
    }

    /// Adds a client (player) to a chess game.
    /// Can be White, Black, Random, Both (for analysis) and Spectator.
    /// TODO: instead of Role 'Both', we might want a Role 'Analysis' instead. This way, we could've multiple people analysing.
    /// TODO: But maybe it'd be better to have a GameMode for Analysis instead, so we can swap the behavior of the chess game.
    pub fn add_player(
        &mut self,
        client_id: ClientId,
        side: UserRoleSelection,
    ) -> GameManagerResult<UserRoleSelection> {
        let info = &mut self.details;
        match side {
            UserRoleSelection::Black => {
                if info.black_player.is_some() {
                    Err(GameManagerError::InvalidGameStatus(
                        "Black side already taken".to_string(),
                    ))
                } else {
                    info.black_player = Some(client_id);
                    Ok(UserRoleSelection::Black)
                }
            }
            UserRoleSelection::White => {
                if info.white_player.is_some() {
                    Err(GameManagerError::InvalidGameStatus(
                        "White side already taken".to_string(),
                    ))
                } else {
                    info.white_player = Some(client_id);
                    Ok(UserRoleSelection::White)
                }
            }
            UserRoleSelection::Random => {
                if info.white_player.is_some() && info.black_player.is_some() {
                    return Err(GameManagerError::InvalidGameStatus(
                        "Game already full".to_string(),
                    ));
                };

                let side: bool = (SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .subsec_nanos()
                    % 2)
                    != 0;

                match side {
                    false => {
                        if info.black_player.is_none() {
                            info.black_player = Some(client_id);
                            Ok(UserRoleSelection::Black)
                        } else {
                            info.white_player = Some(client_id);
                            Ok(UserRoleSelection::White)
                        }
                    }
                    true => {
                        if info.white_player.is_none() {
                            info.white_player = Some(client_id);
                            Ok(UserRoleSelection::White)
                        } else {
                            info.black_player = Some(client_id);
                            Ok(UserRoleSelection::Black)
                        }
                    }
                }
            }
            UserRoleSelection::Spectator => {
                info.spectators.push(client_id);
                Ok(UserRoleSelection::Spectator)
            }
            UserRoleSelection::Both => {
                if info.white_player.is_some() || info.black_player.is_some() {
                    return Err(GameManagerError::InvalidGameStatus(
                        "Cannot control both colors in non-empty games".to_string(),
                    ));
                }
                info.white_player = Some(client_id);
                info.black_player = Some(client_id);
                Ok(UserRoleSelection::Both)
            }
        }
    }

    pub fn remove_player(&mut self, client_id: ClientId) -> Option<UserRoleSelection> {
        let info = &mut self.details;
        let mut side = None;
        if let Some(id) = info.white_player {
            if id == client_id {
                side = Some(UserRoleSelection::White);
                info.white_player = None;
            }
        }
        if let Some(id) = info.black_player {
            if id == client_id {
                side = Some(UserRoleSelection::Black);
                info.black_player = None;
            }
        }
        side
    }

    /// Returns all players and spectators of a chess game.
    pub fn get_participants(&self) -> Vec<ClientId> {
        let info = &self.details;
        let mut participants = Vec::new();
        if let Some(id) = info.white_player {
            participants.push(id);
        }
        if let Some(id) = info.black_player {
            if !participants.contains(&id) {
                participants.push(id);
            }
        }
        for &id in &info.spectators {
            if !participants.contains(&id) {
                participants.push(id);
            }
        }
        participants
    }

    pub fn make_move(
        &mut self,
        mov: ChessMove,
        client_id: ClientId,
    ) -> ChessResult<(Vec<(Tile, Option<Piece>)>)> {
        let is_current_player = match self.chess.get_active_player() {
            ChessColor::White => self.details.white_player == Some(client_id),
            ChessColor::Black => self.details.black_player == Some(client_id),
        };

        if !is_current_player {
            return Err(ChessError::NotYourTurn);
        }
        match self.chess.make_move(mov) {
            Ok(ret) => Ok(ret),
            Err(e) => Err(e),
        }
    }

    pub fn get_game_state(&mut self) -> ChessGameState {
        if self.chess.is_checkmate() {
            return ChessGameState::Checkmate(self.chess.get_active_player());
        }
        if self.chess.is_stalemate() {
            return ChessGameState::Stalemate;
        }
        ChessGameState::Running
    }
}

pub enum ChessGameState {
    Running,
    Checkmate(ChessColor),
    Stalemate,
}
