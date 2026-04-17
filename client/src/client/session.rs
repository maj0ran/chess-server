use bevy::prelude::Resource;

#[derive(Resource)]
pub struct ClientSession {
    pub name: String,
}
