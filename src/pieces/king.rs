use crate::field::Field;
use crate::pieces::{Board, Piece, PieceInfo};

pub fn move_rules_king(piece: &PieceInfo, board: &Board, pos: &Field) -> Vec<Field> {
    let mut all_moves = vec![
        pos.up(),
        pos.down(),
        pos.left(),
        pos.right(),
        pos.up().left(),
        pos.up().right(),
        pos.down().left(),
        pos.down().right(),
    ];
    all_moves.retain(|p| (p.file >= 'a' && p.file <= 'h' && p.rank >= '1' && p.rank <= '8'));
    let moves: Vec<&Field> = all_moves
        .iter()
        .filter(|m| match board.peek(**m) {
            None => true,
            Some(Piece { info: other, .. }) => other.color != piece.color,
        })
        .collect();

    let mut pseudolegal_moves = Vec::<Field>::new();
    for m in moves {
        pseudolegal_moves.push(*m);
    }
    pseudolegal_moves
}
