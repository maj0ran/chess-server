use crate::state::{ClientState, Overlay};
use crate::ui::ButtonColors;
use crate::{btn_green, btn_red, spawn_dialog};
use crate::{spawn_button, spawn_label};
use bevy::prelude::*;
use chess_core::{ClientMessage, NewGameParams};

#[derive(Component)]
pub struct CreateDialogComponent;

pub fn setup_create_dialog(mut commands: Commands) {
    spawn_dialog!(
        commands,
        CreateDialogComponent,
        Val::Percent(30.0),
        Val::Percent(80.0),
        |p| {
            spawn_label!(p, "Create Game", 30.0, Color::WHITE);
            spawn_button!(p, "Confirm", CreateAction::Confirm, btn_green!());
            spawn_button!(p, "Cancel", CreateAction::Cancel, btn_red!());
        }
    );
}

#[derive(Component)]
pub enum CreateAction {
    Confirm,
    Cancel,
}

pub fn cleanup_create_dialog(
    mut commands: Commands,
    query: Query<Entity, With<CreateDialogComponent>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn create_dialog_action_system(
    mut interaction_query: Query<
        (&Interaction, &CreateAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<ClientState>,
    mut next_overlay: ResMut<NextState<Overlay>>,
) {
    for (interaction, action) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            match action {
                CreateAction::Confirm => {
                    state.menu_state.is_loading = true;
                    state.network.send(ClientMessage::NewGame(NewGameParams {
                        mode: 0,
                        time: state.create_settings.time as u32 * 60,
                        time_inc: state.create_settings.increment as u32,
                    }));
                    next_overlay.set(Overlay::None);
                }
                CreateAction::Cancel => {
                    next_overlay.set(Overlay::None);
                }
            }
        }
    }
}
