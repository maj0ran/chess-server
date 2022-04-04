use bevy::prelude::*;

pub mod macros;
pub mod views;

pub use views::gameview::game;
pub use views::gameview::game_over_dialog;
pub use views::menuview::create_game_dialog;
pub use views::menuview::join_game_dialog;
pub use views::menuview::menu;

#[derive(Component)]
pub struct ButtonColors {
    pub normal: Color,
    pub hovered: Color,
    pub pressed: Color,
}

impl Default for ButtonColors {
    fn default() -> Self {
        ButtonColors {
            normal: Color::srgb(0.3, 0.3, 0.3),
            hovered: Color::srgb(0.4, 0.4, 0.4),
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
