#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Color {
    Black = 0,
    White = 1,
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
