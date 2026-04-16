use crate::backend::client::GameOverEvent;
use crate::ui::{Overlay, Screen};
use bevy::prelude::*;
use bevy_flair::prelude::*;

#[derive(Component)]
pub struct GameOverDialogComponent;

pub fn on_game_over(
    ev: On<GameOverEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut next_overlay: ResMut<NextState<Overlay>>,
) {
    let reason = Some(ev.reason);
    let winner = ev.reason.get_winner();

    next_overlay.set(Overlay::GameOver);

    let dialog_text = if let Some(p) = winner {
        format!("{} won by {}!", p, reason.unwrap())
    } else {
        format!("Draw by {}!", reason.unwrap())
    };
    commands.spawn((
        Node::default(),
        NodeStyleSheet::new(asset_server.load("style.css")),
        GameOverDialogComponent,
        children![(
            Node::default(),
            ClassList::new("dialog-content column-align"),
            children![
                (Text::new("Game Over"), ClassList::new("label-large")),
                (Text::new(dialog_text), ClassList::new("label-small")),
                (
                    Button,
                    Interaction::default(),
                    GameOverAction::Ok,
                    children![Text::new("OK")],
                )
            ],
        )],
    ));
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
