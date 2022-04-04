use crate::state::{ClientState, Overlay};
use crate::ui::ButtonColors;
use crate::{btn_red, spawn_dialog};
use crate::{spawn_button, spawn_label};
use bevy::prelude::*;
use chess_core::{ClientMessage, JoinGameParams, UserRoleSelection};

#[derive(Component)]
pub struct JoinDialogComponent;

pub fn setup_join_dialog(mut commands: Commands) {
    spawn_dialog!(
        commands,
        JoinDialogComponent,
        Val::Px(300.0),
        Val::Px(350.0),
        |p| {
            spawn_label!(p, "Select Side", 30.0, Color::WHITE);
            spawn_button!(p, "White", JoinAction::Select(UserRoleSelection::White));
            spawn_button!(p, "Black", JoinAction::Select(UserRoleSelection::Black));
            spawn_button!(p, "Random", JoinAction::Select(UserRoleSelection::Random));
            spawn_button!(p, "Both", JoinAction::Select(UserRoleSelection::Both));

            spawn_button!(p, "Cancel", JoinAction::Cancel, btn_red!());
        }
    );
}

#[derive(Component)]
pub enum JoinAction {
    Select(UserRoleSelection),
    Cancel,
}

pub fn cleanup_join_dialog(
    mut commands: Commands,
    query: Query<Entity, With<JoinDialogComponent>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn join_dialog_action_system(
    mut interaction_query: Query<(&Interaction, &JoinAction), (Changed<Interaction>, With<Button>)>,
    mut state: ResMut<ClientState>,
    mut next_overlay: ResMut<NextState<Overlay>>,
) {
    for (interaction, action) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            match action {
                JoinAction::Select(side) => {
                    if let Some(game_id) = state.menu_state.selected_game {
                        state.menu_state.is_loading = true;
                        state.network.send(ClientMessage::JoinGame(JoinGameParams {
                            game_id,
                            side: side.clone(),
                        }));
                    }
                    next_overlay.set(Overlay::None);
                }
                JoinAction::Cancel => {
                    next_overlay.set(Overlay::None);
                }
            }
        }
    }
}
