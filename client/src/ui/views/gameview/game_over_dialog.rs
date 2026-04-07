use crate::state::{ClientState, Overlay, Screen};
use bevy::prelude::*;
use bevy_flair::prelude::*;

#[derive(Component)]
pub struct GameOverDialogComponent;

pub fn setup_game_over_dialog(
    mut commands: Commands,
    state: ResMut<ClientState>,
    asset_server: Res<AssetServer>,
) {
    let winner = state.game_state.winner.unwrap();
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
            GameOverDialogComponent,
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
                    p.spawn((Text::new("Game Over"), ClassList::new("label-large")));
                    p.spawn((
                        Text::new(format!("{} won", winner)),
                        ClassList::new("label-small"),
                    ));
                    p.spawn((
                        Button,
                        Interaction::default(),
                        ClassList::new(""),
                        GameOverAction::Ok,
                    ))
                    .with_children(|btn| {
                        btn.spawn(Text::new("OK"));
                    });
                });
        })
        .id();
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
