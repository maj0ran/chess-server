use super::super::GameScreenComponent;
use super::assets::ChessAssets;
use super::piece::ChessPiece;
use super::square::ChessSquare;
use super::*;
use crate::backend::client::{BoardUpdate, ClientBackend, Overlay};
use crate::ui::{COLOR_LIGHT2, COLOR_MID};

use crate::ui::views::gameview::game_screen::{DESTINATION_COLOR, SOURCE_COLOR};
use bevy::color::Color;
use bevy::math::{Vec2, Vec3};

/// Event from the backend that is triggered when the server has accepted or rejected a move.
#[derive(Event)]
pub struct ResetSelection;

/// The `ChessBoard` is a simple plane that acts as a reference position for anything else.
/// The plane alone isn't even chessy - it has no squares. Squares are standalone components
/// that will be rendered onto the board.
#[derive(Component)]
pub struct ChessBoard;

impl ChessBoard {
    pub fn new() -> (ChessBoard, GameScreenComponent, Sprite, Transform) {
        (
            ChessBoard,
            GameScreenComponent,
            Sprite {
                color: Color::srgb_u8(150, 50, 150),
                custom_size: Some(Vec2::splat(BOARD_SIZE)),
                ..default()
            },
            Transform {
                translation: Vec3::new(0.0, 0.0, 1.0),
                ..default()
            },
        )
    }
}
/// Helper method to draw the chessboard. Spawns the `ChessBoard` plane and
/// 64 `ChessSquare` as children onto it.
pub fn draw_chessboard(commands: &mut Commands) {
    commands.spawn(ChessBoard::new()).with_children(|parent| {
        let mut file = 'a';
        let mut rank = '1';
        for r in 0..8 {
            for f in 0..8 {
                let color = if (f + r) % 2 == 0 {
                    COLOR_LIGHT2
                } else {
                    COLOR_MID
                };
                let name = format!("{file}{rank}");
                parent
                    .spawn(ChessSquare::new(
                        name,
                        Vec2::new(
                            (-BOARD_SIZE / 2.0) + (SQUARE_SIZE / 2.0) + (f as f32 * SQUARE_SIZE),
                            (-BOARD_SIZE / 2.0) + (SQUARE_SIZE / 2.0) + (r as f32 * SQUARE_SIZE),
                        ),
                        color,
                    ))
                    .observe(select_square);

                file = ((file as u8) + 1) as char;
                if file > 'h' {
                    file = 'a';
                    rank = ((rank as u8) + 1) as char;
                }
            }
        }
    });
}

/// Triggered on `BoardUpdate` event which is sent by the backend when it receives an update from the server.
/// Draws all pieces according to the internal board state.
pub fn draw_pieces(
    _board_update: On<BoardUpdate>,
    mut commands: Commands,
    assets: Res<ChessAssets>,
    squares: Query<(Entity, &ChessSquare), With<ChessSquare>>,
    backend: ResMut<ClientBackend>,
) {
    let pieces = &backend.game_state.internal_board;

    // we iterate through all squares. We remove the piece on every square from
    // the previous draw, no matter what. Then we draw the current piece on the square.
    // TODO: this could be better by just updating those squares that have changed.
    for s in squares {
        let square_entity = s.0;
        let square_name = &s.1.name;
        let p = pieces.get(square_name);

        commands.entity(square_entity).despawn_children();
        match p {
            None => {}
            Some(p) => {
                let piece_entity = commands.spawn((ChessPiece::new(*p, &assets),)).id();
                commands.entity(square_entity).add_child(piece_entity);
            }
        };
    }
}

/// Observer that is connected to each square. It is triggered when the square is clicked.
/// This adds the clicked square to the global resource for either source or destination selection
/// and colors the square accordingly. If both, source and destination, is selected after the click,
/// a `RequestMove` event is triggered that is caught by the backend to send the move to the server.
/// When the backend receives the answer to the move (accepted or rejected), it will again trigger an event
/// that is caught by `reset_selections` (below) to undo the coloring.
pub fn select_square(
    ev: On<Pointer<Press>>,
    mut src: ResMut<SourceSelect>,
    mut dst: ResMut<DestinationSelect>,
    mut query: Query<&mut Sprite, With<ChessSquare>>,
) {
    let clicked: Entity = ev.event_target();

    if src.entity == None {
        if let Ok(mut sprite) = query.get_mut(clicked) {
            src.entity = Some(clicked);
            sprite.color = SOURCE_COLOR;
        }
    } else if dst.entity == None {
        if let Ok(mut sprite) = query.get_mut(clicked) {
            dst.entity = Some(clicked);
            sprite.color = DESTINATION_COLOR;
        }
    }
}

