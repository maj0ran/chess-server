mod buffer;
mod connection;

use crate::net::buffer::Buffer;
use crate::net::connection::Connection;
use crate::util::*;
use core::fmt;
use std::ops::{self, RangeTo};

#[allow(unused)]
use log::{debug, error, info, trace, warn};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::game::Chess;
use crate::tile::ToChessMove;

// bytes representing commands to server
const NEW_GAME: u8 = 0xA;
const JOIN_GAME: u8 = 0xB;
const SET_NAME: u8 = 0xC;

// read and write buffer have a static maximum length
// we don't really need more for chess
const BUF_LEN: usize = 64;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug)]
struct NewGame {
    mode: String,
    hoster_side: PlayerSideRequest,
    name: String,
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

struct Response {
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
    Invalid = 0xFF,
}

#[repr(u8)]
#[derive(Debug)]
enum PlayerSideRequest {
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


pub struct Interface {
    _listener: Option<TcpListener>,
}

impl Interface {
    pub fn new() -> Interface {
        Interface { _listener: None }
    }

    pub async fn listen(&self) -> Result<()> {
        info!("Listening...");
        let listener = TcpListener::bind("127.0.0.1:7878").await?;

        loop {
            let (socket, addr) = listener.accept().await?;
            info!("got connection from {}!", addr);
            let mut hndl = Connection {
                client: socket,
                nickname: String::new(),
                chess: None,
                in_buf: Buffer::new(),
                out_buf: Buffer::new(),
            };
            tokio::spawn(async move {
                hndl.run().await;
            });
        }
    }
}
