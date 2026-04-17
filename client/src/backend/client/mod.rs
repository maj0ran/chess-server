use bevy::prelude::Resource;

pub mod game;
pub mod lobby;

#[derive(Resource)]
pub struct ClientSession {
    pub name: String,
}
