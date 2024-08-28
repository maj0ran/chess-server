use crate::color::Color;
use crate::pieces::Chess;
use crate::tile::Tile;

pub fn get_tiles_control_pawn(board: &Chess, pos: Tile) -> Vec<Tile> {
    let this = board[pos].as_ref().unwrap();
    let mut tiles = Vec::<Tile>::new();

    let attack = if this.color == Color::White {
        [Tile::UPLEFT, Tile::UPRIGHT]
    } else {
        [Tile::DOWNLEFT, Tile::DOWNRIGHT]
    };

    // filters out tiles over the edge
    for tile in attack {
        if let Some(t) = pos + tile {
            tiles.push(t);
        }
    }
    tiles
}

pub fn get_moves_pawn(board: &Chess, pos: Tile) -> Vec<Tile> {
    let this = board[pos].as_ref().unwrap();

    let mut tiles = Vec::<Tile>::new();

    let (forward, attack) = if this.color == Color::White {
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
        let start_rank = if this.color == Color::White { '2' } else { '7' };
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
                if p.color != this.color {
                    tiles.push(t);
                }
            }
            if let Some(e) = board.en_passant {
                if e == t {
                    tiles.push(t)
                }
            }
        }
    }

    tiles
}
