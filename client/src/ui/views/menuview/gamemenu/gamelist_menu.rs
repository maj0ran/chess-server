use crate::backend::client::LobbyState;
use crate::backend::network::NetworkSend;
use crate::ui::Overlay;
use crate::ui::views::menuview::MenuTabComponent;
use crate::ui::views::menuview::menuroot::MenuTabContainer;
use bevy::prelude::*;
use bevy::ui_widgets::{ControlOrientation, CoreScrollbarThumb, Scrollbar};
use bevy_flair::prelude::*;
use chess_core::protocol::messages::ClientMessage;

pub struct MenuPlugin;

#[derive(Component)]
pub struct MenuScreenComponent;

#[derive(Event)]
pub struct UpdateGamesList;

pub fn setup_gamelist_menu(
    mut commands: Commands,
    container_query: Query<Entity, With<MenuTabContainer>>,
) {
    let container = container_query.single();

    let menu_node = commands
        .spawn((
            Node::default(),
            MenuScreenComponent,
            MenuTabComponent,
            ClassList::new("menu"),
            children![],
        ))
        .with_children(|p| {
            p.spawn((
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![(
                    Node {
                        column_gap: Val::Px(50.0),
                        ..default()
                    },
                    children![
                        (
                            Button,
                            Interaction::default(),
                            ClassList::new(""),
                            MenuAction::CreateGame,
                            children![Text::new("Create Game")],
                        ),
                        (
                            Button,
                            Interaction::default(),
                            ClassList::new(""),
                            MenuAction::ListGames,
                            children![Text::new("List Games")],
                        )
                    ],
                )],
            ));

            // Games List wrapper
            p.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(500.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
                ClassList::new("game-list-container"),
                children![(
                    Node::default(),
                    ClassList::new("game-header"),
                    children![
                        (Text::new("Game ID"), ClassList::new("label-small")),
                        (Text::new("Players"), ClassList::new("label-small")),
                        (Text::new("Action"), ClassList::new("label-small")),
                    ],
                ),],
            ))
            .with_children(|parent| {
                parent
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(500.0),
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    })
                    .with_children(|parent| {
                        // Games List container (scrolling content)
                        let game_list = parent
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    display: Display::Grid,
                                    grid_template_columns: vec![GridTrack::flex(1.0)],
                                    grid_auto_rows: vec![GridTrack::auto()],
                                    overflow: Overflow::scroll_y(),
                                    ..default()
                                },
                                GamesListContainer,
                                children![],
                            ))
                            .id();

                        // Scrollbar
                        parent.spawn((
                            Node {
                                width: Val::Px(10.0),
                                height: Val::Percent(100.0),
                                position_type: PositionType::Absolute,
                                right: Val::Px(2.0),
                                top: Val::Px(0.0),
                                ..default()
                            },
                            Scrollbar {
                                orientation: ControlOrientation::Vertical,
                                target: game_list,
                                min_thumb_length: 20.0,
                            },
                            Children::spawn(Spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    border_radius: BorderRadius::all(Val::Px(5.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.5, 0.4, 0.3)),
                                CoreScrollbarThumb,
                            ))),
                        ));
                    });
            });
        })
        .id();

    if let Ok(container) = container {
        commands.entity(container).add_child(menu_node);
    }
}

#[derive(Component)]
pub enum MenuAction {
    CreateGame,
    ListGames,
    JoinGame(chess_core::GameId),
}

#[derive(Component)]
pub struct GamesListContainer;

pub fn cleanup_gamelist_menu(
    mut commands: Commands,
    query: Query<Entity, With<MenuScreenComponent>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn gamelist_menu_action_system(
    mut interaction_query: Query<(&Interaction, &MenuAction), (Changed<Interaction>, With<Button>)>,
    mut lobby: ResMut<LobbyState>,
    mut next_overlay: ResMut<NextState<Overlay>>,
    mut commands: Commands,
) {
    for (interaction, action) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            match action {
                MenuAction::CreateGame => {
                    next_overlay.set(Overlay::CreateDialog);
                }
                MenuAction::ListGames => {
                    commands.trigger(NetworkSend(ClientMessage::QueryGames));
                }
                MenuAction::JoinGame(gid) => {
                    lobby.pending_join_game = Some(*gid);
                    next_overlay.set(Overlay::JoinDialog);
                }
            }
        }
    }
}

pub fn update_games_list(
    _clicked: On<UpdateGamesList>,
    lobby: Res<LobbyState>,
    mut commands: Commands,
    container_query: Query<Entity, With<GamesListContainer>>,
    children_query: Query<&Children, With<GamesListContainer>>,
) {
    // get the container where the game list is rendered into.
    // see menuroot.rs for this.
    let container = container_query.single().unwrap();

    // despawn previous game list
    if let Ok(children) = children_query.get(container) {
        for child in children {
            commands.entity(*child).despawn();
        }
    }

    // render new game list
    commands.entity(container).with_children(|parent| {
        for (gid, details) in &lobby.games {
            let game_info = if let Some(d) = details {
                // from the Client ID, get the name from the internal client list,
                // otherwise use the ID as a fallback (should not happen tbh).
                // finally, use "-" if both are missing. (this just means that no player
                // is connected to a game)
                let white = d
                    .white_player
                    .and_then(|id| lobby.client_names.get(&id))
                    .cloned()
                    .unwrap_or_else(|| {
                        d.white_player
                            .map(|id| id.to_string())
                            .unwrap_or("-".to_string())
                    });
                // ditto
                let black = d
                    .black_player
                    .and_then(|id| lobby.client_names.get(&id))
                    .cloned()
                    .unwrap_or_else(|| {
                        d.black_player
                            .map(|id| id.to_string())
                            .unwrap_or("-".to_string())
                    });
                format!("White: {} | Black: {}", white, black)
            } else {
                "Loading details...".to_string()
            };

            parent.spawn((
                Node::default(),
                ClassList::new("game-item"),
                children![
                    (
                        Text::new(format!("#{}", gid)),
                        ClassList::new("label-small"),
                    ),
                    (Text::new(game_info), ClassList::new("label-small")),
                    (
                        Button,
                        Interaction::default(),
                        ClassList::new("join-button"),
                        MenuAction::JoinGame(*gid),
                        children![Text::new("Join")],
                    )
                ],
            ));
        }
    });
}
