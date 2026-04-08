use crate::state::{ClientState, Overlay};
use crate::ui::views::menuview::menuroot::MenuTabContainer;
use crate::ui::views::menuview::MenuTabComponent;
use bevy::prelude::*;
use bevy::ui_widgets::{ControlOrientation, CoreScrollbarThumb, Scrollbar};
use bevy_flair::prelude::*;
use chess_core::ClientMessage;

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
                children![
                    (Text::new("Schach!"), ClassList::new("label-large")),
                    (
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
                    )
                ],
            ));

            // Games List wrapper
            p.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(400.0),
                    display: Display::Grid,
                    grid_template_columns: vec![GridTrack::flex(1.0)],
                    grid_template_rows: vec![GridTrack::flex(1.0)],
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                children![Node::default()],
            ))
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
                    ))
                    .id();

                // Scrollbar (sibling of game_list)
                parent.spawn((
                    Node {
                        width: Val::Px(8.0),
                        height: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        right: Val::Px(0.0),
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
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.8, 0.8, 0.8)),
                        CoreScrollbarThumb,
                    ))),
                ));
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

pub fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MenuScreenComponent>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn gamelist_menu_action_system(
    mut interaction_query: Query<(&Interaction, &MenuAction), (Changed<Interaction>, With<Button>)>,
    mut state: ResMut<ClientState>,
    mut next_overlay: ResMut<NextState<Overlay>>,
) {
    for (interaction, action) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            match action {
                MenuAction::CreateGame => {
                    next_overlay.set(Overlay::CreateDialog);
                }
                MenuAction::ListGames => {
                    state.network.send(ClientMessage::QueryGames);
                }
                MenuAction::JoinGame(id) => {
                    state.menu_state.selected_game = Some(*id);
                    next_overlay.set(Overlay::JoinDialog);
                }
            }
        }
    }
}

pub fn update_games_list(
    _clicked: On<UpdateGamesList>,
    state: Res<ClientState>,
    mut commands: Commands,
    container_query: Query<Entity, With<GamesListContainer>>,
    children_query: Query<&Children, With<GamesListContainer>>,
) {
    // get the container where the game list is rendered into.
    // see menuroot.rs for this.
    let container = match container_query.single() {
        Ok(c) => c,
        Err(_) => return,
    };

    // despawn previous game list
    if let Ok(children) = children_query.get(container) {
        for child in children {
            commands.entity(*child).despawn();
        }
    }

    // render new game list
    commands.entity(container).with_children(|parent| {
        for (game_id, details) in &state.menu_state.games {
            parent
                .spawn((Node::default(), ClassList::new("game-item")))
                .with_children(|p| {
                    p.spawn(Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    })
                    .with_children(|info| {
                        info.spawn((
                            Text::new(format!("Game #{}", game_id)),
                            ClassList::new("label-small"),
                        ));

                        let players_text = if let Some(d) = details {
                            let white = d
                                .white_player
                                .map(|id| id.to_string())
                                .unwrap_or("?".to_string());
                            let black = d
                                .black_player
                                .map(|id| id.to_string())
                                .unwrap_or("?".to_string());
                            format!("White: {} vs Black: {}", white, black)
                        } else {
                            "Loading details...".to_string()
                        };

                        info.spawn((Text::new(players_text), ClassList::new("label-small")));
                    });

                    p.spawn((
                        Button,
                        Interaction::default(),
                        ClassList::new(""),
                        MenuAction::JoinGame(*game_id),
                        children![Text::new("Join")],
                    ));
                });
        }
    });
}
