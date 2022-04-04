use crate::*;
use std::fmt;
use std::ops::{self, Range, RangeTo};

/// A trivial network buffer class for sending and receiving data.
/// Has a fixed max length because it's freaking chess.
/// Is used by a connection to buffer sending and receiving data.
#[derive(Clone, Copy)]
pub struct Buffer {
    pub buf: [u8; Buffer::BUF_LEN],
    pub len: usize,
}

impl Buffer {
    pub const BUF_LEN: usize = 256;

    pub fn new() -> Buffer {
        Buffer {
            buf: [0; Buffer::BUF_LEN],
            len: 0,
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

/// For printing the buffer, we have some special colors to better visualize
/// different parts of the messages.
/// TODO: not very consistent yet.
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
