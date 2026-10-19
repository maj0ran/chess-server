use bevy::{
    prelude::*,
    app::{App, Plugin},
};

use crate::ui::Overlay;
use game_over::*;
use promotion::*;
use resign::*;

pub mod game_over;
pub mod promotion;
pub mod resign;

pub struct GamescreenDialogsPlugin;

impl Plugin for GamescreenDialogsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_game_over)
            .add_systems(
                OnEnter(Overlay::GameOver),
                (game_over_dialog_scene.spawn(), set_game_over_message).chain(),
            )
            .add_systems(OnExit(Overlay::GameOver), cleanup_game_over_dialog)
            .add_systems(
                OnEnter(Overlay::Promotion),
                (promotion_dialog_scene.spawn(), initialize_promotion_images).chain(),
            )
            .add_systems(OnEnter(Overlay::Resign), resign_dialog_scene.spawn());
    }
}
