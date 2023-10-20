use crate::{game::Game, net::Interface};

pub struct Server {
    game: Game,
    net: Interface,
}

impl Server {
    pub fn new() -> Server {
        Server {
            game: Game::new(),
            net: Interface::new(),
        }
    }
    pub async fn run(&mut self) {
        println!("Running");
        let e = self.net.listen().await;
    }
}
