use super::GameScreenComponent;
use crate::client::game::{ActiveGame, GameJoinedEvent};
use crate::client::lobby::LobbyState;
use crate::ui::views::gameview::chessboard::board::{ChessBoard, RotateBoardEvent};
use crate::ui::views::gameview::historypanel::movehistory::{MoveHistory, update_move_history};
use crate::ui::{Overlay, Screen};
use bevy::prelude::*;
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

#[derive(Component)]
pub struct WhitePlayerLabel;
#[derive(Component)]
pub struct BlackPlayerLabel;

#[derive(Event)]
pub struct GameScreenInitialized;

/// Sets up the in-game screen.
/// Draws the chessboard and triggers a `BoardUpdate` event to trigger piece position retrievement.
fn setup_gamescreen(mut commands: Commands, win_query: Query<(Entity, &Window)>) {
    log::info!("Setting up gamescreen");

    commands.spawn(ChessBoard::new());

    commands.spawn((
        GameScreenComponent,
        WhitePlayerLabel,
        Node {
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::srgb(1.0, 0.0, 0.0)),
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
        BackgroundColor(Color::srgb(1.0, 0.0, 0.0)),
        Text::new("Waiting for Black..."),
        TextFont {
            font_size: 24.0,
            ..default()
        },
    ));

    commands.spawn(MoveHistory::new());

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

/// Computes scale and position of all game screen components based on the current window size.
/// This method is kinda complex due to the need to adjust various UI elements dynamically based on
/// window size and available screen space.
pub fn on_resize(
    mut resize_reader: MessageReader<bevy::window::WindowResized>,
    mut queries: ParamSet<(
        Single<&mut Transform, With<ChessBoard>>,
        Single<(&mut Node, &mut TextFont), With<MoveHistory>>,
        Single<(&mut Node, &mut TextFont), With<WhitePlayerLabel>>,
        Single<(&mut Node, &mut TextFont), With<BlackPlayerLabel>>,
    )>,
) {
    // Get the new window size
    let mut new_size = None;
    for e in resize_reader.read() {
        new_size = Some(Vec2::new(e.width, e.height));
    }
    let Some(win_size) = new_size else {
        return;
    };

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

    /////////////////
    // Board scale //
    /////////////////
    let is_rotated = {
        let mut board_transform = queries.p0();

        board_transform.scale = Vec3::splat(scale);
        // we need the orientation later for the player labels
        let is_rotated = board_transform.rotation.z.abs() > 0.1; // PI rotation check
        is_rotated
    };

    let board_px = RESOLUTION * scale;
    let padding_px = history_padding * scale;
    let history_px = history_base_width * scale;

    //////////////////
    // Move History //
    //////////////////
    {
        let (mut mh_node, mut mh_font) = queries.p1().into_inner();
        // Center of window + half board width + padding
        mh_node.left = Val::Px(win_size.x / 2.0 + board_px / 2.0 + padding_px);
        // Align top of history with top of the board (board is centered vertically)
        mh_node.top = Val::Px((win_size.y - board_px) / 2.0);

        mh_node.width = Val::Px(history_px);
        mh_node.height = Val::Px(board_px);

        // Scale the font sizes
        mh_font.font_size = 24.0 * scale;
    }

    // Player Labels - Compute separately because we can borrow only one part from the query.
    let board_top = (win_size.y - board_px) / 2.0;
    let board_bottom = (win_size.y + board_px) / 2.0;
    let board_center_x = win_size.x / 2.0;
    let label_width = RESOLUTION * scale;
    let label_size = 32.0;

    ////////////////////////
    // White Player Label //
    ////////////////////////
    {
        let (mut white_node, mut white_font) = queries.p2().into_inner();

        white_font.font_size = label_size * scale;
        if !is_rotated {
            white_node.top = Val::Px(board_bottom + padding_px);
            white_node.bottom = Val::Auto;
        } else {
            white_node.bottom = Val::Px(win_size.y - board_top + padding_px);
            white_node.top = Val::Auto;
        }
        white_node.width = Val::Px(label_width);
        white_node.left = Val::Px(board_center_x - label_width / 2.0);
        white_node.justify_content = JustifyContent::Center;
    }
    ////////////////////////
    // Black Player Label //
    ////////////////////////
    {
        let (mut black_node, mut black_font) = queries.p3().into_inner();

        black_font.font_size = label_size * scale;
        if !is_rotated {
            black_node.bottom = Val::Px(win_size.y - board_top + padding_px);
            black_node.top = Val::Auto;
        } else {
            black_node.top = Val::Px(board_bottom + padding_px);
            black_node.bottom = Val::Auto;
        }
        black_node.width = Val::Px(label_width);
        black_node.left = Val::Px(board_center_x - label_width / 2.0);
        black_node.justify_content = JustifyContent::Center;
    }
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
