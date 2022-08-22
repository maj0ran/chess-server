use crate::color::Color;
use crate::field::Field;
use crate::game::Board;
use core::fmt;

pub mod bishop;
pub mod king;
pub mod knight;
pub mod pawn;
pub mod queen;
pub mod rook;

#[macro_export]
macro_rules! piece {
    ($p:expr) => {{
        let p = match $p {
            'K' => Piece::new(PieceId::King, Color::White, move_rules_king),
            'k' => Piece::new(PieceId::King, Color::Black, move_rules_king),
            'Q' => Piece::new(PieceId::Queen, Color::White, move_rules_queen),
            'q' => Piece::new(PieceId::Queen, Color::Black, move_rules_queen),
            'R' => Piece::new(PieceId::Rook, Color::White, move_rules_rook),
            'r' => Piece::new(PieceId::Rook, Color::Black, move_rules_rook),
            'N' => Piece::new(PieceId::Knight, Color::White, move_rules_knight),
            'n' => Piece::new(PieceId::Knight, Color::Black, move_rules_knight),
            'B' => Piece::new(PieceId::Bishop, Color::White, move_rules_bishop),
            'b' => Piece::new(PieceId::Bishop, Color::Black, move_rules_bishop),
            'P' => Piece::new(PieceId::Pawn, Color::White, move_rules_pawn),
            'p' => Piece::new(PieceId::Pawn, Color::Black, move_rules_pawn),
            _ => panic!(),
        };
        p
    }};
}

#[derive(Copy, Clone, Debug)]
pub enum PieceId {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

#[derive(Copy, Clone, Debug)]
pub struct PieceInfo {
    pub id: PieceId,
    pub color: Color,
}

#[derive(Copy, Clone)]
pub struct Piece {
    pub info: PieceInfo,
    pub move_rule: fn(&PieceInfo, &Board, &Field) -> Vec<Field>,
}

impl Piece {
    pub fn new(
        id: PieceId,
        color: Color,
        move_rule: fn(&PieceInfo, &Board, &Field) -> Vec<Field>,
    ) -> Piece {
        Piece {
            info: PieceInfo { id, color },
            move_rule,
        }
    }

    pub fn get_legal_fields(&self, board: &Board, from: &Field) -> Vec<Field> {
        (self.move_rule)(&self.info, board, from)
    }
}

impl fmt::Display for PieceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            PieceInfo {
                id: PieceId::King,
                color: Color::Black,
                ..
            } => "♔",
            PieceInfo {
                id: PieceId::Queen,
                color: Color::Black,
                ..
            } => "♕",
            PieceInfo {
                id: PieceId::Rook,
                color: Color::Black,
                ..
            } => "♖",
            PieceInfo {
                id: PieceId::Bishop,
                color: Color::Black,
                ..
            } => "♗",
            PieceInfo {
                id: PieceId::Knight,
                color: Color::Black,
                ..
            } => "♘",
            PieceInfo {
                id: PieceId::Pawn,
                color: Color::Black,
                ..
            } => "♙",

            PieceInfo {
                id: PieceId::King,
                color: Color::White,
                ..
            } => "♚",
            PieceInfo {
                id: PieceId::Queen,
                color: Color::White,
                ..
            } => "♛",
            PieceInfo {
                id: PieceId::Rook,
                color: Color::White,
                ..
            } => "♜",
            PieceInfo {
                id: PieceId::Bishop,
                color: Color::White,
                ..
            } => "♝",
            PieceInfo {
                id: PieceId::Knight,
                color: Color::White,
                ..
            } => "♞",
            PieceInfo {
                id: PieceId::Pawn,
                color: Color::White,
                ..
            } => "♟",
        };

        write!(f, "{}", symbol)
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.info)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
