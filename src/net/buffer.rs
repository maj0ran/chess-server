use core::fmt;
use std::ops::{self, RangeTo};

use log::{debug, error, warn};

use crate::{
    net::{PlayerSideRequest, *},
    pieces::Piece,
    tile::Tile,
    util::*,
};

use super::{Command, NewGame, Parameter, BUF_LEN};

pub struct Buffer {
    pub buf: [u8; BUF_LEN],
    pub len: usize,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            buf: [0; BUF_LEN],
            len: 0,
        }
    }

    pub fn write_move_response(&mut self, changes: &[(Tile, Option<Piece>)]) {
        self.len = changes.len() * 2 + 1;
        self[0] = (changes.len() * 2) as u8;

        for (i, (tile, piece)) in changes.iter().enumerate() {
            let tile_byte = tile.to_index();
            let piece_byte = match piece {
                None => 0,
                Some(p) => p.typ as u8 | p.color as u8,
            };
            self[1 + (i * 2)] = tile_byte;
            self[1 + (i * 2 + 1)] = piece_byte;
        }
    }

    pub fn parse(&mut self) -> Option<Command> {
        let tokens: Vec<&[u8]> = self[..self.len].split(|s| &b' ' == s).collect();
        let cmd = tokens[0];
        let params = &tokens[1..];
        if cmd.len() == 0 {
            warn!("got message but no content");
            return None;
        }
        debug!("cmd: {}", cmd[0]);
        let ret = match cmd[0] {
            NEW_GAME => {
                if params.len() != 2 {
                    error!("host: invalid number of params received!: {}", params.len());
                    return None;
                }
                let mode = params[0].to_val();
                let side: u8 = params[1].to_val();
                let side = PlayerSideRequest::try_from(side);
                let side = match side {
                    Ok(s) => s,
                    Err(_) => {
                        warn!("invalid Side chosen! default to random");
                        PlayerSideRequest::Random
                    }
                };
                let new_game = NewGame::new(mode, side);
                Some(Command::NewGame(new_game))
            }

            JOIN_GAME => {
                if params.len() != 1 {
                    error!("join: invalid number of params received!: {}", params.len());
                    return None;
                }
                let game_name = params[0].to_val();
                Some(Command::JoinGame(game_name))
            }
            SET_NAME => Some(Command::Nickname(params[0].to_val())),
            _ => {
                // ingame Move
                let mov = String::from_utf8_lossy(cmd).to_string();
                Some(Command::Move(mov))
            }
        };
        // we got a new message so we clear our read buffer
        // //for i in 0..BUF_LEN {
        //     self[i as usize] = 0;
        // };
        ret
    }
}

impl ops::Index<usize> for Buffer {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buf[index]
    }
}
impl ops::Index<RangeTo<usize>> for Buffer {
    type Output = [u8];

    fn index(&self, index: RangeTo<usize>) -> &Self::Output {
        &self.buf[index]
    }
}
impl ops::IndexMut<usize> for Buffer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.buf[index]
    }
}
impl ops::IndexMut<RangeTo<usize>> for Buffer {
    fn index_mut(&mut self, index: RangeTo<usize>) -> &mut Self::Output {
        &mut self.buf[index]
    }
}

impl fmt::Display for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf_str = style_bold.to_string();
        for i in &self.buf[..self.len] {
            let col = match i {
                b' ' => fg_red,
                _ if i < &32 => fg_yellow,
                _ => fg_blue,
            };
            buf_str = buf_str + &format!("{col}[{i}]");
        }
        buf_str = buf_str + fg_reset + style_reset;
        write!(f, "{}", buf_str)
    }
}
