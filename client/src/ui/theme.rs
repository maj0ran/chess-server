use bevy::color::{Alpha, Color};
use bevy::feathers::{
    dark_theme::create_dark_theme,
    theme::{SemanticToken, UiTheme},
    tokens,
};
use bevy::prelude::*;

/// A soft, low-saturation wooden palette used by the client UI.
pub const THEME_BACKGROUND: Color = Color::srgb(0.18, 0.13, 0.09);
pub const THEME_SURFACE: Color = Color::srgb(0.29, 0.22, 0.16);
pub const THEME_SURFACE_LIGHT: Color = Color::srgb(0.38, 0.30, 0.22);
pub const THEME_BUTTON: Color = Color::srgb(0.34, 0.27, 0.20);
pub const THEME_PENDING_DRAW_OFFER: Color = Color::srgb(0.32, 0.52, 0.34);
pub const THEME_BORDER: Color = Color::srgb(0.55, 0.43, 0.29);
pub const THEME_GOLD: Color = Color::srgb(0.68, 0.54, 0.32);
pub const THEME_GOLD_LIGHT: Color = Color::srgb(0.82, 0.70, 0.48);
pub const THEME_TEXT: Color = Color::srgb(0.93, 0.87, 0.74);
pub const THEME_TEXT_DIM: Color = Color::srgb(0.76, 0.68, 0.55);
pub const THEME_OVERLAY: Color = Color::srgba(0.04, 0.02, 0.01, 0.78);

pub const LEFT_PANEL_RATIO: f32 = 0.10;
pub const RIGHT_PANEL_RATIO: f32 = 0.25;
pub const BOARD_AREA_RATIO: f32 = 1.0 - LEFT_PANEL_RATIO - RIGHT_PANEL_RATIO;
pub const MIN_WINDOW_WIDTH: f32 = 640.0;
pub const MIN_WINDOW_HEIGHT: f32 = 480.0;
pub const PLAYER_LABEL_HEIGHT: f32 = 48.0;
pub const PLAYER_LABEL_OFFSET: f32 = 16.0;
pub const BOARD_EDGE_GAP: f32 = 12.0;

// default shadow for UI elements
pub fn default_ui_shadow() -> BoxShadow {
    BoxShadow::new(
        Color::srgba(0.0, 0.0, 0.0, 0.6),
        px(0),
        px(6),
        px(2),
        px(10),
    )
}

pub fn default_dialog_node() -> Node {
    Node {
        width: px(480),
        padding: UiRect::all(px(24)),
        flex_direction: FlexDirection::Column,
        row_gap: px(16),
        align_items: AlignItems::Center,
        border: UiRect::all(px(2)),
        border_radius: BorderRadius::all(px(10)),
        ..Default::default()
    }
}

pub fn default_button_row() -> Node {
    Node { width: percent(100),
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::Center,
        column_gap: px(12),
        ..Default::default()
    }
}

/// Default Button Nodes. I do it like this to have standardized button sizes
/// and consistency throughout the UI design.
pub fn default_button_size_180() -> Node {
    default_button(180, 50)
}
pub fn default_button_size_150() -> Node {
    default_button(150, 50)
}
pub fn default_button_size_120() -> Node {
    default_button(120, 50)
}

pub fn default_button_icon() -> Node {
    default_button(100, 80)
}

fn default_button(width: usize, height: usize) -> Node {
    Node {
        width: px(width),
        height: px(height),
        justify_content: JustifyContent::Center, // horizontal alignment
        align_items: AlignItems::Center, // vertical alignment ... the fuck is up with these CSS property names?!
        border_radius: BorderRadius::all(px(8)),
        ..Default::default()
    }
}

/// Feathers only uses the theme for design-token colors. Layout, borders, and shadows remain
/// native Bevy UI components and are configured by the individual scenes.
pub fn wood_theme() -> UiTheme {
    let mut theme = create_dark_theme();

    let overrides = [
        (tokens::WINDOW_BG, THEME_BACKGROUND),
        (tokens::TEXT_MAIN, THEME_TEXT),
        (tokens::TEXT_DIM, THEME_TEXT_DIM),
        (tokens::BUTTON_BG, THEME_BUTTON),
        (tokens::BUTTON_BG_HOVER, THEME_SURFACE_LIGHT),
        (tokens::BUTTON_BG_PRESSED, THEME_GOLD),
        (tokens::BUTTON_BG_DISABLED, THEME_SURFACE),
        (tokens::BUTTON_TEXT, THEME_TEXT),
        (
            tokens::BUTTON_TEXT_DISABLED,
            THEME_TEXT_DIM.with_alpha(0.5),
        ),
        (tokens::BUTTON_PRIMARY_BG, THEME_GOLD),
        (tokens::BUTTON_PRIMARY_BG_HOVER, THEME_GOLD_LIGHT),
        (tokens::BUTTON_PRIMARY_BG_PRESSED, THEME_BORDER),
        (tokens::BUTTON_PRIMARY_BG_DISABLED, THEME_SURFACE),
        (tokens::BUTTON_PRIMARY_TEXT, THEME_BACKGROUND),
        (
            tokens::BUTTON_PRIMARY_TEXT_DISABLED,
            THEME_BACKGROUND.with_alpha(0.5),
        ),
        (tokens::BUTTON_PLAIN_BG, Color::NONE),
        (tokens::BUTTON_PLAIN_BG_HOVER, THEME_SURFACE),
        (tokens::BUTTON_PLAIN_BG_PRESSED, THEME_SURFACE_LIGHT),
        (tokens::BUTTON_PLAIN_BG_DISABLED, Color::NONE),
        (tokens::SCROLLBAR_BG, THEME_BACKGROUND),
        (tokens::SCROLLBAR_THUMB, THEME_GOLD),
        (tokens::SCROLLBAR_THUMB_HOVER, THEME_GOLD_LIGHT),
        (tokens::FOCUS_RING, THEME_GOLD_LIGHT.with_alpha(0.8)),
    ];

    for (token, color) in overrides {
        let sem = SemanticToken::new(token.to_string().into());
        theme.token_assignments.insert(token, sem.clone());
        theme.semantic_base.insert(sem, color);
    }

    UiTheme(theme)
}
