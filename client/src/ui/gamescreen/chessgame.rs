use bevy::{
    prelude::*,
    feathers::controls::FeathersButton,
    feathers::display::label,
    feathers::theme::*,
    feathers::tokens,
    picking::Pickable,
    ui_widgets::Activate,
};

use crate::{
    backend::{ActiveGame, ClientDetailsEvent, GameJoinedEvent, LobbyState},
    backend::net::NetworkSend,
    ui::{
        Overlay,
        Screen,
        theme::*,
        gamescreen::move_history::*,
        gamescreen::dialogs::GamescreenDialogsPlugin,
        gamescreen::chessboard::RotateBoardEvent,
        gamescreen::chessboard::ChessboardPlugin,
        gamescreen::chessboard::ChessBoard,
    }
};

use chess_core::protocol::messages::ClientMessage;
use chess_core::*;
use ::log;

use std::f32::consts::PI;
use crate::ui::gamescreen::BOARD_SIZE;

/// Marker component for everything that belongs to the in-game screen.
#[derive(Component, Clone, Copy, Default)]
pub struct ChessSceneRootNode;
#[derive(Component, Clone, Copy, Default)]
pub struct WhitePlayerLabel;
#[derive(Component, Clone, Copy, Default)]
pub struct BlackPlayerLabel;
#[derive(Component, Clone, Copy, Default)]
pub struct DrawOfferButton;
#[derive(Event)]
pub struct ChessGameInitialized;
#[derive(Event)]
pub struct DrawOffered(pub bool);

pub struct ChessGamePlugin;

impl Plugin for ChessGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ChessboardPlugin)
            .add_systems(
                OnEnter(Screen::Game),
                (
                    chess_ui_scene.spawn(),
                    setup_chessboard,
                )
                    .chain(),
            )
            .add_systems(OnExit(Screen::Game), cleanup_chess_scene)
            .add_systems(Update, listen_keyboard_input.run_if(in_state(Screen::Game)))
            .add_systems(
                Update,
                (
                    (send_scroll_events, resize_move_history_text),
                )
                    .run_if(in_state(Screen::Game)),
            )
            .add_observer(on_game_joined)
            .add_observer(update_player_names)
            .add_observer(on_rotate)
            .add_observer(update_move_history)
            .add_observer(refresh_move_history)
            .add_observer(on_scroll_handler)
            .add_observer(on_draw_offered)
            .add_systems(
                Update,
                (
                    on_resize,
                )
                    .run_if(in_state(Screen::Game)),
            )
        .add_plugins(GamescreenDialogsPlugin);
    }
}

/// Helper method that gives the 'Resign' and the 'Draw' button the same style.
fn left_panel_button_node() -> Node {
    Node {
        width: percent(80),
        height: Val::Auto,
        aspect_ratio: Some(1.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border: UiRect::all(px(2)),
        border_radius: BorderRadius::all(px(8)),
        ..default()
    }
}
/// `chess_ui_scene` creates all the UI elements around the chess game: the player labels,
/// the panel for resign/draw buttons, and the move history.
/// The actual chess board is NOT a UI element but a real game element.
/// It is thus managed by `setup_chessboard()`.
fn chess_ui_scene() -> impl Scene {
    bsn! {
        // Root Element
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100)
        }
        ChessSceneRootNode
        Pickable::IGNORE
        InheritableThemeTextColor(tokens::TEXT_MAIN)
        Children [
            // White Player Label
            Node {
                position_type: PositionType::Absolute,
                height: px(48),
                bottom: px(16),
                left: percent(12),
                right: percent(27),
                padding: UiRect::all(px(8)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(5))
            }
            WhitePlayerLabel
            BackgroundColor(THEME_SURFACE)
            BorderColor::all(THEME_BORDER)
            Children [ Text("Waiting for White...") ThemedText ]
            --
            // Black Player Label
            Node {
                position_type: PositionType::Absolute,
                height: px(48),
                top: px(16),
                left: percent(12),
                right: percent(27),
                padding: UiRect::all(px(8)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(5))
            }
            BlackPlayerLabel
            BackgroundColor(THEME_SURFACE)
            BorderColor::all(THEME_BORDER)
            Children [ Text("Waiting for Black...") ThemedText ]
            --
            // Left Panel (Game Buttons)
            Node {
                position_type: PositionType::Absolute,
                height: percent(100),
                width: percent(LEFT_PANEL_RATIO * 100.0),
                left: px(0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(10),
                padding: UiRect::all(px(12)),
                border: UiRect::right(px(2)),
                border_radius: BorderRadius::all(px(6))
            }
            BackgroundColor(THEME_SURFACE)
            BorderColor::all(THEME_BORDER)
            default_ui_shadow()
            Children [
                @FeathersButton { @caption: bsn! { @label("R") } }
                left_panel_button_node()
                on(|_: On<Activate>, mut overlay: ResMut<NextState<Overlay>>| overlay.set(Overlay::Resign))
                --
                @FeathersButton { @caption: bsn! { @label("D") } }
                DrawOfferButton
                left_panel_button_node()
                on(|_: On<Activate>, game: Res<ActiveGame>, mut commands: Commands| {
                    commands.trigger(NetworkSend(ClientMessage::OfferDraw(game.gid)));
                })
            ]
            --
            // Right Panel (Move History)
            MoveHistory
            Node {
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                height: percent(100),
                width: percent(RIGHT_PANEL_RATIO * 100.0),
                right: px(0),
                top: px(0),
                padding: UiRect::all(px(12)),
                border: UiRect::left(px(2)),
                border_radius: BorderRadius::all(px(6)),
                overflow: Overflow::scroll_y(),
            }
            BackgroundColor(THEME_SURFACE)
            BorderColor::all(THEME_BORDER)
            default_ui_shadow()
            ]
    }
}

