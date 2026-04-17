use crate::backend::client::ClientRequest;
use crate::ui::Overlay;
use bevy::prelude::*;
use bevy_flair::prelude::*;
use chess_core::protocol::NewGameParams;
use chess_core::protocol::messages::ClientMessage;

#[derive(Component)]
pub struct CreateDialogComponent;

pub fn setup_create_dialog(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Node::default(),
        NodeStyleSheet::new(asset_server.load("style.css")),
        CreateDialogComponent,
        children![(
            Node::default(),
            ClassList::new("dialog-content column-align"),
            children![
                (Text::new("Create Game"), ClassList::new("label-large")),
                (
                    Button,
                    Interaction::default(),
                    ClassList::new("button-green"),
                    CreateAction::Confirm,
                    children![Text::new("Create")],
                ),
                (
                    Button,
                    Interaction::default(),
                    ClassList::new("button-red"),
                    CreateAction::Cancel,
                    children![Text::new("Cancel")],
                )
            ],
        )],
    ));
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
    mut next_overlay: ResMut<NextState<Overlay>>,
    mut commands: Commands,
) {
    for (interaction, action) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            match action {
                CreateAction::Confirm => {
                    commands.trigger(ClientRequest(ClientMessage::NewGame(NewGameParams {
                        mode: 0,
                        time: 600,
                        time_inc: 10,
                    })));
                    next_overlay.set(Overlay::None);
                }
                CreateAction::Cancel => {
                    next_overlay.set(Overlay::None);
                }
            }
        }
    }
}
