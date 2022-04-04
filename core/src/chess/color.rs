use core::fmt;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ChessColor {
    Black = 0,
    White = 1,
}

impl std::ops::Not for ChessColor {
    type Output = ChessColor;

    fn not(self) -> ChessColor {
        match self {
            ChessColor::White => ChessColor::Black,
            ChessColor::Black => ChessColor::White,
        }
    }
}

impl fmt::Display for ChessColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match self {
            ChessColor::Black => "Black",
            ChessColor::White => "White",
        };
        write!(f, "{}", c)
    }
}
