mod color;
mod game;
mod net;
mod pieces;
mod server;
mod tile;
use crate::server::Server;
use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::init();
    let mut server = Server::new();
    let _ = server.run().await;

    println!("Exited Gracefully.");
    Ok(())
}

