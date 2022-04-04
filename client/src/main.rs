pub mod network;
pub mod state;
pub mod ui;

use crate::network::poll_network;
use crate::state::{ClientState, Overlay, Screen};
use crate::ui::menu::MenuPlugin;
use bevy::prelude::*;
use bevy::ui_widgets::ScrollbarPlugin;
use bevy::window::WindowResolution;
use ui::views::gameview::game::GamePlugin;

fn main() {
    env_logger::init();

    App::new()
        .add_plugins((
            // always needed
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Chess Client".to_string(),
                        resolution: WindowResolution::new(1280, 720),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: "res".to_string(),
                    ..default()
                }),
            // for scrollbars in UI elements (used for games list)
            ScrollbarPlugin,
        ))
        .init_state::<Screen>()
        .init_state::<Overlay>()
        .insert_resource(ClientState::new())
        .add_systems(Startup, setup_camera)
        .add_systems(Update, (poll_network, ui::button_system))
        .add_plugins(GamePlugin)
        .add_plugins(MenuPlugin)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
