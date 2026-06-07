use crate::ui::homescreen::dialogs::create_game::{cleanup_create_dialog, create_dialog_scene, initialize_create_dialog};
use crate::ui::homescreen::dialogs::join_game::join_dialog_scene;
use crate::ui::{Dialog, Overlay};
use bevy::app::{App, Plugin};
use bevy::prelude::*;

pub mod create_game;
pub mod join_game;

pub struct HomescreenDialogsPlugin;

impl Plugin for HomescreenDialogsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                OnEnter(Overlay::CreateGame),
                (initialize_create_dialog, create_dialog_scene.spawn()).chain(),
            )
            .add_systems(OnExit(Overlay::CreateGame), cleanup_create_dialog)
            .add_systems(OnEnter(Overlay::JoinGame), join_dialog_scene.spawn());
    }
}
