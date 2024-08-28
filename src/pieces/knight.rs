use crate::pieces::*;
use crate::tile::Tile;

pub fn get_tiles_control_knight(board: &Chess, pos: Tile) -> Vec<Tile> {
    get_moves_knight(board, pos)
}

pub fn get_moves_knight(board: &Chess, pos: Tile) -> Vec<Tile> {
    let this = &board[pos];
    let this = this.as_ref().unwrap();

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
                if p.color != this.color {
                    tiles.push(t);
                }
            } else {
                tiles.push(t)
            }
        }
    }
    tiles
}
