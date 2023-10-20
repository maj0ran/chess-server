use crate::color::*;
use crate::game::Game;
use crate::tile::Tile;
use core::fmt;

pub mod bishop;
pub mod king;
pub mod knight;
pub mod pawn;
pub mod queen;
pub mod rook;

#[derive(Copy, Clone, Debug)]
pub enum ChessPiece {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

#[macro_export]
macro_rules! piece {
    ($p:expr) => {{
        let p: Box<dyn PieceTrait + Send> = match $p {
            'K' => Box::new(King {
                color: Color::White,
            }),
            'Q' => Box::new(Queen {
                color: Color::White,
            }),
            'R' => Box::new(Rook {
                color: Color::White,
            }),
            'B' => Box::new(Bishop {
                color: Color::White,
            }),
            'N' => Box::new(Knight {
                color: Color::White,
            }),
            'P' => Box::new(Pawn {
                color: Color::White,
            }),
            'k' => Box::new(King {
                color: Color::Black,
            }),
            'q' => Box::new(Queen {
                color: Color::Black,
            }),
            'r' => Box::new(Rook {
                color: Color::Black,
            }),
            'n' => Box::new(Knight {
                color: Color::Black,
            }),
            'b' => Box::new(Bishop {
                color: Color::Black,
            }),
            'p' => Box::new(Pawn {
                color: Color::Black,
            }),
            _ => panic!(),
        };
        p
    }};
}

pub trait PieceTrait {
    fn color(&self) -> Color;
    fn id(&self) -> ChessPiece;
    fn get_moves(&self, board: &Game, pos: Tile) -> Vec<Tile>;
}

impl fmt::Display for dyn PieceTrait + Send {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = self.id();
        let color = self.color();
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
