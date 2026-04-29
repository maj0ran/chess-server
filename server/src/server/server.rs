use super::manager::GameManager;
use super::session::ClientSession;
use crate::net::tls::TlsServer;
use chess_core::ClientId;
use smol::channel::unbounded;
use smol::net::*;
use std::sync::Arc;

pub type Result<T> = std::result::Result<T, std::io::Error>;

/// TCP server to handle incoming network connections and setting up clients for the Game Manager.
///
/// This server first sets up the internal Game Manager and then listens for incoming network
/// connections. An accepted connection will first be transformed to a ClientSession,
/// then linked to the Game Manager via internal channels, and finally moved into its own task.
pub struct Server {
    _listener: Option<TcpListener>, // listen port for incoming connections
    client_id_counter: ClientId,
    tls_server: Arc<TlsServer>,
}

impl Server {
    pub fn new() -> Server {
        let tls_server = TlsServer::new().expect("failed to create TlsServer");

        Server {
            _listener: None,
            client_id_counter: 0,
            tls_server: Arc::new(tls_server),
        }
    }

    /// run the server.
    /// this creates the GameManager task and listens for incoming connections,
    /// which will then be converted to client tasks and linked to the Game Manager.
    pub async fn run(&mut self, port: u16) -> Result<()> {
        // N-to-1 client-Server channel
        // server sets up the channel through which clients communicate to server.
        // client_tx: transmitter for the client to the server.
        // srv_rx: receiver for the server for client messages.
        let (client_tx, srv_rx) = unbounded();

        // Game Manager gets the receiver of the channel
        let mut game_manager = GameManager::new(srv_rx);
        smol::spawn(async move {
            game_manager.run().await;
        })
        .detach();

        log::info!("start listening on port {}.", port);
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
        // listen for incoming connections. An accepted connection will be converted to a client task.
        loop {
            let (socket, addr) = listener.accept().await?;
            log::info!("accepted connection from {}!", addr);

            let tls_server = self.tls_server.clone();
            let client_tx = client_tx.clone();

            self.client_id_counter += 1;
            let id = self.client_id_counter;

            smol::spawn(async move {
                match tls_server.to_tls(socket).await {
                    Ok(tls_stream) => {
                        let net_client = ClientSession::new(id, tls_stream, client_tx).await;
                        net_client.run().await;
                    }
                    Err(e) => {
                        log::error!("TLS handshake failed for {}: {}", addr, e);
                    }
                }
            })
            .detach();
        }
    }
}
