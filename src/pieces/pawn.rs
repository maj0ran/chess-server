use crate::color::Color;
use crate::field::Field;
use crate::pieces::{Board, Piece, PieceInfo};

pub fn move_rules_pawn(piece: &PieceInfo, board: &Board, pos: &Field) -> Vec<Field> {
    let mut all_moves = Vec::<Field>::new();

    let mov = pos.up();
    if piece.color == Color::White {
        match board.peek(mov) {
            None => {
                all_moves.push(mov);
                if pos.rank == '2' {
                    let mov = pos.up().up();
                    match board.peek(mov) {
                        None => all_moves.push(mov),
                        Some(_) => {}
                    };
                }
            }
            Some(_) => {}
        }
    }

    let mov = pos.down();
    if piece.color == Color::Black {
        match board.peek(mov) {
            None => {
                all_moves.push(mov);
                if pos.rank == '7' {
                    let mov = pos.down().down();
                    match board.peek(mov) {
                        None => all_moves.push(mov),
                        Some(_) => {}
                    };
                }
            }
            Some(_) => {}
        }
    }

    all_moves
}
