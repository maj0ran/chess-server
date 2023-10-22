use crate::pieces::Chess;
use crate::tile::Tile;

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
    tiles
}
