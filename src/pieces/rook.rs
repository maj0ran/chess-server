use crate::pieces::{Board, ChessField, MoveRuleset, Piece, PieceInfo};

pub struct RookMoveRuleset;
impl MoveRuleset for RookMoveRuleset {
    fn get_valid_fields<S: Into<String>>(board: &Board, piece: &PieceInfo, pos: S) -> Vec<String> {
        let pos = pos.into();
        let mut all_moves = vec![];
        let mut prev = pos.clone();
        while prev.up().rank() <= '8' {
            let next = prev.up();
            match board.peek(&next) {
                None => {
                    all_moves.push(next.clone());
                    prev = next;
                }
                Some(Piece { color, .. }) => {
                    if color != piece.color {
                        all_moves.push(next.clone());
                    }
                    break;
                }
            };
        }
        prev = pos.clone();
        while prev.down().rank() >= '1' {
            let next = prev.down();
            match board.peek(&next) {
                None => {
                    all_moves.push(next.clone());
                    prev = next;
                }
                Some(Piece { color, .. }) => {
                    if color != piece.color {
                        all_moves.push(next.clone());
                    }
                    break;
                }
            };
        }
        prev = pos.clone();
        while prev.right().file() <= 'h' {
            let next = prev.right();
            match board.peek(&next) {
                None => {
                    all_moves.push(next.clone());
                    prev = next;
                }
                Some(Piece { color, .. }) => {
                    if color != piece.color {
                        all_moves.push(next.clone());
                    }
                    break;
                }
            };
        }
        prev = pos.clone();
        while prev.left().file() >= 'a' {
            let next = prev.left();
            match board.peek(&next) {
                None => {
                    all_moves.push(next.clone());
                    prev = next;
                }
                Some(Piece { color, .. }) => {
                    if color != piece.color {
                        all_moves.push(next.clone());
                    }
                    break;
                }
            };
        }

        all_moves
    }
}
