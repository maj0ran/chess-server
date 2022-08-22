use crate::color::Color;
use crate::pieces::{Game, PieceTrait};
use crate::tile::Tile;

use super::ChessPiece;

pub struct Pawn {
    pub color: Color,
}
impl PieceTrait for Pawn {
    fn color(&self) -> Color {
        self.color
    }

    fn id(&self) -> ChessPiece {
        ChessPiece::Pawn
    }

    fn get_moves(&self, board: &Game, pos: Tile) -> Vec<Tile> {
        let mut tiles = Vec::<Tile>::new();

        let (forward, attack) = if self.color() == Color::White {
            let forward = Tile::UP;
            let attack = [Tile::UPLEFT, Tile::UPRIGHT];
            (forward, attack)
        } else {
            let forward = Tile::DOWN;
            let attack = [Tile::DOWNLEFT, Tile::DOWNRIGHT];
            (forward, attack)
        };

        let dst = pos + forward;
        if let Some(t) = dst {
            if board[t].is_none() {
                tiles.push(t);
            }
            let start_rank = if self.color() == Color::White {
                '2'
            } else {
                '7'
            };
            if pos.rank == start_rank {
                let t2 = (t + forward).unwrap();
                if board[t2].is_none() {
                    tiles.push(t2);
                }
            }
        }
        for d in attack {
            let dst = pos + d;
            if let Some(t) = dst {
                if let Some(p) = board.peek(t) {
                    if p.color() != self.color() {
                        tiles.push(t);
                    }
                }
            }
        }

        tiles
    }
}
