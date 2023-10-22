use log::{debug, error, info, trace, warn};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::color::Color;
use crate::game::Chess;
use crate::pieces::ChessPiece;
use crate::tile::ToChessMove;
use crate::util::*;

use super::buffer::Buffer;
use super::{Command, NewGame, Response, BUF_LEN};

pub struct Connection {
    pub client: TcpStream,
    pub nickname: String,
    pub chess: Option<Chess>,
    pub in_buf: Buffer,
    pub out_buf: Buffer,
}

impl Connection {
    fn is_ingame(&self) -> bool {
        self.chess.is_some()
    }
    async fn wait_for_message(&mut self) -> bool {
        // read the first byte which indicates the length.
        // this value will be discarded and not be part of the read buffer
        let len = self.client.read_u8().await;
        let len = match len {
            Ok(n) => {
                if n as usize > BUF_LEN {
                    error!("message-length too big!: {}", n);
                    return false;
                }
                n
            }
            Err(e) => {
                error!("error at reading message length: {}", e);
                panic!("eof");
            }
        };

        self.in_buf.len = len as usize;
        // read the actual message into the read buffer
        let n = self
            .client
            .read_exact(&mut self.in_buf[..len as usize])
            .await;
        match n {
            Ok(0) => {
                info!("remote closed connection!");
                false
            } // connection closed
            Err(e) => {
                error!("Error at reading TcpStream: {}", e);
                false
            }
            Ok(n) => {
                trace!("In Buffer: {} (Length: {n})", self.in_buf);
                true
            }
        }
    }

    fn new_game(&mut self, new_game: NewGame) {
        info!(
            "New Game! (\"{}\" ({}) hoster side: {:?})",
            new_game.name, new_game.mode, new_game.hoster_side
        );
        self.chess = Some(Chess::new());
    }

    fn exec(&mut self, cmd: Command) -> bool {
        match cmd {
            Command::Nickname(name) => {
                self.nickname = name;
                true
            }
            Command::NewGame(new_game) => {
                self.new_game(new_game);
                true
            }
            Command::JoinGame(id) => {
                self.join_game(id);
                true
            }
            Command::Move(mov) => {
                if self.is_ingame() {
                    let (src, dst) = if let Some(unpacked_mov) = mov.to_chess() {
                        unpacked_mov
                    } else {
                        warn!(
                            "cannot parse move: {style_bold}{fg_red}{}{style_reset}{fg_reset}",
                            mov
                        );
                        return false;
                    };
                    let chess = self.chess.as_mut().unwrap(); // we are ingame, so there must be a
                    let changes = chess.make_move(src, dst);
                    debug!("make move!");

                    if changes.is_empty() {
                        info!("illegal chess move: {style_bold}{fg_yellow}{}{}{style_reset}{fg_reset}", src, dst);
                        self.out_buf[0] = 1;
                        self.out_buf[1] = 1;
                        self.out_buf.len = 2;
                        trace!("Out Buffer: {}", self.out_buf);
                        return false;
                    };

                    debug!(
                        "executed move: {style_bold}{fg_green}{}{}{style_reset}{fg_reset}!",
                        src, dst
                    );
                    println!("{}", chess);
                    self.out_buf.len = changes.len() * 2 + 1;
                    self.out_buf[0] = (changes.len() * 2) as u8;

                    for (i, (tile, piece)) in changes.iter().enumerate() {
                        let tile_byte = tile.to_index();
                        let piece_byte = match piece {
                            None => 0,
                            Some(p) => p.typ as u8 | p.color as u8,
                        };
                        self.out_buf[1 + (i * 2)] = tile_byte;
                        self.out_buf[1 + (i * 2 + 1)] = piece_byte;
                    }
                    trace!("Out Buffer: {}", self.out_buf);
                    true
                } else {
                    false // ingame but not a chess move
                }
            }
            Command::Invalid => {
                warn!("Invalid Command received!: {:?}", cmd);
                false
            }
        }
    }

    fn join_game(&self, id: String) {
        info!("Join Game. id: {:?}", id)
    }
    pub async fn run(&mut self) {
        println!("Hello, {}!", self.nickname);
        loop {
            // only reading the message, no further validation.
            // this blocks the task until a full message is available
            if self.wait_for_message().await {
            } else {
                error!("error while waiting for message!");
                continue;
            };

            // now we interpret the message
            let cmd = if let Some(cmd) = self.in_buf.parse() {
                cmd
            } else {
                error!("invalid command received!");
                continue;
            };

            if !self.exec(cmd) {
                info!("{fg_red}Command failed!{fg_reset}");
            } else {
                info!("{fg_green}Command executed!{fg_reset}");
            }
            let _ = self
                .client
                .write_all(&self.out_buf.buf[..self.out_buf.len + 1])
                .await; // send data to client
            let _ = self.client.flush().await;
        }
    }
}
