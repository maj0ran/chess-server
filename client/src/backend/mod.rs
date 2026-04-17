use crate::backend::client::{ClientConfig, LobbyState};
use crate::backend::config::Config;
use crate::backend::network::{
    NetTransport, handle_client_requests, on_move_request, poll_network,
};
use bevy::app::{App, Plugin};
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
        app.add_observer(on_move_request);
        app.add_observer(handle_client_requests);
    }
}
