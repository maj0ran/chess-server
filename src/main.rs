mod game;
use crate::game::board::*;
use crate::game::Game;
use std::io;

fn main() -> io::Result<()> {
    let mut src = String::new();
    let mut dst = String::new();
    let mut chess = Game::new();
    let mut buffer = String::new();
    chess.setup();
    println!("{}", chess.board);
    'gameloop: loop {
        loop {
            src.clear();
            io::stdin().read_line(&mut src)?;
            if chess.select(pos!(src)) {
                break;
            }
        }

        loop {
            dst.clear();
            io::stdin().read_line(&mut dst)?;
            match dst.as_str().trim() {
                "?" => {
                    for m in chess.get_valid_moves() {
                        println!("{}", m)
                    }
                }
                "q" => break 'gameloop,
                _ => {
                    chess.move_to(pos!(src), pos!(dst));
                    println!("{}", chess.board);
                    break;
                }
            }
        }
    }

    Ok(())
}
