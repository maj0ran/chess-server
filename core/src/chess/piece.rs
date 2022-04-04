use crate::ChessColor;
use core::fmt;

/// Raw chess piece, without taking sides.
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

/// A lightweight chess piece for client use, containing only type and color.
#[derive(Copy, Clone, PartialEq)]
pub struct WoodPiece {
    pub typ: ChessPiece,
    pub color: ChessColor,
}

impl WoodPiece {
    pub fn new(typ: ChessPiece, color: ChessColor) -> WoodPiece {
        WoodPiece { typ, color }
    }

    /// used for sending pieces over the network
    pub fn as_byte(&self) -> char {
        match (self.typ, self.color) {
            (ChessPiece::King, ChessColor::Black) => 'k',
            (ChessPiece::King, ChessColor::White) => 'K',
            (ChessPiece::Queen, ChessColor::Black) => 'q',
            (ChessPiece::Queen, ChessColor::White) => 'Q',
            (ChessPiece::Rook, ChessColor::Black) => 'r',
            (ChessPiece::Rook, ChessColor::White) => 'R',
            (ChessPiece::Bishop, ChessColor::Black) => 'b',
            (ChessPiece::Bishop, ChessColor::White) => 'B',
            (ChessPiece::Knight, ChessColor::Black) => 'n',
            (ChessPiece::Knight, ChessColor::White) => 'N',
            (ChessPiece::Pawn, ChessColor::Black) => 'p',
            (ChessPiece::Pawn, ChessColor::White) => 'P',
        }
    }

    pub fn from_char(c: char) -> Option<Self> {
        let (typ, color) = match c {
            'K' => (ChessPiece::King, ChessColor::White),
            'Q' => (ChessPiece::Queen, ChessColor::White),
            'R' => (ChessPiece::Rook, ChessColor::White),
            'B' => (ChessPiece::Bishop, ChessColor::White),
            'N' => (ChessPiece::Knight, ChessColor::White),
            'P' => (ChessPiece::Pawn, ChessColor::White),
            'k' => (ChessPiece::King, ChessColor::Black),
            'q' => (ChessPiece::Queen, ChessColor::Black),
            'r' => (ChessPiece::Rook, ChessColor::Black),
            'b' => (ChessPiece::Bishop, ChessColor::Black),
            'n' => (ChessPiece::Knight, ChessColor::Black),
            'p' => (ChessPiece::Pawn, ChessColor::Black),
            _ => return None,
        };
        Some(WoodPiece { typ, color })
    }
}

impl fmt::Display for WoodPiece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match (self.typ, self.color) {
            (ChessPiece::King, ChessColor::Black) => "♔",
            (ChessPiece::Queen, ChessColor::Black) => "♕",
            (ChessPiece::Rook, ChessColor::Black) => "♖",
            (ChessPiece::Bishop, ChessColor::Black) => "♗",
            (ChessPiece::Knight, ChessColor::Black) => "♘",
            (ChessPiece::Pawn, ChessColor::Black) => "♙",
            (ChessPiece::King, ChessColor::White) => "♚",
            (ChessPiece::Queen, ChessColor::White) => "♛",
            (ChessPiece::Rook, ChessColor::White) => "♜",
            (ChessPiece::Bishop, ChessColor::White) => "♝",
            (ChessPiece::Knight, ChessColor::White) => "♞",
            (ChessPiece::Pawn, ChessColor::White) => "♟",
        };
        write!(f, "{}", symbol)
    }
}

impl fmt::Debug for WoodPiece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
