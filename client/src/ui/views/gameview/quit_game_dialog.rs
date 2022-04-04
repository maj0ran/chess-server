use crate::state::{ClientState, Overlay, Screen};
use crate::ui::ButtonColors;
use crate::{btn_green, btn_red, spawn_dialog};
use crate::{spawn_button, spawn_label};
use bevy::prelude::*;
use chess_core::ClientMessage;

/// Marker that indicates a dialog for quitting the current game
#[derive(Component)]
pub struct QuitGameDialog;

/// Available actions for the dialog
#[derive(Component)]
pub enum QuitGameAction {
    Confirm,
    Cancel,
}

pub fn setup_quit_game_dialog(mut commands: Commands) {
    spawn_dialog!(
        commands,
        QuitGameDialog,
        Val::Px(300.0),
        Val::Px(250.0),
        |p| {
            spawn_label!(p, "Are you sure?", 30.0, Color::WHITE);
            spawn_button!(p, "Confirm", QuitGameAction::Confirm, btn_green!());
            spawn_button!(p, "Cancel", QuitGameAction::Cancel, btn_red!());
        }
    );
}

pub fn cleanup_quit_game_dialog(
    mut commands: Commands,
    query: Query<Entity, With<QuitGameDialog>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn quit_game_dialog_action_system(
    mut interaction_query: Query<
        (&Interaction, &QuitGameAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_overlay: ResMut<NextState<Overlay>>,
    mut next_screen: ResMut<NextState<Screen>>,
    client: ResMut<ClientState>,
) {
    for (interaction, action) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            match action {
                QuitGameAction::Confirm => {
                    client.network.send(ClientMessage::LeaveGame(
                        client.menu_state.selected_game.unwrap(),
                    ));
                    next_screen.set(Screen::Menu);
                    next_overlay.set(Overlay::None);
                }
                QuitGameAction::Cancel => {
                    next_overlay.set(Overlay::None);
                }
            }
        }
    }
}
