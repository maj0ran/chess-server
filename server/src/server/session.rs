use crate::net::tls::TlsStream;
use chess_core::net::Connection;
use chess_core::protocol::messages::{ClientMessage, ServerMessage};
use chess_core::protocol::parser::NetMessage;
use chess_core::*;
use ::log;
use smol::channel::{unbounded, Receiver, Sender};
use smol::future;
use smol::net::TcpStream;

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
    pub conn: Connection<TlsStream<TcpStream>>, // connection to the outside client
    pub srv: ServerInterface,                   // connection to the internal server
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
        stream: TlsStream<TcpStream>,
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
            Err(e) => log_proc!(error, "got new client but failed to register with game manager: {e}"),
        }

        let mut client = ClientSession {
            id,
            conn: Connection::new(stream),
            srv: ServerInterface { tx, rx },
        };

        // send the assigned client ID back over the network immediately, so the remote client knows it
        if let Err(e) = client
            .conn
            .write_out(&ServerMessage::LoginAccepted(id).to_bytes())
            .await
        {
            log_send!(error, "failed to send login event to client #{}: {}", id, e);
        }

        client
    }

    /// Run the `ClientSession`.
    /// Here, we listen periodically for messages from both sides: The remote client and the
    /// internal `GameManager`. `GameManager` messages will be forwarded to the remote client,
    /// and client messages will be forwarded to the `GameManager`.
    /// listening on either side is done by two separate sub-tasks that run concurrently.
    pub async fn run(self) {
        let id = self.id;
        let srv_tx = self.srv.tx;
        let srv_rx = self.srv.rx;

        // network connections for sending and receiving to/from the remote client.
        // we split the connection we have into a read and write half. This way,
        // we can move each half into their own task, effectively using them
        // concurrently.
        let (mut conn_in, mut conn_out) = self.conn.split();

        // Sub-task that first receives a `ClientMessage` from the remote client and then sends it
        // to the internal `GameManager` along with the client's ID that is attached to this session.
        let listen_on_client = async move {
            loop {
                match conn_in.read_msg::<ClientMessage>().await {
                    Ok(cmd) => {
                        log_recv!(debug, "message from client #{}: {:?}", id, cmd);
                        if let Err(e) = srv_tx.send((id, cmd)).await {
                            log_recv!(warn, "error sending client message to GM!: {}", e);
                        };
                    }
                    Err(e) => {
                        match e {
                            NetError::Io(_) => {}
                            NetError::Protocol(_) => {}
                            NetError::Disconnected => {
                                let msg = ClientMessage::LeaveGame(0); // 0 = All Games
                                let _ = srv_tx.send((id, msg)).await.unwrap();
                                break;
                            }
                        }
                        log_recv!(warn, "failed to read message from client #{}: {}", id, e);
                        break;
                    }
                }
            }
        };

        // Sub-task that first receives messages from the internal `GameManager`
        // and then sends it to the remote client.
        let listen_on_game_manager = async move {
            let id = self.id;
            loop {
                match srv_rx.recv().await {
                    Ok(msg) => {
                        match conn_out.write_out(&msg.to_bytes()).await {
                           Ok(_) => {
                               log_send!(debug, "message to client #{}: {:?}", id, msg);
                           }
                           Err(e) => {
                               log_send!(error, "error sending message to client #{}: {}", id, e);
                           }
                        }
                    }
                    Err(e) => {
                        log_send!(warn, "error receiving channel message from GM!: {}", e);
                        break;
                    }
                }
            }
        };

        future::zip(listen_on_client, listen_on_game_manager).await;
    }
}
