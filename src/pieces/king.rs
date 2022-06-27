use crate::pieces::{Board, ChessField, MoveRuleset, Piece, PieceInfo};

struct KingMoveRuleset;
impl MoveRuleset for KingMoveRuleset {
    fn get_valid_fields<S: Into<String>>(board: &Board, piece: &PieceInfo, pos: S) -> Vec<String> {
        let pos = pos.into();
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

        all_moves.retain(|m| Board::index(m) > 0 && Board::index(m) < 64);
        let moves: Vec<&String> = all_moves
            .iter()
            .filter(|m| match board.peek(*m) {
                None => true,
                Some(Piece { color, .. }) => color != piece.color,
            })
            .collect();

        let mut pseudolegal_moves = Vec::<String>::new();
        for m in moves {
            pseudolegal_moves.push(String::from(m.clone()));
        }
        pseudolegal_moves
    }
}
