use crate::game::Chess;
use smol::lock::Mutex;
use std::sync::Arc;

use super::*;

use bytes::BytesMut;
use smol::net::*;
pub struct Server {
    _listener: Option<TcpListener>,
    pub chess: Arc<Mutex<Chess>>,
    clients: Vec<Arc<Mutex<Handler>>>,
}

impl Server {
    pub fn new() -> Server {
        Server {
            _listener: None,
            chess: Arc::new(Mutex::new(Chess::new())),
            clients: vec![],
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
                buffer: BytesMut::zeroed(64),
            };

            info!("handler initialized with name: {}", handler.name);

            smol::spawn(async move {
                loop {
                    handler.run().await;
                }
            })
            .detach();
        }
    }
}
