pub mod network;
pub mod state;
pub mod ui;

use crate::network::poll_network;
use crate::state::{ClientState, MenuTab, Overlay, Screen};
use crate::ui::COLOR_DARK;
use ui::views::gameview::game::GamePlugin;
use ui::views::menuview::menuroot::MenuRootPlugin;

use bevy::prelude::*;
use bevy::ui_widgets::ScrollbarPlugin;
use bevy::window::WindowResolution;
use bevy_flair::prelude::*;

fn main() {
    env_logger::init();

    App::new()
        .add_plugins((
            // always needed
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Chess Client".to_string(),
                    resolution: WindowResolution::new(1280, 720),
                    ..default()
                }),
                ..default()
            }),
            //  ui::UiPlugin,
            // for scrollbars in UI elements (used for games list)
            ScrollbarPlugin,
            FlairPlugin,
        ))
        .insert_resource(ClientState::new())
        .init_state::<Screen>()
        .init_state::<MenuTab>()
        .init_state::<Overlay>()
        .add_systems(Startup, setup_camera)
        .add_systems(Update, poll_network)
        .add_plugins(GamePlugin)
        .add_plugins(MenuRootPlugin)
        .insert_resource(ClearColor(COLOR_DARK))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
