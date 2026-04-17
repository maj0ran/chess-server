use crate::backend::network::{on_move_request, poll_network};
use bevy::app::{App, Plugin};
use bevy::prelude::*;

pub mod client;
pub mod config;
pub mod network;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, poll_network);
        app.insert_resource(Time::<Fixed>::from_hz(30.0)); // FixedUpdate tick-rate
        app.add_observer(on_move_request);
    }
}
