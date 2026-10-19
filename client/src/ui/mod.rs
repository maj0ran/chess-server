pub mod gamescreen;
pub mod homescreen;
pub mod theme;

use bevy::prelude::*;

use crate::ui::theme::THEME_OVERLAY;
use gamescreen::ChessGamePlugin;
use homescreen::HomescreenPlugin;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum Screen {
    #[default]
    Menu,
    Game,
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum Overlay {
    #[default]
    None,
    CreateGame,
    JoinGame,
    Resign,
    GameOver,
    Promotion,
}

#[derive(Component, Clone, Copy, Default)]
pub struct Dialog;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                HomescreenPlugin,
                ChessGamePlugin,
            ))
            .add_systems(Update, setup_dialogs)
            .add_systems(OnEnter(Overlay::None), despawn_dialog);
    }
}

/// When a dialog is spawned, this system will add a slightly opaque, dark overlay to the
/// whole screen. This will first, darken the background behind the dialog and second, disable
/// any clicking on the screen behind the dialog.
fn setup_dialogs(mut commands: Commands, dialog: Single<Entity, Added<Dialog>>) {
    let entity = dialog.entity();
        commands.entity(entity).insert((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(THEME_OVERLAY),
        ));
}

fn despawn_dialog(mut commands: Commands, dialogs: Query<Entity, With<Dialog>>) {
    for entity in &dialogs {
        commands.entity(entity).despawn();
    }
}
