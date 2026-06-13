use crate::backend::net::NetworkSend;
use crate::backend::{ActiveGame, GameJoinedEvent, LobbyState};
use crate::ui::gamescreen::chessboard::{BOARD_SIZE, ChessBoard, ChessboardPlugin, RotateBoardEvent};
use crate::ui::gamescreen::dialogs::GamescreenDialogsPlugin;
use crate::ui::gamescreen::move_history::*;
use crate::ui::theme::*;
use crate::ui::{Overlay, Screen};
use bevy::feathers::controls::FeathersButton;
use bevy::feathers::display::label;
use bevy::feathers::theme::{InheritableThemeTextColor, ThemedText};
use bevy::feathers::tokens;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use chess_core::ClientId;
use chess_core::protocol::messages::ClientMessage;
use std::f32::consts::PI;

/// Marker component for everything that belongs to the in-game screen.
#[derive(Component, Clone, Copy, Default)]
pub struct ChessSceneRootNode;

pub const SOURCE_COLOR: Color = Color::srgb_u8(250, 113, 113);
pub const DESTINATION_COLOR: Color = Color::srgb_u8(113, 250, 113);

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
                    update_player_names.run_if(
                        resource_exists::<ActiveGame>.and_then(
                            resource_exists_and_changed::<LobbyState>
                                .or_else(resource_exists_and_changed::<ActiveGame>),
                        ),
                    ),
                    (send_scroll_events, resize_move_history_text),
                )
                    .run_if(in_state(Screen::Game)),
            )
            .add_observer(on_game_joined)
            .add_observer(on_rotate)
            .add_observer(update_move_history)
            .add_observer(refresh_move_history)
            .add_observer(on_scroll_handler)
            .add_observer(on_draw_offered)
            .add_systems(
                Update,
                (
                    on_resize,
                    update_draw_offer_button,
                )
                    .run_if(in_state(Screen::Game)),
            )
        .add_plugins(GamescreenDialogsPlugin);
    }
}

fn on_game_joined(
    _ev: On<GameJoinedEvent>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_overlay: ResMut<NextState<Overlay>>,
) {
    next_screen.set(Screen::Game);
    next_overlay.set(Overlay::None);
}

#[derive(Component, Clone, Copy, Default)]
pub struct WhitePlayerLabel;
#[derive(Component, Clone, Copy, Default)]
pub struct BlackPlayerLabel;
#[derive(Component, Clone, Copy, Default)]
pub struct GameActionButton;
#[derive(Component, Clone, Copy, Default)]
pub struct DrawOfferButton;
#[derive(Component, Clone, Copy, Default)]
pub struct DrawOfferActive;
#[derive(Event)]
pub struct ChessGameInitialized;
#[derive(Event)]
pub struct DrawOffered(pub bool);

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
            BackgroundColor(THEME_SURFACE)
            BorderColor::all(THEME_BORDER)
            WhitePlayerLabel
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
            BackgroundColor(THEME_SURFACE)
            BorderColor::all(THEME_BORDER)
            BlackPlayerLabel
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
                GameActionButton
                Node {
                    width: percent(80),
                    height: Val::Auto,
                    aspect_ratio: { Some(1.0) },
                    justify_content: JustifyContent::Center, // horizontal alignment
                    align_items: AlignItems::Center, // vertical alignment ... the fuck is up with these CSS property names?!
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(8))
                }
                on(|_: On<Activate>, mut overlay: ResMut<NextState<Overlay>>| overlay.set(Overlay::Resign))
                --
                @FeathersButton { @caption: bsn! { @label("D") ThemedText } }
                GameActionButton
                DrawOfferButton
                Node {
                    width: percent(80),
                    height: Val::Auto,
                    aspect_ratio: { Some(1.0) },
                    justify_content: JustifyContent::Center, // horizontal alignment
                    align_items: AlignItems::Center, // vertical alignment ... the fuck is up with these CSS property names?!
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(8))
                }
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
fn setup_chessboard(mut commands: Commands, window_query: Single<(Entity, &Window)>) {
    log::info!("Setting up chessboard");

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

    commands.trigger(ChessGameInitialized);
}

/// Despawn all entities that are part of the chess game screen,
/// that is, the UI and the chessboard.
/// Obviously happens when we leave a game.
fn cleanup_chess_scene(
    mut commands: Commands,
    query_ui: Single<Entity, With<ChessSceneRootNode>>,
    query_board: Single<Entity, With<ChessBoard>>,
) {
    log::debug!("Cleaning up chess scene");

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

fn update_player_names(
    game: Res<ActiveGame>,
    lobby: Res<LobbyState>,
    query_white: Single<&Children, With<WhitePlayerLabel>>,
    query_black: Single<&Children, (With<BlackPlayerLabel>, Without<WhitePlayerLabel>)>,
    mut query_text: Query<&mut Text>,
) {
    let game_info = game.game_info;

    update_player_label(
        &query_white,
        game_info.white_player,
        "Waiting for White...",
        &lobby,
        &mut query_text,
    );
    update_player_label(
        &query_black,
        game_info.black_player,
        "Waiting for Black...",
        &lobby,
        &mut query_text,
    );
}

fn update_player_label(
    children: &Children,
    player_id: Option<ClientId>,
    waiting_text: &str,
    lobby: &LobbyState,
    query_text: &mut Query<&mut Text>,
) {
    let Some(&text_entity) = children.first() else {
        return;
    };
    let Ok(mut text) = query_text.get_mut(text_entity) else {
        return;
    };

    let name = player_id
        .map(|id| {
            lobby
                .get_client_info(id)
                .cloned()
                .unwrap_or_else(|| format!("Player {id}"))
        })
        .unwrap_or_else(|| waiting_text.to_string());

    if text.0 != name {
        log::debug!("Updating player name to {name}");
        text.0 = name;
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

fn update_draw_offer_button(
    mut buttons: Query<(&mut BackgroundColor, Option<&DrawOfferActive>), With<DrawOfferButton>>,
) {
    for (mut background, active) in &mut buttons {
        background.0 = if active.is_some() {
            THEME_PENDING_DRAW_OFFER
        } else {
            THEME_BUTTON
        };
    }
}

fn on_rotate(
    _ev: On<RotateBoardEvent>,
    mut board: Single<&mut Transform, With<ChessBoard>>,
    mut white_node: Single<&mut Node, With<WhitePlayerLabel>>,
    mut black_node: Single<&mut Node, (With<BlackPlayerLabel>, Without<WhitePlayerLabel>)>,
    mut commands: Commands,
    window_query: Single<(Entity, &Window)>,
) {
    log::debug!("Rotating game view");

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

fn on_draw_offered(
    ev: On<DrawOffered>,
    mut commands: Commands,
    button: Single<Entity, With<DrawOfferButton>>,
) {
    log::debug!("Draw offer state changed: {}", ev.0);

    if ev.0 {
        commands.entity(*button).insert(DrawOfferActive);
    } else {
        commands.entity(*button).remove::<DrawOfferActive>();
    }
}
