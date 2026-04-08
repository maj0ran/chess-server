use crate::state::{ClientState, Overlay};
use bevy::prelude::*;
use bevy_flair::prelude::*;
use chess_core::{ClientMessage, JoinGameParams, UserRoleSelection};

#[derive(Component)]
pub struct JoinDialogComponent;

pub fn setup_join_dialog(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Node::default(),
        JoinDialogComponent,
        NodeStyleSheet::new(asset_server.load("style.css")),
        ClassList::new("dialog-overlay"),
        children![(
            Node::default(),
            ClassList::new("dialog-content column-align"),
            children![
                (Text::new("Select Side"), ClassList::new("label-large")),
                (
                    Button,
                    Interaction::default(),
                    ClassList::new(""),
                    JoinAction::Select(UserRoleSelection::White),
                    children![Text::new("White")],
                ),
                (
                    Button,
                    Interaction::default(),
                    ClassList::new(""),
                    JoinAction::Select(UserRoleSelection::Black),
                    children![Text::new("Black")],
                ),
                (
                    Button,
                    Interaction::default(),
                    ClassList::new(""),
                    JoinAction::Select(UserRoleSelection::Random),
                    children![Text::new("Random")],
                ),
                (
                    Button,
                    Interaction::default(),
                    ClassList::new(""),
                    JoinAction::Select(UserRoleSelection::Both),
                    children![Text::new("Both")],
                ),
                (
                    Button,
                    Interaction::default(),
                    ClassList::new(""),
                    JoinAction::Select(UserRoleSelection::Spectator),
                    children![Text::new("Spectator")],
                ),
                (
                    Button,
                    Interaction::default(),
                    ClassList::new("button-red"),
                    JoinAction::Cancel,
                    children![Text::new("Cancel")],
                )
            ],
        )],
    ));
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
    state: ResMut<ClientState>,
    mut next_overlay: ResMut<NextState<Overlay>>,
) {
    for (interaction, action) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            match action {
                JoinAction::Select(side) => {
                    if let Some(game_id) = state.menu_state.selected_game {
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
