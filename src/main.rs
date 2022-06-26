mod game;
mod pieces;

use crate::game::*;
use crate::pieces::*;
use std::io;

fn main() -> io::Result<()> {
    let mut chess = Board::new();
    let p = Piece::new(PieceType::King, Color::White);

    let start_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq f3 0 1";
    let mut chess = Board::load_fen(start_fen);
    println!("{}", chess);

    let mut src = String::new();
    let mut dst = String::new();
    loop {
        src.clear();
        dst.clear();
        io::stdin().read_line(&mut src)?;
        io::stdin().read_line(&mut dst)?;
        chess.move_to(&src, &dst);
        println!("{}", chess);
        println!("{:?}", chess.get_moves("e1"));
    }
}
