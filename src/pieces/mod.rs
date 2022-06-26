use crate::game::Board;
use core::fmt;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Color {
    Black,
    White,
}

impl std::ops::Not for Color {
    type Output = Color;

    fn not(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

#[derive(Copy, Clone, Debug)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
}

pub trait ChessPiece {
    fn get_moves<S: Into<String>>(&self, board: &Board, pos: S) -> Vec<String>;
}

impl Piece {
    pub fn new(piece_type: PieceType, color: Color) -> Piece {
        Piece { piece_type, color }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            Piece {
                piece_type: PieceType::King,
                color: Color::Black,
                ..
            } => "♔",
            Piece {
                piece_type: PieceType::Queen,
                color: Color::Black,
                ..
            } => "♕",
            Piece {
                piece_type: PieceType::Rook,
                color: Color::Black,
                ..
            } => "♖",
            Piece {
                piece_type: PieceType::Bishop,
                color: Color::Black,
                ..
            } => "♗",
            Piece {
                piece_type: PieceType::Knight,
                color: Color::Black,
                ..
            } => "♘",
            Piece {
                piece_type: PieceType::Pawn,
                color: Color::Black,
                ..
            } => "♙",

            Piece {
                piece_type: PieceType::King,
                color: Color::White,
                ..
            } => "♚",
            Piece {
                piece_type: PieceType::Queen,
                color: Color::White,
                ..
            } => "♛",
            Piece {
                piece_type: PieceType::Rook,
                color: Color::White,
                ..
            } => "♜",
            Piece {
                piece_type: PieceType::Bishop,
                color: Color::White,
                ..
            } => "♝",
            Piece {
                piece_type: PieceType::Knight,
                color: Color::White,
                ..
            } => "♞",
            Piece {
                piece_type: PieceType::Pawn,
                color: Color::White,
                ..
            } => "♟",
        };

        write!(f, "{}", symbol)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