/// Sets up the chess board and triggers the initial board draw.
fn setup_chessboard(
    mut commands: Commands,
    window_query: Single<(Entity, &Window)>,
    game: Option<Res<ActiveGame>>,
    lobby: Res<LobbyState>,
) {
    log_proc!(info, "Setting up chessboard");

    commands.spawn(ChessBoard::new());

    // We trigger a WindowResized event manually so the board and other items scale
    // themselves properly at startup.
    let (window_entity, window) = *window_query;
    let win_size = window.size();
    commands.write_message(bevy::window::WindowResized {
        window: window_entity,
        width: win_size.x,
        height: win_size.y,
    });

    // after we set up the board, we look into the ActiveGame resource
    // to see who is connected to the game. If we have their names, we
    // send the `ClientDetailsEvent` to the UI system, which updates the player labels.
    // if we don't have their names, we need to send `QueryClientDetails` to the server.
    if let Some(game) = game {
        if let Some(white_id) = game.game_info.white_player {
            if let Some(name) = lobby.get_client_info(white_id) {
                commands.trigger(ClientDetailsEvent {
                    cid: white_id,
                    name: name.clone(),
                });
            } else {
                commands.trigger(NetworkSend(ClientMessage::QueryClientDetails(white_id)));
            }
        }
        if let Some(black_id) = game.game_info.black_player {
            if let Some(name) = lobby.get_client_info(black_id) {
                commands.trigger(ClientDetailsEvent {
                    cid: black_id,
                    name: name.clone(),
                });
            } else {
                commands.trigger(NetworkSend(ClientMessage::QueryClientDetails(black_id)));
            }
        }
    }

    commands.trigger(ChessGameInitialized);
}

/// Despawn all entities that are part of the chess game screen,
/// that is, the UI and the chessboard.
/// Obviously happens when we leave a game.
fn cleanup_chess_scene(mut commands: Commands,
                       query_ui: Single<Entity, With<ChessSceneRootNode>>,
                       query_board: Single<Entity, With<ChessBoard>>) {
    log_proc!(debug, "Cleaning up chess scene");

    let ui = query_ui.entity();
    commands.entity(ui).despawn();

    let board = query_board.entity();
    commands.entity(board).despawn();
}

fn listen_keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_overlay: ResMut<NextState<Overlay>>,
    overlay: Res<State<Overlay>>,
) {
    if keys.just_pressed(KeyCode::Escape) && *overlay.get() == Overlay::None {
        next_overlay.set(Overlay::Resign);
    }
}

/// Event Handler for `GameJoinedEvent`.
/// When we join a game, switch to the game screen and dismiss any overlay.
pub fn on_game_joined(
    ev: On<GameJoinedEvent>,
    screen: Res<State<Screen>>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_overlay: ResMut<NextState<Overlay>>,
) {
    log_proc!(debug, "Received GameJoinedEvent for game ID {}", ev.gid);
    // When we join a game, switch to the game screen and dismiss any overlay.
    if *screen.get() != Screen::Game {
        next_screen.set(Screen::Game);
        next_overlay.set(Overlay::None);
    }
}

