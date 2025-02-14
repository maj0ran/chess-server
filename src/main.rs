mod chessmove;
mod color;
mod game;
mod net;
mod pieces;
mod test;
mod tile;
mod util;
use std::io;

use crate::net::server::Server;
use smol::*;

use smol_macros::main;

main! {
    async fn main() -> io::Result<()> {
        env_logger::init();

        let mut server = Server::new();
        let _ = server.listen().await;

        println!("Exited Gracefully.");
        Ok(())
    }
}
