use super::manager::GameManager;
use super::*;
use smol::channel::{unbounded, Receiver, Sender};
use smol::net::*;

pub struct Client {
    pub id: ClientId,
    pub tx: Sender<ServerMessage>,
}

impl Client {
    pub fn new(tx: Sender<ServerMessage>) -> Self {
        Client { id: 1, tx }
    }
}

pub struct Server {
    _listener: Option<TcpListener>,
    clients: Vec<Client>,

    client_tx: Sender<ServerMessage>,
    game_manager_rx: Receiver<ServerMessage>,

    client_id_counter: ClientId,
}

impl Server {
    pub fn new() -> Server {
        let (srv_tx, srv_rx) = unbounded();
        Server {
            _listener: None,
            clients: vec![],
            client_tx: srv_tx,
            game_manager_rx: srv_rx,
            client_id_counter: 0,
        }
    }

    pub async fn create_client(
        &mut self,
        socket: TcpStream,
        tx_channel: Sender<ServerMessage>,
    ) -> NetClient {
        self.client_id_counter += 1;
        NetClient::new(self.client_id_counter, socket, tx_channel).await
    }

    /*
     * run the server.
     * this creates the GameManager task and listens for incoming connections,
     * which will then converted to client tasks.
     */
    pub async fn run(&mut self) -> Result<()> {
        // N-to-1 channel Client-Server
        // server sets up the channel through which clients communicate to server.
        let (client_tx, srv_rx) = unbounded();

        let mut game_manager = GameManager::new(srv_rx);
        smol::spawn(async move {
            loop {
                game_manager.run().await;
            }
        })
        .detach();

        log::info!("start Listening.");
        let listener = TcpListener::bind("127.0.0.1:7878").await?;

        loop {
            let (socket, addr) = listener.accept().await?;
            let mut net_client = self.create_client(socket, client_tx.clone()).await;

            log::info!("accepted connection from {}!", addr);

            smol::spawn(async move {
                loop {
                    net_client.run().await;
                }
            })
            .detach();
        }
    }
}