pub fn update_player_names(
    ev: On<ClientDetailsEvent>,
    game: Option<Res<ActiveGame>>,
    query_white: Query<&Children, With<WhitePlayerLabel>>,
    query_black: Query<&Children, (With<BlackPlayerLabel>, Without<WhitePlayerLabel>)>,
    mut query_text: Query<&mut Text>,
) {
    let Some(game) = game else {
        return;
    };

    if game.game_info.white_player == Some(ev.cid) {
        if let Ok(children) = query_white.single() {
            if let Some(&text_entity) = children.first() {
                if let Ok(mut text) = query_text.get_mut(text_entity) {
                    if text.0 != ev.name {
                        log_proc!(debug, "Updating white player name to {}", ev.name);
                        text.0 = ev.name.clone();
                    }
                }
            }
        }
    }

    if game.game_info.black_player == Some(ev.cid) {
        if let Ok(children) = query_black.single() {
            if let Some(&text_entity) = children.first() {
                if let Ok(mut text) = query_text.get_mut(text_entity) {
                    if text.0 != ev.name {
                        log_proc!(debug, "Updating black player name to {}", ev.name);
                        text.0 = ev.name.clone();
                    }
                }
            }
        }
    }
}

/// Triggered when the user resizes the window.
/// The UI elements will resize automatically as they are created with a
/// percentage width/height in `chess_ui_scene()`. The board, however,
/// is not a UI element, so we need to resize it manually.
fn on_resize(
    mut resize_reader: MessageReader<bevy::window::WindowResized>,
    mut board_transform: Single<&mut Transform, With<ChessBoard>>,
) {
    // Collect the latest window size.
    let mut last_size = None;
    for e in resize_reader.read() {
        last_size = Some(Vec2::new(e.width, e.height));
    }
    let Some(win_size) = last_size else {
        return;
    };

    resize_board(win_size, &mut board_transform);
}

/// helper method to resize the board.
fn resize_board(win_size: Vec2, board_transform: &mut Transform) {
    // Keep the board inside the central area, below the top label, above the bottom label,
    // and away from all four neighboring UI regions.
    let available_width = (win_size.x * BOARD_AREA_RATIO - 2.0 * BOARD_EDGE_GAP).max(1.0);
    let label_reserved_height = PLAYER_LABEL_OFFSET + PLAYER_LABEL_HEIGHT + BOARD_EDGE_GAP;
    let available_height = (win_size.y - 2.0 * label_reserved_height).max(1.0);
    let board_size = available_width.min(available_height);
    let scale = board_size / BOARD_SIZE;

    board_transform.scale = Vec3::splat(scale);
    board_transform.translation.x = (LEFT_PANEL_RATIO - RIGHT_PANEL_RATIO) * win_size.x / 2.0;
}

fn on_rotate(
    _ev: On<RotateBoardEvent>,
    mut board: Single<&mut Transform, With<ChessBoard>>,
    mut white_node: Single<&mut Node, With<WhitePlayerLabel>>,
    mut black_node: Single<&mut Node, (With<BlackPlayerLabel>, Without<WhitePlayerLabel>)>,
    mut commands: Commands,
    window_query: Single<(Entity, &Window)>,
) {
    log_proc!(debug, "Rotating game view");

    // rotate the board 180'
    board.rotate_z(PI);
    let is_rotated = board.rotation.z.abs() > 0.1;

    // Update Player Labels
    let set_label = |node: &mut Node, is_top: bool| {
        if is_top {
            node.top = Val::Px(PLAYER_LABEL_OFFSET);
            node.bottom = Val::Auto;
        } else {
            node.bottom = Val::Px(PLAYER_LABEL_OFFSET);
            node.top = Val::Auto;
        }
    };

    // White is normally at the bottom, Black at the top.
    // Swap if the board is rotated.
    set_label(&mut white_node, is_rotated);
    set_label(&mut black_node, !is_rotated);

    // Trigger a resize to update UI node positions
    let (entity, window) = *window_query;
    let win_size = window.size();
    commands.write_message(bevy::window::WindowResized {
        window: entity,
        width: win_size.x,
        height: win_size.y,
    });
}

pub fn on_draw_offered(
    ev: On<DrawOffered>,
    button: Single<(Entity, &ThemeBackgroundColor), With<DrawOfferButton>>,
    theme: Res<UiTheme>,
    mut commands: Commands,
) {
    let (entity, bg_token) = *button;
    let context = if ev.0 {
        SurfaceLevel::Higher
    } else {
        SurfaceLevel::Base
    };
    commands.entity(entity).insert((
        ThemeContext(context),
        BackgroundColor(theme.context_color(&bg_token.0, context)),
    ));
}