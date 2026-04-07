use crate::state::{ClientState, Overlay, Screen};
use bevy::prelude::*;
use bevy_flair::prelude::*;
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

pub fn setup_quit_game_dialog(mut commands: Commands, asset_server: Res<AssetServer>) {
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
            QuitGameDialog,
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
                    p.spawn((Text::new("Are you sure?"), ClassList::new("label-large")));
                    p.spawn((
                        Button,
                        Interaction::default(),
                        ClassList::new("button-green"),
                        QuitGameAction::Confirm,
                    ))
                    .with_children(|btn| {
                        btn.spawn(Text::new("Confirm"));
                    });
                    p.spawn((
                        Button,
                        Interaction::default(),
                        ClassList::new("button-red"),
                        QuitGameAction::Cancel,
                    ))
                    .with_children(|btn| {
                        btn.spawn(Text::new("Cancel"));
                    });
                });
        })
        .id();
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
