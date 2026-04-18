use super::GameScreenComponent;
use crate::ClassList;
use crate::client::game::{ActiveGame, GameJoinedEvent};
use crate::client::lobby::LobbyState;
use crate::ui::views::gameview::chessboard::board::{ChessBoard, RotateBoardEvent};
use crate::ui::{Overlay, Screen};
use bevy::prelude::*;
use bevy::text::TextBounds;
use bevy::ui::AlignItems::Center;
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
                update_player_names.run_if(in_state(Screen::Game)).run_if(
                    resource_exists::<ActiveGame>.and(
                        resource_exists_and_changed::<LobbyState>
                            .or(resource_exists_and_changed::<ActiveGame>),
                    ),
                ),
            )
            .add_observer(on_game_joined)
            .add_observer(on_rotate)
            .add_observer(update_move_history)
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

/// Container for all game-screen elements. The board, player names, move history...
/// We use this container to scale all elements when resizing the window.
#[derive(Component)]
pub struct GameScreenContainer;

#[derive(Component)]
pub struct WhitePlayerLabel;
#[derive(Component)]
pub struct BlackPlayerLabel;

#[derive(Component)]
pub struct MoveHistory;
#[derive(Component)]
pub struct MoveHistoryContainer;
#[derive(Event)]
pub struct MoveHistoryUpdated;

#[derive(Event)]
pub struct GameScreenInitialized;

