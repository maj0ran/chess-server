use core::fmt;
use std::ops::{self, RangeTo};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{
    net::{frame::Frame, *},
    pieces::Piece,
    tile::Tile,
    util::*,
};

use super::BUF_LEN;

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
    pub async fn read_frame(&mut self, conn: &mut TcpStream) -> Option<Frame> {
        log::trace!("In Buffer: {} (Length: {})", &self, self.len);
        // read the first byte which indicates the length.
        // this value will be discarded and not be part of the read buffer
        let len = conn.read_u8().await;
        let len = match len {
            Ok(n) => {
                if n as usize > BUF_LEN {
                    log::error!("message-length too big!: {}", n);
                    return None;
                }
                n
            }
            Err(e) => {
                log::error!("error at reading message length: {}", e);
                panic!("EOF when reading frame");
            }
        };

        self.len = len as usize;
        // read the actual message into the read buffer
        let n = conn.read_exact(&mut self.buf[..len as usize]).await;
        match n {
            Ok(0) => {
                log::info!("remote closed connection!");
                None
            } // connection closed
            Err(e) => {
                log::error!("Error at reading TcpStream: {}", e);
                None
            }
            Ok(n) => Some(Frame {
                len: n as u8,
                content: self.buf,
            }),
        }
    }

    pub fn write_frame(&mut self, data: &[u8]) {
        self[0] = data.len() as u8;
        self.len = self[0] as usize + 1;
        for (i, val) in data.iter().enumerate() {
            self[i + 1] = *val;
        }
    }

    pub async fn write(&mut self, conn: &mut TcpStream) -> bool {
        trace!("Out Buffer: {} (Length: {})", &self, self.len);

        let len = self.len;
        let r = conn.write_all(&self[..len as usize]).await; // send data to client
                                                             //
        match r {
            Ok(_) => {
                debug!("wrote {} bytes", len);
                true
            }
            Err(e) => {
                error!("Error writing stream: {}", e);
                false
            }
        }
    }

    /*
     * write a list of all fields that have changed into the buffer for writing out
     */
    pub fn fields_to_buffer(&mut self, changes: &[(Tile, Option<Piece>)]) {
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
