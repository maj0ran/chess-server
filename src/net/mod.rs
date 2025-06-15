pub mod buffer;
pub mod connection;
pub mod frame;
pub mod game;
pub mod handler;
pub mod manager;
pub mod server;
use smol::channel::Sender;
use std::fmt;

use crate::net::handler::NetClient;
use crate::pieces::Piece;
use crate::{chessmove::ChessMove, tile::Tile};

pub type GameId = u32;
pub type ClientId = usize;

// read and write buffer have a static maximum length
// we don't really need more for chess
const BUF_LEN: usize = 64;

/*
 * we do NOT use an enum here, because rust safety concerns disallow us to map conveniently u8 <->
 * enum in both directions as in C. Because the mapping is non-exhaustive (not all u8 values map to
 * an opcode), a cast from u8 to enum could lead to undefined behavior. This is only fixed by
 * defining the mapping two times, first in enum OpCode { FOO = 1 } and second in impl From<u8> ...
 * with a match for every u8 => OpCode. This duplicated definition is ugly and error-prone. Using a
 * crate that gives us an magic macro would be another choice, but I want to minimize my
 * dependencies.
 * This solution seems to be the least invasive, and stupid enough for code that
 * doesn't want to hate itself.
 */
mod opcode {
    pub const LOGIN: u8 = 0xF0;
    pub const NEW_GAME: u8 = 0xA;
    pub const MAKE_MOVE: u8 = 0xC;

    pub const GAME_CREATED: u8 = 0x81;
    pub const JOIN_GAME: u8 = 0x82;
    pub const BOARD_UPDATED: u8 = 0x82;
}

/*
 * Parameter Structs for the Commands
 */

#[derive(Debug)]
pub struct NewGameParams {
    pub mode: u8,      // chess mode. only standard chess is supported anway.
    pub time: u32,     // clock time in seconds
    pub time_inc: u32, // clock increment after each turn in seconds
}

#[derive(Debug)]
pub struct JoinGameParams {
    pub game_id: u32,
    pub side: PlayerRole,
}

struct _Response {
    len: u8,
    content: [u8; BUF_LEN],
}

#[derive(Debug)]
#[repr(u8)]
pub enum Command {
    Register(Sender<ServerToClientMessage>),

    NewGame(NewGameParams),
    JoinGame(JoinGameParams),
    Move(GameId, ChessMove),
    _Invalid = 0xFF,
}

#[derive(Debug)]
pub struct ServerToClientMessage {
    msg: Response,
}

#[derive(Debug)]
#[repr(u8)]
pub enum Response {
    Update(Vec<(Tile, Option<Piece>)>) = 0x81,
    GameCreated(GameId, ClientId) = 0x82,
    GameJoined(GameId, ClientId, PlayerRole) = 0x83,
}
#[derive(Debug)]
pub struct ClientToServerMessage {
    client_id: ClientId,
    cmd: Command,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Command::NewGame(_) => "New Game",
            Command::JoinGame(_) => "Join Game",
            Command::Move(_, _) => "Make Chess Move",
            Command::_Invalid => "Invalid Comand sent!",
            Command::Register(_) => todo!(),
        };
        write!(f, "{}", str)
    }
}

#[repr(u8)]
#[derive(Debug)]
pub enum PlayerRole {
    Black = 0,
    White = 1,
    Random = 2,
    Spectator = 3,
    Both = 4,
}

// A token after a command is a Parameter.
// Parameters can be different types so we have
// to define some conversions.
trait Parameter<T> {
    fn to_param(&self) -> T;
}

impl Parameter<String> for &[u8] {
    fn to_param(&self) -> String {
        String::from_utf8_lossy(self).to_string()
    }
}
impl Parameter<u8> for &[u8] {
    fn to_param(&self) -> u8 {
        self[0]
    }
}
impl Parameter<u32> for &[u8] {
    fn to_param(&self) -> u32 {
        u32::from_ne_bytes(self[0..4].try_into().unwrap())
    }
}

impl Parameter<PlayerRole> for &[u8] {
    fn to_param(&self) -> PlayerRole {
        match self[0] {
            0 => PlayerRole::Black,
            1 => PlayerRole::White,
            2 => PlayerRole::Random,
            3 => PlayerRole::Spectator,
            4 => PlayerRole::Both,
            _ => PlayerRole::Spectator,
        }
    }
}

pub type Result<T> = std::result::Result<T, std::io::Error>;
// --- Error Types ---
//#[derive(Debug, thiserror::Error)]
//pub enum ServerError {
//    #[error("Network I/O error: {0}")]
//    Io(#[from] std::io::Error),
//    #[error("Client disconnected")]
//    ClientDisconnected,
//    #[error("Channel send error")]
//    SendError, // Could be more specific
//    #[error("Game logic error: {0}")]
//    GameError(String),
//    #[error("Invalid command: {0}")]
//    InvalidCommand(String),
//    #[error("Resource not found: {0}")]
//    NotFound(String),
//}
