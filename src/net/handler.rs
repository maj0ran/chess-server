use smol::lock::Mutex;
use std::sync::Arc;

use bytes::BytesMut;

use crate::chessmove::*;
use crate::game::Chess;
use crate::net::connection::connection::Connection;
use crate::net::Command;
use crate::net::*;
use crate::{pieces::Piece, tile::Tile};

pub struct Handler {
    pub name: String,
    pub chess: Arc<Mutex<Chess>>,
    pub conn: Connection,
    pub buffer: BytesMut,
}

impl Handler {
    pub async fn run(&mut self) {
        debug!("running handler: {}", self.name);
        // handler switches between two tasks: listening to the net client and listening for
        // processed moves on the server
        let res = self.conn.read().await;
        self.handle_request().await;
    }

    fn parse(&self) -> Option<Command> {
        let len = self.conn.buf.len;
        if len == 0 {
            log::warn!("parse: zero-length message");
            return None;
        }

        log::debug!("parsing message with length {}", len);

        let content = &self.conn.buf[..len];
        let cmd = content[0];
        let params = &content[2..len];
        log::info!("got len: {}, cmd: {}", len, cmd);
        let params: Vec<&[u8]> = params.split(|c| *c == b' ' as u8).collect();
        log::debug!("{:?}", params);

        let ret = match cmd {
            NEW_GAME => {
                if params.len() != 2 {
                    log::error!("host: invalid number of params received!: {}", params.len());
                    return None;
                }
                let mode = params[0].to_val();
                let side: u8 = params[1].to_val();
                let side = PlayerSideRequest::try_from(side);
                let side = match side {
                    Ok(s) => s,
                    Err(_) => {
                        log::warn!("invalid side chosen! default to random");
                        PlayerSideRequest::Random
                    }
                };
                let new_game = NewGame::new(mode, side);
                Some(Command::NewGame(new_game))
            }

            JOIN_GAME => {
                if params.len() != 1 {
                    log::error!("join: invalid number of params received!: {}", params.len());
                    return None;
                }
                let game_name = params[0].to_val();
                Some(Command::JoinGame(game_name))
            }
            SET_NAME => Some(Command::Nickname(params[0].to_val())),
            MAKE_MOVE => {
                // ingame Move
                let mov = params[0].to_val();
                Some(Command::Move(mov))
            }
            _ => {
                log::error!("parse: invalid command");
                None
            }
        };
        // we got a new message so we clear our read buffer
        // //for i in 0..BUF_LEN {
        //     self[i as usize] = 0;
        // };
        ret
    }

    async fn handle_request(&mut self) -> bool {
        let cmd = self.parse(); // parse the current buffer state
        self.exec(cmd.unwrap()).await;

        true
    }

    pub fn new_game(&mut self, new_game: NewGame) {
        info!(
            "New Game! (\"{}\" ({}) hoster side: {:?})",
            new_game.name, new_game.mode, new_game.hoster_side
        );
    }

    pub async fn make_move(&mut self, chessmove: ChessMove) -> Vec<(Tile, Option<Piece>)> {
        let mut chess = self.chess.lock().await;
        let changes = chess.make_move(chessmove);

        changes
    }

    pub fn join_game(&self, id: String) {
        info!("Join Game. id: {:?}", id)
    }
    pub async fn exec(&mut self, cmd: Command) -> Result<()> {
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
                let mut chess = self.chess.lock().await;
                let updated_tiles = chess.make_move(chessmove);
                /* build response message */
                let msg = build_update_message(updated_tiles);

                self.conn.write_out(msg.as_slice()).await;

                Ok(())
            }
            Command::_Invalid => {
                warn!("Invalid Command received!: {:?}", cmd);
                Ok(())
            }
            Command::Update(vec) => todo!(),
        }
    }
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
