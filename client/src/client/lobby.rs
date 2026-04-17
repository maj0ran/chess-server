use crate::client::game::GameDetails;
use bevy::prelude::Resource;
use chess_core::{ClientId, GameId};
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct LobbyState {
    games: HashMap<GameId, GameDetails>,
    clients: HashMap<ClientId, String>,
    pub pending_join_game: Option<GameId>,
}

impl LobbyState {
    pub fn get_games(&self) -> &HashMap<GameId, GameDetails> {
        &self.games
    }

    pub fn get_game_info(&self, gid: GameId) -> Option<&GameDetails> {
        self.games.get(&gid)
    }

    pub fn update_game_info(&mut self, gid: GameId, details: GameDetails) {
        self.games.insert(gid, details);
    }

    pub fn update_client_info(&mut self, cid: usize, name: String) {
        self.clients.insert(cid, name);
    }

    pub fn get_client_info(&self, cid: usize) -> Option<&String> {
        self.clients.get(&cid)
    }

    pub fn has_client_info(&self, cid: ClientId) -> bool {
        self.clients.contains_key(&cid)
    }
}
