use bevy::prelude::*;

pub mod views;
pub use views::menuview::gamemenu::gamelist_menu;

pub const COLOR_DARK: Color = Color::srgb(0.125, 0.125, 0.125);
pub const COLOR_DARK2: Color = Color::srgb(0.325, 0.282, 0.255);
pub const COLOR_LIGHT: Color = Color::srgb(0.867, 0.863, 0.608);
pub const COLOR_LIGHT2: Color = Color::srgb(0.722, 0.647, 0.443);
pub const COLOR_MID: Color = Color::srgb(0.533, 0.451, 0.349);

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum Screen {
    #[default]
    Menu,
    Game,
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum MenuTab {
    #[default]
    None,
    Games,
    Analysis,
    Puzzle,
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum Overlay {
    #[default]
    None,
    CreateDialog,
    JoinDialog,
    QuitGameDialog,
    GameOver,
    Promotion,
}