/// Handles a chess move.
/// This system is automatically called when the DestinationSelect Resource is changed.
/// Which happens after the user clicks two squares on the chessboard.
/// Here we construct the move from the selected squares. There are two cases:
///     - A normal move like d4d5, which will trigger the `RequestMove` event to send the move
/// to the backend.
///     - A promotion move that happens when a pawn moves on the last rank. This will store the move
/// in the `PendingMove` resource and sets `Overlay::Promotion`, which will spawn the promotion dialog.
/// When a promotion button on this dialog is clicked, it will load the `PendingMove` resource. There, we
/// trigger `RequestMove` together with the promotion information.
pub fn handle_move(
    mut commands: Commands,
    src: ResMut<SourceSelect>,
    dst: ResMut<DestinationSelect>,
    query: Query<(&ChessSquare, Option<&Children>)>,
    piece_query: Query<&ChessPiece>,
    mut next_overlay: ResMut<NextState<Overlay>>,
) {
    let src_entity = src.entity.unwrap();
    let dst_entity = dst.entity.unwrap();

    let (src_sq, src_children) = query.get(src_entity).unwrap();
    let (dst_sq, _) = query.get(dst_entity).unwrap();

    /* Some hassle to get the piece that sits on the square... */
    let chess_piece = src_children
        .and_then(|children| children.last())
        .map(|&p| piece_query.get(p).unwrap());

    if let Some(piece) = chess_piece {
        // is it a promotion move?...
        let is_pawn = piece.id == 'P' || piece.id == 'p';
        let is_last_rank = dst_sq.name.ends_with('8') || dst_sq.name.ends_with('1');
        if is_pawn && is_last_rank {
            let pending_move = PendingMove {
                src: src_sq.name.clone(),
                dst: dst_sq.name.clone(),
            };
            // ...save the move in the pending move resource and call the promotion dialog...
            commands.insert_resource(pending_move);
            next_overlay.set(Overlay::Promotion);
        } else {
            // ...otherwise, it's a normal move, so we can send the move to the backend directly.
            commands.trigger(RequestMove {
                source: src_sq.name.clone(),
                destination: dst_sq.name.clone(),
                promotion: None,
            });
        }
    } else {
        // no piece on source square, do nothing except reset selection
        commands.trigger(ResetSelection);
    }
}

/// Triggered by the backend when it receives a response on a move request.
/// I.e., the backend sends a move to the server, waits for a response, and on the response,
/// it sends back an event here to reset the selection.
pub fn reset_selections(
    _ev: On<ResetSelection>,
    mut src: ResMut<SourceSelect>,
    mut dst: ResMut<DestinationSelect>,
    mut query: Query<(Entity, &mut ChessSquare, &mut Sprite)>,
) {
    for square_entity in query.iter_mut() {
        let entity = square_entity.0;
        let square = square_entity.1;
        let mut sprite = square_entity.2;

        if src.entity == Some(entity) {
            sprite.color = square.color;
        } else if dst.entity == Some(entity) {
            sprite.color = square.color;
        }
    }

    // Resetting the selection resources must not trigger a change detection
    // that would call handle_move again.
    src.bypass_change_detection().entity = None;
    dst.bypass_change_detection().entity = None;
}

pub fn on_resize_board(
    mut resize_reader: MessageReader<bevy::window::WindowResized>,
    mut board_query: Query<(Entity, &mut Transform), With<ChessBoard>>,
) {
    // Trigger only when there was at least one resize event
    let mut new_size = None;
    for e in resize_reader.read() {
        new_size = Some(Vec2::new(e.width, e.height));
    }
    let Some(size) = new_size else {
        return;
    };

    let size = if size.x < size.y { size.x } else { size.y };

    let mut board = board_query.single_mut().unwrap(); // we're in-game, so a board must exist.

    board.1.scale.x = (size / BOARD_SIZE) * 0.7;
    board.1.scale.y = (size / BOARD_SIZE) * 0.7;

    log::debug!("Resized board to {}px", board.1.scale.x * BOARD_SIZE);
}
