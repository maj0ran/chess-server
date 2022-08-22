use crate::color::Color;
use crate::pieces::ChessPiece;
use crate::pieces::{Game, PieceTrait};
use crate::tile::Tile;

pub struct King {
    pub color: Color,
}

impl PieceTrait for King {
    fn color(&self) -> crate::color::Color {
        self.color
    }

    fn id(&self) -> ChessPiece {
        ChessPiece::King
    }

    fn get_moves(&self, board: &Game, pos: Tile) -> Vec<Tile> {
        let mut tiles = vec![];

        for d in [
            Tile::UP,
            Tile::DOWN,
            Tile::LEFT,
            Tile::RIGHT,
            Tile::UPLEFT,
            Tile::UPRIGHT,
            Tile::DOWNLEFT,
            Tile::DOWNRIGHT,
        ] {
            let dst = pos + d;
            if let Some(t) = dst {
                if let Some(p) = board.peek(t) {
                    if p.color() != self.color() {
                        tiles.push(t);
                    }
                } else {
                    tiles.push(t);
                }
            }
        }
        tiles
    }
}
