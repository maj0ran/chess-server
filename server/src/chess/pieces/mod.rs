use core::fmt;
use std::ops::Deref;

pub mod bishop;
pub mod king;
pub mod knight;
pub mod pawn;
pub mod queen;
pub mod rook;

use crate::chess::chess::Chess;
use chess_core::{ChessColor, ChessPiece, Tile, WoodPiece};

pub use bishop::*;
pub use king::*;
pub use knight::*;
pub use pawn::*;
pub use queen::*;
pub use rook::*;

/// A full-fledged chess piece, with color and functionality.
/// Pieces are combined with two callback functions that operate on a chess board.
#[derive(Copy, Clone)]
pub struct Piece {
    pub piece: WoodPiece,
    pub get_moves: fn(&Chess, Tile) -> Vec<Tile>,
    pub get_tiles_controlled: fn(&Chess, Tile) -> Vec<Tile>,
}

impl Deref for Piece {
    type Target = WoodPiece;

    fn deref(&self) -> &Self::Target {
        &self.piece
    }
}

impl Piece {
    pub fn new(
        typ: ChessPiece,
        color: ChessColor,
        move_function: fn(&Chess, Tile) -> Vec<Tile>,
        get_tiles_controlled_function: fn(&Chess, Tile) -> Vec<Tile>,
    ) -> Piece {
        Piece {
            piece: WoodPiece::new(typ, color),
            get_moves: move_function,
            get_tiles_controlled: get_tiles_controlled_function,
        }
    }

    /// used for sending pieces over the network
    pub fn as_byte(&self) -> char {
        self.piece.as_byte()
    }
}

/// creates a full piece from a char
#[macro_export]
macro_rules! piece {
    ($p:expr) => {{
        use crate::chess::pieces::*;
        let p: Option<Piece> = match $p {
            'K' => Some(Piece::new(
                ChessPiece::King,
                ChessColor::White,
                get_moves_king,
                get_tiles_control_king,
            )),
            'Q' => Some(Piece::new(
                ChessPiece::Queen,
                ChessColor::White,
                get_moves_queen,
                get_tiles_control_queen,
            )),
            'R' => Some(Piece::new(
                ChessPiece::Rook,
                ChessColor::White,
                get_moves_rook,
                get_tiles_control_rook,
            )),
            'B' => Some(Piece::new(
                ChessPiece::Bishop,
                ChessColor::White,
                get_moves_bishop,
                get_tiles_control_bishop,
            )),
            'N' => Some(Piece::new(
                ChessPiece::Knight,
                ChessColor::White,
                get_moves_knight,
                get_tiles_control_knight,
            )),
            'P' => Some(Piece::new(
                ChessPiece::Pawn,
                ChessColor::White,
                get_moves_pawn,
                get_tiles_control_pawn,
            )),
            'k' => Some(Piece::new(
                ChessPiece::King,
                ChessColor::Black,
                get_moves_king,
                get_tiles_control_king,
            )),
            'q' => Some(Piece::new(
                ChessPiece::Queen,
                ChessColor::Black,
                get_moves_queen,
                get_tiles_control_queen,
            )),
            'r' => Some(Piece::new(
                ChessPiece::Rook,
                ChessColor::Black,
                get_moves_rook,
                get_tiles_control_rook,
            )),
            'n' => Some(Piece::new(
                ChessPiece::Knight,
                ChessColor::Black,
                get_moves_knight,
                get_tiles_control_knight,
            )),
            'b' => Some(Piece::new(
                ChessPiece::Bishop,
                ChessColor::Black,
                get_moves_bishop,
                get_tiles_control_bishop,
            )),
            'p' => Some(Piece::new(
                ChessPiece::Pawn,
                ChessColor::Black,
                get_moves_pawn,
                get_tiles_control_pawn,
            )),

            _ => None,
        };
        p
    }};
}

/// used to display pieces on CLI boards
impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.piece)
    }
}

impl fmt::Debug for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.piece)
    }
}
