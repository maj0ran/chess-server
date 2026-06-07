use bevy::app::{App, Plugin};
use bevy::prelude::*;
use crate::ui::gamescreen::dialogs::game_over::{cleanup_game_over_dialog, game_over_dialog_scene, on_game_over, set_game_over_message};
use crate::ui::gamescreen::dialogs::promotion::{initialize_promotion_images, promotion_dialog_scene};
use crate::ui::gamescreen::dialogs::resign::resign_dialog_scene;
use crate::ui::{Dialog, Overlay};

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
