use crate::state::{ClientState, Overlay};
use crate::ui::ButtonColors;
use crate::ui::COLOR_DARK;
use crate::ui::COLOR_DARK2;
use crate::ui::COLOR_LIGHT;
use crate::ui::COLOR_MID;
use bevy::prelude::*;
use chess_core::{ClientMessage, NewGameParams};

#[derive(Component)]
pub struct CreateDialogComponent;

pub fn setup_create_dialog(mut commands: Commands) {
    spawn_dialog!(
        commands,
        CreateDialogComponent,
        Val::Px(500.0),
        Val::Px(300.0),
        |p| {
            spawn_label!(p, "Create Game", 30.0);
            spawn_button!(p, "Confirm", CreateAction::Confirm, ButtonColors::green());
            spawn_button!(p, "Cancel", CreateAction::Cancel, ButtonColors::red());
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
