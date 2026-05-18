use bevy::prelude::*;

use crate::ui::theme::WOOD_OVERLAY;

pub mod theme;
pub mod views;

pub const COLOR_DARK: Color = Color::srgb(0.125, 0.125, 0.125);
pub const COLOR_LIGHT2: Color = Color::srgb(0.722, 0.647, 0.443);
pub const COLOR_MID: Color = Color::srgb(0.533, 0.451, 0.349);

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
    CreateDialog,
    JoinDialog,
    ResignDialog,
    GameOver,
    Promotion,
}

#[derive(Component, Clone, Copy, Default)]
pub struct Dialog;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, setup_dialogs);
    }
}

fn setup_dialogs(mut commands: Commands, dialogs: Query<Entity, Added<Dialog>>) {
    for entity in dialogs.iter() {
        commands.entity(entity).insert((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(WOOD_OVERLAY),
        ));
    }
}
