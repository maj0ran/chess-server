use crate::{net::*, pieces::Piece, tile::Tile, util::*};
use core::fmt;
use smol::net::*;
use std::ops::{self, Range, RangeTo};

use super::BUF_LEN;

pub struct Buffer {
    pub buf: [u8; BUF_LEN],
    pub len: usize,
    cursor: usize,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            buf: [0; BUF_LEN],
            len: 0,
            cursor: 0,
        }
    }

    /* touches the cursor */
    pub fn get_byte(&mut self) -> u8 {
        let byte = self.buf[self.cursor];
        self.cursor += 1;
        byte
    }

    pub fn get_to_end(&mut self) -> &[u8] {
        &self.buf[self.cursor..self.len]
    }
    pub async fn write(&mut self, conn: &mut TcpStream) -> bool {
        trace!("Out Buffer: {} (Length: {})", &self, self.len);

        //    let len = self.len;
        //    let r = conn.write_all(&self[..len as usize]).await; // send data to client
        //                                                         //
        //    match r {
        //        Ok(_) => {
        //            debug!("wrote {} bytes", len);
        //            true
        //        }
        //        Err(e) => {
        //            error!("Error writing stream: {}", e);
        //            false
        //        }
        //    }
        true
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

impl ops::Index<Range<usize>> for Buffer {
    type Output = [u8];

    fn index(&self, index: Range<usize>) -> &Self::Output {
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
