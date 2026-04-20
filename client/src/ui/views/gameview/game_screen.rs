use super::GameScreenComponent;
use crate::client::game::{ActiveGame, GameJoinedEvent};
use crate::client::lobby::LobbyState;
use crate::client::network::NetworkSend;
use crate::ui::views::gameview::chessboard::board::{ChessBoard, RotateBoardEvent};
use crate::ui::views::gameview::historypanel::movehistory::{
    MoveHistory, on_scroll_handler, refresh_move_history, send_scroll_events, update_move_history,
};
use crate::ui::views::menuview::gamemenu::dialogs::create_game_dialog::CreateAction;
use crate::ui::{Overlay, Screen};
use bevy::prelude::*;
use bevy_flair::prelude::{ClassList, NodeStyleSheet};
use chess_core::protocol::NewGameParams;
use chess_core::protocol::messages::ClientMessage;
use std::f32::consts::PI;

pub const SOURCE_COLOR: Color = Color::srgb_u8(250, 113, 113);
pub const DESTINATION_COLOR: Color = Color::srgb_u8(113, 250, 113);
pub const RESOLUTION: f32 = 800.0;

pub struct GameScreenPlugin;

impl Plugin for GameScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Game), setup_gamescreen)
            .add_systems(OnExit(Screen::Game), cleanup_gamescreen)
            // listening for keyboard input (only ESC for now)
            .add_systems(Update, listen_keyboard_input.run_if(in_state(Screen::Game)))
            .add_systems(
                Update,
                (
                    update_player_names.run_if(
                        resource_exists::<ActiveGame>.and(
                            resource_exists_and_changed::<LobbyState>
                                .or(resource_exists_and_changed::<ActiveGame>),
                        ),
                    ),
                    send_scroll_events,
                )
                    .run_if(in_state(Screen::Game)),
            )
            .add_observer(on_game_joined)
            .add_observer(on_rotate)
            .add_observer(update_move_history)
            .add_observer(refresh_move_history)
            .add_observer(on_scroll_handler)
            .add_systems(
                Update,
                gamescreen_button_system.run_if(in_state(Screen::Game)),
            )
            .add_systems(Update, on_resize.run_if(in_state(Screen::Game)));
    }
}

pub fn on_game_joined(
    _ev: On<GameJoinedEvent>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_overlay: ResMut<NextState<Overlay>>,
) {
    next_screen.set(Screen::Game);
    next_overlay.set(Overlay::None);
}

#[derive(Component)]
pub struct WhitePlayerLabel;
#[derive(Component)]
pub struct BlackPlayerLabel;
#[derive(Component)]
pub struct ResignButton;

#[derive(Event)]
pub struct GameScreenInitialized;

/// Sets up the in-game screen.
/// Draws the chessboard and triggers a `BoardUpdate` event to trigger piece position retrievement.
fn setup_gamescreen(
    mut commands: Commands,
    win_query: Query<(Entity, &Window)>,
    asset_server: Res<AssetServer>,
) {
    log::info!("Setting up gamescreen");

    commands.spawn(ChessBoard::new());

    commands.spawn((
        GameScreenComponent,
        WhitePlayerLabel,
        NodeStyleSheet::new(asset_server.load("style.css")),
        ClassList::new("player-label"),
        Node {
            bottom: Val::Px(50.0),
            left: Val::Px(0.0),
            ..default()
        },
        children![Text::new("Waiting for White...")],
    ));

    commands.spawn((
        GameScreenComponent,
        BlackPlayerLabel,
        NodeStyleSheet::new(asset_server.load("style.css")),
        ClassList::new("player-label"),
        Node {
            top: Val::Px(50.0),
            left: Val::Px(0.0),
            ..default()
        },
        children![Text::new("Waiting for Black...")],
    ));

    commands.spawn((
        GameScreenComponent,
        NodeStyleSheet::new(asset_server.load("style.css")),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            height: Val::Percent(100.0),
            width: Val::Auto,
            justify_items: JustifyItems::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        ClassList::new("column-align"),
        children![
            (
                Button,
                Interaction::default(),
                GameAction::Resign,
                children![Text::new("R"), TextFont { ..default() }],
                ClassList::new("game-button"),
            ),
            (
                Button,
                Interaction::default(),
                GameAction::OfferDraw,
                children![Text::new("D"), TextFont { ..default() }],
                ClassList::new("game-button"),
            ),
        ],
    ));

    commands.spawn(MoveHistory::new(&asset_server));

    // We trigger a WindowResized event manually so the board and other items scale
    // themselves properly at startup.
    let window = win_query.single().unwrap();
    let win_size = window.1.size();
    commands.write_message(bevy::window::WindowResized {
        window: window.0,
        width: win_size.x,
        height: win_size.y,
    });

    commands.trigger(GameScreenInitialized);
}

/// Despawn all entities that are part of the in-game screen.
/// Obviously happens when we leave a game.
fn cleanup_gamescreen(
    mut commands: Commands,
    query: Query<Entity, With<GameScreenComponent>>,
    mut ui_scale: ResMut<UiScale>,
) {
    log::debug!("Cleaning up gamescreen");

    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    // rest scaling so the main menu isn't affected
    ui_scale.0 = 1.0;
}

