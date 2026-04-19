use bevy::prelude::Resource;
use chess_core::ClientId;

#[derive(Resource)]
pub struct ClientSession {
    pub name: String,
    pub id: Option<ClientId>,
}
