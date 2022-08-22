use crate::color::Color;
use crate::pieces::*;
use crate::tile::Tile;

pub struct Rook {
    pub color: Color,
}

impl PieceTrait for Rook {
    fn color(&self) -> crate::color::Color {
        self.color
    }

    fn id(&self) -> super::ChessPiece {
        ChessPiece::Rook
    }

    fn get_moves(&self, board: &Game, pos: Tile) -> Vec<Tile> {
        let dirs = [Tile::UP, Tile::DOWN, Tile::RIGHT, Tile::LEFT];

        let mut tiles = vec![];
        for d in dirs {
            let mut ray = board.ray(pos, d);
            if let Some(t) = ray.last() {
                if let Some(p) = board.peek(*t) {
                    if p.color() == self.color() {
                        let _ = ray.pop();
                    }
                }
            }
            tiles.append(&mut ray)
        }

        tiles
    }
}
