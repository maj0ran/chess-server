use chess_core::error::*;
use chess_core::net::*;
use chess_core::protocol::messages::{ClientMessage, ServerMessage};
use chess_core::protocol::parser::NetMessage;
use chess_core::ClientId;
use smol::channel::{unbounded, Receiver, Sender};
use smol::future;
use smol::net::TcpStream;

pub type Result<T> = std::result::Result<T, std::io::Error>;

/// Interface for a client session to the internal `GameManager`.
/// Outside clients are represented by `ClientSession`, and a `ClientSession` will have
/// this interface to further communicate with the internal `GameManager`.
/// This is needed because `ClientSession` and `GameManager` are in different tasks.
pub struct ServerInterface {
    pub tx: Sender<(ClientId, ClientMessage)>, // sends messages to the internal server
    pub rx: Receiver<ServerMessage>,           // receives messages from the internal server
}

/// A `ClientSession` is a connection between a remote network client and the server.
/// A `ClientSession`s is constructed by the server for each accepted network connection.
/// The `ClientSession` is then moved into its own thread and linked to the `GameManager` via channels.
/// I.e., further communication is between `ClientSession` and the `GameManager`, not the `Server`, which
/// is only responsible for accepting incoming connections.
pub struct ClientSession {
    id: ClientId,
    pub conn: Connection,     // connection to the outside client
    pub srv: ServerInterface, // connection to the internal server
}

impl ClientSession {
    /// Creates a new `ClientSession` for a connected client.
    /// This is the interface to the remote network client.
    /// Params:
    /// id: the internal client ID
    /// tx: Transmitter to the Game Manager.
    ///     This is set up by the `Server` and used to communicate with the `GameManager`.
    pub async fn new(
        id: ClientId,
        socket: TcpStream,
        tx: Sender<(ClientId, ClientMessage)>,
    ) -> Self {
        // From the parameter we already got the transmitter to the Game Manager, which is constructed by the server.
        // Now we construct another channel for the reverse direction.
        // We use the transmitter we have to send the transmitter to this client to the Game Manager.
        // Now have a (tx,rx) pair from a client to game manager and a (tx,rx) pair from game manager to a client.
        let (srv_tx, rx) = unbounded();
        let res = tx.send((id, ClientMessage::Register(srv_tx))).await;

        match res {
            Ok(_) => {}
            Err(e) => log::error!("got new client but failed to register with game manager: {e}"),
        }

        let mut client = ClientSession {
            id,
            conn: Connection::new(socket),
            srv: ServerInterface { tx, rx },
        };

        // send the assigned client ID back over the network immediately, so the remote client knows it
        if let Err(e) = client
            .conn
            .write_out(&ServerMessage::LoginAccepted(id).to_bytes())
            .await
        {
            log::error!("failed to send login event to client #{}: {}", id, e);
        }

        client
    }

    /// Takes a `ClientMessage` that has been received from the remote client and sends it to the internal
    /// `GameManager` along with the clients ID.
    /// Note: The original message has no client ID because the client is bound to its
    /// session. But from here, we forward the message to the `GameManager`,
    /// so the client ID gets attached here, so the `GameManager` can identify the client.
    pub async fn handle_incoming_message(
        id: ClientId,
        srv_tx: &Sender<(ClientId, ClientMessage)>,
        cmd: ClientMessage,
    ) -> Result<()> {
        log::debug!("receiving message from client: {:?}", cmd);
        if let Err(e) = srv_tx.send((id, cmd)).await {
            log::warn!("error sending client message to GM!: {}", e);
        };

        Ok(())
    }

    /// takes a message that has been received from the internal `GameManager`
    /// and sends it to the remote client.
    pub async fn handle_outgoing_message(
        conn: &mut Connection,
        msg: ServerMessage,
    ) -> NetResult<()> {
        log::debug!("sending message to client: {:?}", msg);
        conn.write_out(&msg.to_bytes()).await
    }

    /// Run the `ClientSession`.
    /// Here, we listen periodically for messages from both sides: The remote client and the
    /// internal `GameManager`. `GameManager` messages will be forwarded to the remote client,
    /// and client messages will be forwarded to the `GameManager`.
    pub async fn run(&mut self) {
        let id = self.id;

        // network connections for sending and receiving to/from the remote client.
        // we split the connection we have into a read and write half. This way,
        // we can move each half into their own task, effectively using them
        // concurrently.
        // Using 'clone()' on rust TcpStreams is fine as those are only handles and
        // safety is ensured by the TcpStream implementation.
        let mut conn_in = self.conn.clone();
        let mut conn_out = &mut self.conn;
        // channel connections for game manager communication.
        let srv_tx = &self.srv.tx;
        let srv_rx = &self.srv.rx;

        // receiving messages from the remote client and forwarding them to the GameManager
        let listen_on_client = async move {
            loop {
                match conn_in.read_msg::<ClientMessage>().await {
                    Ok(cmd) => {
                        let result = ClientSession::handle_incoming_message(id, &srv_tx, cmd).await;
                        if let Err(e) = result {
                            log::error!("error handling incoming message: {}", e);
                        }
                    }
                    Err(e) => {
                        match e {
                            NetError::Io(_) => {}
                            NetError::Protocol(_) => {}
                            NetError::Disconnected => {
                                let msg = ClientMessage::LeaveGame;
                                srv_tx.send((id, msg)).await.unwrap();
                            }
                        }
                        log::warn!("failed to read message from client #{}: {}", id, e);
                        break;
                    }
                }
            }
        };

        // receiving messages from the GameManager and forwarding them to the network client
        let listen_on_game_manager = async move {
            loop {
                match srv_rx.recv().await {
                    Ok(msg) => {
                        if let Err(e) =
                            ClientSession::handle_outgoing_message(&mut conn_out, msg).await
                        {
                            log::error!("error sending message to client #{}: {}", id, e);
                            break;
                        }
                    }
                    Err(e) => {
                        log::warn!("error receiving channel message from GM!: {}", e);
                        break;
                    }
                }
            }
        };

        future::zip(listen_on_client, listen_on_game_manager).await;
    }
}
