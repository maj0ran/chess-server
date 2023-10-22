use crate::{game::Chess, net::Interface};

pub struct Server {
    game: Chess,
    net: Interface,
}

impl Server {
    pub fn new() -> Server {
        Server {
            game: Chess::new(),
            net: Interface::new(),
        }
    }
    pub async fn run(&mut self) {
        println!("Running");
        let e = self.net.listen().await;
    }
}
