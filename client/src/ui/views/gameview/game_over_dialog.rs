use crate::state::{ClientState, Overlay, Screen};
use crate::ui::ButtonColors;
use crate::ui::*;
use crate::{spawn_button, spawn_dialog, spawn_label};
use bevy::prelude::*;
#[derive(Component)]
pub struct GameOverDialogComponent;

pub fn setup_game_over_dialog(mut commands: Commands, state: ResMut<ClientState>) {
    let winner = state.game_state.winner.unwrap();
    spawn_dialog!(
        commands,
        GameOverDialogComponent,
        Val::Px(300.0),
        Val::Px(200.0),
        |p| {
            spawn_label!(p, "Game Over", 30.0, COLOR_LIGHT);
            spawn_label!(p, format!("{} won", winner), 20.0, COLOR_LIGHT);
            spawn_button!(p, "OK", GameOverAction::Ok);
        }
    );
}

#[derive(Component)]
pub enum GameOverAction {
    Ok,
}

pub fn cleanup_game_over_dialog(
    mut commands: Commands,
    query: Query<Entity, With<GameOverDialogComponent>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn game_over_dialog_action_system(
    mut interaction_query: Query<
        (&Interaction, &GameOverAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_overlay: ResMut<NextState<Overlay>>,
) {
    for (interaction, action) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            match action {
                GameOverAction::Ok => {
                    next_screen.set(Screen::Menu);
                    next_overlay.set(Overlay::None);
                }
            }
        }
    }
}
