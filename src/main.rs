mod color;
mod field;
mod game;
mod pieces;
use crate::game::*;
use std::io;

fn main() -> io::Result<()> {
    let start_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq f3 0 1";
    //    let start_fen = "RRRRRRKK/KKKrKKKK/8/8/8/8/KKKKKKKK/KKKKKKKK w KQkq f3 0 1";
    let mut chess = Board::load_fen(start_fen);
    println!("{}", chess);

    loop {
        let mut src = String::new();
        let mut dst = String::new();
        io::stdin().read_line(&mut src)?;
        io::stdin().read_line(&mut dst)?;
        chess.is_valid(src.into(), dst.into());
        println!("{}", chess);
    }
}
