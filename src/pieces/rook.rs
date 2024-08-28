use crate::pieces::*;
use crate::tile::Tile;

pub fn get_tiles_control_rook(board: &Chess, pos: Tile) -> Vec<Tile> {
    get_moves_rook(board, pos)
}

pub fn get_moves_rook(board: &Chess, pos: Tile) -> Vec<Tile> {
    let this = board[pos].as_ref().unwrap();
    let dirs = [Tile::UP, Tile::DOWN, Tile::RIGHT, Tile::LEFT];

    let mut tiles = vec![];
    for d in dirs {
        let mut ray = board.ray(pos, d);
        if let Some(t) = ray.last() {
            if let Some(p) = board.peek(*t) {
                if p.color == this.color {
                    let _ = ray.pop();
                }
            }
        }
        tiles.append(&mut ray)
    }

    tiles
}
