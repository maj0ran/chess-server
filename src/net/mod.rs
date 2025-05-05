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

pub type GameId = usize;
pub type ClientId = usize;

// bytes representing commands to server
pub const NEW_GAME: u8 = 0xA;
pub const JOIN_GAME: u8 = 0xB;
pub const SET_NAME: u8 = 0xC;
pub const MAKE_MOVE: u8 = 0xD;
pub const UPDATE_BOARD: u8 = 0xE;
// read and write buffer have a static maximum length
// we don't really need more for chess
const BUF_LEN: usize = 64;
pub type Result<T> = std::result::Result<T, ServerError>;

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
    Move(GameId, ChessMove),
    Update(Vec<(Tile, Option<Piece>)>),
    Register(Sender<ServerMessage>),
    _Invalid = 0xFF,
}

#[derive(Debug)]
pub struct ServerMessage {
    client_id: ClientId,
    cmd: Command,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Command::NewGame(_) => "New Game",
            Command::JoinGame(_) => "Join Game",
            Command::Nickname(_) => "Setting Nickname",
            Command::Move(_, _) => "Make Chess Move",
            Command::Update(_) => "Update Command",
            Command::_Invalid => "Invalid Comand sent!",
            Command::Register(_) => todo!(),
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

// --- Error Types ---
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("Network I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Client disconnected")]
    ClientDisconnected,
    #[error("Channel send error")]
    SendError, // Could be more specific
    #[error("Game logic error: {0}")]
    GameError(String),
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    #[error("Resource not found: {0}")]
    NotFound(String),
}

// --- Client -> Server Commands ---
#[derive(Debug)]
pub enum ClientCommand {
    CreateGame,
    JoinGame(GameId),
    SpectateGame(GameId),
    MakeMove(String), // Placeholder for move data
    ChatMessage(String),
    // Add other commands like Resign, OfferDraw, etc.
}

// --- Server -> Client Messages ---

// --- Message for GameManager Task ---
// This wraps client commands with metadata needed by the manager
#[derive(Debug)]
pub enum ManagerMessage {
    NewClient(ClientId, Sender<ServerMessage>), // Tell manager about a new client and how to talk back
    ClientDisconnected(ClientId),
    ClientCommand(ClientId, ClientCommand),
}
