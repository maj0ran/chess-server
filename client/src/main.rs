mod config;
pub mod network;
pub mod state;
pub mod ui;

use crate::network::poll_network;
use crate::state::{ClientBackend, MenuTab, Overlay, Screen};
use crate::ui::COLOR_DARK;
use ui::views::gameview::game::GamePlugin;
use ui::views::menuview::menuroot::MenuRootPlugin;

use bevy::prelude::*;
use bevy::ui_widgets::ScrollbarPlugin;
use bevy::window::{WindowResized, WindowResolution};
use bevy_flair::prelude::*;

#[derive(Resource)]
pub struct WindowSize {
    pub width: f32,
    pub height: f32,
}

/// Window resize event handler
fn on_resize_system(
    mut resize_reader: MessageReader<WindowResized>,
    mut win_res: ResMut<WindowSize>,
) {
    for e in resize_reader.read() {
        let text = format!("{:.1} x {:.1}", e.width, e.height);
        log::warn!("Window resized: {}", text);
        win_res.width = e.width;
        win_res.height = e.height;
    }
}

fn main() {
    env_logger::init();

    let config = crate::config::Config::read("settings.cfg");

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
        .insert_resource(ClientBackend::with_config(config))
        .init_state::<Screen>()
        .init_state::<MenuTab>()
        .init_state::<Overlay>()
        .add_systems(Startup, setup_camera)
        .add_systems(Update, poll_network)
        .add_systems(Update, on_resize_system)
        .add_plugins(GamePlugin)
        .add_plugins(MenuRootPlugin)
        .insert_resource(ClearColor(COLOR_DARK))
        .insert_resource(WindowSize {
            width: 0.0,
            height: 0.0,
        })
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
