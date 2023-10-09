use crate::{
    game::{self, Game},
    net::{self, Interface},
};

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
    pub fn run(&mut self) {
        loop {
            let msg = self.net.wait_for_message();
            self.game.make_move(msg.0.into(), msg.1.into());
            println!("{}", self.game)
        }
    }
}
