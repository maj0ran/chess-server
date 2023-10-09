mod color;
mod game;
mod net;
mod pieces;
mod server;
mod tile;
use crate::{game::*, net::Interface, server::Server, tile::Tile};
use std::io;

fn main() -> io::Result<()> {
    let mut server = Server::new();
    server.run();

    let start_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq f3 0 1";
    let mut chess = Game::load_fen(start_fen);
    println!("{}", chess);

    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        // d2d4 + 'RETURN' = 5 symbols
        if input.len() != 5 {
            println!("invalid");
            continue;
        }

        let src: Tile = input[0..2].to_string().into();
        let dst: Tile = input[2..4].to_string().into();

        match chess.make_move(src, dst) {
            true => {
                println!("{}", chess);
            }
            false => {
                println!("Invalid Move!: {} -> {}", &src, &dst);
                print!("Valid Moves are: ");
                let tiles = chess.get_moves(src);
                for t in tiles {
                    print!("{}{} ", src, t)
                }
                println!();
            }
        }
    }
}
