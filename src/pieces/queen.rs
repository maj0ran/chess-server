use crate::color::Color;
use crate::pieces::{Game, PieceTrait};
use crate::tile::Tile;

use super::ChessPiece;

pub struct Queen {
    pub color: Color,
}

impl PieceTrait for Queen {
    fn color(&self) -> Color {
        self.color
    }

    fn id(&self) -> ChessPiece {
        ChessPiece::Queen
    }
    fn get_moves(&self, board: &Game, pos: Tile) -> Vec<Tile> {
        let dirs = [
            Tile::UP,
            Tile::DOWN,
            Tile::RIGHT,
            Tile::LEFT,
            Tile::UPLEFT,
            Tile::DOWNLEFT,
            Tile::UPRIGHT,
            Tile::DOWNLEFT,
        ];

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
