use crate::chess::chess::Chess;
use crate::chess::pieces::Piece;
use chess_core::protocol::UserRoleSelection;
use chess_core::states::{ChessGameState, GameOverReason};
use chess_core::*;
use std::time::{SystemTime, UNIX_EPOCH};

/// A `ChessGame` represents a real chess game between two players.
/// It wraps the 'raw' `Chess` struct, which is basically only the board and the rules,
/// and adds all the stuff around a chess game: The players, the clock, the move history, etc.
pub struct ChessGame {
    pub id: GameId,
    pub chess: Chess,

    pub _started: bool,
    pub white_player: Option<ClientId>,
    pub black_player: Option<ClientId>,
    pub spectators: Vec<ClientId>,

    pub _time: u32,
    pub _time_inc: u32,

    pub draw_offer_pending: bool,

    pub move_history: Vec<String>,
}

impl ChessGame {
    /// Starts a chess game.
    /// Chess game can only start when two players are joined.
    pub fn _start_game(&mut self) -> GameManagerResult<()> {
        if self.white_player.is_none() {
            return Err(GameManagerError::InvalidGameStatus(
                "White player missing".to_string(),
            ));
        }
        if self.black_player.is_none() {
            return Err(GameManagerError::InvalidGameStatus(
                "Black player missing".to_string(),
            ));
        }
        if self._started {
            return Err(GameManagerError::InvalidGameStatus(
                "Game already started".to_string(),
            ));
        }
        self._started = true;
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
        match side {
            UserRoleSelection::Black => {
                if self.black_player.is_some() {
                    Err(GameManagerError::InvalidGameStatus(
                        "Black side already taken".to_string(),
                    ))
                } else {
                    self.black_player = Some(client_id);
                    Ok(UserRoleSelection::Black)
                }
            }
            UserRoleSelection::White => {
                if self.white_player.is_some() {
                    Err(GameManagerError::InvalidGameStatus(
                        "White side already taken".to_string(),
                    ))
                } else {
                    self.white_player = Some(client_id);
                    Ok(UserRoleSelection::White)
                }
            }
            UserRoleSelection::Random => {
                if self.white_player.is_some() && self.black_player.is_some() {
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
                        if self.black_player.is_none() {
                            self.black_player = Some(client_id);
                            Ok(UserRoleSelection::Black)
                        } else {
                            self.white_player = Some(client_id);
                            Ok(UserRoleSelection::White)
                        }
                    }
                    true => {
                        if self.white_player.is_none() {
                            self.white_player = Some(client_id);
                            Ok(UserRoleSelection::White)
                        } else {
                            self.black_player = Some(client_id);
                            Ok(UserRoleSelection::Black)
                        }
                    }
                }
            }
            UserRoleSelection::Spectator => {
                self.spectators.push(client_id);
                Ok(UserRoleSelection::Spectator)
            }
            UserRoleSelection::Both => {
                if self.white_player.is_some() || self.black_player.is_some() {
                    return Err(GameManagerError::InvalidGameStatus(
                        "Cannot control both colors in non-empty games".to_string(),
                    ));
                }
                self.white_player = Some(client_id);
                self.black_player = Some(client_id);
                Ok(UserRoleSelection::Both)
            }
        }
    }

    pub fn remove_player(&mut self, client_id: ClientId) -> Option<UserRoleSelection> {
        let mut side = None;
        if let Some(id) = self.white_player {
            if id == client_id {
                side = Some(UserRoleSelection::White);
                self.white_player = None;
            }
        }
        if let Some(id) = self.black_player {
            if id == client_id {
                side = Some(UserRoleSelection::Black);
                self.black_player = None;
            }
        }
        side
    }

    /// Returns all players and spectators of a chess game.
    pub fn get_participants(&self) -> Vec<ClientId> {
        let mut participants = Vec::new();
        if let Some(id) = self.white_player {
            participants.push(id);
        }
        if let Some(id) = self.black_player {
            if !participants.contains(&id) {
                participants.push(id);
            }
        }
        for &id in &self.spectators {
            if !participants.contains(&id) {
                participants.push(id);
            }
        }
        participants
    }

    pub fn get_white(&self) -> Option<ClientId> {
        self.white_player
    }

    pub fn get_black(&self) -> Option<ClientId> {
        self.black_player
    }

    pub fn get_opponent(&self, cid: ClientId) -> Option<ClientId> {
        if self.white_player == Some(cid) {
            self.black_player
        } else if self.black_player == Some(cid) {
            self.white_player
        } else {
            None
        }
    }

    pub fn make_move(
        &mut self,
        mov: ChessMove,
        client_id: ClientId,
    ) -> ChessResult<Vec<(Tile, Option<Piece>)>> {
        let is_current_player = match self.chess.active_player {
            ChessColor::White => self.white_player == Some(client_id),
            ChessColor::Black => self.black_player == Some(client_id),
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
            return ChessGameState::Finished(GameOverReason::Checkmate(!self.chess.active_player));
        }
        if self.chess.is_stalemate() {
            return ChessGameState::Finished(GameOverReason::Stalemate);
        }
        if self.chess.is_fifty_moves_rule() {
            return ChessGameState::Finished(GameOverReason::FiftyMovesRule);
        }

        if self.chess.is_repetition() {
            return ChessGameState::Finished(GameOverReason::ThreefoldRepetition);
        }

        ChessGameState::Running
    }

    pub fn get_side(&self, client_id: ClientId) -> Option<ChessColor> {
        if self.white_player == Some(client_id) {
            Some(ChessColor::White)
        } else if self.black_player == Some(client_id) {
            Some(ChessColor::Black)
        } else {
            log::warn!("Client is not part of the game");
            None
        }
    }
}
