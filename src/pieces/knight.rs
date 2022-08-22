use crate::field::Field;
use crate::pieces::{Board, Piece, PieceInfo};

pub fn move_rules_knight(piece: &PieceInfo, board: &Board, pos: &Field) -> Vec<Field> {
    let mut all_moves = vec![
        pos.up().up().left(),
        pos.up().up().right(),
        pos.down().down().left(),
        pos.down().down().right(),
        pos.right().right().up(),
        pos.right().right().down(),
        pos.left().left().up(),
        pos.left().left().down(),
    ];

    all_moves.retain(|p| (p.file >= 'a' && p.file <= 'h' && p.rank >= '1' && p.rank <= '8'));

    all_moves.retain(|p| match board.peek(*p) {
        None => true,
        Some(Piece { info: other, .. }) => other.color != piece.color,
    });

    all_moves
}
