use crate::{NetError, NetResult};

pub mod messages;
pub mod parser;

#[derive(Debug, Clone)]
pub struct NewGameParams {
    pub mode: u8,
    pub time: u32,
    pub time_inc: u32,
}

impl NewGameParams {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![self.mode, 0];
        bytes.extend_from_slice(&self.time.to_le_bytes());
        bytes.extend_from_slice(&self.time_inc.to_le_bytes());
        bytes
    }
}

#[derive(Debug, Clone)]
pub struct JoinGameParams {
    pub game_id: u32,
    pub side: UserRoleSelection,
}

impl JoinGameParams {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend_from_slice(&self.game_id.to_le_bytes());
        bytes.push(self.side as u8);
        bytes
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserRoleSelection {
    Black = 0,
    White = 1,
    Random = 2,
    Spectator = 3,
    Both = 4,
}

impl UserRoleSelection {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => UserRoleSelection::Black,
            1 => UserRoleSelection::White,
            2 => UserRoleSelection::Random,
            3 => UserRoleSelection::Spectator,
            4 => UserRoleSelection::Both,
            _ => UserRoleSelection::Spectator,
        }
    }
}

/// A trivial Reader for network messages. Gives us the possibility to read
/// fields from the byte stream in a simpler way.
pub struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    pub fn read_u8(&mut self) -> NetResult<u8> {
        if self.offset < self.bytes.len() {
            let val = self.bytes[self.offset];
            self.offset += 1;
            Ok(val)
        } else {
            Err(NetError::Protocol("Unexpected end of data".to_string()))
        }
    }

    pub fn read_u32_le(&mut self) -> NetResult<u32> {
        if self.offset + 4 <= self.bytes.len() {
            let val =
                u32::from_le_bytes(self.bytes[self.offset..self.offset + 4].try_into().unwrap());
            self.offset += 4;
            Ok(val)
        } else {
            Err(NetError::Protocol("Unexpected end of data".to_string()))
        }
    }

    pub fn read_str(&mut self, len: usize) -> NetResult<&'a str> {
        if self.offset + len <= self.bytes.len() {
            let s = std::str::from_utf8(&self.bytes[self.offset..self.offset + len])
                .map_err(|_| NetError::Protocol("Failed to parse string".to_string()))?;
            self.offset += len;
            Ok(s)
        } else {
            Err(NetError::Protocol("Unexpected end of data".to_string()))
        }
    }

    pub fn remaining(&self) -> &[u8] {
        &self.bytes[self.offset..]
    }
}
