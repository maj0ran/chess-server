use crate::state::{ClientState, Overlay};
use crate::ui::views::menuview::menuroot::MenuTabContainer;
use crate::ui::views::menuview::MenuTabComponent;
use crate::ui::*;
use bevy::color::palettes::basic::*;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::BorderColor;
use bevy::ui_widgets::{ControlOrientation, CoreScrollbarThumb, Scrollbar};
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
    let container = container_query.single().ok();

    let menu_node = commands
        .spawn((
            Node {
                width: Val::Percent(80.0),
                height: Val::Percent(100.0),
                display: Display::Grid,
                grid_template_columns: vec![GridTrack::flex(1.0)],
                grid_template_rows: vec![GridTrack::auto(), GridTrack::flex(1.0)],
                row_gap: Val::Px(20.0),
                align_items: AlignItems::Center,
                justify_items: JustifyItems::Center,
                ..default()
            },
            MenuScreenComponent,
            MenuTabComponent,
        ))
        .with_children(|p| {
            p.spawn(Node {
                display: Display::Grid,
                grid_template_columns: vec![GridTrack::flex(1.0)],
                grid_auto_rows: vec![GridTrack::auto()],
                row_gap: Val::Px(10.0),
                align_items: AlignItems::Center,
                justify_items: JustifyItems::Center,
                border: UiRect::bottom(Val::Px(2.0)),
                ..default()
            })
            .with_children(|p| {
                // Title Label
                spawn_label!(p, "Schach!", 40.0, Color::WHITE);
                // Buttons container
                p.spawn(Node {
                    display: Display::Grid,
                    grid_template_columns: vec![GridTrack::px(210.0), GridTrack::px(210.0)],
                    grid_template_rows: vec![GridTrack::auto()],
                    column_gap: Val::Px(10.0),
                    align_items: AlignItems::Center,
                    justify_items: JustifyItems::Center,
                    ..default()
                })
                .with_children(|parent| {
                    spawn_button!(
                        parent,
                        "Create Game",
                        MenuAction::CreateGame,
                        ButtonColors::default(),
                        Val::Px(200.0),
                        Val::Px(50.0)
                    );
                    spawn_button!(
                        parent,
                        "List Games",
                        MenuAction::ListGames,
                        ButtonColors::default(),
                        Val::Px(200.0),
                        Val::Px(50.0)
                    );
                });
            });

            // Games List wrapper
            p.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Px(400.0),
                display: Display::Grid,
                grid_template_columns: vec![GridTrack::flex(1.0)],
                grid_template_rows: vec![GridTrack::flex(1.0)],
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
                            display: Display::Grid,
                            grid_template_columns: vec![GridTrack::flex(1.0)],
                            grid_auto_rows: vec![GridTrack::auto()],
                            overflow: Overflow::scroll_y(),
                            ..default()
                        },
                        BackgroundColor(COLOR_DARK2),
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
        })
        .id();

    if let Some(container) = container {
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
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(60.0),
                        display: Display::Grid,
                        grid_template_columns: vec![GridTrack::flex(1.0), GridTrack::px(120.0)],
                        grid_template_rows: vec![GridTrack::flex(1.0)],
                        padding: UiRect::horizontal(Val::Px(10.0)),
                        border: UiRect::bottom(Val::Px(1.0)),
                        align_items: AlignItems::Center,
                        justify_items: JustifyItems::Start,
                        ..default()
                    },
                    BorderColor::all(Color::srgb(0.3, 0.3, 0.3)),
                ))
                .with_children(|p| {
                    p.spawn(Node {
                        display: Display::Grid,
                        grid_template_columns: vec![GridTrack::flex(1.0)],
                        grid_auto_rows: vec![GridTrack::auto()],
                        ..default()
                    })
                    .with_children(|info| {
                        info.spawn((
                            Text::new(format!("Game #{}", game_id)),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(COLOR_LIGHT),
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

                    spawn_button!(
                        p,
                        "Join",
                        MenuAction::JoinGame(*game_id),
                        ButtonColors::default(),
                        Val::Px(100.0),
                        Val::Px(40.0)
                    );
                });
        }
    });
}
