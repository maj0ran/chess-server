use smol::channel::{unbounded, Receiver, Sender};
use smol::future;
use smol::net::TcpStream;

use crate::net::connection::connection::Connection;
use crate::net::ServerMessage;
use crate::net::*;
use crate::{pieces::Piece, tile::Tile};

pub struct ServerInterface {
    pub tx: Sender<ServerMessage>,
    pub rx: Receiver<ServerMessage>,
}

pub struct NetClient {
    id: ClientId,

    pub conn: Connection,
    pub srv: ServerInterface,
}

impl NetClient {
    /*
     * create a new network handler.
     * this is the interface to the remote network client.
     */
    pub async fn new(id: ClientId, socket: TcpStream, tx: Sender<ServerMessage>) -> Self {
        // 1-to-1 channel Client-Server
        // client sents up the channel through which server communicates to client.
        let (srv_tx, rx) = unbounded();
        tx.send(ServerMessage {
            client_id: id,
            cmd: Command::Register(srv_tx),
        })
        .await;

        NetClient {
            id: 1,
            conn: Connection::new(socket),
            srv: ServerInterface { tx, rx },
        }
    }

    pub async fn send_message(&self, cmd: Command) -> Result<()> {
        let client_id = self.id;
        let msg = ServerMessage { client_id, cmd };
        match self.srv.tx.send(msg).await {
            Ok(_) => Ok(()),
            Err(_) => todo!(),
        }
    }

    /*
     * read from buffer and execute the given command
     */
    pub async fn handle_request(&mut self, cmd: Option<Command>) -> bool {
        let cmd = match cmd {
            Some(c) => c,
            None => {
                log::warn!("could not parse request!");
                Command::_Invalid
            }
        };

        match cmd {
            Command::Update(items) => {
                log::info!("[ClientHandler] received Update from GameManager");
                let update_msg = NetClient::build_update_message(items);
                self.conn.write_out(update_msg.as_slice()).await;
            }
            _ => {
                let res = self.send_message(cmd).await;
                log::info!("[ClientHandler] received Command from Network");
                match res {
                    Ok(_) => {}
                    Err(e) => log::warn!("request handling failed!: {e}"),
                }
            }
        }

        true
    }

    /*
     * run the handler.
     * this listens for new incoming messages on the network interface.
     */
    pub async fn run(&mut self) {
        let conn = &mut self.conn;
        let srv = &mut self.srv;

        let f1 = async {
            let cmd = conn.read().await;
            cmd
        };

        let f2 = async {
            let msg = srv.rx.recv().await;
            match msg {
                Ok(msg) => Some(msg.cmd),
                Err(e) => {
                    log::warn!("[ClientHandler] error receiving channel message!: {}", e);
                    None
                }
            }
        };

        let cmd = future::or(f1, f2).await;

        self.handle_request(cmd).await;
    }

    fn build_update_message(tiles: Vec<(Tile, Option<Piece>)>) -> Vec<u8> {
        let mut msg = vec![UPDATE_BOARD]; // vector with update opcode as first entry
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
