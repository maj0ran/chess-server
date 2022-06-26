mod game;
mod pieces;

use crate::game::*;
use std::io;

fn main() -> io::Result<()> {
    let start_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq f3 0 1";
    let start_fen = "KKKKKKKK/KKKKKKKK/8/8/8/8/KKKKKKKK/KKKKKKKK w KQkq f3 0 1";
    let mut chess = Board::load_fen(start_fen);
    println!("{}", chess);

    let mut src = String::new();
    let mut dst = String::new();
    loop {
        src.clear();
        dst.clear();
        io::stdin().read_line(&mut src)?;
        io::stdin().read_line(&mut dst)?;
        println!("{:?}", chess.get_moves(&src));
        chess.move_to(&src, &dst);
        println!("{}", chess);
    }
}
