use super::backend::network::*;
use crate::backend::client::*;
use crate::backend::config::*;
use bevy::prelude::*;

pub mod client;
pub mod config;
pub mod network;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        let config = Config::read("settings.cfg");
        app.insert_resource(NetTransport::with_config(config.clone()));
        app.insert_resource(ClientConfig { name: config.name });
        app.init_resource::<LobbyState>();

        app.add_systems(FixedUpdate, poll_network);
        app.insert_resource(Time::<Fixed>::from_hz(30.0)); // FixedUpdate tick-rate
        app.add_observer(send_message);
    }
}
