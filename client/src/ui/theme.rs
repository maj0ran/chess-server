use bevy::color::{Alpha, Color};
use bevy::feathers::{dark_theme::create_dark_theme, theme::UiTheme};

/// A soft, low-saturation wooden palette used by the client UI.
pub const WOOD_BACKGROUND: Color = Color::srgb(0.18, 0.13, 0.09);
pub const WOOD_SURFACE: Color = Color::srgb(0.29, 0.22, 0.16);
pub const WOOD_SURFACE_LIGHT: Color = Color::srgb(0.38, 0.30, 0.22);
pub const WOOD_BUTTON: Color = Color::srgb(0.34, 0.27, 0.20);
pub const WOOD_DRAW_OFFER: Color = Color::srgb(0.32, 0.52, 0.34);
pub const WOOD_BORDER: Color = Color::srgb(0.55, 0.43, 0.29);
pub const WOOD_GOLD: Color = Color::srgb(0.68, 0.54, 0.32);
pub const WOOD_GOLD_LIGHT: Color = Color::srgb(0.82, 0.70, 0.48);
pub const WOOD_TEXT: Color = Color::srgb(0.93, 0.87, 0.74);
pub const WOOD_TEXT_DIM: Color = Color::srgb(0.76, 0.68, 0.55);
pub const WOOD_OVERLAY: Color = Color::srgba(0.04, 0.02, 0.01, 0.78);
pub const LEFT_PANEL_RATIO: f32 = 0.10;
pub const RIGHT_PANEL_RATIO: f32 = 0.25;
pub const BOARD_AREA_RATIO: f32 = 1.0 - LEFT_PANEL_RATIO - RIGHT_PANEL_RATIO;
pub const MIN_WINDOW_WIDTH: f32 = 640.0;
pub const MIN_WINDOW_HEIGHT: f32 = 480.0;
pub const PLAYER_LABEL_HEIGHT: f32 = 48.0;
pub const PLAYER_LABEL_OFFSET: f32 = 16.0;
pub const BOARD_EDGE_GAP: f32 = 12.0;

/// Feathers only uses the theme for design-token colors. Layout, borders, and shadows remain
/// native Bevy UI components and are configured by the individual scenes.
pub fn wood_theme() -> UiTheme {
    let mut theme = UiTheme(create_dark_theme());

    theme.set_color("feathers.window.bg", WOOD_BACKGROUND);
    theme.set_color("feathers.text.main", WOOD_TEXT);
    theme.set_color("feathers.text.dim", WOOD_TEXT_DIM);

    theme.set_color("feathers.button.bg", WOOD_BUTTON);
    theme.set_color("feathers.button.bg.hover", WOOD_SURFACE_LIGHT);
    theme.set_color("feathers.button.bg.pressed", WOOD_GOLD);
    theme.set_color("feathers.button.bg.disabled", WOOD_SURFACE);
    theme.set_color("feathers.button.txt", WOOD_TEXT);
    theme.set_color(
        "feathers.button.txt.disabled",
        WOOD_TEXT_DIM.with_alpha(0.5),
    );

    theme.set_color("feathers.button.primary.bg", WOOD_GOLD);
    theme.set_color("feathers.button.primary.bg.hover", WOOD_GOLD_LIGHT);
    theme.set_color("feathers.button.primary.bg.pressed", WOOD_BORDER);
    theme.set_color("feathers.button.primary.bg.disabled", WOOD_SURFACE);
    theme.set_color("feathers.button.primary.txt", WOOD_BACKGROUND);
    theme.set_color(
        "feathers.button.primary.txt.disabled",
        WOOD_BACKGROUND.with_alpha(0.5),
    );

    theme.set_color("feathers.button.plain.bg", Color::NONE);
    theme.set_color("feathers.button.plain.bg.hover", WOOD_SURFACE);
    theme.set_color("feathers.button.plain.bg.pressed", WOOD_SURFACE_LIGHT);
    theme.set_color("feathers.button.plain.bg.disabled", Color::NONE);

    theme.set_color("feathers.scrollbar.bg", WOOD_BACKGROUND);
    theme.set_color("feathers.scrollbar.thumb", WOOD_GOLD);
    theme.set_color("feathers.scrollbar.thumb.hover", WOOD_GOLD_LIGHT);

    theme.set_color("feathers.focus", WOOD_GOLD_LIGHT.with_alpha(0.8));
    theme
}
