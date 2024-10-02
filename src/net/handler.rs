use std::sync::Arc;
use std::sync::Mutex;

use bytes::BytesMut;
use server::Server;
use tokio::io::ReadHalf;
use tokio::io::WriteHalf;
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpStream;
use tokio::sync::broadcast;

use crate::chessmove::*;
use crate::game::Chess;
use crate::net::connection::connection::Connection;
use crate::net::frame::Frame;
use crate::net::Command;
use crate::net::*;
use crate::util::*;
use crate::{pieces::Piece, tile::Tile};

pub struct Handler {
    pub name: String,
    pub chess: Arc<Mutex<Chess>>,
    pub conn: Connection,
    pub notify_move: broadcast::Receiver<Vec<(Tile, Option<Piece>)>>,
    pub buffer: BytesMut,
}

impl Handler {
    pub async fn run(&mut self) {
        let cmd = tokio::select! {
            res = self.conn.read() => () ,
            res = self.notify_move.recv() => ()
        };
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
            len: self.conn.buf.len as u8,
        }
    }
}
