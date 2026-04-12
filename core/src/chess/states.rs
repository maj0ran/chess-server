use crate::ChessColor;

pub enum ChessGameState {
    Running,
    Finished(ChessGameOutcome),
}

pub enum ChessGameOutcome {
    Checkmate(ChessColor),
    Resignation(ChessColor),
    TimeOut(ChessColor),
    Draw(DrawType),
}

#[derive(Debug, Clone, Copy)]
pub enum DrawType {
    Stalemate,
    ThreefoldRepetition,
    InsufficientMaterial,
    FiftyMoveRule,
}
