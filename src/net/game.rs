use crate::game::Chess;
use crate::net::*;

pub struct OnlineGame {
    pub id: GameId,
    pub chess: Chess,
    pub started: bool,
    pub white_player: Option<ClientId>,
    pub black_player: Option<ClientId>,
    pub spectators: Vec<ClientId>,
}

impl OnlineGame {
    pub fn new(id: GameId) -> OnlineGame {
        OnlineGame {
            id: 1,
            chess: Chess::new(),
            started: false,
            white_player: None,
            black_player: None,
            spectators: vec![],
        }
    }

    pub fn start_game(&mut self) -> bool {
        if self.white_player.is_some() && self.black_player.is_some() && !self.started {
            self.started = true;
            true
        } else {
            false
        }
    }

    pub fn add_player(&mut self, player_id: ClientId) -> Result<()> {
        if self.white_player.is_none() {
            self.white_player = Some(player_id);
            Ok(())
        } else if self.black_player.is_none() && self.white_player != Some(player_id) {
            self.black_player = Some(player_id);
            Ok(())
        } else if self.white_player == Some(player_id) || self.black_player == Some(player_id) {
            Err(ServerError::InvalidCommand("Already in game".to_string()))
        } else {
            Err(ServerError::InvalidCommand("Game is full".to_string()))
        }
    }

    pub fn get_participants(&self) -> impl Iterator<Item = ClientId> + '_ {
        self.white_player
            .iter()
            .chain(self.black_player.iter())
            .chain(self.spectators.iter())
            .copied()
    }
}
