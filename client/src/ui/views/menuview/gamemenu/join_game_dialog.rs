use crate::state::{ClientState, Overlay};
use bevy::prelude::*;
use bevy_flair::prelude::*;
use chess_core::{ClientMessage, JoinGameParams, UserRoleSelection};

#[derive(Component)]
pub struct JoinDialogComponent;

pub fn setup_join_dialog(mut commands: Commands, asset_server: Res<AssetServer>) {
    let dialog = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            JoinDialogComponent,
            NodeStyleSheet::new(asset_server.load("style.css")),
            ClassList::new("dialog-overlay"),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ClassList::new("dialog-content"),
                ))
                .with_children(|p| {
                    p.spawn((Text::new("Select Side"), ClassList::new("label-large")));
                    p.spawn((
                        Button,
                        Interaction::default(),
                        ClassList::new(""),
                        JoinAction::Select(UserRoleSelection::White),
                    ))
                    .with_children(|btn| {
                        btn.spawn(Text::new("White"));
                    });
                    p.spawn((
                        Button,
                        Interaction::default(),
                        ClassList::new(""),
                        JoinAction::Select(UserRoleSelection::Black),
                    ))
                    .with_children(|btn| {
                        btn.spawn(Text::new("Black"));
                    });
                    p.spawn((
                        Button,
                        Interaction::default(),
                        ClassList::new(""),
                        JoinAction::Select(UserRoleSelection::Random),
                    ))
                    .with_children(|btn| {
                        btn.spawn(Text::new("Random"));
                    });
                    p.spawn((
                        Button,
                        Interaction::default(),
                        ClassList::new(""),
                        JoinAction::Select(UserRoleSelection::Both),
                    ))
                    .with_children(|btn| {
                        btn.spawn(Text::new("Both"));
                    });
                    p.spawn((
                        Button,
                        Interaction::default(),
                        ClassList::new(""),
                        JoinAction::Select(UserRoleSelection::Spectator),
                    ))
                    .with_children(|btn| {
                        btn.spawn(Text::new("Spectator"));
                    });

                    p.spawn((
                        Button,
                        Interaction::default(),
                        ClassList::new("button-red"),
                        JoinAction::Cancel,
                    ))
                    .with_children(|btn| {
                        btn.spawn(Text::new("Cancel"));
                    });
                });
        })
        .id();
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
