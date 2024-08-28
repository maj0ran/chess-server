use crate::color::*;
use crate::game::Chess;
use crate::tile::Tile;
use core::fmt;

pub mod bishop;
pub mod king;
pub mod knight;
pub mod pawn;
pub mod queen;
pub mod rook;

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum ChessPiece {
    King = (1 << 1),
    Queen = (1 << 2),
    Rook = (1 << 3),
    Bishop = (1 << 4),
    Knight = (1 << 5),
    Pawn = (1 << 6),
}

#[derive(Copy, Clone)]
pub struct Piece {
    pub typ: ChessPiece,
    pub color: Color,
    pub get_moves: fn(&Chess, Tile) -> Vec<Tile>,
    pub get_tiles_controlled: fn(&Chess, Tile) -> Vec<Tile>,
}

impl Piece {
    pub fn new(
        typ: ChessPiece,
        color: Color,
        move_function: fn(&Chess, Tile) -> Vec<Tile>,
        get_tiles_controlled_function: fn(&Chess, Tile) -> Vec<Tile>,
    ) -> Piece {
        Piece {
            typ,
            color,
            get_moves: move_function,
            get_tiles_controlled: get_tiles_controlled_function,
        }
    }
}

#[macro_export]
macro_rules! piece {
    ($p:expr) => {{
        let p: Piece = match $p {
            'K' => Piece::new(
                ChessPiece::King,
                Color::White,
                get_moves_king,
                get_tiles_control_king,
            ),
            'Q' => Piece::new(
                ChessPiece::Queen,
                Color::White,
                get_moves_queen,
                get_tiles_control_queen,
            ),
            'R' => Piece::new(
                ChessPiece::Rook,
                Color::White,
                get_moves_rook,
                get_tiles_control_rook,
            ),
            'B' => Piece::new(
                ChessPiece::Bishop,
                Color::White,
                get_moves_bishop,
                get_tiles_control_bishop,
            ),
            'N' => Piece::new(
                ChessPiece::Knight,
                Color::White,
                get_moves_knight,
                get_tiles_control_knight,
            ),
            'P' => Piece::new(
                ChessPiece::Pawn,
                Color::White,
                get_moves_pawn,
                get_tiles_control_pawn,
            ),
            'k' => Piece::new(
                ChessPiece::King,
                Color::Black,
                get_moves_king,
                get_tiles_control_king,
            ),
            'q' => Piece::new(
                ChessPiece::Queen,
                Color::Black,
                get_moves_queen,
                get_tiles_control_queen,
            ),
            'r' => Piece::new(
                ChessPiece::Rook,
                Color::Black,
                get_moves_rook,
                get_tiles_control_rook,
            ),
            'n' => Piece::new(
                ChessPiece::Knight,
                Color::Black,
                get_moves_knight,
                get_tiles_control_knight,
            ),
            'b' => Piece::new(
                ChessPiece::Bishop,
                Color::Black,
                get_moves_bishop,
                get_tiles_control_bishop,
            ),
            'p' => Piece::new(
                ChessPiece::Pawn,
                Color::Black,
                get_moves_pawn,
                get_tiles_control_pawn,
            ),

            _ => panic!(),
        };
        p
    }};
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = self.typ;
        let color = self.color;
        let symbol = match (id, color) {
            (ChessPiece::King, Color::Black) => "♔",
            (ChessPiece::Queen, Color::Black) => "♕",
            (ChessPiece::Rook, Color::Black) => "♖",
            (ChessPiece::Bishop, Color::Black) => "♗",
            (ChessPiece::Knight, Color::Black) => "♘",
            (ChessPiece::Pawn, Color::Black) => "♙",
            (ChessPiece::King, Color::White) => "♚",
            (ChessPiece::Queen, Color::White) => "♛",
            (ChessPiece::Rook, Color::White) => "♜",
            (ChessPiece::Bishop, Color::White) => "♝",
            (ChessPiece::Knight, Color::White) => "♞",
            (ChessPiece::Pawn, Color::White) => "♟",
        };

        write!(f, "{}", symbol)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
