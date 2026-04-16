pub mod backend;
pub mod ui;

use bevy::prelude::*;
use bevy::ui_widgets::ScrollbarPlugin;
use bevy::window::{WindowResized, WindowResolution};
use bevy_flair::prelude::*;

use backend::client::ClientBackend;
use ui::COLOR_DARK;

use crate::ui::{MenuTab, Overlay, Screen};
use backend::ClientPlugin;
use ui::views::gameview::chessboard::ChessboardPlugin;
use ui::views::gameview::game_screen::GameScreenPlugin;
use ui::views::menuview::menuroot::MenuRootPlugin;

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
        win_res.width = e.width;
        win_res.height = e.height;
    }
}

fn main() {
    env_logger::init();

    let config = backend::config::Config::read("settings.cfg");

    App::new()
        .add_plugins((
            // always needed
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Chess Client".to_string(),
                    resolution: WindowResolution::new(1280, 800),
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
        .add_systems(Update, on_resize_system)
        .add_plugins((
            MenuRootPlugin,
            GameScreenPlugin,
            ChessboardPlugin,
            ClientPlugin,
        ))
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
