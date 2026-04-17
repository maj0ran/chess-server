use crate::backend::client::game::GameDetails;
use bevy::prelude::Resource;
use chess_core::GameId;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct LobbyState {
    pub games: HashMap<GameId, Option<GameDetails>>,
    pub client_names: HashMap<usize, String>,
    pub pending_join_game: Option<GameId>,
}
