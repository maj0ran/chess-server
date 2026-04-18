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
        children![
            (
                WhitePlayerLabel,
                Text2d::new("Waiting for White..."),
                Transform::from_xyz(0.0, -450.0, 1.0)
            ),
            (
                BlackPlayerLabel,
                Text2d::new("Waiting for Black..."),
                Transform::from_xyz(0.0, 450.0, 1.0),
            ),
        ],
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
    mut query_white: Query<&mut Text2d, With<WhitePlayerLabel>>,
    mut query_black: Query<&mut Text2d, (With<BlackPlayerLabel>, Without<WhitePlayerLabel>)>,
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
    mut history_query: Query<&mut TextFont, With<MoveHistory>>,
    mut mh_container: Query<&mut Node, With<MoveHistoryContainer>>,
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

    // Apply the final scale to the game container (board and player labels).
    container.1.scale = Vec3::splat(scale);

    // Position and size the Move History UI Node.
    // The UI coordinate system usually starts at top-left for Nodes.
    let board_px = RESOLUTION * scale;
    let padding_px = history_padding * scale;
    let history_px = history_base_width * scale;

    // Center of window + half board width + padding
    mh.left = Val::Px(win_size.x / 2.0 + board_px / 2.0 + padding_px);
    // Align top of history with top of the board (board is centered vertically)
    mh.top = Val::Px((win_size.y - board_px) / 2.0);

    mh.width = Val::Px(history_px);
    mh.height = Val::Px(board_px);

    // 5. Scale the font size to match the UI scaling.
    let mut font = history_query.single_mut().unwrap();
    font.font_size = 24.0 * scale;
}
pub fn on_rotate(
    _ev: On<RotateBoardEvent>,
    mut query: ParamSet<(
        Query<&mut Transform, With<ChessBoard>>,
        Query<&mut Transform, With<WhitePlayerLabel>>,
        Query<&mut Transform, With<BlackPlayerLabel>>,
    )>,
) {
    log::debug!("Rotating game view");

    // rotate the board 180'
    {
        let mut board_query = query.p0();
        let mut board = board_query.single_mut().unwrap();
        board.rotate_z(PI);
    }

    // switch player label positions
    {
        let mut query_white = query.p1();
        let mut white_label = query_white.single_mut().unwrap();
        white_label.translation.y = -white_label.translation.y;
    }
    {
        let mut query_black = query.p2();
        let mut black_label = query_black.single_mut().unwrap();
        black_label.translation.y = -black_label.translation.y;
    }
}
