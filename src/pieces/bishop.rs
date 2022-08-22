use crate::field::Field;
use crate::pieces::{Board, Piece, PieceInfo};

pub fn move_rules_bishop(piece: &PieceInfo, board: &Board, pos: &Field) -> Vec<Field> {
    let mut all_moves = vec![];

    let mut prev = pos.clone();
    while prev.right().file <= 'h' && prev.up().rank <= '8' {
        let next = prev.up().right();
        match board.peek(next) {
            None => {
                all_moves.push(next.clone());
                prev = next;
            }
            Some(Piece {
                info: other_piece, ..
            }) => {
                if other_piece.color != piece.color {
                    all_moves.push(next.clone());
                }
                break;
            }
        };
    }
    prev = pos.clone();
    while prev.right().file <= 'h' && prev.down().rank >= '1' {
        let next = prev.down().right();
        match board.peek(next) {
            None => {
                all_moves.push(next.clone());
                prev = next;
            }
            Some(Piece {
                info: other_piece, ..
            }) => {
                if other_piece.color != piece.color {
                    all_moves.push(next.clone());
                }
                break;
            }
        };
    }
    prev = pos.clone();
    while prev.left().file >= 'a' && prev.down().rank >= '1' {
        let next = prev.left().down();
        match board.peek(next) {
            None => {
                all_moves.push(next.clone());
                prev = next;
            }
            Some(Piece {
                info: other_piece, ..
            }) => {
                if other_piece.color != piece.color {
                    all_moves.push(next.clone());
                }
                break;
            }
        };
    }
    prev = pos.clone();
    while prev.left().file >= 'a' && prev.up().rank <= '8' {
        let next = prev.left();
        match board.peek(next) {
            None => {
                all_moves.push(next.clone());
                prev = next;
            }
            Some(Piece {
                info: other_piece, ..
            }) => {
                if other_piece.color != piece.color {
                    all_moves.push(next.clone());
                }
                break;
            }
        };
    }

    all_moves
}
