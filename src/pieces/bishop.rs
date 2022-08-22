use crate::color::Color;
use crate::pieces::Game;
use crate::tile::Tile;

use super::PieceTrait;

pub struct Bishop {
    pub color: Color,
}

impl PieceTrait for Bishop {
    fn color(&self) -> crate::color::Color {
        self.color
    }

    fn id(&self) -> super::ChessPiece {
        super::ChessPiece::Bishop
    }

    fn get_moves(&self, board: &Game, pos: Tile) -> Vec<Tile> {
        let dirs = [Tile::UPLEFT, Tile::DOWNLEFT, Tile::UPRIGHT, Tile::DOWNRIGHT];

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
