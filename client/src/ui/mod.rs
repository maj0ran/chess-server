use bevy::prelude::*;

#[macro_use]
pub mod macros;
pub mod views;

pub use views::menuview::create_game_dialog;
pub use views::menuview::join_game_dialog;
pub use views::menuview::menu;

pub const fn from_rgb_u32(value: u32) -> Color {
    let r = ((value >> 16) & 0xFF) as f32 / 255.0;
    let g = ((value >> 8) & 0xFF) as f32 / 255.0;
    let b = (value & 0xFF) as f32 / 255.0;
    Color::srgb(r, g, b)
}

pub const COLOR_DARK: Color = from_rgb_u32(0x202020);
pub const COLOR_DARK2: Color = from_rgb_u32(0x534841);
pub const COLOR_LIGHT: Color = from_rgb_u32(0xDDDC9B);
pub const COLOR_LIGHT2: Color = from_rgb_u32(0xB8A571);
pub const COLOR_MID: Color = from_rgb_u32(0x887359);

#[derive(Component, Clone, Copy, Debug)]
pub struct ButtonColors {
    pub normal: Color,
    pub hovered: Color,
    pub pressed: Color,
}

impl ButtonColors {
    pub fn red() -> Self {
        ButtonColors {
            normal: Color::srgb(0.5, 0.2, 0.2),
            hovered: Color::srgb(0.6, 0.3, 0.3),
            pressed: Color::srgb(0.2, 0.2, 0.2),
        }
    }
    pub fn green() -> Self {
        ButtonColors {
            normal: Color::srgb(0.2, 0.5, 0.2),
            hovered: Color::srgb(0.3, 0.6, 0.3),
            pressed: Color::srgb(0.2, 0.2, 0.2),
        }
    }
}

impl Default for ButtonColors {
    fn default() -> Self {
        ButtonColors {
            normal: COLOR_LIGHT2,
            hovered: COLOR_LIGHT,
            pressed: Color::srgb(0.2, 0.2, 0.2),
        }
    }
}

pub fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &ButtonColors),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, button_colors) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = button_colors.pressed.into();
            }
            Interaction::Hovered => {
                *color = button_colors.hovered.into();
            }
            Interaction::None => {
                *color = button_colors.normal.into();
            }
        }
    }
}
