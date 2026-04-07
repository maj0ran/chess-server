use crate::state::{ClientState, Overlay};
use bevy::prelude::*;
use bevy_flair::prelude::*;
use chess_core::{ClientMessage, NewGameParams};

#[derive(Component)]
pub struct CreateDialogComponent;

pub fn setup_create_dialog(mut commands: Commands, asset_server: Res<AssetServer>) {
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
            NodeStyleSheet::new(asset_server.load("style.css")),
            CreateDialogComponent,
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
                    p.spawn((Text::new("Create Game"), ClassList::new("label-large")));
                    p.spawn((
                        Button,
                        Interaction::default(),
                        ClassList::new("button-green"),
                        CreateAction::Confirm,
                    ))
                    .with_children(|btn| {
                        btn.spawn(Text::new("Confirm"));
                    });
                    p.spawn((
                        Button,
                        Interaction::default(),
                        ClassList::new("button-red"),
                        CreateAction::Cancel,
                    ))
                    .with_children(|btn| {
                        btn.spawn(Text::new("Cancel"));
                    });
                });
        })
        .id();
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
    state: ResMut<ClientState>,
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
