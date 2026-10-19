use bevy::{
    app::{App, Plugin},
    prelude::*,
};

use join_game::join_dialog_scene;
use create_game::*;
use crate::ui::Overlay;

pub mod create_game;
pub mod join_game;

pub struct HomescreenDialogsPlugin;

impl Plugin for HomescreenDialogsPlugin {
    fn build(&self, app: &mut App) {
        app
            // Create game dialog
            .add_systems(
                OnEnter(Overlay::CreateGame),
                (initialize_create_dialog, create_dialog_scene.spawn()).chain(),
            )
            // Join game dialog
            .add_systems(
                OnEnter(Overlay::JoinGame),
                join_dialog_scene.spawn());
    }
}
