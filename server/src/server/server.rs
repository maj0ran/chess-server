use super::manager::GameManager;
use super::session::ClientSession;
use chess_core::{ClientId, ClientMessage};
use smol::channel::{unbounded, Sender};
use smol::net::*;

pub type Result<T> = std::result::Result<T, std::io::Error>;

/// TCP server to handle incoming network connections and setting up clients for the Game Manager.
///
/// This server first sets up the internal Game Manager and then listens for incoming network
/// connections. An accepted connection will first be transformed to a ClientSession,
/// then linked to the Game Manager via internal channels, and finally moved into its own task.
pub struct Server {
    _listener: Option<TcpListener>, // listen port for incoming connections
    client_id_counter: ClientId,
}

impl Server {
    pub fn new() -> Server {
        Server {
            _listener: None,
            client_id_counter: 0,
        }
    }

    /// Creates a client handler for a new client connection.
    pub async fn create_client(
        &mut self,
        socket: TcpStream,
        tx_channel: Sender<(ClientId, ClientMessage)>,
    ) -> ClientSession {
        self.client_id_counter += 1;
        ClientSession::new(self.client_id_counter, socket, tx_channel).await
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
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
        // listen for incoming connections. An accepted connection will be converted to a client task.
        loop {
            let (socket, addr) = listener.accept().await?;
            // each client gets its own tx, all of them are bound to srv_rx
            let mut net_client = self.create_client(socket, client_tx.clone()).await;

            log::info!("accepted connection from {}!", addr);

            smol::spawn(async move {
                net_client.run().await;
            })
            .detach();
        }
    }
}
