mod color;
mod game;
mod net;
mod pieces;
mod tile;
mod util;
use std::io;

use crate::net::server::Server;

#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::init();
    let mut server = Server::new();
    let _ = server.listen().await;

    println!("Exited Gracefully.");
    Ok(())
}
