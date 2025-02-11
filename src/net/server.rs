use std::sync::{Arc, Mutex};

use crate::{game::Chess, pieces::Piece, tile::Tile};

use super::*;

use bytes::BytesMut;
use tokio::{net::TcpListener, sync::broadcast};

pub struct Server {
    _listener: Option<TcpListener>,
    pub chess: Arc<Mutex<Chess>>,
    clients: Vec<Arc<Mutex<Handler>>>,
    notify_move: broadcast::Sender<Vec<(Tile, Option<Piece>)>>,
}

impl Server {
    pub fn new() -> Server {
        let (notify_move, _) = broadcast::channel(16);
        Server {
            _listener: None,
            chess: Arc::new(Mutex::new(Chess::new())),
            clients: vec![],
            notify_move,
        }
    }

    pub async fn listen(&mut self) -> Result<()> {
        info!("Listening...");
        let listener = TcpListener::bind("127.0.0.1:7878").await?;

        loop {
            let (socket, addr) = listener.accept().await?;
            info!("got connection from {}!", addr);

            let mut handler = Handler {
                name: "Marian".to_string(),
                chess: self.chess.clone(),
                conn: Connection::new(socket),
                notify_move: self.notify_move.subscribe(),
                buffer: BytesMut::zeroed(64),
            };

            info!("handler initialized with name: {}", handler.name);

            tokio::spawn(async move {
                loop {
                    handler.run().await;
                }
            });
        }
    }
}
