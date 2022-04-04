use crate::btn_default;
use crate::state::{ClientState, Overlay, Screen};
use crate::ui::create_game_dialog::{
    cleanup_create_dialog, create_dialog_action_system, setup_create_dialog,
};
use crate::ui::join_game_dialog::{
    cleanup_join_dialog, join_dialog_action_system, setup_join_dialog,
};
use crate::ui::ButtonColors;
use crate::{spawn_button, spawn_label};
use bevy::color::palettes::basic::*;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::BorderColor;
use bevy::ui_widgets::{ControlOrientation, CoreScrollbarThumb, Scrollbar};
use chess_core::ClientMessage;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        // Main Menu
        app.add_systems(OnEnter(Screen::Menu), setup_menu)
            .add_systems(OnExit(Screen::Menu), cleanup_menu)
            .add_systems(Update, menu_action_system.run_if(in_state(Screen::Menu)))
            .add_systems(Update, update_games_list.run_if(in_state(Screen::Menu)))
            // Create Game Dialog
            .add_systems(OnEnter(Overlay::CreateDialog), setup_create_dialog)
            .add_systems(OnExit(Overlay::CreateDialog), cleanup_create_dialog)
            .add_systems(
                Update,
                create_dialog_action_system.run_if(in_state(Overlay::CreateDialog)),
            )
            // Join Game Dialog
            .add_systems(OnEnter(Overlay::JoinDialog), setup_join_dialog)
            .add_systems(OnExit(Overlay::JoinDialog), cleanup_join_dialog)
            .add_systems(
                Update,
                join_dialog_action_system.run_if(in_state(Overlay::JoinDialog)),
            );
    }
}

#[derive(Component)]
pub struct MenuScreenComponent;

pub fn setup_menu(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            MenuScreenComponent,
        ))
        .with_children(|p| {
            // Title Label
            spawn_label!(p, "Schach!", 40.0, Color::WHITE);
            // Buttons container
            p.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            })
            .with_children(|parent| {
                spawn_button!(
                    parent,
                    "Create Game",
                    MenuAction::CreateGame,
                    btn_default!(),
                    Val::Px(200.0),
                    Val::Px(50.0)
                );
                spawn_button!(
                    parent,
                    "List Games",
                    MenuAction::ListGames,
                    btn_default!(),
                    Val::Px(200.0),
                    Val::Px(50.0)
                );
            });

            // Games List wrapper
            p.spawn(Node {
                width: Val::Px(400.0),
                height: Val::Px(300.0),
                flex_direction: FlexDirection::Row,
                margin: UiRect::all(Val::Px(10.0)),
                ..default()
            })
            .with_children(|parent| {
                // Games List container (scrolling content)
                let game_list = parent
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            overflow: Overflow::scroll_y(),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
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
                        Hovered::default(),
                        BackgroundColor(Color::srgb(0.8, 0.8, 0.8)),
                        CoreScrollbarThumb,
                    ))),
                ));
            });
        });
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

pub fn menu_action_system(
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
                    state.menu_state.is_loading = true;
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
    state: Res<ClientState>,
    mut commands: Commands,
    container_query: Query<Entity, With<GamesListContainer>>,
    children_query: Query<&Children, With<GamesListContainer>>,
) {
    if !state.is_changed() {
        return;
    }

    let container = match container_query.single() {
        Ok(c) => c,
        Err(_) => return,
    };

    if let Ok(children) = children_query.get(container) {
        for child in children {
            commands.entity(*child).despawn();
        }
    }

    commands.entity(container).with_children(|parent| {
        let mut sorted_games: Vec<_> = state.menu_state.games.keys().collect();
        sorted_games.sort();

        for &game_id in sorted_games {
            let details = state
                .menu_state
                .games
                .get(&game_id)
                .and_then(|d| d.as_ref());

            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(60.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        padding: UiRect::horizontal(Val::Px(10.0)),
                        border: UiRect::bottom(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor::all(Color::srgb(0.3, 0.3, 0.3)),
                ))
                .with_children(|p| {
                    p.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        flex_grow: 1.0,
                        ..default()
                    })
                    .with_children(|info| {
                        info.spawn((
                            Text::new(format!("Game #{}", game_id)),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
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

                        info.spawn((
                            Text::new(players_text),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::from(GRAY)),
                        ));
                    });

                    spawn_button!(p, "Join", MenuAction::JoinGame(game_id));
                });
        }
    });
}
