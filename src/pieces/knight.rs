use crate::color::Color;
use crate::pieces::*;
use crate::tile::Tile;

pub struct Knight {
    pub color: Color,
}

impl PieceTrait for Knight {
    fn color(&self) -> Color {
        self.color
    }

    fn id(&self) -> ChessPiece {
        ChessPiece::Knight
    }

    fn get_moves(&self, board: &Game, pos: Tile) -> Vec<Tile> {
        let mut tiles = vec![];
        for d in [
            (2, 1),
            (2, -1),
            (-2, 1),
            (-2, -1),
            (1, 2),
            (1, -2),
            (-1, 2),
            (-1, -2),
        ] {
            let dst = pos + d;
            if let Some(t) = dst {
                if let Some(p) = board.peek(t) {
                    if p.color() != self.color() {
                        tiles.push(t);
                    }
                } else {
                    tiles.push(t)
                }
            }
        }
        tiles
    }
}
