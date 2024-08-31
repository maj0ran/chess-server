use std::sync::{Arc, Mutex};

use crate::{
    chessmove::{ChessMove, ToChessMove},
    game::Chess,
};

use super::*;

use frame::Frame;
use tokio::net::TcpListener;

pub struct Server {
    _listener: Option<TcpListener>,
    pub chess: Arc<Mutex<Chess>>,
}

impl Server {
    pub fn new() -> Server {
        Server {
            _listener: None,
            chess: Arc::new(Mutex::new(Chess::new())),
        }
    }

    pub async fn listen(&mut self) -> Result<()> {
        info!("Listening...");
        let listener = TcpListener::bind("127.0.0.1:7878").await?;

        loop {
            let (conn, addr) = listener.accept().await?;
            info!("got connection from {}!", addr);

            let mut client = Client::new("Marian".to_string(), conn, Arc::clone(&self.chess));
            tokio::spawn(async move {
                loop {
                    let cmd = client.read().await;
                    match cmd {
                        Some(cmd) => match client.exec(cmd) {
                            Ok(_) => {
                                info!("executed command")
                            }
                            Err(_) => warn!("error executing command"),
                        },
                        None => warn!("received command not executable"),
                    }
                }
            });
        }
    }
}
