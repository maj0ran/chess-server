mod chess;
mod server;
use server::server::Server;

use smol_macros::main;
use std::io;

main! {
    async fn main() -> io::Result<()> {
        env_logger::init();

        let mut server = Server::new();
        let _ = server.run(7878).await;

        println!("Exited Gracefully.");
        Ok(())
    }
}
