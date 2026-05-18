pub mod client;
pub mod net;
pub mod ui;

use bevy::feathers::FeathersPlugins;
use bevy::prelude::*;
use bevy::window::{WindowResizeConstraints, WindowResolution};
use ui::COLOR_DARK;
use ui::theme::{MIN_WINDOW_HEIGHT, MIN_WINDOW_WIDTH};

use crate::ui::{Overlay, Screen};
use client::ClientPlugin;
use ui::views::gameview::chessboard::ChessboardPlugin;
use ui::views::gameview::game_screen::GameScreenPlugin;
use ui::views::menuview::main_menu::MenuRootPlugin;

fn main() {
    env_logger::init();

    App::new()
        // .insert_resource(DefaultErrorHandler(error_handler))
        .add_plugins((
            // always needed
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Chess Client".to_string(),
                    resolution: WindowResolution::new(1280, 800),
                    resize_constraints: WindowResizeConstraints {
                        min_width: MIN_WINDOW_WIDTH,
                        min_height: MIN_WINDOW_HEIGHT,
                        ..default()
                    },
                    ..default()
                }),
                ..default()
            }),
            //  ui::UiPlugin,
            // for scrollbars in UI elements (used for games list)
            //    ScrollbarPlugin,
            //     FlairPlugin,
        ))
        .init_state::<Screen>()
        .init_state::<Overlay>()
        .add_systems(Startup, setup_camera)
        .add_plugins((
            FeathersPlugins,
            ui::UiPlugin,
            MenuRootPlugin,
            GameScreenPlugin,
            ChessboardPlugin,
            ClientPlugin,
        ))
        .insert_resource(ClearColor(COLOR_DARK))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
