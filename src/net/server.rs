use super::*;
use crate::game::Chess;
use dashmap::DashMap;
use smol::lock::Mutex;
use std::sync::Arc;

use bytes::BytesMut;
use smol::net::*;

pub struct ServerState {
    pub games: Arc<DashMap<usize, Arc<Mutex<Chess>>>>,
}

impl ServerState {
    pub fn new() -> Arc<ServerState> {
        Arc::new(ServerState {
            games: Arc::new(DashMap::new()),
        })
    }
}

pub struct Server {
    _listener: Option<TcpListener>,
    clients: Vec<Arc<Mutex<Handler>>>,
    pub state: Arc<ServerState>,
}

impl Server {
    pub fn new() -> Server {
        Server {
            _listener: None,
            clients: vec![],
            state: ServerState::new(),
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
                chess: None,
                conn: Connection::new(socket),
                buffer: BytesMut::zeroed(64),
                server_state: self.state.clone(),
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
