use crate::ChessColor;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameOverReason {
    Checkmate(ChessColor),
    Resignation(ChessColor),
    TimeOut(ChessColor),
    Stalemate,
    ThreefoldRepetition,
    InsufficientMaterial,
    FiftyMovesRule,
    DrawAgreement,
}

impl GameOverReason {
    pub fn to_u8(&self) -> u8 {
        match self {
            GameOverReason::Checkmate(_) => 1,
            GameOverReason::Resignation(_) => 2,
            GameOverReason::TimeOut(_) => 3,
            GameOverReason::Stalemate => 4,
            GameOverReason::ThreefoldRepetition => 5,
            GameOverReason::InsufficientMaterial => 6,
            GameOverReason::FiftyMovesRule => 7,
            GameOverReason::DrawAgreement => 8,
        }
    }

    pub fn get_winner(&self) -> Option<ChessColor> {
        match self {
            GameOverReason::Checkmate(c)
            | GameOverReason::Resignation(c)
            | GameOverReason::TimeOut(c) => Some(*c),
            _ => None,
        }
    }
}

impl Display for GameOverReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            GameOverReason::Checkmate(_) => "Checkmate",
            GameOverReason::Resignation(_) => "Resignation",
            GameOverReason::TimeOut(_) => "Time Out",
            GameOverReason::Stalemate => "Stalemate",
            GameOverReason::ThreefoldRepetition => "3-Fold-Repetition",
            GameOverReason::InsufficientMaterial => "Insufficient Material",
            GameOverReason::FiftyMovesRule => "50-Moves-Rule",
            GameOverReason::DrawAgreement => "Agreement",
        };
        write!(f, "{}", text)
    }
}

pub enum ChessGameState {
    Running,
    Finished(GameOverReason),
}
