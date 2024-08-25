use crate::pieces::Chess;
use crate::tile::Tile;

use super::{ChessPiece, Color};

pub fn get_moves_king(board: &Chess, pos: Tile) -> Vec<Tile> {
    let this = board[pos].as_ref().unwrap();
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
                if p.color != this.color {
                    tiles.push(t);
                }
            } else {
                tiles.push(t);
            }
        }
    }

    // castling
    // the castle rights offset is the offset in memory for the castle rights, depending on white
    // or black:
    // K Q k q
    // ^   ^
    // |   |
    // 0   2
    let castle_rights = board.castle_rights;
    let castle_rights_offset = if this.color == Color::White { 0 } else { 2 };

    // shoot a ray kingside to determine king-side castling legality
    if castle_rights[castle_rights_offset + 0] {
        let kingside_ray = board.ray(pos, Tile::RIGHT);
        // should never be 'None' in regular chess but just in case we allow weird setups
        // always 'Some' if the king isn't standing at the border left or right
        match kingside_ray.last() {
            // check if the ray ended at a rook
            Some(tile) => match board[*tile] {
                Some(p) => {
                    if p.typ == ChessPiece::Rook && p.color == this.color && p.move_count == 0 {
                        if let Some(castling_tile) = pos + (2, 0) {
                            tiles.push(castling_tile);
                        }
                    }
                }
                None => todo!(),
            },
            None => {}
        }
    }

    // shoot a ray queenside to determine queen-side castling legality
    if castle_rights[castle_rights_offset + 1] {
        let queenside_ray = board.ray(pos, Tile::LEFT);
        // should never be 'None' in regular chess but just in case we allow weird setups
        // always 'Some' if the king isn't standing at the border left or right
        match queenside_ray.last() {
            // check if the ray ended at a rook
            Some(tile) => match board[*tile] {
                Some(p) => {
                    if p.typ == ChessPiece::Rook && p.color == this.color && p.move_count == 0 {
                        if let Some(castling_tile) = pos + (-2, 0) {
                            tiles.push(castling_tile);
                        }
                    }
                }
                None => todo!(),
            },
            None => {}
        }
    }

    tiles
}
