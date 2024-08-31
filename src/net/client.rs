use std::sync::Arc;
use std::sync::Mutex;

use server::Server;
use tokio::net::TcpStream;

use crate::chessmove::*;
use crate::game::Chess;
use crate::net::connection::connection::Connection;
use crate::net::frame::Frame;
use crate::net::Command;
use crate::net::*;
use crate::util::*;
use crate::{pieces::Piece, tile::Tile};

pub struct Client {
    pub name: String,
    conn: TcpStream,
    buf: Buffer,
    pub chess: Arc<Mutex<Chess>>,
}

impl Client {
    pub fn new(name: String, conn: TcpStream, chess: Arc<Mutex<Chess>>) -> Client {
        Client {
            name,
            conn,
            buf: Buffer::new(),
            chess,
        }
    }

    pub fn new_game(&mut self, new_game: NewGame) {
        info!(
            "New Game! (\"{}\" ({}) hoster side: {:?})",
            new_game.name, new_game.mode, new_game.hoster_side
        );
    }

    pub fn make_move(&mut self, chessmove: ChessMove) -> Vec<(Tile, Option<Piece>)> {
        let mut chess = self.chess.lock().unwrap();
        let changes = chess.make_move(chessmove);

        changes
    }

    pub fn join_game(&self, id: String) {
        info!("Join Game. id: {:?}", id)
    }
    pub async fn read(&mut self) -> Option<Command> {
        // only reading the message, no further validation.
        // this blocks the task until a full message is available
        let frame: Option<Frame> = self.buf.read_frame(&mut self.conn).await;

        debug!("got new message");
        let frame = if let Some(f) = frame {
            f
        } else {
            warn!("error reading message");
            return None;
        };

        // now we interpret the message
        let cmd = if let Some(cmd) = frame.parse() {
            cmd
        } else {
            error!("{fg_red}invalid command received: !{fg_reset}");
            return None;
        };

        info!("{fg_green}Received command: {cmd}!{fg_reset}");

        Some(cmd)
        // and execute the command
        // let response = self.exec(cmd);
        // finally sent respond to client
        // if !self.buf.write(&mut self.conn, response).await {
        //     info!("{fg_red}sending command failed!: {fg_reset}");
        // }
    }
    pub fn exec(&mut self, cmd: Command) -> Result<()> {
        match cmd {
            Command::Nickname(name) => Ok(()),
            Command::NewGame(new_game) => Ok(()),
            Command::JoinGame(id) => Ok(()),
            Command::Move(mov) => {
                let chessmove = match mov.parse() {
                    Some(cm) => cm,
                    None => {
                        warn!("could not parse move: {}", mov);
                        return Ok(());
                    }
                };
                let mut chess = self.chess.lock().unwrap();
                chess.make_move(chessmove);
                Ok(())
            }
            Command::_Invalid => {
                warn!("Invalid Command received!: {:?}", cmd);
                Ok(())
            }
        }
    }

    pub fn create_frame(&self) -> Frame {
        Frame {
            len: self.buf.len as u8,
            content: self.buf.buf,
        }
    }
}
