use bevy::prelude::*;

pub mod config;
pub mod game;
pub mod lobby;
pub mod session;
pub mod net;

pub use config::Config;
pub use game::{
    ActiveGame, BoardUpdate, ClientDetails, ClientDetailsEvent, GameDetails, GameJoined,
    GameJoinedEvent, GameOverEvent,
};
pub use lobby::LobbyState;
pub use session::ClientSession;
use crate::backend::net::{poll_network, NetTransport, send_message};

pub struct ClientBackendPlugin;

impl Plugin for ClientBackendPlugin {
    fn build(&self, app: &mut App) {
        let config = Config::read("settings.cfg");
        let addr = config.server;
        app.insert_resource(NetTransport::new(addr));
        app.insert_resource(ClientSession {
            name: config.name,
            id: None,
        });
        app.init_resource::<LobbyState>();
        app.add_systems(FixedUpdate, poll_network);
        app.insert_resource(Time::<Fixed>::from_hz(30.0)); // FixedUpdate tick-rate
        app.add_observer(send_message);
    }
}
