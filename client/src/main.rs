pub mod backend;
pub mod ui;

use bevy::feathers::FeathersPlugins;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::window::{WindowResizeConstraints, WindowResolution};

use crate::backend::ClientBackendPlugin;
use crate::ui::theme::{MIN_WINDOW_HEIGHT, MIN_WINDOW_WIDTH};
use crate::ui::{COLOR_DARK, Overlay, Screen, UiPlugin};

fn main() {
    // A convenient way to set logging messages is
    // RUST_LOG=chess=debug,bevy=warn
    // giving all debug messages from our chess program and only warnings/errors from bevy
    // in case we mis-use the engine.
    env_logger::init();

    App::new()
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
                // Disable bevy log plugin since we are using env_logger manually.
                // Note that we still log bevy messages this way.
            }).build().disable::<LogPlugin>(),
        ))
        .init_state::<Screen>()
        .init_state::<Overlay>()
        .add_systems(Startup, setup_camera)
        .add_plugins((
            FeathersPlugins,
            ClientBackendPlugin,
            UiPlugin,
        ))
        .insert_resource(ClearColor(COLOR_DARK))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
