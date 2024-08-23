use crate::pieces::Chess;
use crate::tile::Tile;

use super::ChessPiece;

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
    /* we use move_count to determine our castle_rights instead of the information in FEN
     * this is easier to implement, although it decouples the FEN information with the game logic
     * and we have to be careful to update both manually */
    if this.move_count == 0 {
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
    tiles
}