/// Sets up the in-game screen.
/// Draws the chessboard and triggers a `BoardUpdate` event to trigger piece position retrievement.
fn setup_gamescreen(mut commands: Commands, win_query: Query<(Entity, &Window)>) {
    log::info!("Setting up gamescreen");

    commands.spawn((
        GameScreenComponent,
        GameScreenContainer,
        Transform::default(),
    ));

    commands.spawn((
        GameScreenComponent,
        WhitePlayerLabel,
        Node {
            position_type: PositionType::Absolute,
            ..default()
        },
        Text::new("Waiting for White..."),
        TextFont {
            font_size: 24.0,
            ..default()
        },
    ));

    commands.spawn((
        GameScreenComponent,
        BlackPlayerLabel,
        Node {
            position_type: PositionType::Absolute,
            ..default()
        },
        Text::new("Waiting for Black..."),
        TextFont {
            font_size: 24.0,
            ..default()
        },
    ));

    commands.spawn(((
        GameScreenComponent,
        MoveHistoryContainer,
        Node {
            position_type: PositionType::Absolute,
            height: Val::Percent(100.0),
            width: Val::Px(250.0),
            top: Val::Px(100.0),
            justify_items: JustifyItems::Start,
            ..default()
        },
        BackgroundColor(Color::srgb(1.0, 0.0, 0.0)),
        children![(
            MoveHistory,
            Text::new(""),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextLayout {
                justify: Justify::Justified,
                ..default()
            },
        )],
    ),));

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
fn cleanup_gamescreen(mut commands: Commands, query: Query<Entity, With<GameScreenComponent>>) {
    log::debug!("Cleaning up gamescreen");

    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
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
    mut query_white: Query<&mut Text, With<WhitePlayerLabel>>,
    mut query_black: Query<&mut Text, (With<BlackPlayerLabel>, Without<WhitePlayerLabel>)>,
) {
    let game_info = game.game_info;

    if let Ok(mut white_text) = query_white.single_mut() {
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

    if let Ok(mut black_text) = query_black.single_mut() {
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

fn update_move_history(
    _ev: On<MoveHistoryUpdated>,
    mut query_display: Query<&mut Text, With<MoveHistory>>,
    history: ResMut<ActiveGame>,
) {
    if let Ok(mut text) = query_display.single_mut() {
        log::debug!("Updating move history");
        let last_move = &history.move_history.last().unwrap();
        text.0.push_str(&format!("{} ", last_move));
        if &history.move_history.len() % 2 == 0 {
            text.0.push_str("\n");
        } else {
            let spacing = " ".repeat(10 - last_move.len());
            text.0.push_str(&spacing);
        }
    }
}

pub fn on_resize(
    mut resize_reader: MessageReader<bevy::window::WindowResized>,
    mut container_query: Query<(Entity, &mut Transform), With<GameScreenContainer>>,
    mut history_query: Query<
        &mut TextFont,
        (
            With<MoveHistory>,
            Without<WhitePlayerLabel>,
            Without<BlackPlayerLabel>,
            Without<MoveHistoryContainer>,
        ),
    >,
    mut mh_container: Query<
        &mut Node,
        (
            With<MoveHistoryContainer>,
            Without<WhitePlayerLabel>,
            Without<BlackPlayerLabel>,
        ),
    >,
    mut white_label_query: Query<
        (&mut Node, &mut TextFont),
        (
            With<WhitePlayerLabel>,
            Without<MoveHistoryContainer>,
            Without<BlackPlayerLabel>,
        ),
    >,
    mut black_label_query: Query<
        (&mut Node, &mut TextFont),
        (
            With<BlackPlayerLabel>,
            Without<MoveHistoryContainer>,
            Without<WhitePlayerLabel>,
        ),
    >,
    board_query: Query<&Transform, (With<ChessBoard>, Without<GameScreenContainer>)>,
) {
    let mut new_size = None;
    for e in resize_reader.read() {
        new_size = Some(Vec2::new(e.width, e.height));
    }
    let Some(win_size) = new_size else {
        return;
    };

    let mut container = container_query.single_mut().unwrap();
    let mut mh = mh_container.single_mut().unwrap();

    // Initial scale calculation based on the smallest window dimension.
    // We want the board to take up about 75% of the screen height or width initially.
    let min_dim = win_size.x.min(win_size.y);
    let mut scale = (min_dim / RESOLUTION) * 0.75;

    // Layout constants for the Move History panel.
    let history_base_width = 250.0; // Width of history panel at scale 1.0
    let history_padding = 20.0; // Gap between board and history panel

    // Ensure the Move History panel fits within the window.
    // The board is centered at (win_size.x / 2.0).
    // Its right edge is at: (win_size.x / 2.0) + (RESOLUTION * scale / 2.0).
    // We need additional space for padding and the history panel width.
    let half_res = RESOLUTION / 2.0;
    let total_right_offset_base = half_res + history_padding + history_base_width;

    let current_right_edge_px = total_right_offset_base * scale;
    let available_right_space_px = win_size.x / 2.0;

    // If the history panel goes off-screen, we downscale everything to fit.
    if current_right_edge_px > available_right_space_px {
        // We add a small 0.95 factor to keep it away from the very edge of the window.
        scale = (available_right_space_px / total_right_offset_base) * 0.95;
    }

    // Apply the final scale to the game container (board).
    container.1.scale = Vec3::splat(scale);

    // Position and size the Move History UI Node.
    let board_px = RESOLUTION * scale;
    let padding_px = history_padding * scale;
    let history_px = history_base_width * scale;

    // Center of window + half board width + padding
    mh.left = Val::Px(win_size.x / 2.0 + board_px / 2.0 + padding_px);
    // Align top of history with top of the board (board is centered vertically)
    mh.top = Val::Px((win_size.y - board_px) / 2.0);

    mh.width = Val::Px(history_px);
    mh.height = Val::Px(board_px);

    // Scale the font sizes
    let mut history_font = history_query.single_mut().unwrap();
    history_font.font_size = 24.0 * scale;

    let (mut white_node, mut white_font) = white_label_query.single_mut().unwrap();
    let (mut black_node, mut black_font) = black_label_query.single_mut().unwrap();

    white_font.font_size = 32.0 * scale;
    black_font.font_size = 32.0 * scale;

    // Position player labels relative to the board
    // They should be centered horizontally and slightly above/below the board
    let label_offset = 20.0 * scale;
    let board_top = (win_size.y - board_px) / 2.0;
    let board_bottom = (win_size.y + board_px) / 2.0;
    let board_center_x = win_size.x / 2.0;

    // Determine which label goes where based on board rotation
    let is_rotated = if let Ok(bt) = board_query.single() {
        bt.rotation.z.abs() > 0.1 // PI rotation check
    } else {
        false
    };

    if !is_rotated {
        // White is bottom, Black is top
        white_node.top = Val::Px(board_bottom + label_offset);
        white_node.bottom = Val::Auto;
        black_node.bottom = Val::Px(win_size.y - board_top + label_offset);
        black_node.top = Val::Auto;
    } else {
        // White is top, Black is bottom
        white_node.bottom = Val::Px(win_size.y - board_top + label_offset);
        white_node.top = Val::Auto;
        black_node.top = Val::Px(board_bottom + label_offset);
        black_node.bottom = Val::Auto;
    }

    // Center labels horizontally (using left with a trick or just absolute pos)
    // To center an absolute node, we can't easily use justify_self if parent is not a node.
    // But they are at the root. So we use left: 50% and transform: translateX(-50%) if possible,
    // but here we can just calculate left: (win_size.x / 2.0) and hope they are small or we use a container.
    // Actually, we should probably give them a width and set left.
    let label_width = 400.0 * scale;
    white_node.width = Val::Px(label_width);
    black_node.width = Val::Px(label_width);
    white_node.left = Val::Px(board_center_x - label_width / 2.0);
    black_node.left = Val::Px(board_center_x - label_width / 2.0);
    white_node.justify_content = JustifyContent::Center;
    black_node.justify_content = JustifyContent::Center;
}
pub fn on_rotate(
    _ev: On<RotateBoardEvent>,
    mut query: Query<&mut Transform, With<ChessBoard>>,
    mut commands: Commands,
    win_query: Query<(Entity, &Window)>,
) {
    log::debug!("Rotating game view");

    // rotate the board 180'
    for mut board in query.iter_mut() {
        board.rotate_z(PI);
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
