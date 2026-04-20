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

    commands.spawn((
        GameScreenComponent,
        NodeStyleSheet::new(asset_server.load("style.css")),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            height: Val::Percent(100.0),
            width: Val::Auto,
            justify_content: JustifyContent::Center,
            justify_items: JustifyItems::Center,
            align_items: AlignItems::Center,
            justify_self: JustifySelf::Center,
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
        Single<(&mut Node, &mut TextFont), With<WhitePlayerLabel>>,
        Single<(&mut Node, &mut TextFont), With<BlackPlayerLabel>>,
    )>,
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

    // 1. Calculate the Scaling Factor
    // -------------------------------------------------------------------------
    // The board's intrinsic size is RESOLUTION (800x800).
    // Initially, we want the board to occupy 75% of the shortest window dimension.
    let min_dim = win_size.x.min(win_size.y);
    let mut scale = (min_dim / RESOLUTION) * 0.75;

    // Layout constants for unscaled UI (as if resolution was 1:1)
    let half_res = RESOLUTION / 2.0;
    let padding = 20.0;
    let history_width = 400.0;

    // Ensure board + history fits horizontally without overlap.
    // Board is centered, so we need space on the right for half the board + history.
    let right_side_needed = half_res + padding + history_width;
    let available_right_space = win_size.x / 2.0;

    if (right_side_needed * scale) > available_right_space {
        scale = available_right_space / right_side_needed;
    }

    // 2. Apply Scale to the chess board
    // -------------------------------------------------------------------------
    // The board is not an UI element, so it's NOT affected by UiScale.
    // We scale it manually.
    let is_rotated = {
        let mut board_transform = queries.p0();
        board_transform.scale = Vec3::splat(scale);
        board_transform.rotation.z.abs() > 0.1 // Rotation check for labels
    };

    // 3. UI Positioning using "Unscaled" Pixels
    // -------------------------------------------------------------------------
    // Since we use Bevy's UiScale, all Val::Px(x) values will be MULTIPLIED by `scale`.
    // To place a UI element at an ABSOLUTE screen position (P), we must set its
    // Val::Px value to (P / scale).
    // Elements defined relative to the board (which uses `scale`) use base constants.

    // Center of the window in "UI-scaled" coordinate space.
    let ui_center_x = (win_size.x / 2.0) / scale;
    let ui_center_y = (win_size.y / 2.0) / scale;

    // Board bounds in "UI-scaled" space (matching the board's visual size).
    let board_top = ui_center_y - half_res;
    let board_bottom = ui_center_y + half_res;

    // Update Move History
    // It's anchored on the right in MoveHistory::new() and scales with UiScale.

    // Update Player Labels
    let label_size = 32.0;
    let label_width = RESOLUTION;

    // Helper to position a label either above or below the board.
    let set_label = |node: &mut Node, font: &mut TextFont, is_top: bool| {
        font.font_size = label_size;
        node.width = Val::Px(label_width);
        node.left = Val::Px(ui_center_x - label_width / 2.0);
        node.justify_content = JustifyContent::Center;

        if is_top {
            // "Unscaled" distance from top to place it above the board.
            node.bottom = Val::Px((win_size.y / scale) - board_top + padding);
            node.top = Val::Auto;
        } else {
            // "Unscaled" distance from top to place it below the board.
            node.top = Val::Px(board_bottom + padding);
            node.bottom = Val::Auto;
        }
    };

    // White is normally at the bottom, Black at the top.
    // Swap if the board is rotated.
    {
        let (mut white_node, mut white_font) = queries.p1().into_inner();
        set_label(&mut white_node, &mut white_font, is_rotated);
    }
    {
        let (mut black_node, mut black_font) = queries.p2().into_inner();
        set_label(&mut black_node, &mut black_font, !is_rotated);
    }

    // Finally, update the global UI scale.
    ui_scale.0 = scale;
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