fn listen_keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_overlay: ResMut<NextState<Overlay>>,
    overlay: Res<State<Overlay>>,
) {
    if keys.just_pressed(KeyCode::Escape) && *overlay.get() == Overlay::None {
        next_overlay.set(Overlay::QuitGameDialog);
    }
}

fn update_player_names(
    game: Res<ActiveGame>,
    lobby: Res<LobbyState>,
    query_white: Single<&Children, With<WhitePlayerLabel>>,
    query_black: Single<&Children, (With<BlackPlayerLabel>, Without<WhitePlayerLabel>)>,
    mut query_text: Query<&mut Text>,
) {
    let game_info = game.game_info;

    if let Some(&text_entity) = query_white.first() {
        if let Ok(mut white_text) = query_text.get_mut(text_entity) {
            let name = if let Some(wid) = game_info.white_player {
                lobby
                    .get_client_info(wid)
                    .cloned()
                    .unwrap_or_else(|| format!("Player {}", wid))
            } else {
                "Waiting for White...".to_string()
            };

            if white_text.0 != name {
                log::debug!("Updating white player name to {}", name);
                white_text.0 = name;
            }
        }
    }

    if let Some(&text_entity) = query_black.first() {
        if let Ok(mut black_text) = query_text.get_mut(text_entity) {
            let name = if let Some(bid) = game_info.black_player {
                lobby
                    .get_client_info(bid)
                    .cloned()
                    .unwrap_or_else(|| format!("Player {}", bid))
            } else {
                "Waiting for Black...".to_string()
            };

            if black_text.0 != name {
                log::debug!("Updating black player name to {}", name);
                black_text.0 = name;
            }
        }
    }
}

/// Computes scale and position of all game screen components based on the current window size.
/// This method is kinda complex due to the need to adjust various UI elements dynamically based on
/// window size and available screen space.
pub fn on_resize(
    mut resize_reader: MessageReader<bevy::window::WindowResized>,
    mut queries: ParamSet<(Single<&mut Transform, With<ChessBoard>>,)>,
    mut ui_scale: ResMut<UiScale>,
) {
    // Collect the latest window size.
    let mut last_size = None;
    for e in resize_reader.read() {
        last_size = Some(Vec2::new(e.width, e.height));
    }
    let Some(win_size) = last_size else {
        return;
    };

    let history_width = 400.0;
    let padding = 10.0;

    // Available space for the board is constrained by:
    // 1. window_height (board must fit vertically)
    // 2. window_width - history_width (to keep it centered and not overlap history)
    let max_board_dim = win_size.y.min(win_size.x - (history_width + padding));
    let scale = (max_board_dim / RESOLUTION) * 0.75;

    // The board is not a UI element, so it's NOT affected by UiScale.
    // We scale it manually.
    {
        let mut board_transform = queries.p0();
        board_transform.scale = Vec3::splat(scale);
    }

    // Finally, update the global UI scale.
    ui_scale.0 = scale;
}
pub fn on_rotate(
    _ev: On<RotateBoardEvent>,
    mut queries: ParamSet<(
        Query<&mut Transform, With<ChessBoard>>,
        Single<&mut Node, With<WhitePlayerLabel>>,
        Single<&mut Node, (With<BlackPlayerLabel>, Without<WhitePlayerLabel>)>,
    )>,
    mut commands: Commands,
    win_query: Query<(Entity, &Window)>,
) {
    log::debug!("Rotating game view");

    // rotate the board 180'
    let is_rotated = {
        let mut board_query = queries.p0();
        let mut rotated = false;
        for mut board in board_query.iter_mut() {
            board.rotate_z(PI);
            rotated = board.rotation.z.abs() > 0.1;
        }
        rotated
    };

    // Update Player Labels
    let set_label = |node: &mut Node, is_top: bool| {
        if is_top {
            node.top = Val::Px(50.0);
            node.bottom = Val::Auto;
        } else {
            node.bottom = Val::Px(50.0);
            node.top = Val::Auto;
        }
    };

    // White is normally at the bottom, Black at the top.
    // Swap if the board is rotated.
    {
        let mut white_node = queries.p1();
        set_label(&mut white_node, is_rotated);
    }
    {
        let mut black_node = queries.p2();
        set_label(&mut black_node, !is_rotated);
    }

    // Trigger a resize to update UI node positions
    if let Ok((entity, window)) = win_query.single() {
        let win_size = window.size();
        commands.write_message(bevy::window::WindowResized {
            window: entity,
            width: win_size.x,
            height: win_size.y,
        });
    }
}

#[derive(Component)]
pub enum GameAction {
    Resign,
    OfferDraw,
}

pub fn gamescreen_button_system(
    mut interaction_query: Query<(&Interaction, &GameAction), (Changed<Interaction>, With<Button>)>,
    mut commands: Commands,
    game: ResMut<ActiveGame>,
) {
    for (interaction, action) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            match action {
                GameAction::Resign => {
                    commands.trigger(NetworkSend(ClientMessage::Resign(game.gid)));
                }
                GameAction::OfferDraw => {
                    commands.trigger(NetworkSend(ClientMessage::OfferDraw(game.gid)))
                }
            }
        }
    }
}
