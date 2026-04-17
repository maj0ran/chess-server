use super::client::network::*;
use crate::client::config::*;
use bevy::prelude::*;
use lobby::LobbyState;
use session::ClientSession;

pub mod config;
pub mod game;
pub mod lobby;
pub mod network;
pub mod session;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        let config = Config::read("settings.cfg");
        let addr = config.server;
        app.insert_resource(NetTransport::new(addr));
        app.insert_resource(ClientSession { name: config.name });
        app.init_resource::<LobbyState>();
        app.add_systems(FixedUpdate, poll_network);
        app.insert_resource(Time::<Fixed>::from_hz(30.0)); // FixedUpdate tick-rate
        app.add_observer(send_message);
    }
}
