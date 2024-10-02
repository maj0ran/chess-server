pub mod buffer;
pub mod connection;
pub mod frame;
pub mod handler;
pub mod server;
use std::fmt;

use crate::net::buffer::Buffer;
use crate::net::connection::connection::Connection;
use crate::net::handler::Handler;

pub use log::{debug, error, info, trace, warn};
#[allow(unused)]

// bytes representing commands to server
const NEW_GAME: u8 = 0xA;
const JOIN_GAME: u8 = 0xB;
const SET_NAME: u8 = 0xC;

// read and write buffer have a static maximum length
// we don't really need more for chess
const BUF_LEN: usize = 64;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug)]
pub struct NewGame {
    pub mode: String,
    pub hoster_side: PlayerSideRequest,
    pub name: String,
}
impl NewGame {
    pub fn new(mode: String, side: PlayerSideRequest) -> NewGame {
        NewGame {
            mode,
            hoster_side: side,
            name: "test123".to_string(),
        }
    }
}

struct _Response {
    len: u8,
    content: [u8; BUF_LEN],
}

#[derive(Debug)]
#[repr(u8)]
pub enum Command {
    NewGame(NewGame),
    JoinGame(String),
    Nickname(String),
    Move(String),
    _Invalid = 0xFF,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Command::NewGame(_) => "New Game",
            Command::JoinGame(_) => "Join Game",
            Command::Nickname(_) => "Setting Nickname",
            Command::Move(_) => "Make Chess Move",
            Command::_Invalid => "Invalid Comand sent!",
        };
        write!(f, "{}", str)
    }
}

#[repr(u8)]
#[derive(Debug)]
pub enum PlayerSideRequest {
    Black = 0,
    White = 1,
    Random = 2,
}

impl TryFrom<u8> for PlayerSideRequest {
    type Error = ();

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            x if x == PlayerSideRequest::White as u8 => Ok(PlayerSideRequest::White),
            x if x == PlayerSideRequest::Black as u8 => Ok(PlayerSideRequest::Black),
            x if x == PlayerSideRequest::Random as u8 => Ok(PlayerSideRequest::Random),
            _ => Err(()),
        }
    }
}

// A token after a command is a Parameter.
// Parameters can be different types so we have
// to define some conversions.
trait Parameter<T> {
    fn to_val(&self) -> T;
}
impl Parameter<String> for &[u8] {
    fn to_val(&self) -> String {
        String::from_utf8_lossy(self).to_string()
    }
}
impl Parameter<u8> for &[u8] {
    fn to_val(&self) -> u8 {
        self[0]
    }
}
