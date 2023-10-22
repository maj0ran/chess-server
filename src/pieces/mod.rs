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
}

impl Piece {}

#[macro_export]
macro_rules! piece {
    ($p:expr) => {{
        let p: Piece = match $p {
            'K' => Piece {
                typ: ChessPiece::King,
                color: Color::White,
                get_moves: get_moves_king,
            },
            'Q' => Piece {
                typ: ChessPiece::Queen,
                color: Color::White,
                get_moves: get_moves_queen,
            },

            'R' => Piece {
                typ: ChessPiece::Rook,
                color: Color::White,
                get_moves: get_moves_rook,
            },

            'B' => Piece {
                typ: ChessPiece::Bishop,
                color: Color::White,
                get_moves: get_moves_bishop,
            },

            'N' => Piece {
                typ: ChessPiece::Knight,
                color: Color::White,
                get_moves: get_moves_knight,
            },
            'P' => Piece {
                typ: ChessPiece::Pawn,
                color: Color::White,
                get_moves: get_moves_pawn,
            },

            'k' => Piece {
                typ: ChessPiece::King,
                color: Color::Black,
                get_moves: get_moves_king,
            },

            'q' => Piece {
                typ: ChessPiece::Queen,
                color: Color::Black,
                get_moves: get_moves_queen,
            },

            'r' => Piece {
                typ: ChessPiece::Rook,
                color: Color::Black,
                get_moves: get_moves_rook,
            },

            'n' => Piece {
                typ: ChessPiece::Knight,
                color: Color::Black,
                get_moves: get_moves_knight,
            },

            'b' => Piece {
                typ: ChessPiece::Bishop,
                color: Color::Black,
                get_moves: get_moves_bishop,
            },

            'p' => Piece {
                typ: ChessPiece::Pawn,
                color: Color::Black,
                get_moves: get_moves_pawn,
            },

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
