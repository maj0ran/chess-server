use smol::channel::{unbounded, Receiver, Sender};
use smol::future;
use smol::net::TcpStream;

use crate::net::connection::connection::Connection;
use crate::net::ClientToServerMessage;
use crate::net::*;
use crate::{pieces::Piece, tile::Tile};

/* wrapper for both method directions.
 * This seems a bit wonky, but the reason is, that we have two tasks in the run() method, one for
 * incoming and one for outgoing messages. We listen on these tasks to determine which returns data
 * first and they both have to return the same data type.
 */
pub enum Message {
    FromClient(ClientToServerMessage),
    FromServer(ServerToClientMessage),
}
pub struct ServerInterface {
    pub tx: Sender<ClientToServerMessage>, // sends messages to the internal server
    pub rx: Receiver<ServerToClientMessage>, // receives messages from the internal server
}

pub struct NetClient {
    id: ClientId,
    pub conn: Connection,     // connection to the outside client
    pub srv: ServerInterface, // connection to the internal server
}

impl NetClient {
    /*
     * create a new network handler.
     * this is the interface to the remote network client.
     * param:
     * id: the internal client ID
     * tx: Transmitter which is setup by the internal server.
     *     This is used by the NetClient to communicate back to the internal server
     */
    pub async fn new(id: ClientId, socket: TcpStream, tx: Sender<ClientToServerMessage>) -> Self {
        // 1-to-1 channel Client-Server
        // client sents up the channel through which server communicates to client.
        // `srv_tx` is the transmitter for the server. This transmitter is sent back via `tx` which we
        // got from the server.
        let (srv_tx, rx) = unbounded();
        let res = tx
            .send(ClientToServerMessage {
                client_id: id,
                cmd: Command::Register(srv_tx),
            })
            .await;

        match res {
            Ok(_) => {}
            Err(e) => log::error!("got new client but failed to register with game manager: {e}"),
        }

        let mut client = NetClient {
            id,
            conn: Connection::new(socket),
            srv: ServerInterface { tx, rx },
        };
        // send the ID back immediately so the remote client knows it
        let id_bytes = id.to_le_bytes();
        client
            .conn
            .write_out(&[
                opcode::LOGIN,
                id_bytes[0],
                id_bytes[1],
                id_bytes[2],
                id_bytes[3],
            ])
            .await;

        client
    }

    pub async fn handle_incoming_message(&self, cmd: Command) -> Result<()> {
        let msg = ClientToServerMessage {
            client_id: self.id,
            cmd,
        };
        match self.srv.tx.send(msg).await {
            Ok(_) => Ok(()),
            Err(_) => todo!(),
        }
    }

    pub async fn handle_outgoing_message(&mut self, msg: ServerToClientMessage) {
        match msg.msg {
            Response::Update(items) => {
                log::info!("received board update from GameManager");
                let update_msg = NetClient::build_update_message(items);
                self.conn.write_out(update_msg.as_slice()).await;
            }
            Response::GameCreated(game_id, client_id) => {
                log::info!("received new game from GameManager");
                let game_id = game_id.to_le_bytes();
                let client_id = client_id.to_le_bytes();
                let data = [
                    opcode::GAME_CREATED,
                    game_id[0],
                    game_id[1],
                    game_id[2],
                    game_id[3],
                    0x20,
                    client_id[0],
                    client_id[1],
                    client_id[2],
                    client_id[3],
                ];
                self.conn.write_out(&data).await;
            }
            Response::GameJoined(game_id, client_id, side) => {
                log::info!("received game joined from GameManager");
                let game_id = game_id.to_le_bytes();
                let client_id = client_id.to_le_bytes();
                let data = [
                    opcode::JOIN_GAME,
                    game_id[0],
                    game_id[1],
                    game_id[2],
                    game_id[3],
                    0x20,
                    client_id[0],
                    client_id[1],
                    client_id[2],
                    client_id[3],
                    0x20,
                    side as u8,
                ];
                self.conn.write_out(&data).await;
            }
        }
    }

    /*
     * Forwards a message either to the internal server or to the remote client, depending on the
     * message.
     */
    pub async fn forward_message(&mut self, cmd: Message) -> bool {
        match cmd {
            Message::FromServer(msg) => {
                self.handle_outgoing_message(msg).await;
            }
            Message::FromClient(msg) => {
                self.handle_incoming_message(msg.cmd).await;
            }
        }

        true
    }

    /*
     * run the handler.
     * listens for new incoming messages on the network interface.
     */

    pub async fn run(&mut self) {
        let conn = &mut self.conn;
        let srv = &mut self.srv;

        // this task listens on the remote client connection for new messages coming from over the
        // network. These messages will be forwarded to the internal server.
        let listen_on_client = async {
            let cmd = conn.read().await.unwrap();
            let msg = ClientToServerMessage {
                client_id: self.id,
                cmd,
            };
            Message::FromClient(msg)
        };

        // this task listens on the internal channel for new messages coming from the
        // internal server. These messages will be forwarded to the remote client.
        let listen_on_server = async {
            let msg = srv.rx.recv().await;
            match msg {
                Ok(msg) => Message::FromServer(msg),
                Err(e) => {
                    log::warn!("[ClientHandler] error receiving channel message!: {}", e);
                    panic!()
                }
            }
        };

        // get data from whichever tasks signals first.
        let message = future::or(listen_on_client, listen_on_server).await;

        // we have to handle both possible data sets (incoming & outgoing) here; handling them
        // independently inside the listener-tasks above would require to borrow &self in both
        // tasks, which is invalid.
        // TODO: maybe we could seperate outgoing and incoming logic in seperate structs, so
        // borrowing the whole self is not needed. this would mean some refactoring, but we could
        // also get rid of the Message-Wrapper.
        self.forward_message(message).await;
    }

    fn build_update_message(tiles: Vec<(Tile, Option<Piece>)>) -> Vec<u8> {
        let mut msg = vec![opcode::BOARD_UPDATED]; // vector with update opcode as first entry
        for u in &tiles {
            let mut tile = u.0.to_string().as_bytes().to_owned();
            let piece = match u.1 {
                Some(p) => p.as_byte(),
                None => 'X', // tile updated to 'empty'
            };

            msg.append(&mut tile);
            msg.push(piece as u8); // each updated tile is 5 bytes
        }

        msg
    }
}
