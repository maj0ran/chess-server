use super::GameScreenComponent;
use crate::client::game::{ActiveGame, GameJoinedEvent};
use std::f32::consts::PI;

use crate::client::lobby::LobbyState;
use crate::ui::views::gameview::chessboard::board::{ChessBoard, RotateBoardEvent};
use crate::ui::{Overlay, Screen};
use bevy::prelude::*;

pub const SOURCE_COLOR: Color = Color::srgb_u8(250, 113, 113);
pub const DESTINATION_COLOR: Color = Color::srgb_u8(113, 250, 113);
pub const RESOLUTION: f32 = 1024.0;

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

#[derive(Event)]
pub struct GameScreenInitialized;

/// Sets up the in-game screen.
/// Draws the chessboard and triggers a `BoardUpdate` event to trigger piece position retrievement.
fn setup_gamescreen(mut commands: Commands) {
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
                Transform::from_xyz(0.0, 450.0, 1.0)
            )
        ],
    ));

    commands.trigger(GameScreenInitialized);
}
/// Despawn all entities that are part of the in-game screen.
/// Obviously happens when we leave a game.
fn cleanup_gamescreen(mut commands: Commands, query: Query<Entity, With<GameScreenComponent>>) {
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

pub fn on_resize(
    mut resize_reader: MessageReader<bevy::window::WindowResized>,
    mut container_query: Query<(Entity, &mut Transform), With<GameScreenContainer>>,
) {
    // Trigger only when there was at least one resize event
    let mut new_size = None;
    for e in resize_reader.read() {
        new_size = Some(Vec2::new(e.width, e.height));
    }
    let Some(size) = new_size else {
        return;
    };

    let mut container = container_query.single_mut().unwrap(); // we're in-game, so a board must exist.

    let size = if size.x < size.y { size.x } else { size.y };

    container.1.scale.x = size / RESOLUTION;
    container.1.scale.y = size / RESOLUTION;
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
