use crate::ChessColor;

pub enum ChessGameState {
    Running,
    Finished(ChessGameOutcome),
}

pub enum ChessGameOutcome {
    Victory(VictoryType),
    Draw(DrawType),
}

#[derive(Debug, Clone, Copy)]
pub enum DrawType {
    Stalemate,
    ThreefoldRepetition,
    InsufficientMaterial,
    FiftyMoveRule,
}

#[derive(Clone, Debug, Copy)]
#[repr(u8)]
pub enum VictoryType {
    Checkmate(ChessColor) = 1,
    Resignation(ChessColor) = 2,
    TimeOut(ChessColor) = 3,
}

impl VictoryType {
    pub fn get_winner(&self) -> ChessColor {
        match self {
            VictoryType::Checkmate(c) => *c,
            VictoryType::Resignation(c) => *c,
            VictoryType::TimeOut(c) => *c,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            VictoryType::Checkmate(_) => 1,
            VictoryType::Resignation(_) => 2,
            VictoryType::TimeOut(_) => 3,
        }
    }

    pub fn from_u8(v: u8, winner: ChessColor) -> Self {
        match v {
            1 => VictoryType::Checkmate(winner),
            2 => VictoryType::Resignation(winner),
            3 => VictoryType::TimeOut(winner),
            _ => panic!("Invalid victory type"),
        }
    }
}
