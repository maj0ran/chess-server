pub mod create_game;
pub mod game_over;
pub mod join_game;
pub mod promotion;
pub mod resign;

use bevy::prelude::*;

use crate::ui::{Dialog, Overlay};
use create_game::{cleanup_create_dialog, create_dialog_scene, initialize_create_dialog};
use game_over::{
    cleanup_game_over_dialog, game_over_dialog_scene, on_game_over, set_game_over_message,
};
use join_game::join_dialog_scene;
use promotion::{initialize_promotion_images, promotion_dialog_scene};
use resign::resign_dialog_scene;

pub struct DialogsPlugin;

impl Plugin for DialogsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_game_over)
            .add_systems(
                OnEnter(Overlay::CreateGame),
                (initialize_create_dialog, create_dialog_scene.spawn()).chain(),
            )
            .add_systems(OnExit(Overlay::CreateGame), cleanup_create_dialog)
            .add_systems(OnEnter(Overlay::JoinGame), join_dialog_scene.spawn())
            .add_systems(OnExit(Overlay::JoinGame), despawn_dialog)
            .add_systems(
                OnEnter(Overlay::GameOver),
                (game_over_dialog_scene.spawn(), set_game_over_message).chain(),
            )
            .add_systems(OnExit(Overlay::GameOver), cleanup_game_over_dialog)
            .add_systems(
                OnEnter(Overlay::Promotion),
                (promotion_dialog_scene.spawn(), initialize_promotion_images).chain(),
            )
            .add_systems(OnExit(Overlay::Promotion), despawn_dialog)
            .add_systems(OnEnter(Overlay::Resign), resign_dialog_scene.spawn())
            .add_systems(OnExit(Overlay::Resign), despawn_dialog);
    }
}

fn despawn_dialog(mut commands: Commands, dialogs: Query<Entity, With<Dialog>>) {
    for entity in &dialogs {
        commands.entity(entity).despawn();
    }
}
